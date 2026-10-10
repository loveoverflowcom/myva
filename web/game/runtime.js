// Game document: tải Bevy WASM, nối vòng đời với shell qua postMessage có version/session.
// Bevy (winit) tự đọc bàn phím, chạm và gamepad trên canvas; file này không dịch input.
const V = 2;
const session = new URLSearchParams(location.search).get('session');
const canvas = document.querySelector('#game-canvas');
const label = document.querySelector('#game-status');
const started = performance.now();
let game;
let exited = false;
let failed = false;
let shellPaused = false;
let initialized = false;
let timer;
const lifetime = new AbortController();
const options = { signal: lifetime.signal };
const metrics = window.__myvaMetrics = { startupMs: null, samples: [], errors: [], inputSamples: [] };
// Phím ra hành động: đo từ keydown DOM tới lúc tick mô phỏng nhận khung có nút vừa nhấn.
// Không phải input-to-photon; chạm và gamepad không có mốc DOM nên không được đo ở đây.
const actionKeys = new Set(['Space', 'ShiftLeft', 'ShiftRight', 'KeyJ', 'KeyK', 'KeyQ', 'KeyE', 'KeyR']);
const pendingPresses = [];
let lastPresses = 0;

function send(type, payload = {}) {
    if (parent !== window) parent.postMessage({ v: V, session, type, ...payload }, location.origin);
}
function show(message) {
    label.textContent = message;
    label.hidden = !message;
}
function fail(message) {
    if (failed) return;
    failed = true;
    metrics.errors.push(message);
    show(message);
    game?.set_paused(true);
    send('error', { message });
}
function state() {
    return game && !failed ? JSON.parse(game.telemetry() || 'null') : null;
}
function pause() {
    const paused = failed || shellPaused || document.hidden || !initialized;
    game?.set_paused(paused);
    // Lỗi giữ nguyên thông báo; trước init nhãn vẫn là trạng thái tải.
    if (initialized && !failed) show(paused ? 'Tạm dừng' : '');
}
function dispose() {
    exited = true;
    game?.set_paused(true);
    clearInterval(timer);
    lifetime.abort();
    // The parent removes this entire document; no unsupported Bevy restart is attempted.
}
// Chỉ gửi số liệu tóm tắt; shell không nhận ECS world hay state nội bộ của lõi.
function summary(s) {
    return {
        mode: s.mode, round: s.round, tick: s.tick, phase: s.phase, paused: s.paused,
        outcome: s.outcome, verified: s.verified, verifiedTicks: s.verified_ticks, hash: s.hash,
        autopilot: s.autopilot, touch: s.touch, gamepads: s.gamepads,
        player: { hp: s.player.hp, maxHp: s.player.max_hp, mach: s.player.mach },
        rival: { hp: s.rival.hp, maxHp: s.rival.max_hp },
        tally: s.tally.player,
    };
}
async function exportReplay() {
    if (!game || failed) return;
    game.request_replay();
    for (let attempt = 0; attempt < 100 && !exited; attempt++) {
        const text = game.take_replay();
        if (text) {
            const s = state();
            send('replay', { text, name: `myva-${s.mode}-r${s.round}-t${s.tick}.myva-replay` });
            return;
        }
        await new Promise(resolve => setTimeout(resolve, 50));
    }
    send('error', { message: 'Không lấy được replay' });
}

window.addEventListener('message', event => {
    const data = event.data;
    if (event.origin !== location.origin || event.source !== parent || !data ||
        data.v !== V || data.session !== session) return;
    if (data.type === 'init') { initialized = true; pause(); }
    else if (data.type === 'pause' && typeof data.paused === 'boolean') { shellPaused = data.paused; pause(); }
    else if (data.type === 'command') {
        if (data.command === 'rematch') game?.rematch();
        else if (data.command === 'mode') game?.switch_mode();
        else if (data.command === 'hitboxes') game?.toggle_hitboxes();
        else if (data.command === 'autopilot' && typeof data.value === 'boolean') game?.set_autopilot(data.value);
        else if (data.command === 'touch' && typeof data.value === 'boolean') game?.show_touch_controls(data.value);
        else if (data.command === 'replay') exportReplay();
    }
    else if (data.type === 'exit') dispose();
}, options);
canvas.addEventListener('keydown', event => {
    if (!actionKeys.has(event.code) || event.repeat) return;
    pendingPresses.push(performance.now());
    if (pendingPresses.length > 8) pendingPresses.shift();
}, options);
// winit chỉ nhận phím khi canvas có focus; chạm/nhấp đưa focus về canvas.
canvas.addEventListener('pointerdown', () => canvas.focus(), options);
document.addEventListener('visibilitychange', pause, options);
window.addEventListener('pagehide', dispose, options);
canvas.addEventListener('webglcontextlost', event => {
    event.preventDefault();
    fail('Mất kết nối đồ họa. Hãy rời trận và mở lại');
}, options);
window.addEventListener('error', event => fail(event.message || 'Lỗi runtime'), options);
window.addEventListener('unhandledrejection', event => fail(String(event.reason)), options);

try {
    const probe = document.createElement('canvas');
    const gl = probe.getContext('webgl2');
    if (!gl) throw new Error('Trình duyệt này chưa hỗ trợ WebGL2');
    gl.getExtension('WEBGL_lose_context')?.loseContext();
    const loadedGame = await import('./pkg/myva_web_game.js');
    await loadedGame.default();
    // Export wrappers exist before WASM initializes; calling them on fetch
    // failure would throw again and hide the original error from the shell.
    game = loadedGame;
    if (!exited) {
        game.set_paused(true);
        game.start_game();
        if (matchMedia('(pointer: coarse)').matches) game.show_touch_controls(true);
        let ready = false;
        let last = performance.now();
        let updates = 0;
        timer = setInterval(() => {
            const s = state();
            if (!s) return;
            const now = performance.now();
            if (!ready && s.ready) {
                ready = true;
                metrics.startupMs = now - started;
                show('');
                send('ready', { startupMs: metrics.startupMs, renderer: 'webgl2' });
            }
            const sample = { at: now, ...s, updateHz: (s.updates - updates) * 1000 / (now - last), hidden: document.hidden };
            metrics.samples.push(sample);
            if (metrics.samples.length > 120) metrics.samples.shift();
            updates = s.updates;
            last = now;
            if (ready) send('event', { battle: summary(s) });
        }, 250);
        const observeInput = () => {
            if (exited) return;
            const s = state();
            if (s && s.presses !== lastPresses) {
                const now = performance.now();
                // Mỗi khung có nút nhấn tiêu thụ tối đa một mốc keydown còn chờ.
                for (let i = lastPresses; i < s.presses && pendingPresses.length; i++) {
                    metrics.inputSamples.push(now - pendingPresses.shift());
                }
                if (metrics.inputSamples.length > 120) metrics.inputSamples.splice(0, metrics.inputSamples.length - 120);
                lastPresses = s.presses;
            }
            while (pendingPresses.length && performance.now() - pendingPresses[0] > 2000) pendingPresses.shift();
            requestAnimationFrame(observeInput);
        };
        requestAnimationFrame(observeInput);
    }
} catch (error) { fail(error.message || String(error)); }
