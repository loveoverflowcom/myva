//! The shell owns DOM, navigation and text input. The game gets its own document.
use myva_sim::catalog::{KitId, ReviewStatus};
use myva_sim::character::{CharacterCatalog, CharacterError, CharacterIcon, Inheritance};
use myva_sim::registry::{KitRequest, KitRevision};

#[derive(Clone, Debug)]
pub struct CharacterChoice {
    pub id: KitId,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub inheritance: String,
    pub provenance: String,
    pub skills: [&'static str; 3],
    pub review: &'static str,
    pub revision: Option<KitRevision>,
    pub disabled_reason: Option<String>,
}

/// Shared Rust catalog, also exercised in native tests; no JS copy of roster/kit parameters.
pub fn character_choices() -> Result<Vec<CharacterChoice>, CharacterError> {
    let catalog = CharacterCatalog::baseline()?;
    catalog
        .registry()
        .catalog()
        .kits
        .iter()
        .map(|record| {
            let view = catalog.lookup(record.id.as_str())?;
            let CharacterIcon::Placeholder(icon) = view.details.icon;
            let revision = view.implementation.map(|e| e.revision);
            let disabled_reason = match revision {
                None => Some("Kit chưa triển khai; chưa được chọn chơi".to_owned()),
                Some(revision) => catalog
                    .select(KitRequest {
                        id: record.id.as_str(),
                        revision,
                    })
                    .err()
                    .map(|e| e.to_string()),
            };
            let inheritance = match view.inheritance {
                Inheritance::UnmappedFixture => view.inheritance.label().to_owned(),
                Inheritance::Mapped(h) => {
                    let domain = catalog.registry().catalog();
                    let labels = [
                        domain
                            .factions
                            .iter()
                            .find(|r| r.id == h.faction)
                            .map(|r| r.metadata.display_name),
                        domain
                            .sources
                            .iter()
                            .find(|r| r.id == h.source)
                            .map(|r| r.metadata.display_name),
                        domain
                            .lineages
                            .iter()
                            .find(|r| r.id == h.lineage)
                            .map(|r| r.metadata.display_name),
                        domain
                            .branches
                            .iter()
                            .find(|r| r.id == h.branch)
                            .map(|r| r.metadata.display_name),
                    ];
                    labels
                        .into_iter()
                        .map(|label| label.unwrap_or("metadata unresolved"))
                        .collect::<Vec<_>>()
                        .join(" → ")
                }
            };
            Ok(CharacterChoice {
                id: record.id,
                name: record.metadata.display_name,
                description: view.details.description,
                icon,
                inheritance,
                provenance: record
                    .metadata
                    .provenance
                    .iter()
                    .map(|p| p.reference)
                    .collect::<Vec<_>>()
                    .join(" · "),
                skills: view.details.skills,
                review: match record.metadata.review {
                    ReviewStatus::Proposed => "Chưa duyệt lore",
                    ReviewStatus::Approved(_) => "Metadata đã duyệt",
                },
                revision,
                disabled_reason,
            })
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use leptos::{ev, prelude::*};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/web/shell-bridge.js")]
    extern "C" {
        #[wasm_bindgen(js_name = mountGame)]
        fn mount_game(kit: &str, version: u32, config_hash: &str, display_name: &str);
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
    fn CharacterCard(choice: CharacterChoice) -> impl IntoView {
        let pin = choice
            .revision
            .map(|r| format!("Fixture v{}", r.version))
            .unwrap_or_else(|| "Chưa có implementation".into());
        view! {
            <article class="character-card" aria-label=format!("Kit {}", choice.name)>
                <div class="character-heading">
                    <span class="character-icon" aria-label="Icon placeholder">{choice.icon}</span>
                    <div><h2>{choice.name}</h2><p class="character-id"><code>{choice.id.as_str()}</code>" · "{pin}</p></div>
                </div>
                <p>{choice.description}</p>
                <p class="catalog-status">{choice.inheritance}</p>
                <p>{choice.review}" · Icon và mỹ thuật placeholder"</p>
                <p>"Thuật: "{choice.skills.join(" · ")}</p>
                <details><summary>"Nguồn metadata"</summary><p class="catalog-source">{choice.provenance}</p></details>
                {choice.disabled_reason.map(|reason| view! { <p role="status">{reason}</p> })}
            </article>
        }
    }

    #[component]
    fn Shell() -> impl IntoView {
        let choices_result = character_choices();
        let catalog_error = choices_result.as_ref().err().map(ToString::to_string);
        let choices = choices_result.unwrap_or_default();
        let card_choices = choices.clone();
        let enter_choices = choices.clone();
        let disabled_choices = choices.clone();
        let (selected, set_selected) = signal(KitId::LONG_LUU.as_str().to_owned());
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
                    <p>"Chọn kit đã triển khai để vào đấu trường graybox. Vùng vàng báo trước nơi đòn boss sắp đánh; né, đỡ hoặc nhảy đúng lúc rồi phản công. Boss đổi pha ở 70% và 35% sinh lực."</p>
                    <div class="character-catalog">
                        <label for="kit-choice">"Kit offline"</label>
                        <select id="kit-choice" aria-describedby="kit-selection-note" on:change=move |event| set_selected.set(event_target_value(&event))>
                            {choices.into_iter().map(|choice| view! {
                                <option value=choice.id.as_str() disabled=choice.disabled_reason.is_some() selected=choice.id == KitId::LONG_LUU>{choice.name}</option>
                            }).collect_view()}
                        </select>
                        <p id="kit-selection-note">"Lựa chọn khóa trước trận. Chưa có roster trực hệ đã duyệt; địa lý và đội chiến đấu độc lập với thế lực."</p>
                        {card_choices.into_iter().map(|choice| {
                            let id = choice.id;
                            view! { <div class:hidden=move || selected.get() != id.as_str()><CharacterCard choice=choice /></div> }
                        }).collect_view()}
                        {catalog_error.map(|error| view! { <p role="alert">{error}</p> })}
                    </div>
                    <label for="nickname">"Tên người chơi"</label>
                    <input id="nickname" maxlength="32" placeholder="Khách Vân Thủy" autocomplete="off"
                        on:input=move |event| set_name.set(event_target_value(&event)) />
                    <p class="muted">"Chơi thử tại máy, chưa có tài khoản hay lưu tiến trình. Hình khối là placeholder, chưa phải mỹ thuật thành phẩm."</p>
                    <button id="enter-game" class="primary" disabled=move || !disabled_choices.iter().any(|c| c.id.as_str() == selected.get() && c.disabled_reason.is_none()) on:click=move |_| {
                        let id = selected.get();
                        let Some(choice) = enter_choices.iter().find(|c| c.id.as_str() == id && c.disabled_reason.is_none()) else {
                            set_status.set("Kit chưa được chọn chơi".into()); return;
                        };
                        let Some(revision) = choice.revision else { set_status.set("Kit chưa triển khai".into()); return; };
                        set_playing.set(true);
                        restart();
                        mount_game(choice.id.as_str(), revision.version, &format!("{:016x}", revision.config_hash), choice.name);
                    }>"Chơi fixture offline →"</button>
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
                        <p><strong>"Mạch công/thủ"</strong>": chạm rồi thả R / RB+B / Triều dùng thuật thường; giữ 450 ms yêu cầu đại thuật (100 Mạch + 35 năng lượng, hồi chiêu chung). Khi bị choáng, thả rồi nhấn đỡ mới để Thoát Mạch (50 Mạch, hồi 20 giây). Trúng, bị đỡ, hụt và lý do không nối hiện trong luyện tập. Các giá trị đang thử nghiệm."</p>
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_choices_follow_catalog_and_registry_without_proposed_roster() {
        let choices = character_choices().unwrap();
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].id, KitId::LONG_LUU);
        assert!(choices[0].disabled_reason.is_none());
        assert!(choices[0].inheritance.contains("chưa ánh xạ"));
        assert_eq!(choices[0].name, "Long Lưu");
        assert_eq!(choices[0].review, "Chưa duyệt lore");
        assert_eq!(choices[0].icon, "LL");
    }
}
