const VERSION = 1;
let frame = null;
let session = null;
let deadline = null;
let manuallyPaused = false;
let listener = null;

function status(message) {
    window.dispatchEvent(new CustomEvent('myva-status', { detail: message }));
}
function send(type, payload = {}) {
    frame?.contentWindow?.postMessage({ v: VERSION, session, type, ...payload }, location.origin);
}
function pauseState() {
    send('pause', { paused: manuallyPaused || document.hidden });
}
export function mountGame() {
    exitGame();
    session = crypto.randomUUID();
    manuallyPaused = false;
    frame = document.createElement('iframe');
    frame.title = 'MyVa — bãi tập';
    frame.id = 'game-frame';
    frame.allow = 'fullscreen';
    frame.src = new URL(`game/?session=${encodeURIComponent(session)}`, document.baseURI).href;
    listener = (event) => {
        const data = event.data;
        if (event.origin !== location.origin || event.source !== frame?.contentWindow ||
            !data || data.v !== VERSION || data.session !== session) return;
        if (data.type === 'ready') {
            clearTimeout(deadline);
            status('Bãi tập đã sẵn sàng. Chạm hoặc nhấp để bắt đầu.');
            send('init');
            pauseState();
        } else if (data.type === 'event' && Number.isSafeInteger(data.score)) {
            status(`Linh lực: ${data.score} · ${data.paused ? 'Đang tạm dừng' : 'Đang chơi'}`);
        } else if (data.type === 'error' && typeof data.message === 'string') {
            clearTimeout(deadline);
            status(`Không thể mở bãi tập: ${data.message}. Rời bãi tập rồi thử lại.`);
        }
    };
    window.addEventListener('message', listener);
    document.addEventListener('visibilitychange', pauseState);
    document.querySelector('#game-host').replaceChildren(frame);
    status('Đang tải bãi tập… Lần đầu có thể mất một lúc.');
    deadline = setTimeout(() => status('Tải quá lâu. Kiểm tra kết nối rồi rời bãi tập và thử lại.'), 90000);
}
export function exitGame() {
    send('exit');
    // Removing the browsing context is the teardown boundary, independent of App::run.
    frame?.remove();
    frame = null;
    session = null;
    clearTimeout(deadline);
    deadline = null;
    if (listener) window.removeEventListener('message', listener);
    listener = null;
    document.removeEventListener('visibilitychange', pauseState);
    status('Đã rời bãi tập.');
}
export function pauseGame(value) { manuallyPaused = value; pauseState(); }
export function resetGame() { send('command', { command: 'reset' }); }
export function fullscreenGame() {
    if (!frame?.requestFullscreen) { status('Trình duyệt chưa hỗ trợ toàn màn hình.'); return; }
    frame.requestFullscreen().catch(() => status('Không thể bật toàn màn hình trên trình duyệt này.'));
}
window.addEventListener('pagehide', exitGame);
