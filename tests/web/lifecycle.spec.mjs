import { test, expect } from '@playwright/test';
import fs from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

async function telemetry(frame) {
    return frame.evaluate(() => window.__myvaMetrics.samples.at(-1));
}
async function enter(page) {
    await page.locator('#enter-game').click();
    await expect(page.locator('#shell-status')).toContainText(/sẵn sàng|Linh lực:/);
    const frame = page.frames().find(item => item.url().includes('/game/?'));
    expect(frame).toBeTruthy();
    await expect.poll(async () => (await telemetry(frame))?.ready).toBe(true);
    return frame;
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
            await page.keyboard.down('ArrowRight');
            await expect.poll(async () => (await telemetry(frame)).score).toBeGreaterThan(0);
            await page.keyboard.up('ArrowRight');
            await page.locator('#pause-game').click();
            await expect.poll(async () => (await telemetry(frame)).paused).toBe(true);
            const paused = await telemetry(frame);
            await page.waitForTimeout(600);
            expect((await telemetry(frame)).x).toBe(paused.x);
            await page.locator('#pause-game').click();
            await expect.poll(async () => (await telemetry(frame)).paused).toBe(false);
            await page.locator('#chat').fill('Tiếng Việt: Thần Mạch');
            const before = await telemetry(frame);
            await page.keyboard.press('ArrowRight');
            await page.waitForTimeout(600);
            expect((await telemetry(frame)).x).toBe(before.x);
            await page.locator('#reset-game').click();
            await expect.poll(async () => (await telemetry(frame)).score).toBe(0);
            await page.setViewportSize({ width: 900, height: 700 });
            await expect.poll(async () => canvas.evaluate(el => el.width)).toBeGreaterThan(500);
            await page.setViewportSize({ width: 1280, height: 900 });
            await canvas.click();
            await page.keyboard.down('ArrowRight');
            await page.waitForTimeout(300);
            await page.locator('#chat').focus();
            await page.keyboard.up('ArrowRight');
            await page.waitForTimeout(600);
            const blurred = await telemetry(frame);
            await page.waitForTimeout(600);
            expect((await telemetry(frame)).x).toBe(blurred.x);
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

test('touch/cancel, DPR, unsupported renderer, load failure and context loss', async ({ browser }) => {
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
        const canvas = frame.locator('canvas');
        const size = await canvas.boundingBox();
        const cdp = await context.newCDPSession(page);
        const point = { x: size.x + size.width / 2, y: size.y + size.height / 2, id: 1 };
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [point] });
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchMove', touchPoints: [{ ...point, x: point.x + 70 }] });
        await expect.poll(async () => (await telemetry(frame)).x).toBeGreaterThan(-180);
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchCancel', touchPoints: [] });
        await page.waitForTimeout(600);
        const after = await telemetry(frame);
        await page.waitForTimeout(600);
        expect((await telemetry(frame)).x).toBe(after.x);
        expect(await canvas.evaluate(el => el.width / el.getBoundingClientRect().width)).toBeCloseTo(2, 0);
        await page.screenshot({ path: 'target/web-evidence/touch-emulation.png', fullPage: true });
        await fs.writeFile('target/web-evidence/touch-emulation.json', JSON.stringify({
            disclaimer: 'Chromium desktop touch emulation and process scale factor 2, NOT a physical mobile browser',
            metrics: await frame.evaluate(() => window.__myvaMetrics), errors,
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
    await expect(failedPage.locator('#shell-status')).toContainText('Không thể mở bãi tập');
    expect(failedErrors).toEqual([]);
    await fs.writeFile('target/web-evidence/load-failure.json', JSON.stringify({
        status: await failedPage.locator('#shell-status').textContent(), pageErrors: failedErrors,
        fault: 'WASM request intentionally aborted; error must reach shell without calling uninitialized exports',
    }, null, 2));
    await failed.close();
});
