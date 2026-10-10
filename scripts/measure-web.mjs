// Run after npm ci, building the web shell, and starting its static server:
//   CHROME_PATH=/usr/bin/google-chrome node scripts/measure-web.mjs
//   BASE_URL=http://127.0.0.1:8081/ node scripts/measure-web.mjs
// CHROME_PATH is optional; otherwise Playwright uses its installed Chromium.
// The headed comparison needs an available desktop display and opens a window.
// Each scene has a 2-second warm-up and one 5-second observation. These are RAF
// and Bevy Update rates, not presentation FPS or a production budget verdict.
// The combat scene keeps running during the sample (boss AI included).

import { chromium } from '@playwright/test';
import fs from 'node:fs/promises';

const baseURL = new URL(process.env.BASE_URL || 'http://127.0.0.1:8080/');
if (!baseURL.pathname.endsWith('/')) baseURL.pathname += '/';
const outputDirectory = new URL('../target/web-evidence/', import.meta.url);
const output = new URL('rate-diagnostics.json', outputDirectory);
const scenarios = [
    { name: 'headless-forced-swiftshader', headless: true, args: ['--enable-unsafe-swiftshader', '--use-angle=swiftshader'] },
    { name: 'headless-default', headless: true, args: [] },
    { name: 'headed-default', headless: false, args: [] },
];
const results = [];

for (const scenario of scenarios) {
    let browser;
    const errors = [];
    const warnings = [];
    const record = {
        scenario,
        recorded_at: new Date().toISOString(),
        baseURL: baseURL.href,
        measurement_note: 'One 5-second scene sample after 2-second warm-up; RAF and Bevy Update rates are not GPU presentation FPS. Input latency is DOM keydown (Playwright CDP key events, not keyboard hardware) to the fixed tick that consumed the press, observed on RAF. A headed desktop may receive real user input; inspect observedInputEvents and before/after state for contamination.',
    };
    if (!scenario.headless && process.platform === 'linux' && !process.env.DISPLAY) {
        record.skipped = 'No DISPLAY is available for the headed Linux comparison.';
        results.push(record);
        console.log(JSON.stringify(record));
        await fs.mkdir(outputDirectory, { recursive: true });
        await fs.writeFile(output, JSON.stringify(results, null, 2) + '\n');
        continue;
    }
    try {
        browser = await chromium.launch({
            executablePath: process.env.CHROME_PATH || undefined,
            headless: scenario.headless,
            args: scenario.args,
            timeout: 20000,
        });
        record.browser = browser.version();
        const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
        const page = await context.newPage();
        page.on('pageerror', error => errors.push(error.message));
        page.on('console', message => {
            if (message.type() === 'error') errors.push(message.text());
            if (message.type() === 'warning') warnings.push(message.text());
        });
        const response = await page.request.get(new URL('bundle-report.json', baseURL).href);
        if (!response.ok()) throw new Error(`Build report request returned ${response.status()}`);
        const build = await response.json();
        record.build = Object.fromEntries(['revision', 'dirty', 'profile', 'rustc'].map(key => [key, build[key]]));
        await page.goto(baseURL.href);
        await page.locator('#enter-game').click();
        await page.waitForFunction(
            () => document.querySelector('#shell-status').textContent.includes('Đánh boss'),
            null,
            { timeout: 30000 },
        );
        const frame = page.frames().find(item => item.url().includes('/game/?'));
        if (!frame) throw new Error('Game document did not mount');
        await frame.locator('canvas').click();
        await page.waitForTimeout(2000);
        record.sample = await frame.evaluate(async () => {
            const game = await import('./pkg/myva_web_game.js');
            const gl = document.querySelector('canvas').getContext('webgl2');
            if (!gl) throw new Error('Game canvas has no WebGL2 context');
            const debug = gl.getExtension('WEBGL_debug_renderer_info');
            const renderer = {
                vendor: gl.getParameter(gl.VENDOR),
                renderer: gl.getParameter(gl.RENDERER),
                unmaskedVendor: debug ? gl.getParameter(debug.UNMASKED_VENDOR_WEBGL) : null,
                unmaskedRenderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null,
            };
            const inputObserver = new AbortController();
            const observedInputEvents = {};
            for (const type of ['keydown', 'keyup', 'pointerdown', 'pointermove', 'pointerup', 'blur']) {
                window.addEventListener(type, event => {
                    if (event.isTrusted) observedInputEvents[type] = (observedInputEvents[type] || 0) + 1;
                }, { capture: true, signal: inputObserver.signal });
            }
            const before = JSON.parse(game.telemetry());
            const start = performance.now();
            let raf = 0;
            const rafGaps = [];
            let last = null;
            await new Promise(resolve => {
                let request;
                const finish = () => { clearTimeout(timer); cancelAnimationFrame(request); resolve(); };
                // A hidden/stalled document must produce a bounded observation.
                const timer = setTimeout(finish, 6000);
                function next(timestamp) {
                    raf++;
                    // RAF timestamps precede callback execution; performance.now()
                    // is not a valid baseline for the first RAF timestamp.
                    if (last !== null) rafGaps.push(timestamp - last);
                    last = timestamp;
                    if (performance.now() - start >= 5000) finish();
                    else request = requestAnimationFrame(next);
                }
                request = requestAnimationFrame(next);
            });
            const end = performance.now();
            const after = JSON.parse(game.telemetry());
            inputObserver.abort();
            return {
                renderer, before, after, elapsedMs: end - start,
                rafCallbacks: raf,
                rafRateHz: raf * 1000 / (end - start),
                bevyUpdateRateHz: (after.updates - before.updates) * 1000 / (end - start),
                // Tick luật đã chạy mỗi giây thực; 60 nếu khung hình không vượt max delta 250 ms.
                simTickRateHz: (after.tick - before.tick) * 1000 / (end - start),
                rafGaps, observedInputEvents,
                visibility: document.visibilityState, focused: document.hasFocus(),
                canvas: { width: gl.drawingBufferWidth, height: gl.drawingBufferHeight },
                telemetry: window.__myvaMetrics.samples,
            };
        });
        // Đòn nhẹ cách nhau đủ xa để mỗi lần nhấn vào một tick riêng.
        const canvas = frame.locator('canvas');
        await canvas.focus();
        for (let press = 0; press < 6; press++) {
            await page.keyboard.press('KeyJ');
            await page.waitForTimeout(700);
        }
        record.inputSamplesMs = await frame.evaluate(() => window.__myvaMetrics.inputSamples);
    } catch (error) {
        record.error = String(error);
        process.exitCode = 1;
    } finally {
        await browser?.close();
    }
    record.errors = errors;
    record.warnings = warnings;
    results.push(record);
    console.log(JSON.stringify({
        ...record,
        sample: record.sample && { ...record.sample, rafGaps: undefined, telemetry: undefined },
    }));
    await fs.mkdir(outputDirectory, { recursive: true });
    await fs.writeFile(output, JSON.stringify(results, null, 2) + '\n');
}
console.log(`Saved ${output.pathname}`);
