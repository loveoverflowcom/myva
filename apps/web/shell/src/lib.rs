//! The shell owns DOM, navigation and text input. The game gets its own document.
#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::{ev, prelude::*};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/web/shell-bridge.js")]
    extern "C" {
        #[wasm_bindgen(js_name = mountGame)]
        fn mount_game();
        #[wasm_bindgen(js_name = exitGame)]
        fn exit_game();
        #[wasm_bindgen(js_name = pauseGame)]
        fn pause_game(paused: bool);
        #[wasm_bindgen(js_name = gameCommand)]
        fn game_command(command: &str);
        #[wasm_bindgen(js_name = setAutopilot)]
        fn set_autopilot(on: bool);
        #[wasm_bindgen(js_name = fullscreenGame)]
        fn fullscreen_game();
    }

    #[component]
    fn Shell() -> impl IntoView {
        let (playing, set_playing) = signal(false);
        let (paused, set_paused) = signal(false);
        let (autopilot, set_autopilot_signal) = signal(false);
        let (finished, set_finished) = signal(false);
        let (status, set_status) = signal("Sẵn sàng vào trận.".to_owned());
        let (name, set_name) = signal(String::new());
        let status_listener = window_event_listener(
            ev::Custom::<web_sys::CustomEvent>::new("myva-status"),
            move |event| {
                if let Some(message) = event.detail().as_string() {
                    set_status.set(message);
                }
            },
        );
        let outcome_listener = window_event_listener(
            ev::Custom::<web_sys::CustomEvent>::new("myva-outcome"),
            move |event| {
                let outcome = event.detail().as_string().unwrap_or_default();
                set_finished.set(!outcome.is_empty());
            },
        );
        on_cleanup(move || {
            status_listener.remove();
            outcome_listener.remove();
            exit_game();
        });
        let restart = move || {
            set_paused.set(false);
            set_autopilot_signal.set(false);
            set_finished.set(false);
        };

        view! {
            <header>
                <a class="brand" href="./">"MYVA "<span>"/ THẦN MẠCH"</span></a>
                <span class="badge">"Graybox chiến đấu · #2"</span>
            </header>
            <main>
                <section class="intro" class:hidden=move || playing.get()>
                    <p class="eyebrow">"VÂN THỦY · ĐẬP NƯỚC CŨ"</p>
                    <h1>"Đánh bại Kẻ Giữ Đập."</h1>
                    <p>"Điều khiển Long Lưu trong đấu trường graybox. Vùng vàng báo trước nơi đòn boss sắp đánh; né, đỡ hoặc nhảy đúng lúc rồi phản công. Boss đổi pha ở 70% và 35% sinh lực."</p>
                    <label for="nickname">"Tên người chơi"</label>
                    <input id="nickname" maxlength="32" placeholder="Khách Vân Thủy" autocomplete="off"
                        on:input=move |event| set_name.set(event_target_value(&event)) />
                    <p class="muted">"Chơi thử tại máy, chưa có tài khoản hay lưu tiến trình. Hình khối là placeholder, chưa phải mỹ thuật thành phẩm."</p>
                    <button id="enter-game" class="primary" on:click=move |_| {
                        set_playing.set(true);
                        restart();
                        mount_game();
                    }>"Vào trận →"</button>
                </section>
                <section class="play" class:hidden=move || !playing.get()>
                    <div class="toolbar">
                        <div><p class="eyebrow">"ĐẤU TRƯỜNG GRAYBOX"</p><h1>{move || {
                            let current = name.get();
                            if current.trim().is_empty() { "Khách Vân Thủy".to_owned() } else { current }
                        }}</h1></div>
                        <div class="actions">
                            <button id="pause-game" on:click=move |_| {
                                let value = !paused.get();
                                set_paused.set(value);
                                pause_game(value);
                            }>{move || if paused.get() { "Tiếp tục" } else { "Tạm dừng" }}</button>
                            <button id="rematch-game" class:primary=move || finished.get() on:click=move |_| {
                                set_finished.set(false);
                                game_command("rematch");
                            }>"Đấu lại"</button>
                            <button id="mode-game" on:click=move |_| {
                                set_finished.set(false);
                                game_command("mode");
                            }>"Đổi chế độ"</button>
                            <button id="autopilot-game" aria-pressed=move || autopilot.get().to_string() on:click=move |_| {
                                let value = !autopilot.get();
                                set_autopilot_signal.set(value);
                                set_autopilot(value);
                            }>{move || if autopilot.get() { "Tự điều khiển" } else { "Xem bot đánh mẫu" }}</button>
                            <button id="hitbox-game" on:click=move |_| game_command("hitboxes")>"Hitbox"</button>
                            <button id="replay-game" on:click=move |_| game_command("replay")>"Tải replay"</button>
                            <button id="fullscreen-game" on:click=move |_| fullscreen_game()>"Toàn màn hình"</button>
                            <button id="exit-game" on:click=move |_| {
                                exit_game();
                                set_playing.set(false);
                            }>"Rời trận"</button>
                        </div>
                    </div>
                    <div id="game-host" aria-label="Đấu trường tương tác"></div>
                    <div class="controls">
                        <p><strong>"Bàn phím"</strong>" (nhấp vào đấu trường trước): A/D hoặc ←/→ di chuyển · Space nhảy · Shift lướt · giữ L đỡ · J/K đòn nhẹ/nặng · Q/E/R thuật Lưu Tiễn, Hồi Thế, Triều Dâng · Enter đấu lại · M đổi chế độ · H hitbox."</p>
                        <p><strong>"Gamepad"</strong>": stick trái/D-pad di chuyển · A nhảy · B lướt · giữ LB đỡ · X/Y nhẹ/nặng · giữ RB + X/Y/B thuật 1/2/3 · Start đấu lại sau khi kết thúc."</p>
                        <p><strong>"Cảm ứng"</strong>": kéo ở nửa trái để di chuyển; cụm nút bên phải cho đòn, thuật, nhảy, lướt và đỡ (giữ). Màn hình ngang dễ chơi hơn."</p>
                        <p>"Tab để trở về các nút điều hướng. Replay ghi mọi input; kiểm tra file bằng "<code>"myva-replay verify"</code>"."</p>
                    </div>
                    <label for="chat">"Thử nhập tiếng Việt"</label>
                    <input id="chat" placeholder="Nhập ở đây để kiểm tra bàn phím…" maxlength="120" />
                </section>
                <p id="shell-status" role="status" aria-live="polite">{move || status.get()}</p>
            </main>
            <footer>"MyVa — Thần Mạch · Graybox, chưa phải gameplay thành phẩm."</footer>
        }
    }

    #[wasm_bindgen(start)]
    pub fn start() {
        leptos::mount::mount_to_body(Shell);
    }
}
