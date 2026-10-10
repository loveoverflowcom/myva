//! Chữ hiển thị trong canvas.
//!
//! Chữ viết không dấu: font mặc định của Bevy (`default_font`, FiraMono subset) chỉ có 95 glyph
//! ASCII, và dự án chưa chọn font có giấy phép rõ ràng (CONTRIBUTING.md). Chữ tiếng Việt đầy đủ
//! nằm ở shell Leptos, nơi trình duyệt có font hệ thống.

use myva_sim::State;
use myva_sim::boss::BossPhase;
use myva_sim::snapshot::FighterView;

/// Bỏ dấu tiếng Việt để hiển thị bằng font mặc định.
pub fn ascii(text: &str) -> String {
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

pub fn phase_label(phase: BossPhase) -> &'static str {
    match phase {
        BossPhase::Recognize => "Pha 1 - Nhan dien",
        BossPhase::Terrain => "Pha 2 - Doi dia hinh",
        BossPhase::Combine => "Pha 3 - Phoi hop quy luat",
    }
}

/// Trạng thái hiện tại; đòn đang ra kèm pha và thời gian còn lại của pha đó.
pub fn state_label(fighter: &FighterView) -> String {
    match (fighter.state, fighter.action.zip(fighter.action_spec())) {
        (State::Attack { .. }, Some((action, spec))) => {
            let (word, end) = match action.phase {
                myva_sim::Phase::Startup => ("bao", spec.startup),
                myva_sim::Phase::Active => ("danh", spec.startup + spec.active),
                myva_sim::Phase::Recovery => ("hoi", spec.total()),
            };
            format!(
                "{} - {word} {}",
                ascii(spec.name),
                seconds(end - action.elapsed)
            )
        }
        (State::Neutral, _) if !fighter.grounded => "Tren khong".to_owned(),
        (State::Neutral, _) => "Trung tinh".to_owned(),
        (State::Guard { .. }, _) => match fighter.guard {
            Some(true) => "Do (hoan hao)".to_owned(),
            Some(false) => "Do".to_owned(),
            None => "Dang dua the do".to_owned(),
        },
        (State::Dash { .. }, _) => "Luot".to_owned(),
        (State::Hitstun { .. }, _) => "Choang".to_owned(),
        (State::GuardBreak { .. }, _) => "Vo the do".to_owned(),
        (State::Downed, _) => "Bi ha".to_owned(),
        (State::Attack { .. }, None) => unreachable!("trạng thái đánh luôn có đòn"),
    }
}

/// Tick sang giây với một chữ số thập phân.
pub fn seconds(ticks: u32) -> String {
    let tenths = (u64::from(ticks) * 10).div_ceil(u64::from(myva_sim::tick::TICK_HZ));
    format!("{}.{}s", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_vietnamese_diacritics() {
        assert_eq!(ascii("Long Lưu"), "Long Luu");
        assert_eq!(ascii("Triều Dâng · Hồi Thế"), "Trieu Dang · Hoi The");
        assert_eq!(ascii("Đỡ hoàn hảo"), "Do hoan hao");
    }

    #[test]
    fn seconds_round_up_to_a_tenth() {
        assert_eq!(seconds(0), "0.0s");
        assert_eq!(seconds(1), "0.1s");
        assert_eq!(seconds(180), "3.0s");
        assert_eq!(seconds(181), "3.1s");
    }
}
