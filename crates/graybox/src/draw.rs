//! Vẽ graybox: arena, khối nhân vật, hitbox, HUD và log sự kiện.
//!
//! Chữ viết không dấu: font mặc định của Macroquad thiếu glyph tiếng Việt, và dự án chưa chọn
//! font có giấy phép rõ ràng (CONTRIBUTING.md).

use macroquad::prelude::*;
use myva_sim::fighter::{ARENA_WIDTH, MAX_HP};
use myva_sim::kit::Rect as SimRect;
use myva_sim::meter::Meter;
use myva_sim::tick::TICK_HZ;
use myva_sim::{Fighter, Phase, State};

use crate::Match;

const BACKGROUND: Color = Color::new(0.11, 0.12, 0.15, 1.0);
const GROUND: Color = Color::new(0.55, 0.57, 0.62, 1.0);
const TEXT: Color = Color::new(0.88, 0.89, 0.92, 1.0);
const DIM: Color = Color::new(0.55, 0.57, 0.62, 1.0);
const PLAYER: Color = Color::new(0.30, 0.56, 0.92, 1.0);
const RIVAL: Color = Color::new(0.62, 0.64, 0.70, 1.0);

/// Chiếu tọa độ mô phỏng (mili-pixel, y hướng lên) sang màn hình.
struct View {
    left: f32,
    ground: f32,
    scale: f32,
}

impl View {
    fn new() -> Self {
        let margin = 40.0;
        Self {
            left: margin,
            ground: screen_height() * 0.72,
            scale: (screen_width() - 2.0 * margin) / ARENA_WIDTH as f32,
        }
    }

    fn x(&self, x: i32) -> f32 {
        self.left + x as f32 * self.scale
    }

    fn y(&self, y: i32) -> f32 {
        self.ground - y as f32 * self.scale
    }

    fn rect(&self, r: &SimRect) -> (f32, f32, f32, f32) {
        (
            self.x(r.x0),
            self.y(r.y1),
            (r.x1 - r.x0) as f32 * self.scale,
            (r.y1 - r.y0) as f32 * self.scale,
        )
    }
}

pub(crate) fn frame(game: &Match, show_boxes: bool) {
    clear_background(BACKGROUND);
    let view = View::new();
    draw_line(
        view.x(0),
        view.ground,
        view.x(ARENA_WIDTH),
        view.ground,
        2.0,
        GROUND,
    );

    for projectile in game.world.projectiles() {
        let (x, y, w, h) = view.rect(&projectile.rect);
        let color = if projectile.owner == game.player {
            PLAYER
        } else {
            RIVAL
        };
        draw_rectangle(x, y, w, h, color);
        draw_rectangle_lines(x, y, w, h, 2.0, WHITE);
    }
    for fighter in game.world.fighters() {
        draw_fighter(
            &view,
            fighter,
            game.name(fighter.id),
            fighter.id == game.player,
            show_boxes,
        );
    }

    let hud_width = 300.0;
    for (i, fighter) in game.world.fighters().iter().enumerate() {
        let x = if i == 0 {
            20.0
        } else {
            screen_width() - hud_width - 20.0
        };
        draw_hud(x, 24.0, hud_width, game.name(fighter.id), fighter);
    }

    let title = format!(
        "MyVa - Than Mach graybox  |  tick {}  |  hash {:016x}",
        game.world.tick(),
        game.world.state_hash()
    );
    centered(&title, 24.0, 18.0, DIM);
    if game.is_ko() {
        centered("KO - F5 de dau lai", screen_height() * 0.35, 40.0, TEXT);
    }

    let mut y = view.ground + 40.0;
    for line in &game.log {
        draw_text(line, 20.0, y, 20.0, TEXT);
        y += 22.0;
    }
    let bot = if game.rival_bot.is_some() {
        "bat"
    } else {
        "tat"
    };
    let controls = format!(
        "A/D di chuyen  Space nhay  Shift luot  L do  J/K nhe/nang  Q/E/R thuat  |  B bot ({bot})  H hitbox  F5 dau lai"
    );
    draw_text(&controls, 20.0, screen_height() - 16.0, 18.0, DIM);
}

fn draw_fighter(view: &View, fighter: &Fighter, name: &str, is_player: bool, show_boxes: bool) {
    let base = if is_player { PLAYER } else { RIVAL };
    let mut color = match (fighter.state, fighter.action()) {
        (_, Some((_, _, Phase::Startup))) => YELLOW,
        (_, Some((_, _, Phase::Active))) => RED,
        (_, Some((_, _, Phase::Recovery))) => ORANGE,
        (State::Guard { .. }, _) => SKYBLUE,
        (State::Hitstun { .. }, _) => MAGENTA,
        (State::GuardBreak { .. }, _) => VIOLET,
        (State::Downed, _) => DARKGRAY,
        _ => base,
    };
    if fighter.is_invulnerable() && fighter.state != State::Downed {
        color.a = 0.35;
    }

    let (x, y, w, h) = view.rect(&fighter.hurtbox());
    draw_rectangle(x, y, w, h, color);
    if fighter.guard_active().is_some() {
        let outline = if fighter.guard_active() == Some(true) {
            WHITE
        } else {
            SKYBLUE
        };
        draw_rectangle_lines(x - 3.0, y - 3.0, w + 6.0, h + 6.0, 3.0, outline);
    }
    // Tên giữ màu riêng để phân biệt hai bên khi cùng ra đòn.
    let tag = measure_text(name, None, 18, 1.0).width;
    draw_text(name, x + (w - tag) / 2.0, y - 8.0, 18.0, base);
    // Hướng mặt.
    let eye_x = if fighter.facing > 0 {
        x + w - 8.0
    } else {
        x + 2.0
    };
    draw_rectangle(eye_x, y + 8.0, 6.0, 6.0, BACKGROUND);

    if show_boxes {
        draw_rectangle_lines(x, y, w, h, 1.0, GREEN);
        if let Some(attack) = fighter.attack_box() {
            let (ax, ay, aw, ah) = view.rect(&attack);
            draw_rectangle(ax, ay, aw, ah, Color::new(1.0, 0.2, 0.2, 0.35));
            draw_rectangle_lines(ax, ay, aw, ah, 1.0, RED);
        }
    }
}

fn draw_hud(x: f32, y: f32, width: f32, name: &str, fighter: &Fighter) {
    let header = format!("{name}  {}", ascii(fighter.kit.lineage));
    draw_text(&header, x, y, 22.0, TEXT);
    let mut row = y + 10.0;
    bar(
        x,
        row,
        width,
        fighter.hp as f32 / MAX_HP as f32,
        RED,
        &format!("HP {}", fighter.hp),
    );
    for (label, meter, color) in [
        ("Suc ben", fighter.stamina, GREEN),
        ("Nang luong", fighter.energy, SKYBLUE),
        ("Mach", fighter.mach, GOLD),
    ] {
        row += 20.0;
        bar(
            x,
            row,
            width,
            fill(meter),
            color,
            &format!("{label} {}", meter.points()),
        );
    }

    let cooldowns: Vec<String> = ["Q", "E", "R"]
        .iter()
        .zip(fighter.cooldowns)
        .map(|(key, ticks)| {
            if ticks == 0 {
                format!("{key} san sang")
            } else {
                format!("{key} {:.1}s", ticks as f32 / TICK_HZ as f32)
            }
        })
        .collect();
    draw_text(cooldowns.join("  "), x, row + 34.0, 18.0, DIM);
    draw_text(state_label(fighter), x, row + 54.0, 18.0, DIM);
}

fn fill(meter: Meter) -> f32 {
    meter.sub() as f32 / meter.max_sub().max(1) as f32
}

fn bar(x: f32, y: f32, width: f32, fraction: f32, color: Color, label: &str) {
    let height = 16.0;
    draw_rectangle(x, y, width, height, Color::new(1.0, 1.0, 1.0, 0.08));
    draw_rectangle(x, y, width * fraction.clamp(0.0, 1.0), height, color);
    // Bóng chữ giúp nhãn đọc được cả trên phần đầy lẫn phần trống của thanh.
    draw_text(label, x + 7.0, y + 13.5, 16.0, BLACK);
    draw_text(label, x + 6.0, y + 12.5, 16.0, WHITE);
}

fn state_label(fighter: &Fighter) -> String {
    match (fighter.state, fighter.action()) {
        (_, Some((_, spec, phase))) => format!("{} [{phase:?}]", ascii(spec.name)),
        (State::Neutral, _) if !fighter.is_grounded() => "Tren khong".to_owned(),
        (State::Neutral, _) => "Trung tinh".to_owned(),
        (State::Guard { .. }, _) => "Do".to_owned(),
        (State::Dash { .. }, _) => "Luot".to_owned(),
        (State::Hitstun { .. }, _) => "Choang".to_owned(),
        (State::GuardBreak { .. }, _) => "Vo the do".to_owned(),
        (State::Downed, _) => "Bi ha".to_owned(),
        (State::Attack { .. }, None) => unreachable!("trạng thái đánh luôn có đòn"),
    }
}

fn centered(text: &str, y: f32, size: f32, color: Color) {
    let width = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, (screen_width() - width) / 2.0, y, size, color);
}

/// Bỏ dấu tiếng Việt để hiển thị bằng font mặc định.
pub(crate) fn ascii(text: &str) -> String {
    const GROUPS: [(&str, char); 14] = [
        ("àáạảãâầấậẩẫăằắặẳẵ", 'a'),
        ("ÀÁẠẢÃÂẦẤẬẨẪĂẰẮẶẲẴ", 'A'),
        ("èéẹẻẽêềếệểễ", 'e'),
        ("ÈÉẸẺẼÊỀẾỆỂỄ", 'E'),
        ("ìíịỉĩ", 'i'),
        ("ÌÍỊỈĨ", 'I'),
        ("òóọỏõôồốộổỗơờớợởỡ", 'o'),
        ("ÒÓỌỎÕÔỒỐỘỔỖƠỜỚỢỞỠ", 'O'),
        ("ùúụủũưừứựửữ", 'u'),
        ("ÙÚỤỦŨƯỪỨỰỬỮ", 'U'),
        ("ỳýỵỷỹ", 'y'),
        ("ỲÝỴỶỸ", 'Y'),
        ("đ", 'd'),
        ("Đ", 'D'),
    ];
    text.chars()
        .map(|c| {
            GROUPS
                .iter()
                .find(|(group, _)| group.contains(c))
                .map_or(c, |&(_, plain)| plain)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::ascii;

    #[test]
    fn strips_vietnamese_diacritics() {
        assert_eq!(ascii("Long Lưu"), "Long Luu");
        assert_eq!(ascii("Triều Dâng · Hồi Thế"), "Trieu Dang · Hoi The");
        assert_eq!(ascii("Đỡ hoàn hảo"), "Do hoan hao");
    }
}
