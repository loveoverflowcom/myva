//! Điều khiển cảm ứng (combat.md §12): joystick bên trái, 8 nút bên phải — nhẹ, nặng, ba thuật
//! và nhóm nhảy/lướt/đỡ cạnh cụm đòn. Vùng chạm tối thiểu 48 px logic; không cần double-tap,
//! vuốt hay phân biệt màu.
//!
//! Layout và máy trạng thái chạm là hàm thuần để test; Bevy chỉ đọc `TouchInput` (tọa độ logic,
//! gốc trên-trái) và vẽ bằng UI node. Nút phát ý định ở lúc chạm xuống; đỡ giữ theo ngón tay. Joystick
//! nổi: chạm vào nửa trái ngoài nút đặt tâm cần tại điểm chạm.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use myva_sim::Buttons;

use crate::session::{GraySet, HostCommand, Intent, Session};

/// Đường kính vùng chạm tối thiểu, px logic.
pub const MIN_TARGET: f32 = 48.0;
/// Cần lệch ngang quá ngưỡng này mới đi.
pub const STICK_DEADZONE: f32 = 14.0;
/// Bán kính hiển thị của cần khi kéo.
const STICK_TRAVEL: f32 = 44.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TouchButton {
    Light,
    Heavy,
    /// Ô thuật `0..=2`.
    Skill(u8),
    Jump,
    Dash,
    Guard,
}

impl TouchButton {
    pub const ALL: [Self; 8] = [
        Self::Light,
        Self::Heavy,
        Self::Skill(0),
        Self::Skill(1),
        Self::Skill(2),
        Self::Jump,
        Self::Dash,
        Self::Guard,
    ];

    pub const fn buttons(self) -> Buttons {
        match self {
            Self::Light => Buttons::LIGHT,
            Self::Heavy => Buttons::HEAVY,
            Self::Skill(0) => Buttons::SKILL1,
            Self::Skill(1) => Buttons::SKILL2,
            Self::Skill(_) => Buttons::SKILL3,
            Self::Jump => Buttons::JUMP,
            Self::Dash => Buttons::DASH,
            Self::Guard => Buttons::GUARD,
        }
    }

    /// Mã ASCII ổn định cho bridge và test.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Heavy => "heavy",
            Self::Skill(0) => "skill1",
            Self::Skill(1) => "skill2",
            Self::Skill(_) => "skill3",
            Self::Jump => "jump",
            Self::Dash => "dash",
            Self::Guard => "guard",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Light => "Nhe",
            Self::Heavy => "Nang",
            Self::Skill(0) => "Tien",
            Self::Skill(1) => "The",
            Self::Skill(_) => "Trieu",
            Self::Jump => "Nhay",
            Self::Dash => "Luot",
            Self::Guard => "Do",
        }
    }
}

/// Vùng chạm tròn, px logic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pad {
    pub center: Vec2,
    pub radius: f32,
}

impl Pad {
    pub fn overlaps(&self, other: &Pad) -> bool {
        self.center.distance(other.center) < self.radius + other.radius
    }

    pub fn inside(&self, size: Vec2) -> bool {
        let min = self.center - self.radius;
        let max = self.center + self.radius;
        min.x >= 0.0 && min.y >= 0.0 && max.x <= size.x && max.y <= size.y
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TouchLayout {
    pub size: Vec2,
    /// Vị trí nghỉ của joystick, chỉ để hiển thị.
    pub stick: Pad,
    pub buttons: [(TouchButton, Pad); 8],
    /// Dung sai quanh mỗi nút khi chạm hơi lệch.
    slack: f32,
}

impl TouchLayout {
    /// Nút gần nhất trong vùng chạm (có dung sai).
    pub fn button_at(&self, position: Vec2) -> Option<TouchButton> {
        self.buttons
            .iter()
            .map(|(button, pad)| (*button, pad.center.distance(position) - pad.radius))
            .filter(|&(_, gap)| gap <= self.slack)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(button, _)| button)
    }

    pub fn pad(&self, button: TouchButton) -> Pad {
        self.buttons
            .iter()
            .find(|(b, _)| *b == button)
            .map(|&(_, pad)| pad)
            .expect("layout có đủ 8 nút")
    }

    /// JSON tọa độ nút theo px logic của canvas, cho test trình duyệt.
    pub fn to_json(&self) -> String {
        let buttons: Vec<String> = self
            .buttons
            .iter()
            .map(|(button, pad)| {
                format!(
                    "\"{}\":{{\"x\":{:.1},\"y\":{:.1},\"r\":{:.1}}}",
                    button.id(),
                    pad.center.x,
                    pad.center.y,
                    pad.radius
                )
            })
            .collect();
        format!(
            "{{\"width\":{:.1},\"height\":{:.1},\"stick\":{{\"x\":{:.1},\"y\":{:.1},\"r\":{:.1}}},\"buttons\":{{{}}}}}",
            self.size.x,
            self.size.y,
            self.stick.center.x,
            self.stick.center.y,
            self.stick.radius,
            buttons.join(",")
        )
    }
}

/// Bố cục theo kích thước màn hình. Màn hình ngang đủ rộng dùng hai hàng; màn hình hẹp (dọc)
/// xếp ba hàng sát mép phải để chừa chỗ cho joystick.
pub fn layout(size: Vec2) -> TouchLayout {
    let unit = (size.x.min(size.y) / 6.5).clamp(MIN_TARGET, 72.0);
    let small = unit / 2.0;
    let big = unit * 0.62;
    let gap = unit * 0.18;
    let margin = (unit * 0.3).max(12.0);
    let right = size.x - margin;
    let bottom = size.y - margin;
    let pad = |x: f32, y: f32, radius: f32| Pad {
        center: Vec2::new(x, y),
        radius,
    };

    let light = pad(right - big, bottom - big, big);
    let heavy = pad(light.center.x - 2.0 * big - gap, light.center.y, big);
    let skills_y = light.center.y - big - gap - small;
    let (skills, moves) = if size.x >= unit * 10.0 {
        // Nhóm nhảy/lướt/đỡ cùng hàng, cách cụm đòn một khoảng rộng hơn.
        let guard = pad(
            heavy.center.x - big - 2.0 * gap - small,
            bottom - small,
            small,
        );
        let dash = pad(guard.center.x - 2.0 * small - gap, bottom - small, small);
        let jump = pad(dash.center.x - 2.0 * small - gap, bottom - small, small);
        let skills = [
            pad(heavy.center.x - big - gap - small, skills_y, small),
            pad(heavy.center.x, skills_y, small),
            pad(light.center.x, skills_y, small),
        ];
        (skills, [jump, dash, guard])
    } else {
        let column = |i: f32| right - small - i * (2.0 * small + gap);
        let moves_y = skills_y - 2.0 * small - gap;
        (
            [
                pad(column(2.0), skills_y, small),
                pad(column(1.0), skills_y, small),
                pad(column(0.0), skills_y, small),
            ],
            [
                pad(column(2.0), moves_y, small),
                pad(column(1.0), moves_y, small),
                pad(column(0.0), moves_y, small),
            ],
        )
    };
    let stick_radius = unit * 1.1;
    TouchLayout {
        size,
        stick: pad(margin + stick_radius, bottom - stick_radius, stick_radius),
        buttons: [
            (TouchButton::Light, light),
            (TouchButton::Heavy, heavy),
            (TouchButton::Skill(0), skills[0]),
            (TouchButton::Skill(1), skills[1]),
            (TouchButton::Skill(2), skills[2]),
            (TouchButton::Jump, moves[0]),
            (TouchButton::Dash, moves[1]),
            (TouchButton::Guard, moves[2]),
        ],
        slack: gap / 2.0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stick {
    pub id: u64,
    pub origin: Vec2,
    pub position: Vec2,
}

/// Trạng thái các ngón đang chạm. Mỗi ngón gắn với đúng một nút hoặc joystick từ lúc chạm xuống
/// tới lúc nhấc lên; trượt ra khỏi nút không đổi nút.
#[derive(Resource, Clone, Debug, Default)]
pub struct TouchPad {
    stick: Option<Stick>,
    held: Vec<(u64, TouchButton)>,
    pressed: Buttons,
}

impl TouchPad {
    pub fn begin(&mut self, layout: &TouchLayout, id: u64, position: Vec2) {
        if let Some(button) = layout.button_at(position) {
            self.held.push((id, button));
            self.pressed |= button.buttons();
        } else if self.stick.is_none() && position.x < layout.size.x / 2.0 {
            self.stick = Some(Stick {
                id,
                origin: position,
                position,
            });
        }
    }

    pub fn moved(&mut self, id: u64, position: Vec2) {
        if let Some(stick) = self.stick.as_mut().filter(|stick| stick.id == id) {
            stick.position = position;
        }
    }

    /// Ngón nhấc lên hoặc bị hệ thống hủy.
    pub fn end(&mut self, id: u64) {
        self.held.retain(|&(touch, _)| touch != id);
        if self.stick.is_some_and(|stick| stick.id == id) {
            self.stick = None;
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn move_x(&self) -> i8 {
        match self.stick {
            Some(stick) if (stick.position.x - stick.origin.x).abs() > STICK_DEADZONE => {
                (stick.position.x - stick.origin.x).signum() as i8
            }
            _ => 0,
        }
    }

    /// Chỉ đỡ là nút giữ.
    pub fn held(&self) -> Buttons {
        if self.is_held(TouchButton::Guard) {
            Buttons::GUARD
        } else {
            Buttons::NONE
        }
    }

    pub fn is_held(&self, button: TouchButton) -> bool {
        self.held.iter().any(|&(_, b)| b == button)
    }

    pub fn take_pressed(&mut self) -> Buttons {
        std::mem::take(&mut self.pressed)
    }

    pub fn stick(&self) -> Option<Stick> {
        self.stick
    }
}

/// Lớp điều khiển cảm ứng hiện sau lần chạm đầu tiên, hoặc khi host yêu cầu.
#[derive(Resource, Debug, Default)]
pub struct TouchUi {
    pub visible: bool,
    pub layout: Option<TouchLayout>,
}

/// Bật/tắt lớp cảm ứng từ host (ví dụ thiết bị có con trỏ thô).
#[derive(Message, Clone, Copy, Debug)]
pub struct ShowTouchControls(pub bool);

pub fn plugin(app: &mut App) {
    app.init_resource::<TouchPad>()
        .init_resource::<TouchUi>()
        .add_message::<ShowTouchControls>()
        .add_systems(Startup, spawn_controls)
        .add_systems(
            RunFixedMainLoop,
            (
                clear_on_host_command.in_set(GraySet::Commands),
                read_touches.in_set(GraySet::Devices),
            ),
        )
        .add_systems(Update, (draw_controls, draw_hint));
}

fn clear_on_host_command(mut commands: MessageReader<HostCommand>, mut pad: ResMut<TouchPad>) {
    if commands.read().count() > 0 {
        pad.take_pressed();
    }
}

fn read_touches(
    mut touches: MessageReader<TouchInput>,
    mut show: MessageReader<ShowTouchControls>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut pad: ResMut<TouchPad>,
    mut ui: ResMut<TouchUi>,
    mut intent: ResMut<Intent>,
) {
    for ShowTouchControls(visible) in show.read() {
        ui.visible = *visible;
        if !visible {
            pad.clear();
        }
    }
    let current = layout(window.size());
    for touch in touches.read() {
        ui.visible = true;
        match touch.phase {
            bevy::input::touch::TouchPhase::Started => {
                pad.begin(&current, touch.id, touch.position)
            }
            bevy::input::touch::TouchPhase::Moved => pad.moved(touch.id, touch.position),
            bevy::input::touch::TouchPhase::Ended | bevy::input::touch::TouchPhase::Canceled => {
                pad.end(touch.id);
            }
        }
    }
    ui.layout = ui.visible.then_some(current);
    let pressed = pad.take_pressed();
    intent.add(pad.move_x(), pad.held(), pressed);
}

#[derive(Component)]
struct TouchRoot;

#[derive(Component)]
struct TouchKey(TouchButton);

#[derive(Component)]
struct KeyCooldown(u8);

#[derive(Component)]
enum StickPart {
    Base,
    Knob,
}

#[derive(Component)]
struct RotateHint;

const KEY_IDLE: Color = Color::srgba(0.85, 0.9, 0.95, 0.16);
const KEY_DOWN: Color = Color::srgba(0.95, 0.85, 0.45, 0.55);
const KEY_DIM: Color = Color::srgba(0.4, 0.4, 0.45, 0.25);
const KEY_BORDER: Color = Color::srgba(0.95, 0.97, 1.0, 0.55);

fn spawn_controls(mut commands: Commands) {
    commands
        .spawn((
            TouchRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(10),
        ))
        .with_children(|root| {
            for part in [StickPart::Base, StickPart::Knob] {
                root.spawn((
                    part,
                    Node {
                        position_type: PositionType::Absolute,
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(KEY_IDLE),
                    BorderColor::all(KEY_BORDER),
                ));
            }
            for button in TouchButton::ALL {
                root.spawn((
                    TouchKey(button),
                    Node {
                        position_type: PositionType::Absolute,
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::MAX,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(KEY_IDLE),
                    BorderColor::all(KEY_BORDER),
                ))
                .with_children(|key| {
                    key.spawn((
                        Text::new(button.label()),
                        TextFont::from_font_size(14.0),
                        TextColor(Color::WHITE),
                    ));
                    if let TouchButton::Skill(slot) = button {
                        key.spawn((
                            KeyCooldown(slot),
                            Text::new(""),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(1.0, 0.85, 0.5)),
                        ));
                    }
                });
            }
            root.spawn((
                RotateHint,
                Text::new("Xoay ngang man hinh de choi thoai mai hon"),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(42.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                TextLayout::justify(Justify::Center),
            ));
        });
}

fn place(node: &mut Node, pad: Pad) {
    node.left = Val::Px(pad.center.x - pad.radius);
    node.top = Val::Px(pad.center.y - pad.radius);
    node.width = Val::Px(pad.radius * 2.0);
    node.height = Val::Px(pad.radius * 2.0);
}

#[allow(clippy::type_complexity)]
fn draw_controls(
    ui: Res<TouchUi>,
    pad: Res<TouchPad>,
    session: Res<Session>,
    mut root: Single<&mut Visibility, With<TouchRoot>>,
    mut keys: Query<(&TouchKey, &mut Node, &mut BackgroundColor), Without<StickPart>>,
    mut sticks: Query<(&StickPart, &mut Node, &mut BackgroundColor), Without<TouchKey>>,
    mut cooldowns: Query<(&KeyCooldown, &mut Text)>,
) {
    let Some(layout) = ui.layout.as_ref() else {
        **root = Visibility::Hidden;
        return;
    };
    **root = Visibility::Inherited;
    let fighter = session.battle.world().fighter(session.battle.player());
    for (key, mut node, mut background) in &mut keys {
        place(&mut node, layout.pad(key.0));
        let blocked = match key.0 {
            TouchButton::Skill(slot) => {
                let spec = &fighter.kit.skills[usize::from(slot)];
                fighter.cooldowns[usize::from(slot)] > 0
                    || !fighter.energy.can_spend(spec.energy_cost)
            }
            TouchButton::Heavy => !fighter.stamina.can_spend(fighter.kit.heavy.stamina_cost),
            TouchButton::Dash => !fighter.stamina.can_spend(myva_sim::fighter::DASH_STAMINA),
            _ => false,
        };
        background.0 = match (pad.is_held(key.0), blocked) {
            (true, _) => KEY_DOWN,
            (false, true) => KEY_DIM,
            (false, false) => KEY_IDLE,
        };
    }
    for (slot, mut text) in &mut cooldowns {
        let remaining = fighter.cooldowns[usize::from(slot.0)];
        text.0 = if remaining > 0 {
            crate::text::seconds(remaining)
        } else {
            String::new()
        };
    }
    let stick = pad.stick();
    for (part, mut node, mut background) in &mut sticks {
        let base = match stick {
            Some(stick) => Pad {
                center: stick.origin,
                radius: layout.stick.radius,
            },
            None => layout.stick,
        };
        match part {
            StickPart::Base => {
                place(&mut node, base);
                background.0 = KEY_IDLE.with_alpha(0.08);
            }
            StickPart::Knob => {
                let offset = stick
                    .map(|s| (s.position - s.origin).clamp_length_max(STICK_TRAVEL))
                    .unwrap_or(Vec2::ZERO);
                place(
                    &mut node,
                    Pad {
                        center: base.center + offset,
                        radius: base.radius * 0.45,
                    },
                );
                background.0 = if stick.is_some() { KEY_DOWN } else { KEY_IDLE };
            }
        }
    }
}

/// Màn hình dọc quá hẹp cho cụm nút: gợi ý xoay ngang.
fn draw_hint(
    ui: Res<TouchUi>,
    mut hint: Single<&mut Visibility, (With<RotateHint>, Without<TouchRoot>)>,
) {
    let portrait = ui
        .layout
        .as_ref()
        .is_some_and(|layout| layout.size.y > layout.size.x);
    **hint = if portrait {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREENS: [(f32, f32); 8] = [
        (568.0, 320.0),
        (667.0, 375.0),
        (844.0, 390.0),
        (932.0, 430.0),
        (1280.0, 720.0),
        (1920.0, 1080.0),
        (357.0, 403.0),
        (320.0, 480.0),
    ];

    #[test]
    fn every_layout_fits_without_overlap_and_meets_min_target() {
        for (w, h) in SCREENS {
            let size = Vec2::new(w, h);
            let layout = layout(size);
            let pads: Vec<Pad> = layout.buttons.iter().map(|&(_, pad)| pad).collect();
            for (i, pad) in pads.iter().enumerate() {
                assert!(pad.radius * 2.0 >= MIN_TARGET, "{w}x{h}: nút {i} nhỏ");
                assert!(pad.inside(size), "{w}x{h}: nút {i} ra ngoài màn hình");
                assert!(!pad.overlaps(&layout.stick), "{w}x{h}: nút {i} đè joystick");
                for other in &pads[i + 1..] {
                    assert!(!pad.overlaps(other), "{w}x{h}: nút {i} chồng nút khác");
                }
            }
            assert!(layout.stick.inside(size), "{w}x{h}: joystick ra ngoài");
        }
    }

    #[test]
    fn tapping_a_button_presses_once_and_guard_is_held() {
        let layout = layout(Vec2::new(844.0, 390.0));
        let mut pad = TouchPad::default();
        pad.begin(&layout, 1, layout.pad(TouchButton::Light).center);
        pad.begin(&layout, 2, layout.pad(TouchButton::Guard).center);
        assert_eq!(pad.take_pressed(), Buttons::LIGHT | Buttons::GUARD);
        assert!(pad.take_pressed().is_empty());
        assert_eq!(pad.held(), Buttons::GUARD);
        pad.end(2);
        assert_eq!(pad.held(), Buttons::NONE);
    }

    #[test]
    fn floating_stick_moves_past_the_deadzone_and_stops_on_cancel() {
        let layout = layout(Vec2::new(844.0, 390.0));
        let mut pad = TouchPad::default();
        let origin = Vec2::new(150.0, 300.0);
        pad.begin(&layout, 7, origin);
        pad.moved(7, origin + Vec2::new(STICK_DEADZONE - 1.0, 0.0));
        assert_eq!(pad.move_x(), 0);
        pad.moved(7, origin + Vec2::new(-40.0, 25.0));
        assert_eq!(pad.move_x(), -1);
        // Ngón khác chạm nửa trái không cướp joystick.
        pad.begin(&layout, 8, Vec2::new(60.0, 200.0));
        pad.moved(8, Vec2::new(300.0, 200.0));
        assert_eq!(pad.move_x(), -1);
        pad.end(7);
        assert_eq!(pad.move_x(), 0);
        assert!(pad.take_pressed().is_empty(), "joystick không phát nút");
    }

    #[test]
    fn near_miss_picks_the_closest_button_and_empty_right_side_does_nothing() {
        let layout = layout(Vec2::new(844.0, 390.0));
        let light = layout.pad(TouchButton::Light);
        let near = light.center + Vec2::new(0.0, -(light.radius + 2.0));
        assert_eq!(layout.button_at(near), Some(TouchButton::Light));
        let mut pad = TouchPad::default();
        pad.begin(&layout, 1, Vec2::new(600.0, 40.0));
        assert!(pad.take_pressed().is_empty());
        assert!(pad.stick().is_none());
    }

    #[test]
    fn layout_json_names_every_button() {
        let json = layout(Vec2::new(844.0, 390.0)).to_json();
        for button in TouchButton::ALL {
            assert!(json.contains(&format!("\"{}\":", button.id())));
        }
    }
}
