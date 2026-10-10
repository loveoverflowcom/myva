import { test, expect } from '@playwright/test';
import fs from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

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
// Đấu tập với bot đứng yên: không đòn nào chạm người chơi, nên vị trí chỉ đổi theo input.
async function quietDuel(page, frame) {
    const before = await state(frame);
    await page.locator('#mode-game').click();
    await expect.poll(async () => (await state(frame)).mode).toBe('duel');
    expect((await state(frame)).round).toBe(before.round + 1);
    await frame.locator('#game-canvas').focus();
    await page.keyboard.press('KeyB');
    await expect.poll(async () => (await state(frame)).sparring).toBe(false);
}

// Vị trí người chơi không đổi qua `frames` khung Bevy liên tiếp.
async function stillFor(frame, frames) {
    const first = await state(frame);
    let now = first;
    while (now.updates < first.updates + frames) {
        await new Promise(resolve => setTimeout(resolve, 100));
        now = await state(frame);
    }
    return now.player.x === first.player.x;
}

test('30 document lifecycles, input ownership and runtime evidence', async ({ page, browser }) => {
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    const cdp = await page.context().newCDPSession(page);
    await cdp.send('Performance.enable');
    await cdp.send('HeapProfiler.enable');
    await page.goto('/');
    await expect(page.locator('#enter-game')).toBeVisible();
    await page.locator('#nickname').fill('Khách Thần Mạch');
    const cycles = [];
    let firstLoad;
    for (let index = 0; index < 30; index++) {
        const frame = await enter(page);
        const canvas = frame.locator('canvas');
        await canvas.click();
        if (index === 0) {
            const resources = () => performance.getEntriesByType('resource').map(entry => ({
                name: entry.name, transferSize: entry.transferSize,
                encodedBodySize: entry.encodedBodySize, decodedBodySize: entry.decodedBodySize,
                duration: entry.duration,
            }));
            firstLoad = { shell: await page.evaluate(resources), game: await frame.evaluate(resources),
                note: 'Fresh browser context; resource timing includes response headers in transferSize; local server sends uncompressed bodies. Navigation HTML is recorded separately.',
                navigation: await page.evaluate(() => performance.getEntriesByType('navigation').map(entry => entry.toJSON())) };

            // Đấu lại tạo trận mới ở vòng kế tiếp, tick về đầu.
            await expect.poll(async () => (await state(frame)).tick).toBeGreaterThan(30);
            const running = await state(frame);
            await page.locator('#rematch-game').click();
            await expect.poll(async () => (await state(frame)).round).toBe(running.round + 1);
            expect((await state(frame)).tick).toBeLessThan(running.tick);

            await quietDuel(page, frame);
            const start = await state(frame);
            await page.keyboard.down('ArrowRight');
            await expect.poll(async () => (await state(frame)).player.x).toBeGreaterThan(start.player.x + 20);
            await page.keyboard.up('ArrowRight');

            await page.locator('#pause-game').click();
            await expect.poll(async () => (await state(frame)).paused).toBe(true);
            const paused = await state(frame);
            await page.waitForTimeout(600);
            expect((await state(frame)).tick).toBe(paused.tick);
            await page.locator('#pause-game').click();
            await expect.poll(async () => (await state(frame)).paused).toBe(false);

            // Ô chat của shell giữ phím: mũi tên không tới canvas.
            await page.locator('#chat').fill('Tiếng Việt: Thần Mạch');
            const before = await state(frame);
            await page.keyboard.press('ArrowRight');
            await page.waitForTimeout(600);
            expect((await state(frame)).player.x).toBe(before.player.x);

            await page.setViewportSize({ width: 900, height: 700 });
            await expect.poll(async () => canvas.evaluate(el => el.width)).toBeGreaterThan(500);
            await page.setViewportSize({ width: 1280, height: 900 });

            // Giữ phím rồi chuyển focus: Bevy nhả mọi phím khi canvas mất focus. Khung hình
            // SwiftShader chậm và không đều, nên đợi vị trí đứng yên qua ba khung Bevy rồi mới
            // kiểm tra nó tiếp tục đứng yên, thay vì giả định một khoảng chờ cố định.
            await canvas.click();
            await page.keyboard.down('ArrowRight');
            await page.waitForTimeout(400);
            await page.locator('#chat').focus();
            await page.keyboard.up('ArrowRight');
            await expect.poll(() => stillFor(frame, 3), { timeout: 30000 }).toBe(true);
            const blurred = await state(frame);
            await page.waitForTimeout(600);
            expect((await state(frame)).player.x).toBe(blurred.player.x);
            await page.screenshot({ path: 'target/web-evidence/desktop.png', fullPage: true });
        }
        const metrics = await frame.evaluate(() => window.__myvaMetrics);
        errors.push(...metrics.errors);
        const oldId = frame.url();
        await page.locator('#exit-game').click();
        await expect(page.locator('iframe')).toHaveCount(0);
        expect(page.frames().some(item => item.url() === oldId)).toBe(false);
        await cdp.send('HeapProfiler.collectGarbage');
        const counts = await cdp.send('Memory.getDOMCounters');
        const perf = await cdp.send('Performance.getMetrics');
        cycles.push({ index: index + 1, ...counts, metrics,
            heapBytes: perf.metrics.find(item => item.name === 'JSHeapUsedSize')?.value });
    }
    // No old game timers should dispatch status after its browsing context is gone.
    const status = await page.locator('#shell-status').textContent();
    const idleBefore = await cdp.send('Performance.getMetrics');
    await page.waitForTimeout(1500);
    expect(await page.locator('#shell-status').textContent()).toBe(status);
    const idleAfter = await cdp.send('Performance.getMetrics');
    const bundle = await (await page.request.get('/bundle-report.json')).json();
    await fs.writeFile('target/web-evidence/lifecycle.json', JSON.stringify({
        date: new Date().toISOString(), browser: browser.version(), platform: process.platform,
        revision: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
        renderer: 'WebGL2 via Chromium SwiftShader (software, not target hardware)',
        bundle, firstLoad, cycles, idleBefore, idleAfter, errors,
    }, null, 2));
    // Detect growth in surviving DOM/listeners; WASM/GPU memory needs device tooling too.
    expect(cycles.at(-1).jsEventListeners).toBeLessThanOrEqual(cycles[1].jsEventListeners + 4);
    expect(cycles.at(-1).documents).toBeLessThanOrEqual(cycles[1].documents + 1);
    expect(errors).toEqual([]);
});

test('touch controls, cancel, DPR, unsupported renderer, load failure and context loss', async ({ browser }) => {
    // Chromium's context emulation changes devicePixelRatio but not the native
    // ResizeObserver devicePixelContentBoxSize used by winit. Match both scales.
    const launch = test.info().project.use.launchOptions;
    const scaledBrowser = await browser.browserType().launch({ ...launch,
        args: [...launch.args, '--force-device-scale-factor=2'] });
    const context = await scaledBrowser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
    try {
        const page = await context.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(error.message));
        await page.goto('http://127.0.0.1:8080');
        const frame = await enter(page);
        // Con trỏ thô: lớp nút cảm ứng hiện ngay khi vào trận.
        await expect.poll(async () => (await state(frame)).touch).toBe(true);
        await quietDuel(page, frame);
        const canvas = frame.locator('canvas');
        const box = await canvas.boundingBox();
        const layout = await frame.evaluate(async () => JSON.parse((await import('./pkg/myva_web_game.js')).touch_layout()));
        expect(layout.width).toBeCloseTo(box.width, 0);
        for (const pad of Object.values(layout.buttons)) expect(pad.r * 2).toBeGreaterThanOrEqual(48);
        const cdp = await context.newCDPSession(page);
        const touch = (type, points) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: points });

        // Joystick nổi ở nửa trái: kéo phải thì đi, hủy chạm thì dừng.
        const start = await state(frame);
        const stick = { x: box.x + box.width * 0.15, y: box.y + box.height * 0.45, id: 1 };
        await touch('touchStart', [stick]);
        await touch('touchMove', [{ ...stick, x: stick.x + 70 }]);
        await expect.poll(async () => (await state(frame)).player.x).toBeGreaterThan(start.player.x + 20);
        await touch('touchCancel', []);
        await page.waitForTimeout(600);
        const after = await state(frame);
        await page.waitForTimeout(600);
        expect((await state(frame)).player.x).toBe(after.player.x);

        // Chạm nút nhẹ: đúng một hành động.
        const light = layout.buttons.light;
        await touch('touchStart', [{ x: box.x + light.x, y: box.y + light.y, id: 2 }]);
        await touch('touchEnd', []);
        await expect.poll(async () => (await state(frame)).tally.player.actions).toBe(after.tally.player.actions + 1);

        expect(await canvas.evaluate(el => el.width / el.getBoundingClientRect().width)).toBeCloseTo(2, 0);
        await page.screenshot({ path: 'target/web-evidence/touch-emulation.png', fullPage: true });
        await fs.writeFile('target/web-evidence/touch-emulation.json', JSON.stringify({
            disclaimer: 'Chromium desktop touch emulation and process scale factor 2, NOT a physical mobile browser',
            layout, final: await state(frame), metrics: await frame.evaluate(() => window.__myvaMetrics), errors,
        }, null, 2));
        expect(errors).toEqual([]);
        await canvas.evaluate(el => el.getContext('webgl2').getExtension('WEBGL_lose_context').loseContext());
        await expect(page.locator('#shell-status')).toContainText('Mất kết nối đồ họa');
        await page.waitForTimeout(500);
        await fs.writeFile('target/web-evidence/context-loss.json', JSON.stringify({
            handled: true, status: await page.locator('#shell-status').textContent(), pageErrors: errors,
            runtimeErrors: await frame.evaluate(() => window.__myvaMetrics.errors),
        }, null, 2));
        await page.locator('#exit-game').click();
    } finally {
        await context.close();
        await scaledBrowser.close();
    }

    // Bố cục chính cho điện thoại: màn hình ngang, game toàn màn hình, hai hàng nút bên phải.
    const landscape = await browser.newContext({ viewport: { width: 844, height: 390 }, isMobile: true, hasTouch: true });
    try {
        const page = await landscape.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(error.message));
        await page.goto('http://127.0.0.1:8080');
        const frame = await enter(page);
        await page.locator('#fullscreen-game').click();
        await expect.poll(() => page.evaluate(() => document.fullscreenElement?.id)).toBe('game-frame');
        await expect.poll(async () => (await frame.locator('canvas').boundingBox()).height).toBeGreaterThan(380);
        const readLayout = () => frame.evaluate(async () => JSON.parse((await import('./pkg/myva_web_game.js')).touch_layout()));
        // Layout được tính lại ở khung hình sau khi canvas đổi kích thước.
        await expect.poll(async () => Math.round((await readLayout()).width)).toBe(844);
        const layout = await readLayout();
        const { light, heavy, guard, skill1 } = layout.buttons;
        expect(guard.x).toBeLessThan(heavy.x);
        expect(skill1.y).toBeLessThan(light.y);
        await expect.poll(async () => (await state(frame)).touch).toBe(true);
        await page.waitForTimeout(500);
        await page.screenshot({ path: 'target/web-evidence/touch-landscape.png' });
        await fs.writeFile('target/web-evidence/touch-landscape.json', JSON.stringify({
            disclaimer: 'Chromium touch emulation, 844x390 landscape, iframe fullscreen; NOT a physical phone',
            layout, errors,
        }, null, 2));
        expect(errors).toEqual([]);
    } finally {
        await landscape.close();
    }

    const unsupported = await browser.newContext();
    await unsupported.addInitScript(() => {
        const original = HTMLCanvasElement.prototype.getContext;
        HTMLCanvasElement.prototype.getContext = function (type, ...args) {
            return type === 'webgl2' ? null : original.call(this, type, ...args);
        };
    });
    const unsupportedPage = await unsupported.newPage();
    await unsupportedPage.goto('http://127.0.0.1:8080');
    await unsupportedPage.locator('#enter-game').click();
    await expect(unsupportedPage.locator('#shell-status')).toContainText('chưa hỗ trợ WebGL2');
    await unsupported.close();

    const failed = await browser.newContext();
    await failed.route('**/game/pkg/*.wasm', route => route.abort());
    const failedPage = await failed.newPage();
    const failedErrors = [];
    failedPage.on('pageerror', error => failedErrors.push(error.message));
    await failedPage.goto('http://127.0.0.1:8080');
    await failedPage.locator('#enter-game').click();
    await expect(failedPage.locator('#shell-status')).toContainText('Không thể mở trận');
    expect(failedErrors).toEqual([]);
    await fs.writeFile('target/web-evidence/load-failure.json', JSON.stringify({
        status: await failedPage.locator('#shell-status').textContent(), pageErrors: failedErrors,
        fault: 'WASM request intentionally aborted; error must reach shell without calling uninitialized exports',
    }, null, 2));
    await failed.close();
});
