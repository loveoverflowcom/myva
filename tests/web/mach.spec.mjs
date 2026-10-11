import { test, expect } from '@playwright/test';
import fs from 'node:fs/promises';

const state = frame => frame.evaluate(async () => JSON.parse((await import('./pkg/myva_web_game.js')).telemetry()));

async function enter(page) {
    await page.goto(test.info().project.use.baseURL);
    await page.locator('#enter-game').click();
    await expect(page.locator('#shell-status')).toContainText(/sẵn sàng|Đánh boss/);
    const frame = page.frames().find(item => item.url().includes('/game/?'));
    await expect.poll(async () => (await state(frame)).paused).toBe(false);
    return frame;
}

test('Mạch keyboard: hold requests ultimate without a stray normal cast; tap casts normal once', async ({ page }) => {
    const frame = await enter(page);
    await frame.locator('#game-canvas').click();
    const before = await state(frame);
    expect(before.player.mach).toBe(0);
    // Leaving the canvas releases winit keys; that release must not become a tap.
    await page.keyboard.down('KeyR');
    await page.waitForTimeout(100);
    await page.locator('#chat').focus();
    await page.keyboard.up('KeyR');
    await page.waitForTimeout(600);
    const blurred = await state(frame);
    expect(blurred.tally.player.actions).toBe(before.tally.player.actions);
    expect(blurred.player.cooldowns[2]).toBe(0);
    await frame.locator('#game-canvas').click();
    await page.keyboard.down('KeyR');
    await expect.poll(async () => (await state(frame)).player.denial).toBe('Some(Mach)');
    await page.keyboard.up('KeyR');
    await page.waitForTimeout(500);
    const denied = await state(frame);
    expect(denied.tally.player.actions).toBe(before.tally.player.actions);
    expect(denied.player.energy).toBe(100);
    expect(denied.player.cooldowns[2]).toBe(0);
    await page.keyboard.press('KeyR');
    await expect.poll(async () => (await state(frame)).player.cooldowns[2]).toBeGreaterThan(0);
    const normal = await state(frame);
    expect(normal.tally.player.actions).toBe(before.tally.player.actions + 1);
    expect(normal.player.energy).toBeLessThan(100);
    await page.screenshot({ path: 'target/web-evidence/mach-keyboard.png', fullPage: true });
    await fs.writeFile('target/web-evidence/mach-keyboard.json', JSON.stringify({ before, blurred, denied, normal }, null, 2));
});

test('Mạch touch: eight controls, long hold is explicit and release never double casts', async ({ browser }) => {
    const context = await browser.newContext({ viewport: { width: 844, height: 390 }, isMobile: true, hasTouch: true });
    try {
        const page = await context.newPage();
        const frame = await enter(page);
        await expect.poll(async () => (await state(frame)).touch).toBe(true);
        const box = await frame.locator('#game-canvas').boundingBox();
        const layout = await frame.evaluate(async () => JSON.parse((await import('./pkg/myva_web_game.js')).touch_layout()));
        expect(Object.keys(layout.buttons)).toHaveLength(8);
        const pad = layout.buttons.skill3;
        const cdp = await context.newCDPSession(page);
        const point = { x: box.x + pad.x, y: box.y + pad.y, id: 19 };
        const before = await state(frame);
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [point] });
        await expect.poll(async () => (await state(frame)).player.denial).toBe('Some(Mach)');
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
        await page.waitForTimeout(600);
        const denied = await state(frame);
        expect(denied.tally.player.actions).toBe(before.tally.player.actions);
        expect(denied.player.cooldowns[2]).toBe(0);
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [point] });
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchCancel', touchPoints: [] });
        await page.waitForTimeout(500);
        expect((await state(frame)).tally.player.actions).toBe(before.tally.player.actions);
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [point] });
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
        await expect.poll(async () => (await state(frame)).player.cooldowns[2]).toBeGreaterThan(0);
        const normal = await state(frame);
        expect(normal.tally.player.actions).toBe(before.tally.player.actions + 1);
        await page.screenshot({ path: 'target/web-evidence/mach-touch.png', fullPage: true });
        await fs.writeFile('target/web-evidence/mach-touch.json', JSON.stringify({ disclaimer: 'Desktop Chromium touch emulation; not physical mobile evidence.', layout, before, denied, normal }, null, 2));
    } finally {
        await context.close();
    }
});
