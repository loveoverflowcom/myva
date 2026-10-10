import { test, expect } from '@playwright/test';
import fs from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

// Trạng thái trực tiếp từ bridge WASM (cùng module instance với runtime của game document).
async function state(frame) {
    return frame.evaluate(async () => JSON.parse((await import('./pkg/myva_web_game.js')).telemetry()));
}
async function enter(page) {
    await page.locator('#enter-game').click();
    await expect(page.locator('#shell-status')).toContainText(/sẵn sàng|Đánh boss/);
    const frame = page.frames().find(item => item.url().includes('/game/?'));
    expect(frame).toBeTruthy();
    await expect.poll(async () => (await state(frame)).paused).toBe(false);
    return frame;
}
function collectErrors(page) {
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    return errors;
}
function evidence(name, data) {
    return fs.mkdir('target/web-evidence', { recursive: true })
        .then(() => fs.writeFile(`target/web-evidence/${name}.json`, JSON.stringify({
            date: new Date().toISOString(),
            revision: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
            renderer: 'WebGL2 via Chromium SwiftShader (software, not target hardware)',
            ...data,
        }, null, 2)));
}

test('keyboard: walk, light attack lands on the boss, keydown→tick latency recorded', async ({ page, browser }) => {
    const errors = collectErrors(page);
    await page.goto('/');
    const frame = await enter(page);
    const canvas = frame.locator('#game-canvas');
    await canvas.click();
    const start = await state(frame);
    expect(start.mode).toBe('boss');
    expect(start.player.x).toBe(400);

    // Thực tế người chơi: đi về phía boss tới trong tầm Lưu Tiễn (~520 px), bắn bằng Q, đánh
    // thêm J. Khung hình software chậm làm nhả phím trễ, nên tầm bắn rộng giữ test ổn định;
    // luật do lõi quyết định, test chỉ kiểm kết quả quan sát được.
    const RANGE = 420;
    const deadline = Date.now() + 120000;
    let s = start;
    while (s.tally.player.hits === 0 && s.player.state !== 'downed' && Date.now() < deadline) {
        const gap = s.rival.x - s.player.x;
        if (Math.abs(gap) > RANGE) {
            const key = gap > 0 ? 'ArrowRight' : 'ArrowLeft';
            await page.keyboard.down(key);
            await expect.poll(async () => {
                const now = await state(frame);
                return Math.abs(now.rival.x - now.player.x) <= RANGE || now.player.state === 'hitstun';
            }, { timeout: 60000 }).toBe(true);
            await page.keyboard.up(key);
        }
        await page.keyboard.press(s.player.cooldowns[0] === 0 ? 'KeyQ' : 'KeyJ');
        await page.waitForTimeout(1500);
        s = await state(frame);
    }
    expect(s.tally.player.hits).toBeGreaterThan(0);
    expect(s.rival.hp).toBeLessThan(s.rival.max_hp);
    expect(s.tally.player.actions).toBeGreaterThan(0);
    const metrics = await frame.evaluate(() => window.__myvaMetrics);
    expect(metrics.inputSamples.length).toBeGreaterThan(0);

    await evidence('combat-keyboard', {
        browser: browser.version(),
        disclaimer: 'Latency = DOM keydown to the fixed tick that consumed the press, observed on requestAnimationFrame; not input-to-photon.',
        final: s, inputSamples: metrics.inputSamples, updateHz: metrics.samples.map(item => item.updateHz), errors,
    });
    expect(errors).toEqual([]);
});

test('gamepad: stick moves, X light, RB+X is Lưu Tiễn without a stray light (mocked Gamepad API)', async ({ browser }) => {
    const context = await browser.newContext();
    // Chromium headless không có tay cầm thật; gilrs đọc navigator.getGamepads() mỗi khung nên
    // một đối tượng theo Standard Gamepad mapping đủ để đi qua cùng đường input của Bevy.
    await context.addInitScript(() => {
        const pad = {
            id: 'MyVa Test Pad (STANDARD GAMEPAD Vendor: 1234 Product: 5678)',
            index: 0, connected: true, mapping: 'standard', timestamp: 0,
            axes: [0, 0, 0, 0],
            buttons: Array.from({ length: 17 }, () => ({ pressed: false, touched: false, value: 0 })),
        };
        window.__myvaPad = pad;
        Object.defineProperty(Navigator.prototype, 'getGamepads', { configurable: true, value: () => [pad, null, null, null] });
    });
    const page = await context.newPage();
    const errors = collectErrors(page);
    try {
        await page.goto('http://127.0.0.1:8080/');
        const frame = await enter(page);
        await expect.poll(async () => (await state(frame)).gamepads).toBe(1);
        const set = (index, pressed) => frame.evaluate(([i, p]) => {
            window.__myvaPad.buttons[i].pressed = p;
            window.__myvaPad.buttons[i].value = p ? 1 : 0;
            window.__myvaPad.timestamp += 1;
        }, [index, pressed]);
        const hold = async (indices, ms = 700) => {
            for (const index of indices) await set(index, true);
            await frame.waitForTimeout(ms);
            for (const index of [...indices].reverse()) await set(index, false);
        };

        const before = await state(frame);
        await frame.evaluate(() => { window.__myvaPad.axes[0] = 1; });
        await expect.poll(async () => (await state(frame)).player.x).toBeGreaterThan(before.player.x + 20);
        await frame.evaluate(() => { window.__myvaPad.axes[0] = 0; });

        // X (West, index 2): đòn nhẹ Gợn Sóng.
        const preLight = await state(frame);
        await hold([2]);
        await expect.poll(async () => (await state(frame)).tally.player.actions).toBe(preLight.tally.player.actions + 1);
        await frame.waitForTimeout(800);

        // Giữ RB (index 5) rồi X: Lưu Tiễn, đúng một hành động, tiêu 20 năng lượng.
        const preSkill = await state(frame);
        expect(preSkill.player.cooldowns[0]).toBe(0);
        await set(5, true);
        await frame.waitForTimeout(400);
        await hold([2]);
        await set(5, false);
        await expect.poll(async () => (await state(frame)).player.cooldowns[0]).toBeGreaterThan(0);
        await frame.waitForTimeout(800);
        const after = await state(frame);
        expect(after.tally.player.actions).toBe(preSkill.tally.player.actions + 1);
        expect(after.player.energy).toBeLessThan(preSkill.player.energy);

        await evidence('combat-gamepad', {
            browser: browser.version(),
            disclaimer: 'Mocked navigator.getGamepads() with Standard Gamepad mapping; verifies gilrs → Bevy → sim wiring, not a physical controller.',
            before, preSkill, after, errors,
        });
        expect(errors).toEqual([]);
    } finally {
        await context.close();
    }
});

test('autopilot B1 beats the boss; replay verifies in the browser and natively', async ({ page, browser }) => {
    test.setTimeout(420000);
    const errors = collectErrors(page);
    await page.goto('/');
    const frame = await enter(page);
    await page.locator('#autopilot-game').click();
    await expect.poll(async () => (await state(frame)).autopilot).toBe(true);
    const started = Date.now();
    await expect.poll(async () => (await state(frame)).outcome, { timeout: 360000, intervals: [2000] }).toBe('victory');
    await expect.poll(async () => (await state(frame)).verified, { timeout: 30000 }).toBe(true);
    const final = await state(frame);
    expect(final.rival.hp).toBe(0);
    expect(final.phase).toBe(3);
    await expect(page.locator('#shell-status')).toContainText('replay đã kiểm chứng');

    const downloadPromise = page.waitForEvent('download');
    await page.locator('#replay-game').click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/^myva-boss-r1-t\d+\.myva-replay$/);
    const path = `target/web-evidence/${download.suggestedFilename()}`;
    await download.saveAs(path);
    // Replay ghi trên wasm32 được công cụ native chạy lại: so mọi mốc hash và hash cuối.
    const verify = execFileSync('cargo', ['run', '--quiet', '--locked', '-p', 'myva-sim', '--bin', 'myva-replay', '--', 'verify', path],
        { encoding: 'utf8' });
    expect(verify).toContain(`hash cuối ${final.hash}`);
    expect(verify).toContain('Kẻ Giữ Đập: HP 0/2400, Downed');

    await evidence('combat-autopilot', {
        browser: browser.version(),
        disclaimer: 'Bot B1 (250 ms reaction) pilots the player through the same input path; the wasm32 replay is re-run by native myva-replay.',
        wallSeconds: (Date.now() - started) / 1000, final, replay: download.suggestedFilename(), nativeVerify: verify,
        updateHz: (await frame.evaluate(() => window.__myvaMetrics.samples)).map(item => item.updateHz), errors,
    });
    await page.screenshot({ path: 'target/web-evidence/combat-victory.png', fullPage: true });
    expect(errors).toEqual([]);
});
