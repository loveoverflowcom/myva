import { test, expect } from '@playwright/test';
import fs from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

// D05-A: fixture art tải qua AssetServer của Bevy WASM, kiểm hash trước khi giải mã. Lỗi tải giữ
// cảnh hình khối và trận vẫn chạy. Renderer là SwiftShader: ảnh chụp chỉ để soát hình, không đo GPU.
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
function collect(page) {
    const errors = [];
    const assets = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    page.on('response', response => {
        if (response.url().includes('/game/assets/')) assets.push({ url: new URL(response.url()).pathname, status: response.status() });
    });
    return { errors, assets };
}
async function evidence(name, data) {
    await fs.mkdir('target/web-evidence', { recursive: true });
    await fs.writeFile(`target/web-evidence/${name}.json`, JSON.stringify({
        date: new Date().toISOString(),
        revision: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
        renderer: 'WebGL2 via Chromium SwiftShader (software, not target hardware)',
        ...data,
    }, null, 2));
}

test('art fixture loads through AssetServer, toggles with G, combat state unaffected', async ({ page, browser }) => {
    const { errors, assets } = collect(page);
    await page.goto('/');
    const frame = await enter(page);
    await expect.poll(async () => (await state(frame)).art.status).toBe('ready');
    const ready = await state(frame);
    expect(ready.art).toEqual({ status: 'ready', visible: true, sheets: 5, failure: null });
    // Manifest, 5 atlas, 5 sheet và layout đều tải đúng một lần, không có request .meta.
    const fetched = assets.map(item => item.url).sort();
    expect(fetched.filter(url => url.endsWith('.png'))).toHaveLength(5);
    expect(fetched.filter(url => url.endsWith('.json'))).toHaveLength(7);
    expect(assets.every(item => item.status === 200)).toBe(true);
    expect(fetched.some(url => url.endsWith('.meta'))).toBe(false);

    await frame.locator('#game-canvas').click();
    await page.waitForTimeout(1500);
    await page.screenshot({ path: 'target/web-evidence/art-desktop.png' });

    await page.keyboard.press('KeyG');
    await expect.poll(async () => (await state(frame)).art.visible).toBe(false);
    await page.screenshot({ path: 'target/web-evidence/art-off.png' });
    await page.keyboard.press('KeyG');
    await expect.poll(async () => (await state(frame)).art.visible).toBe(true);

    const after = await state(frame);
    expect(after.tick).toBeGreaterThan(ready.tick);
    await evidence('art-load', { browser: browser.version(), art: after.art, assets, tick: after.tick, errors });
    expect(errors).toEqual([]);
});

test('corrupt atlas fails the hash check and keeps the graybox playable', async ({ page }) => {
    const { errors } = collect(page);
    await page.route('**/game/assets/fixture/long_luu/atlas.png', async route => {
        const response = await route.fetch();
        const body = Buffer.from(await response.body());
        body[body.length >> 1] ^= 0xff;
        await route.fulfill({ response, body });
    });
    await page.goto('/');
    const frame = await enter(page);
    await expect.poll(async () => (await state(frame)).art.status).toBe('failed');
    const failed = await state(frame);
    expect(failed.art).toEqual({ status: 'failed', visible: false, sheets: 0, failure: 'hash_mismatch' });
    // Trận vẫn chạy với hình khối.
    await expect.poll(async () => (await state(frame)).tick).toBeGreaterThan(failed.tick + 30);
    await page.screenshot({ path: 'target/web-evidence/art-fallback.png' });
    await evidence('art-corrupt', { art: failed.art, errors });
    // Bevy ghi lỗi tải asset ra console; đó là chẩn đoán mong đợi, không phải lỗi trang.
    expect(errors.filter(text => !text.includes('hash_mismatch'))).toEqual([]);
});

test('missing manifest reports io failure without breaking the match', async ({ page }) => {
    const { errors } = collect(page);
    await page.route('**/game/assets/fixture/manifest.json', route => route.fulfill({ status: 404, body: 'missing' }));
    await page.goto('/');
    const frame = await enter(page);
    await expect.poll(async () => (await state(frame)).art.status).toBe('failed');
    const failed = await state(frame);
    expect(failed.art.failure).toBe('io');
    await expect.poll(async () => (await state(frame)).tick).toBeGreaterThan(failed.tick + 30);
    // Trình duyệt ghi 404 của request bị chặn; Bevy ghi lỗi tải manifest.
    expect(errors.filter(text => !/404 \(Not Found\)|manifest\.json/.test(text))).toEqual([]);
});
