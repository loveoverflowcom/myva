//! Bàn phím và gamepad → ý định (combat.md §12). Mapping nằm trong các bảng hằng để đổi ở một
//! chỗ; màn hình đổi phím chưa có.
//!
//! Bàn phím: A/D hoặc ←/→ di chuyển, Space nhảy, Shift lướt, giữ L đỡ, J/K nhẹ/nặng, Q/E/R thuật.
//! Gamepad: stick trái/D-pad di chuyển, A nhảy, B lướt, giữ LB đỡ, X/Y nhẹ/nặng, RB + X/Y/B thuật.
//! Mọi gamepad đang nối đều điều khiển người chơi, theo thứ tự input thực tế.

use bevy::prelude::*;
use myva_sim::Buttons;

use crate::fight::{GraySet, HostCommand, Intent, Match};

pub const MOVE_LEFT: [KeyCode; 2] = [KeyCode::KeyA, KeyCode::ArrowLeft];
pub const MOVE_RIGHT: [KeyCode; 2] = [KeyCode::KeyD, KeyCode::ArrowRight];
pub const GUARD_KEY: KeyCode = KeyCode::KeyL;
pub const PRESS_KEYS: [(KeyCode, Buttons); 8] = [
    (KeyCode::Space, Buttons::JUMP),
    (KeyCode::ShiftLeft, Buttons::DASH),
    (KeyCode::ShiftRight, Buttons::DASH),
    (KeyCode::KeyJ, Buttons::LIGHT),
    (KeyCode::KeyK, Buttons::HEAVY),
    (KeyCode::KeyQ, Buttons::SKILL1),
    (KeyCode::KeyE, Buttons::SKILL2),
    (KeyCode::KeyR, Buttons::SKILL3),
];
/// Phím tắt không thuộc mô phỏng. F5 dành cho trình duyệt, nên đấu lại dùng Enter.
pub const SHORTCUTS: [(KeyCode, HostCommand); 4] = [
    (KeyCode::Enter, HostCommand::Rematch),
    (KeyCode::KeyM, HostCommand::SwitchMode),
    (KeyCode::KeyB, HostCommand::ToggleSparringBot),
    (KeyCode::KeyH, HostCommand::ToggleHitboxes),
];

/// Stick phải lệch quá ngưỡng này mới tính là di chuyển, để stick trôi không làm nhân vật bước.
pub const STICK_DEADZONE: f32 = 0.35;
/// Nút giữ để đỡ.
pub const PAD_GUARD: GamepadButton = GamepadButton::LeftTrigger;
/// Modifier chuyển X/Y/B sang thuật 1/2/3.
pub const PAD_SKILL_MODIFIER: GamepadButton = GamepadButton::RightTrigger;

pub fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        (
            shortcuts.in_set(GraySet::Shortcuts),
            (keyboard, gamepads).in_set(GraySet::Devices),
        ),
    );
}

/// Ý định từ bàn phím, đọc qua các hàm truy vấn để test không cần `ButtonInput`.
pub fn keyboard_intent(
    pressed: impl Fn(KeyCode) -> bool,
    just_pressed: impl Fn(KeyCode) -> bool,
) -> (i8, Buttons, Buttons) {
    let left = MOVE_LEFT.into_iter().any(&pressed);
    let right = MOVE_RIGHT.into_iter().any(&pressed);
    let held = if pressed(GUARD_KEY) {
        Buttons::GUARD
    } else {
        Buttons::NONE
    };
    let mut fresh = Buttons::NONE;
    for (key, button) in PRESS_KEYS {
        if just_pressed(key) {
            fresh |= button;
        }
    }
    (i8::from(right) - i8::from(left), held, fresh)
}

/// Ý định từ một gamepad. Modifier được xét trong cùng khung với nút mặt: giữ RB (hoặc nhấn cùng
/// khung) thì X/Y/B thành thuật, không phát đòn cơ bản; thả RB sau đó cũng không sinh thêm đòn,
/// vì đòn chỉ phát ở cạnh nhấn.
pub fn gamepad_intent(
    pressed: impl Fn(GamepadButton) -> bool,
    just_pressed: impl Fn(GamepadButton) -> bool,
    stick_x: f32,
) -> (i8, Buttons, Buttons) {
    let left = pressed(GamepadButton::DPadLeft) || stick_x <= -STICK_DEADZONE;
    let right = pressed(GamepadButton::DPadRight) || stick_x >= STICK_DEADZONE;
    let held = if pressed(PAD_GUARD) {
        Buttons::GUARD
    } else {
        Buttons::NONE
    };
    let chord = pressed(PAD_SKILL_MODIFIER);
    let face = [
        (GamepadButton::West, Buttons::LIGHT, Buttons::SKILL1),
        (GamepadButton::North, Buttons::HEAVY, Buttons::SKILL2),
        (GamepadButton::East, Buttons::DASH, Buttons::SKILL3),
        (GamepadButton::South, Buttons::JUMP, Buttons::JUMP),
    ];
    let mut fresh = Buttons::NONE;
    for (button, plain, with_modifier) in face {
        if just_pressed(button) {
            fresh |= if chord { with_modifier } else { plain };
        }
    }
    (i8::from(right) - i8::from(left), held, fresh)
}

fn keyboard(keys: Res<ButtonInput<KeyCode>>, mut intent: ResMut<Intent>) {
    let (move_x, held, pressed) =
        keyboard_intent(|key| keys.pressed(key), |key| keys.just_pressed(key));
    intent.add(move_x, held, pressed);
}

fn gamepads(pads: Query<&Gamepad>, mut intent: ResMut<Intent>) {
    for pad in &pads {
        let (move_x, held, pressed) = gamepad_intent(
            |button| pad.pressed(button),
            |button| pad.just_pressed(button),
            pad.left_stick().x,
        );
        intent.add(move_x, held, pressed);
    }
}

fn shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<&Gamepad>,
    game: Res<Match>,
    mut commands: MessageWriter<HostCommand>,
) {
    for (key, command) in SHORTCUTS {
        if keys.just_pressed(key) {
            commands.write(command);
        }
    }
    // Start chỉ đấu lại khi trận đã có kết quả, tránh bấm nhầm giữa trận.
    let start = pads
        .iter()
        .any(|pad| pad.just_pressed(GamepadButton::Start));
    if start && game.bout.outcome().is_some() {
        commands.write(HostCommand::Rematch);
    }
    if pads
        .iter()
        .any(|pad| pad.just_pressed(GamepadButton::Select))
    {
        commands.write(HostCommand::ToggleHitboxes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set<T: PartialEq + Copy>(items: &[T]) -> impl Fn(T) -> bool + '_ {
        move |item| items.contains(&item)
    }

    #[test]
    fn keyboard_follows_combat_doc_defaults() {
        let (move_x, held, pressed) = keyboard_intent(
            set(&[KeyCode::ArrowRight, KeyCode::KeyL, KeyCode::KeyJ]),
            set(&[KeyCode::KeyJ, KeyCode::KeyR, KeyCode::ShiftLeft]),
        );
        assert_eq!(move_x, 1);
        assert_eq!(held, Buttons::GUARD);
        assert_eq!(pressed, Buttons::LIGHT | Buttons::SKILL3 | Buttons::DASH);
        let (move_x, ..) = keyboard_intent(set(&[KeyCode::KeyA, KeyCode::KeyD]), set(&[]));
        assert_eq!(move_x, 0, "giữ cả hai hướng thì đứng yên");
    }

    #[test]
    fn holding_a_key_presses_only_once() {
        let (_, _, pressed) = keyboard_intent(set(&[KeyCode::KeyJ]), set(&[]));
        assert!(pressed.is_empty());
    }

    #[test]
    fn pad_modifier_turns_face_buttons_into_skills() {
        use GamepadButton::*;
        let (_, _, plain) = gamepad_intent(set(&[West]), set(&[West, North]), 0.0);
        assert_eq!(plain, Buttons::LIGHT | Buttons::HEAVY);
        let (_, _, chord) = gamepad_intent(
            set(&[RightTrigger, West, North, East]),
            set(&[West, North, East]),
            0.0,
        );
        assert_eq!(chord, Buttons::SKILL1 | Buttons::SKILL2 | Buttons::SKILL3);
        // RB và X cùng khung: vẫn là thuật, không có đòn nhẹ đi kèm.
        let (_, _, same_frame) =
            gamepad_intent(set(&[RightTrigger, West]), set(&[RightTrigger, West]), 0.0);
        assert_eq!(same_frame, Buttons::SKILL1);
        // Thả RB khi vẫn giữ X không phát thêm đòn nhẹ.
        let (_, _, released) = gamepad_intent(set(&[West]), set(&[]), 0.0);
        assert!(released.is_empty());
        // RB một mình không làm gì; nhảy không bị modifier đổi.
        let (_, _, alone) = gamepad_intent(set(&[RightTrigger]), set(&[RightTrigger, South]), 0.0);
        assert_eq!(alone, Buttons::JUMP);
    }

    #[test]
    fn pad_stick_has_a_deadzone_and_guard_is_held() {
        use GamepadButton::*;
        assert_eq!(gamepad_intent(set(&[]), set(&[]), 0.2).0, 0);
        assert_eq!(gamepad_intent(set(&[]), set(&[]), -0.6).0, -1);
        assert_eq!(gamepad_intent(set(&[DPadRight]), set(&[]), 0.0).0, 1);
        let (_, held, pressed) = gamepad_intent(set(&[LeftTrigger]), set(&[LeftTrigger]), 0.0);
        assert_eq!((held, pressed), (Buttons::GUARD, Buttons::NONE));
    }
}
