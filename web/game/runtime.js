const V = 1;
const session = new URLSearchParams(location.search).get('session');
const canvas = document.querySelector('#game-canvas');
const label = document.querySelector('#game-status');
const started = performance.now();
let game;
let exited = false;
let failed = false;
let shellPaused = false;
let initialized = false;
let pointer = null;
let origin = null;
let pointerVector = [0, 0];
let timer;
const keys = new Set();
const inputKeys = new Set(['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'KeyW', 'KeyA', 'KeyS', 'KeyD']);
const lifetime = new AbortController();
const options = { signal: lifetime.signal };
const metrics = window.__myvaMetrics = { startupMs: null, samples: [], errors: [], inputSamples: [] };
let pendingInput = null;

function send(type, payload = {}) {
    if (parent !== window) parent.postMessage({ v: V, session, type, ...payload }, location.origin);
}
function fail(message) {
    if (failed) return;
    failed = true;
    metrics.errors.push(message);
    label.textContent = message;
    game?.set_paused(true);
    send('error', { message });
}
function input() {
    if (!game || failed) return;
    const x = Number(keys.has('ArrowRight') || keys.has('KeyD')) - Number(keys.has('ArrowLeft') || keys.has('KeyA')) + pointerVector[0];
    const y = Number(keys.has('ArrowUp') || keys.has('KeyW')) - Number(keys.has('ArrowDown') || keys.has('KeyS')) + pointerVector[1];
    if ((x || y) && !pendingInput) pendingInput = { at: performance.now(), before: JSON.parse(game.telemetry()) };
    game.set_input(x, y);
}
function clearInput() {
    keys.clear();
    if (pointer !== null && canvas.hasPointerCapture(pointer)) canvas.releasePointerCapture(pointer);
    pointer = null;
    origin = null;
    pointerVector = [0, 0];
    game?.set_input(0, 0);
    pendingInput = null;
}
function pause() { clearInput(); game?.set_paused(failed || shellPaused || document.hidden || !initialized); }
function dispose() {
    exited = true;
    clearInput();
    game?.set_paused(true);
    clearInterval(timer);
    lifetime.abort();
    // The parent removes this entire document; no unsupported Bevy restart is attempted.
}

window.addEventListener('message', event => {
    const data = event.data;
    if (event.origin !== location.origin || event.source !== parent || !data ||
        data.v !== V || data.session !== session) return;
    if (data.type === 'init') { initialized = true; pause(); }
    else if (data.type === 'pause' && typeof data.paused === 'boolean') { shellPaused = data.paused; pause(); }
    else if (data.type === 'command' && data.command === 'reset') { clearInput(); game?.reset_game(); }
    else if (data.type === 'exit') dispose();
}, options);
canvas.addEventListener('keydown', event => {
    if (!inputKeys.has(event.code)) return;
    event.preventDefault();
    keys.add(event.code);
    input();
}, options);
canvas.addEventListener('keyup', event => {
    if (!inputKeys.has(event.code)) return;
    event.preventDefault();
    keys.delete(event.code);
    input();
}, options);
canvas.addEventListener('pointerdown', event => {
    if (pointer !== null || (event.pointerType === 'mouse' && event.button !== 0)) return;
    event.preventDefault();
    canvas.focus();
    pointer = event.pointerId;
    origin = [event.clientX, event.clientY];
    canvas.setPointerCapture(pointer);
}, options);
canvas.addEventListener('pointermove', event => {
    if (event.pointerId !== pointer) return;
    pointerVector = [(event.clientX - origin[0]) / 48, (origin[1] - event.clientY) / 48];
    input();
}, options);
for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) {
    canvas.addEventListener(type, event => { if (event.pointerId === pointer) clearInput(); }, options);
}
canvas.addEventListener('blur', clearInput, options);
window.addEventListener('blur', clearInput, options);
document.addEventListener('visibilitychange', pause, options);
window.addEventListener('pagehide', dispose, options);
canvas.addEventListener('webglcontextlost', event => {
    event.preventDefault();
    fail('Mất kết nối đồ họa. Hãy rời bãi tập và mở lại');
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
        let ready = false;
        let last = performance.now();
        let frames = 0;
        timer = setInterval(() => {
            if (failed) return;
            const state = JSON.parse(game.telemetry());
            const now = performance.now();
            if (!ready && state.ready) {
                ready = true;
                metrics.startupMs = now - started;
                label.textContent = 'Chạm / WASD để di chuyển';
                send('ready', { startupMs: metrics.startupMs, renderer: 'webgl2' });
            }
            const sample = { at: now, ...state, updateHz: (state.updates - frames) * 1000 / (now - last), hidden: document.hidden };
            metrics.samples.push(sample);
            if (metrics.samples.length > 120) metrics.samples.shift();
            frames = state.updates;
            last = now;
            label.textContent = state.paused ? 'Tạm dừng' : `Linh lực ${state.score} · Chạm / WASD`;
            if (ready) send('event', { score: state.score, paused: state.paused });
        }, 500);
        // Input-to-state latency sampled on animation frames, not physical display latency.
        const observeInput = () => {
            if (exited) return;
            if (pendingInput) {
                const state = JSON.parse(game.telemetry());
                if (state.x !== pendingInput.before.x || state.y !== pendingInput.before.y) {
                    metrics.inputSamples.push(performance.now() - pendingInput.at);
                    if (metrics.inputSamples.length > 120) metrics.inputSamples.shift();
                    pendingInput = null;
                }
            }
            requestAnimationFrame(observeInput);
        };
        requestAnimationFrame(observeInput);
    }
} catch (error) { fail(error.message || String(error)); }
