// v2: event mang tóm tắt trận thay cho điểm của spike; thêm lệnh trận và message replay.
const VERSION = 2;
// Lõi ghi bản 2 (thêm seed, phe, NPC); bản 1 vẫn đọc được nên shell nhận cả hai.
const REPLAY_HEADER = /^myva-replay [12]\n/;
let frame = null;
let session = null;
let deadline = null;
let manuallyPaused = false;
let listener = null;
let lastOutcome = null;

function status(message) {
    window.dispatchEvent(new CustomEvent('myva-status', { detail: message }));
}
function send(type, payload = {}) {
    frame?.contentWindow?.postMessage({ v: VERSION, session, type, ...payload }, location.origin);
}
function pauseState() {
    send('pause', { paused: manuallyPaused || document.hidden });
}
const count = value => Number.isSafeInteger(value) && value >= 0;
const MODES = { boss: 'Đánh boss', duel: 'Đấu tập' };
const OUTCOMES = { victory: 'Thắng', defeat: 'Thua', draw: 'Hòa' };

// Chỉ nhận đúng kiểu dữ liệu đã thỏa thuận; giá trị lạ bị bỏ qua thay vì hiển thị.
function validBattle(b) {
    return b && typeof b === 'object' && b.mode in MODES && count(b.round) && count(b.tick) &&
        count(b.phase) && b.phase <= 3 && typeof b.paused === 'boolean' &&
        (b.outcome === null || b.outcome in OUTCOMES) && (b.verified === null || typeof b.verified === 'boolean') &&
        count(b.verifiedTicks) && typeof b.hash === 'string' && /^[0-9a-f]{16}$/.test(b.hash) &&
        b.player && count(b.player.hp) && count(b.player.maxHp) && count(b.player.mach) &&
        b.rival && count(b.rival.hp) && count(b.rival.maxHp);
}
function describe(b) {
    const mode = MODES[b.mode];
    if (b.outcome) {
        const verdict = b.verified === null ? 'đang kiểm chứng replay…'
            : b.verified ? `replay đã kiểm chứng (${b.verifiedTicks} tick, hash ${b.hash})`
                : 'replay LỆCH, hãy gửi file replay để kiểm tra';
        return `${OUTCOMES[b.outcome]} · ${mode} vòng ${b.round} · ${verdict}. Nhấn “Đấu lại” để chơi tiếp.`;
    }
    const rival = b.mode === 'boss' ? `Kẻ Giữ Đập ${b.rival.hp}/${b.rival.maxHp} · pha ${b.phase}` : `Bot ${b.rival.hp}/${b.rival.maxHp}`;
    const pilot = b.autopilot === true ? ' · bot B1 đang lái' : '';
    return `${mode} · Long Lưu ${b.player.hp}/${b.player.maxHp} · Mạch ${b.player.mach} · ${rival}${b.paused ? ' · Đang tạm dừng' : ''}${pilot}`;
}
function download(text, name) {
    const url = URL.createObjectURL(new Blob([text], { type: 'text/plain;charset=utf-8' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = name;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function mountGame() {
    exitGame();
    session = crypto.randomUUID();
    manuallyPaused = false;
    lastOutcome = null;
    frame = document.createElement('iframe');
    frame.title = 'MyVa — trận graybox';
    frame.id = 'game-frame';
    frame.allow = 'fullscreen; gamepad';
    frame.src = new URL(`game/?session=${encodeURIComponent(session)}`, document.baseURI).href;
    listener = (event) => {
        const data = event.data;
        if (event.origin !== location.origin || event.source !== frame?.contentWindow ||
            !data || data.v !== VERSION || data.session !== session) return;
        if (data.type === 'ready') {
            clearTimeout(deadline);
            status('Trận đã sẵn sàng. Nhấp vào đấu trường hoặc chạm để điều khiển.');
            send('init');
            pauseState();
        } else if (data.type === 'event' && validBattle(data.battle)) {
            const battle = data.battle;
            status(describe(battle));
            if (battle.outcome !== lastOutcome) {
                lastOutcome = battle.outcome;
                window.dispatchEvent(new CustomEvent('myva-outcome', { detail: battle.outcome ?? '' }));
            }
        } else if (data.type === 'replay' && typeof data.text === 'string' && REPLAY_HEADER.test(data.text) &&
            typeof data.name === 'string' && /^[a-z0-9-]+\.myva-replay$/.test(data.name)) {
            download(data.text, data.name);
            status(`Đã tải ${data.name}. Kiểm tra bằng: myva-replay verify ${data.name}`);
        } else if (data.type === 'error' && typeof data.message === 'string') {
            clearTimeout(deadline);
            status(`Không thể mở trận: ${data.message}. Rời trận rồi thử lại.`);
        }
    };
    window.addEventListener('message', listener);
    document.addEventListener('visibilitychange', pauseState);
    document.querySelector('#game-host').replaceChildren(frame);
    status('Đang tải trận… Lần đầu có thể mất một lúc.');
    deadline = setTimeout(() => status('Tải quá lâu. Kiểm tra kết nối rồi rời trận và thử lại.'), 90000);
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
    status('Đã rời trận.');
}
export function pauseGame(value) { manuallyPaused = value; pauseState(); }
export function gameCommand(command) { send('command', { command }); }
export function setAutopilot(value) { send('command', { command: 'autopilot', value }); }
export function fullscreenGame() {
    if (!frame?.requestFullscreen) { status('Trình duyệt chưa hỗ trợ toàn màn hình.'); return; }
    frame.requestFullscreen().catch(() => status('Không thể bật toàn màn hình trên trình duyệt này.'));
}
window.addEventListener('pagehide', exitGame);
