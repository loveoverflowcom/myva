import { test, expect } from '@playwright/test';
import fs from 'node:fs/promises';

async function gameState(frame) {
    return frame.evaluate(async () => {
        const game = await import('./pkg/myva_web_game.js');
        return JSON.parse(game.telemetry());
    });
}

async function enter(page) {
    await page.locator('#enter-game').click();
    await expect(page.locator('#shell-status')).toContainText(/sẵn sàng|Đánh boss/);
    const frame = page.frames().find(item => item.url().includes('/game/?'));
    expect(frame).toBeTruthy();
    await expect.poll(async () => (await gameState(frame)).paused).toBe(false);
    return frame;
}

test('bridge rejects stale messages; Tab, fullscreen and synthetic visibility preserve ownership', async ({ page, browser }) => {
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.goto('/');
    let frame = await enter(page);
    let canvas = frame.locator('#game-canvas');

    // A browser-level Tab press catches winit blanket preventDefault regressions.
    await canvas.focus();
    await page.keyboard.press('Tab');
    await expect(page.locator('#chat')).toBeFocused();
    await canvas.focus();
    await page.keyboard.press('Shift+Tab');
    await expect(page.locator('#exit-game')).toBeFocused();

    const fullscreenEnabled = await page.evaluate(() => document.fullscreenEnabled);
    await page.locator('#fullscreen-game').click();
    if (fullscreenEnabled) {
        await expect.poll(() => page.evaluate(() => document.fullscreenElement?.id)).toBe('game-frame');
        await page.evaluate(() => document.exitFullscreen());
        await expect.poll(() => page.evaluate(() => document.fullscreenElement === null)).toBe(true);
    } else {
        await expect(page.locator('#shell-status')).toContainText(/chưa hỗ trợ toàn màn hình|Không thể bật toàn màn hình/);
    }

    const parentRejected = await page.evaluate(() => {
        const frame = document.querySelector('#game-frame');
        const session = new URL(frame.src).searchParams.get('session');
        const messages = [];
        const collect = event => messages.push(event.detail);
        window.addEventListener('myva-status', collect);
        const battle = {
            mode: 'boss', round: 1, tick: 10, phase: 1, paused: false, outcome: null, verified: null,
            verifiedTicks: 0, hash: '0123456789abcdef', autopilot: false,
            player: { hp: 424242, maxHp: 424242, mach: 0 }, rival: { hp: 2400, maxHp: 2400 },
        };
        const valid = { v: 2, session, type: 'event', battle };
        const cases = [
            { origin: 'https://invalid.example', source: frame.contentWindow, data: valid },
            { origin: location.origin, source: window, data: valid },
            { origin: location.origin, source: frame.contentWindow, data: { ...valid, session: 'old-session' } },
            { origin: location.origin, source: frame.contentWindow, data: { ...valid, v: 1 } },
            { origin: location.origin, source: frame.contentWindow, data: { ...valid, battle: { ...battle, player: { ...battle.player, hp: '424242' } } } },
            { origin: location.origin, source: frame.contentWindow, data: { ...valid, battle: { ...battle, hash: '<b>x</b>' } } },
            { origin: location.origin, source: frame.contentWindow, data: { ...valid, battle: { ...battle, outcome: 'jackpot' } } },
            { origin: location.origin, source: frame.contentWindow, data: { v: 2, session, type: 'replay', text: 'not a replay', name: 'x.myva-replay' } },
            { origin: location.origin, source: frame.contentWindow, data: null },
        ];
        for (const message of cases) window.dispatchEvent(new MessageEvent('message', message));
        const rejected = messages.slice();
        // Positive control proves these events reach the actual bridge listener.
        window.dispatchEvent(new MessageEvent('message', {
            origin: location.origin, source: frame.contentWindow, data: valid,
        }));
        window.removeEventListener('myva-status', collect);
        return { rejected, accepted: messages.at(-1), cases: cases.length };
    });
    expect(parentRejected.rejected).toEqual([]);
    expect(parentRejected.accepted).toContain('424242');

    const childRejected = await frame.evaluate(async () => {
        const game = await import('./pkg/myva_web_game.js');
        const session = new URLSearchParams(location.search).get('session');
        const valid = { v: 2, session, type: 'pause', paused: true };
        const rematch = { v: 2, session, type: 'command', command: 'rematch' };
        const before = JSON.parse(game.telemetry());
        const cases = [
            { origin: 'https://invalid.example', source: parent, data: valid },
            { origin: location.origin, source: window, data: valid },
            { origin: location.origin, source: parent, data: { ...valid, session: 'old-session' } },
            { origin: location.origin, source: parent, data: { ...valid, v: 1 } },
            { origin: location.origin, source: parent, data: { ...valid, paused: 'true' } },
            { origin: location.origin, source: parent, data: { ...valid, type: 'unknown' } },
            { origin: 'https://invalid.example', source: parent, data: rematch },
            { origin: location.origin, source: parent, data: { ...rematch, session: 'old-session' } },
            { origin: location.origin, source: parent, data: { ...rematch, command: 'grant-reward' } },
            { origin: location.origin, source: parent, data: { ...rematch, command: 'autopilot', value: 'yes' } },
        ];
        for (const message of cases) window.dispatchEvent(new MessageEvent('message', message));
        // Lệnh hợp lệ chỉ áp ở khung hình kế tiếp; chờ đủ lâu để lệnh lọt lưới kịp lộ ra.
        await new Promise(resolve => setTimeout(resolve, 600));
        return { before, after: JSON.parse(game.telemetry()), cases: cases.length };
    });
    expect(childRejected.after.paused).toBe(false);
    expect(childRejected.after.round).toBe(childRejected.before.round);
    expect(childRejected.after.autopilot).toBe(false);
    expect(childRejected.after.tick).toBeGreaterThan(childRejected.before.tick);

    // These are synthetic visibility events: they verify wiring and manual-pause
    // precedence, and are explicitly not evidence of OS/background throttling.
    await page.evaluate(() => {
        Object.defineProperty(document, 'hidden', { configurable: true, value: true });
        document.dispatchEvent(new Event('visibilitychange'));
    });
    await expect.poll(async () => (await gameState(frame)).paused).toBe(true);
    await page.locator('#pause-game').click();
    await page.evaluate(() => {
        delete document.hidden;
        document.dispatchEvent(new Event('visibilitychange'));
    });
    // Chờ lệnh pause/resume xếp hàng được áp rồi mới chụp trạng thái tạm dừng.
    await page.waitForTimeout(600);
    const manualPause = await gameState(frame);
    expect(manualPause.paused).toBe(true);
    // Allow queued postMessages and an observation interval to run: a late
    // visibility message must not undo the user's explicit pause.
    await page.waitForTimeout(600);
    expect((await gameState(frame)).tick).toBe(manualPause.tick);
    expect((await gameState(frame)).paused).toBe(true);
    await page.locator('#pause-game').click();
    await expect.poll(async () => (await gameState(frame)).paused).toBe(false);

    await frame.evaluate(() => {
        Object.defineProperty(document, 'hidden', { configurable: true, value: true });
        document.dispatchEvent(new Event('visibilitychange'));
    });
    await expect.poll(async () => (await gameState(frame)).paused).toBe(true);
    await frame.evaluate(() => {
        delete document.hidden;
        document.dispatchEvent(new Event('visibilitychange'));
    });
    await expect.poll(async () => (await gameState(frame)).paused).toBe(false);

    // Keep the old WindowProxy to simulate a delayed message after navigation.
    await page.evaluate(() => {
        const frame = document.querySelector('#game-frame');
        window.__oldGameWindow = frame.contentWindow;
        window.__oldGameSession = new URL(frame.src).searchParams.get('session');
    });
    await page.locator('#exit-game').click();
    frame = await enter(page);
    canvas = frame.locator('#game-canvas');
    await expect(canvas).toBeVisible();
    const staleMessages = await page.evaluate(() => {
        const frame = document.querySelector('#game-frame');
        const session = new URL(frame.src).searchParams.get('session');
        const messages = [];
        const collect = event => messages.push(event.detail);
        window.addEventListener('myva-status', collect);
        for (const oldSession of [window.__oldGameSession, session]) {
            window.dispatchEvent(new MessageEvent('message', {
                origin: location.origin, source: window.__oldGameWindow,
                data: { v: 2, session: oldSession, type: 'error', message: 'STALE RUNTIME' },
            }));
        }
        window.removeEventListener('myva-status', collect);
        delete window.__oldGameWindow;
        delete window.__oldGameSession;
        return messages;
    });
    expect(staleMessages).toEqual([]);
    expect((await gameState(frame)).paused).toBe(false);
    await page.locator('#exit-game').click();

    await fs.mkdir('target/web-evidence', { recursive: true });
    await fs.writeFile('target/web-evidence/protocol.json', JSON.stringify({
        date: new Date().toISOString(), browser: browser.version(), fullscreenEnabled,
        parentRejected, childRejected, staleMessages, errors,
        visibility: 'Synthetic document.hidden overrides and visibilitychange events; actual background-tab/device behavior is not verified here.',
        focus: 'Real keyboard Tab and Shift+Tab leave the canvas and focus shell controls.',
    }, null, 2));
    expect(errors).toEqual([]);
});
