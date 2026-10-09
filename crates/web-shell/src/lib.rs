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
        #[wasm_bindgen(js_name = resetGame)]
        fn reset_game();
        #[wasm_bindgen(js_name = fullscreenGame)]
        fn fullscreen_game();
    }

    #[component]
    fn Shell() -> impl IntoView {
        let (playing, set_playing) = signal(false);
        let (paused, set_paused) = signal(false);
        let (status, set_status) = signal("Sẵn sàng khám phá.".to_owned());
        let (name, set_name) = signal(String::new());
        let listener = window_event_listener(
            ev::Custom::<web_sys::CustomEvent>::new("myva-status"),
            move |event| {
                if let Some(message) = event.detail().as_string() {
                    set_status.set(message);
                }
            },
        );
        on_cleanup(move || {
            listener.remove();
            exit_game();
        });

        view! {
            <header>
                <a class="brand" href="./">"MYVA "<span>"/ THẦN MẠCH"</span></a>
                <span class="badge">"Bản thử web · D03"</span>
            </header>
            <main>
                <section class="intro" class:hidden=move || playing.get()>
                    <p class="eyebrow">"VÂN THỦY · BÃI TẬP"</p>
                    <h1>"Đánh thức thần mạch."</h1>
                    <p>"Di chuyển qua bãi tập, chạm vào những đốm sáng và thu thập linh lực. Một cảnh nhỏ để thử cách chơi trên trình duyệt."</p>
                    <label for="nickname">"Tên người chơi"</label>
                    <input id="nickname" maxlength="32" placeholder="Khách Vân Thủy" autocomplete="off"
                        on:input=move |event| set_name.set(event_target_value(&event)) />
                    <p class="muted">"Chơi thử tại máy. Chưa có tài khoản hoặc lưu tiến trình."</p>
                    <button id="enter-game" class="primary" on:click=move |_| {
                        set_playing.set(true);
                        set_paused.set(false);
                        mount_game();
                    }>"Vào bãi tập →"</button>
                </section>
                <section class="play" class:hidden=move || !playing.get()>
                    <div class="toolbar">
                        <div><p class="eyebrow">"BÃI TẬP"</p><h1>{move || {
                            let current = name.get();
                            if current.trim().is_empty() { "Khách Vân Thủy".to_owned() } else { current }
                        }}</h1></div>
                        <div class="actions">
                            <button id="pause-game" on:click=move |_| {
                                let value = !paused.get();
                                set_paused.set(value);
                                pause_game(value);
                            }>{move || if paused.get() { "Tiếp tục" } else { "Tạm dừng" }}</button>
                            <button id="reset-game" on:click=move |_| reset_game()>"Chơi lại"</button>
                            <button id="fullscreen-game" on:click=move |_| fullscreen_game()>"Toàn màn hình"</button>
                            <button id="exit-game" on:click=move |_| {
                                exit_game();
                                set_playing.set(false);
                            }>"Rời bãi tập"</button>
                        </div>
                    </div>
                    <div id="game-host" aria-label="Bãi tập tương tác"></div>
                    <p class="controls">"Nhấp vào bãi tập rồi dùng WASD / phím mũi tên. Trên màn hình cảm ứng, kéo từ điểm chạm để di chuyển. Nhấn Tab để trở về nút điều hướng."</p>
                    <label for="chat">"Thử nhập tiếng Việt"</label>
                    <input id="chat" placeholder="Nhập ở đây để kiểm tra bàn phím…" maxlength="120" />
                </section>
                <p id="shell-status" role="status" aria-live="polite">{move || status.get()}</p>
            </main>
            <footer>"MyVa — Thần Mạch · Prototype, chưa phải gameplay thành phẩm."</footer>
        }
    }

    #[wasm_bindgen(start)]
    pub fn start() {
        leptos::mount::mount_to_body(Shell);
    }
}
