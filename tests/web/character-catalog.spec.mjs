import { test, expect } from '@playwright/test';

test('catalog selection forwards an exact pin, locks the frame and exports replay v4', async ({ page }) => {
    await page.goto('/');
    const choice = page.locator('#kit-choice');
    await expect(choice.locator('option')).toHaveCount(1);
    await expect(choice).toHaveValue('long-luu');
    await expect(page.locator('.character-card')).toContainText('Fixture chưa ánh xạ');
    await expect(page.locator('.character-card')).toContainText('Icon và mỹ thuật placeholder');
    await expect(page.locator('.character-card code')).toHaveText('long-luu');
    await page.locator('#enter-game').click();
    await expect(page.locator('#shell-status')).toContainText(/sẵn sàng|Đánh boss/);
    const frame = page.frames().find(f => f.url().includes('/game/?'));
    const url = new URL(frame.url());
    expect(url.searchParams.get('kit')).toBe('long-luu');
    expect(url.searchParams.get('version')).toBe('1');
    expect(url.searchParams.get('hash')).toBe('a624f529d63112f0');
    await expect.poll(() => frame.evaluate(async () => JSON.parse((await import('./pkg/myva_web_game.js')).telemetry()).player.kit_id)).toBe('long-luu');
    const rejected = await frame.evaluate(async () => {
        const game = await import('./pkg/myva_web_game.js');
        try { game.start_game_with_kit('long-luu', 1, 'a624f529d63112f0'); return false; }
        catch (error) { return String(error).includes('already started'); }
    });
    expect(rejected).toBe(true);
    const download = page.waitForEvent('download');
    await page.locator('#replay-game').click();
    const replay = await download;
    const stream = await replay.createReadStream();
    const chunks = [];
    for await (const chunk of stream) chunks.push(chunk);
    const text = Buffer.concat(chunks).toString();
    expect(text).toMatch(/^myva-replay 4\n/);
    expect(text).toContain('kit long-luu 1 a624f529d63112f0');
    await page.locator('#exit-game').click();
    await expect(choice).toHaveValue('long-luu');
});

test('unknown, mismatched or incomplete startup selections fail explicitly', async ({ page }) => {
    for (const query of [
        'kit=unknown&version=1&hash=a624f529d63112f0',
        'kit=long-luu&version=99&hash=a624f529d63112f0',
        'kit=long-luu&version=1&hash=0000000000000000',
        'kit=long-luu',
    ]) {
        await page.goto(`/game/?session=invalid-selection&${query}`);
        await expect(page.locator('#game-status')).toContainText(/UnknownKit|VersionMismatch|HashMismatch|Incomplete/);
        const started = await page.evaluate(async () => (await import('./pkg/myva_web_game.js')).telemetry());
        expect(started).toBe('');
    }
});
