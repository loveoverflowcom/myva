//! HUD bằng UI node: thanh sinh lực, sức bền, năng lượng và Mạch, hồi chiêu ba thuật, pha boss,
//! kết quả trận cùng trạng thái kiểm chứng replay, log sự kiện và phím. HUD chỉ đọc [`ArenaView`]
//! (component mirror) và [`Match`]; thanh HP là thể hiện, sát thương do lõi tính (combat.md §9).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use myva_sim::FighterId;
use myva_sim::battle::{Cause, Mode, Outcome};
use myva_sim::snapshot::{FighterView, Gauge};

use crate::fight::{ArenaView, CombatLog, Match};
use crate::text::{ascii, phase_label, seconds, state_label};
use crate::touch::TouchUi;

const TEXT: Color = Color::srgb(0.88, 0.89, 0.92);
const DIM: Color = Color::srgb(0.62, 0.64, 0.70);
const TRACK: Color = Color::srgba(1.0, 1.0, 1.0, 0.1);
const HP: Color = Color::srgb(0.86, 0.25, 0.25);
const STAMINA: Color = Color::srgb(0.3, 0.78, 0.4);
const ENERGY: Color = Color::srgb(0.4, 0.72, 0.95);
const MACH: Color = Color::srgb(0.95, 0.75, 0.25);
/// Dưới chiều cao này (px logic) HUD gọn lại để chừa chỗ cho arena và nút cảm ứng.
const COMPACT_HEIGHT: f32 = 480.0;

/// Bóng chữ sát để nhãn đọc được trên cả phần đầy lẫn phần trống của thanh.
fn shadow() -> TextShadow {
    TextShadow {
        offset: Vec2::splat(1.0),
        color: Color::srgba(0.0, 0.0, 0.0, 0.85),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Player,
    Rival,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BarKind {
    Hp,
    Stamina,
    Energy,
    Mach,
}

impl BarKind {
    const ALL: [Self; 4] = [Self::Hp, Self::Stamina, Self::Energy, Self::Mach];

    const fn color(self) -> Color {
        match self {
            Self::Hp => HP,
            Self::Stamina => STAMINA,
            Self::Energy => ENERGY,
            Self::Mach => MACH,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    Header(Side),
    /// Hồi chiêu với người chơi, pha với boss.
    Detail(Side),
    State(Side),
    Banner,
    BannerDetail,
    Log,
    Legend,
    Meta,
}

#[derive(Component)]
struct HudText(Slot);

#[derive(Component)]
struct HudBar(Side, BarKind);

#[derive(Component)]
struct HudFill(Side, BarKind);

#[derive(Component)]
struct HudLabel(Side, BarKind);

/// Vạch ngưỡng đổi pha 70% và 35% trên thanh HP boss.
#[derive(Component)]
struct Threshold;

#[derive(Component)]
struct BannerBox;

/// Phần chỉ hiện khi không dùng cảm ứng, để màn hình nhỏ còn chỗ.
#[derive(Component)]
struct DesktopOnly;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_hud)
        .add_systems(Update, (update_bars, update_texts, update_layout));
}

fn text(slot: Slot, size: f32, color: Color) -> impl Bundle {
    (
        HudText(slot),
        Text::new(""),
        TextFont::from_font_size(size),
        TextColor(color),
        Node::default(),
    )
}

fn spawn_panel(parent: &mut ChildSpawnerCommands, side: Side) {
    let align = match side {
        Side::Player => AlignItems::FlexStart,
        Side::Rival => AlignItems::FlexEnd,
    };
    parent
        .spawn(Node {
            width: Val::Percent(44.0),
            max_width: Val::Px(380.0),
            flex_direction: FlexDirection::Column,
            align_items: align,
            row_gap: Val::Px(3.0),
            ..default()
        })
        .with_children(|panel| {
            panel.spawn(text(Slot::Header(side), 18.0, TEXT));
            for kind in BarKind::ALL {
                panel
                    .spawn((
                        HudBar(side, kind),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(15.0),
                            ..default()
                        },
                        BackgroundColor(TRACK),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            HudFill(side, kind),
                            Node {
                                height: Val::Percent(100.0),
                                width: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(kind.color()),
                        ));
                        if kind == BarKind::Hp && side == Side::Rival {
                            for at in [70.0, 35.0] {
                                bar.spawn((
                                    Threshold,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Percent(at),
                                        top: Val::Px(-3.0),
                                        width: Val::Px(2.0),
                                        height: Val::Percent(140.0),
                                        ..default()
                                    },
                                    BackgroundColor(TEXT),
                                ));
                            }
                        }
                        bar.spawn((
                            HudLabel(side, kind),
                            Text::new(""),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::WHITE),
                            shadow(),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(6.0),
                                top: Val::Px(0.0),
                                ..default()
                            },
                        ));
                    });
            }
            panel.spawn(text(Slot::Detail(side), 13.0, DIM));
            panel.spawn(text(Slot::State(side), 13.0, DIM));
        });
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        })
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|top| {
                spawn_panel(top, Side::Player);
                spawn_panel(top, Side::Rival);
            });
            root.spawn((
                DesktopOnly,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
            ))
            .with_children(|bottom| {
                bottom.spawn(text(Slot::Log, 13.0, TEXT));
                bottom.spawn(text(Slot::Legend, 13.0, DIM));
                bottom.spawn(text(Slot::Meta, 12.0, DIM));
            });
        });
    commands
        .spawn((
            BannerBox,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Percent(26.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|banner| {
            banner.spawn((text(Slot::Banner, 48.0, TEXT), shadow()));
            banner.spawn((
                text(Slot::BannerDetail, 15.0, TEXT),
                shadow(),
                TextLayout::justify(Justify::Center),
            ));
        });
}

fn fill(gauge: Gauge) -> f32 {
    gauge.sub as f32 / gauge.max_sub.max(1) as f32
}

fn fighter_for<'a>(arena: &'a ArenaView, game: &Match, side: Side) -> Option<&'a FighterView> {
    arena.fighter(match side {
        Side::Player => game.bout.player(),
        Side::Rival => game.bout.rival(),
    })
}

#[allow(clippy::type_complexity)]
fn update_bars(
    arena: Res<ArenaView>,
    game: Res<Match>,
    mut bars: Query<(&HudBar, &mut Node), (Without<HudFill>, Without<HudLabel>)>,
    mut fills: Query<(&HudFill, &mut Node), (Without<HudBar>, Without<HudLabel>)>,
    mut labels: Query<(&HudLabel, &mut Text)>,
) {
    let shown = |side: Side, kind: BarKind| {
        // Boss không dùng sức bền, năng lượng hay Mạch.
        kind == BarKind::Hp || fighter_for(&arena, &game, side).is_some_and(|f| !f.kit.body.armored)
    };
    for (bar, mut node) in &mut bars {
        let display = if shown(bar.0, bar.1) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (bar, mut node) in &mut fills {
        let Some(fighter) = fighter_for(&arena, &game, bar.0) else {
            continue;
        };
        let fraction = match bar.1 {
            BarKind::Hp => fighter.hp as f32 / fighter.max_hp.max(1) as f32,
            BarKind::Stamina => fill(fighter.stamina),
            BarKind::Energy => fill(fighter.energy),
            BarKind::Mach => fill(fighter.mach),
        };
        node.width = Val::Percent(fraction.clamp(0.0, 1.0) * 100.0);
    }
    for (label, mut text) in &mut labels {
        let Some(fighter) = fighter_for(&arena, &game, label.0) else {
            continue;
        };
        let value = match label.1 {
            BarKind::Hp => format!("HP {}/{}", fighter.hp, fighter.max_hp),
            BarKind::Stamina => format!("Suc ben {}", fighter.stamina.points()),
            BarKind::Energy => format!("Nang luong {}", fighter.energy.points()),
            BarKind::Mach => format!(
                "Mach {}/{}",
                fighter.mach.points(),
                fighter.mach.max_points()
            ),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

fn cooldowns(fighter: &FighterView) -> String {
    ["Q", "E", "R"]
        .iter()
        .zip(fighter.kit.skills.iter().zip(fighter.cooldowns))
        .map(|(key, (spec, ticks))| {
            let state = if ticks > 0 {
                seconds(ticks)
            } else if !fighter.energy.can_spend(spec.energy_cost) {
                format!("thieu {}NL", spec.energy_cost)
            } else {
                "ok".to_owned()
            };
            format!("{key} {} {state}", ascii(spec.name))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn cause(arena: &ArenaView, game: &Match, id: FighterId) -> Option<String> {
    Some(match game.bout.tally(id).last_hit_by? {
        Cause::Action(attacker, action) => format!(
            "{} cua {}",
            ascii(arena.fighter(attacker)?.kit.spec(action).name),
            game.name(attacker)
        ),
        Cause::Counter(counterer) => format!("phan cong cua {}", game.name(counterer)),
    })
}

fn banner(arena: &ArenaView, game: &Match) -> Option<(String, String)> {
    let outcome = game.bout.outcome()?;
    let title = match (outcome, game.bout.mode()) {
        (Outcome::Victory, Mode::Boss) => "THANG - Ke Giu Dap bi ha",
        (Outcome::Victory, Mode::Duel) => "THANG",
        (Outcome::Defeat, _) => "THUA",
        (Outcome::Draw, _) => "HOA",
    };
    let mut lines = Vec::new();
    if let Some(cause) =
        cause(arena, game, game.bout.player()).filter(|_| outcome != Outcome::Victory)
    {
        lines.push(format!("Bi ha boi {cause}"));
    }
    lines.push(match &game.verification {
        None => "Dang kiem chung replay...".to_owned(),
        Some(Ok(v)) => format!(
            "Replay da kiem chung: {} tick, {} moc hash, hash {:016x}",
            v.ticks, v.checkpoints, v.hash
        ),
        Some(Err(error)) => format!("Replay LECH: {}", ascii(error)),
    });
    lines.push("Enter / Start / nut Dau lai de dau tiep".to_owned());
    Some((title.to_owned(), lines.join("\n")))
}

fn set(text: &mut Text, value: String) {
    if text.0 != value {
        text.0 = value;
    }
}

fn update_texts(
    arena: Res<ArenaView>,
    game: Res<Match>,
    log: Res<CombatLog>,
    mut texts: Query<(&HudText, &mut Text)>,
    mut banner_box: Single<&mut Visibility, With<BannerBox>>,
    mut thresholds: Query<&mut Visibility, (With<Threshold>, Without<BannerBox>)>,
) {
    let banner = banner(&arena, &game);
    **banner_box = if banner.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let boss = arena.boss_phase.is_some();
    for mut visibility in &mut thresholds {
        *visibility = if boss {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (slot, mut text) in &mut texts {
        let value = match slot.0 {
            Slot::Header(side) => {
                let Some(fighter) = fighter_for(&arena, &game, side) else {
                    continue;
                };
                let mut header =
                    format!("{}  {}", game.name(fighter.id), ascii(fighter.kit.lineage));
                if side == Side::Player && game.bout.autopilot() {
                    header.push_str("  [bot B1 lai]");
                }
                if side == Side::Rival && arena.sparring == Some(false) {
                    header.push_str("  [dung yen]");
                }
                header
            }
            Slot::Detail(side) => match (side, arena.boss_phase) {
                (Side::Rival, Some(phase)) => phase_label(phase).to_owned(),
                _ => fighter_for(&arena, &game, side)
                    .map(cooldowns)
                    .unwrap_or_default(),
            },
            Slot::State(side) => fighter_for(&arena, &game, side)
                .map(state_label)
                .unwrap_or_default(),
            Slot::Banner => banner.as_ref().map(|b| b.0.clone()).unwrap_or_default(),
            Slot::BannerDetail => banner.as_ref().map(|b| b.1.clone()).unwrap_or_default(),
            Slot::Log => log.lines.iter().cloned().collect::<Vec<_>>().join("\n"),
            Slot::Legend => {
                let mode = match arena.sparring {
                    None => "M dau tap".to_owned(),
                    Some(on) => {
                        format!("M danh boss  B bot ({})", if on { "bat" } else { "tat" })
                    }
                };
                format!(
                    "A/D di chuyen  Space nhay  Shift luot  L do  J/K nhe/nang  Q/E/R thuat\n\
                     Tay cam: A nhay  B luot  LB do  X/Y nhe/nang  RB+X/Y/B thuat\n\
                     Enter dau lai  {mode}  H hitbox"
                )
            }
            Slot::Meta => format!(
                "Vong {}  tick {}  hash {:016x}",
                game.bout.round(),
                arena.tick,
                arena.hash
            ),
        };
        set(&mut text, value);
    }
}

/// HUD gọn khi màn hình thấp hoặc đang dùng cảm ứng: ẩn log/phím, nhãn phụ và dòng hồi chiêu
/// (nút thuật cảm ứng tự hiện hồi chiêu). Pha boss luôn hiện.
#[allow(clippy::type_complexity)]
fn update_layout(
    window: Single<&Window, With<PrimaryWindow>>,
    touch: Res<TouchUi>,
    arena: Res<ArenaView>,
    mut desktop: Query<&mut Visibility, With<DesktopOnly>>,
    mut bars: Query<(&HudBar, &mut Node)>,
    mut labels: Query<(&HudLabel, &mut Visibility), Without<DesktopOnly>>,
    mut lines: Query<(&HudText, &mut Node), Without<HudBar>>,
) {
    let compact = touch.visible || window.height() < COMPACT_HEIGHT;
    let boss = arena.boss_phase.is_some();
    for (slot, mut node) in &mut lines {
        let hidden = compact
            && match slot.0 {
                Slot::Detail(Side::Rival) => !boss,
                Slot::Detail(Side::Player) | Slot::State(_) => true,
                _ => false,
            };
        let display = if hidden { Display::None } else { Display::Flex };
        if node.display != display {
            node.display = display;
        }
    }
    for mut visibility in &mut desktop {
        *visibility = if compact {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    let height = Val::Px(if compact { 11.0 } else { 15.0 });
    for (_, mut node) in &mut bars {
        if node.height != height {
            node.height = height;
        }
    }
    for (label, mut visibility) in &mut labels {
        *visibility = if compact && label.1 != BarKind::Hp {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
