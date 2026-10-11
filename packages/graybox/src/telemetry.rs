//! Ảnh chụp trạng thái cho host (bridge web, test trình duyệt). Chỉ đọc; không có lệnh nào ở
//! đây đổi được trận. Không serialize ECS world hay state nội bộ của lõi.

use myva_sim::State;
use myva_sim::battle::Tally;
use myva_sim::snapshot::FighterView;

use crate::art::ArtSummary;
use crate::fight::{ArenaView, Match};
use crate::scene::px;
use crate::text::ascii;

/// Trạng thái host tự biết: vòng đời runtime, thiết bị đang nối.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostStatus {
    pub ready: bool,
    pub paused: bool,
    pub updates: u64,
    pub touch: bool,
    pub gamepads: usize,
}

fn state_kind(fighter: &FighterView) -> &'static str {
    match fighter.state {
        State::Neutral if fighter.grounded => "neutral",
        State::Neutral => "airborne",
        State::Attack { .. } => "attack",
        State::Guard { .. } => "guard",
        State::Dash { .. } => "dash",
        State::Hitstun { .. } => "hitstun",
        State::GuardBreak { .. } => "guard_break",
        State::Escape { .. } => "escape",
        State::Downed => "downed",
    }
}

fn fighter_json(fighter: Option<&FighterView>) -> String {
    let Some(fighter) = fighter else {
        return "null".to_owned();
    };
    let action =
        fighter
            .action
            .zip(fighter.action_spec())
            .map_or("null".to_owned(), |(action, spec)| {
                format!(
                    "{{\"name\":\"{}\",\"phase\":\"{:?}\"}}",
                    ascii(spec.name),
                    action.phase
                )
            });
    format!(
        "{{\"x\":{:.1},\"y\":{:.1},\"facing\":{},\"hp\":{},\"max_hp\":{},\"stamina\":{},\"energy\":{},\"mach\":{},\"cooldowns\":[{},{},{}],\"state\":\"{}\",\"action\":{action},\"contact\":\"{:?}\",\"denial\":\"{:?}\",\"escape_cooldown\":{},\"control_ticks\":{},\"control_damage\":{}}}",
        px(fighter.x),
        px(fighter.y),
        fighter.facing,
        fighter.hp,
        fighter.max_hp,
        fighter.stamina.points(),
        fighter.energy.points(),
        fighter.mach.points(),
        fighter.cooldowns[0],
        fighter.cooldowns[1],
        fighter.cooldowns[2],
        state_kind(fighter),
        fighter.feedback.contact,
        fighter.feedback.denial,
        fighter.feedback.escape_cooldown,
        fighter.feedback.control.locked_ticks,
        fighter.feedback.control.damage,
    )
}

fn tally_json(tally: &Tally) -> String {
    format!(
        "{{\"actions\":{},\"hits\":{},\"damage_dealt\":{},\"damage_taken\":{},\"blocks\":{},\"perfect_blocks\":{},\"counters\":{},\"guard_breaks\":{}}}",
        tally.actions,
        tally.hits,
        tally.damage_dealt,
        tally.damage_taken,
        tally.blocks,
        tally.perfect_blocks,
        tally.counters,
        tally.guard_breaks
    )
}

fn art_json(art: &ArtSummary) -> String {
    let failure = art
        .failure
        .as_deref()
        .filter(|code| code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'))
        .map_or("null".to_owned(), |code| format!("\"{code}\""));
    format!(
        "{{\"status\":\"{}\",\"visible\":{},\"sheets\":{},\"failure\":{failure}}}",
        art.status, art.visible, art.sheets
    )
}

/// JSON một dòng, chỉ gồm số, boolean và chuỗi ASCII không cần escape.
pub fn json(game: &Match, arena: &ArenaView, host: HostStatus, art: &ArtSummary) -> String {
    let bout = &game.bout;
    let (player, rival) = (bout.player(), bout.rival());
    let outcome = bout
        .outcome()
        .map_or("null".to_owned(), |o| format!("\"{}\"", o.id()));
    let (verified, verified_ticks) = match &game.verification {
        None => ("null", 0),
        Some(Ok(v)) => ("true", v.ticks),
        Some(Err(_)) => ("false", 0),
    };
    let phase = arena.boss_phase.map_or(0, |phase| phase as u8 + 1);
    let sparring = arena
        .sparring
        .map_or("null".to_owned(), |on| on.to_string());
    format!(
        "{{\"v\":2,\"ready\":{},\"paused\":{},\"updates\":{},\"touch\":{},\"gamepads\":{},\"mode\":\"{}\",\"round\":{},\"tick\":{},\"autopilot\":{},\"sparring\":{sparring},\"hitboxes\":{},\"phase\":{phase},\"projectiles\":{},\"outcome\":{outcome},\"settled\":{},\"verified\":{verified},\"verified_ticks\":{verified_ticks},\"hash\":\"{:016x}\",\"presses\":{},\"art\":{},\"player\":{},\"rival\":{},\"tally\":{{\"player\":{},\"rival\":{}}}}}",
        host.ready,
        host.paused,
        host.updates,
        host.touch,
        host.gamepads,
        bout.mode().id(),
        bout.round(),
        arena.tick,
        bout.autopilot(),
        game.show_boxes,
        arena.projectiles.len(),
        bout.is_settled(arena.tick),
        arena.hash,
        game.applied_presses,
        art_json(art),
        fighter_json(arena.fighter(player)),
        fighter_json(arena.fighter(rival)),
        tally_json(bout.tally(player)),
        tally_json(bout.tally(rival)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use myva_sim::battle::{Battle, Mode};

    #[test]
    fn telemetry_is_plain_ascii_json() {
        let battle = Battle::new(Mode::Boss, 1);
        let game = Match::new(battle.bout().clone());
        let art = ArtSummary {
            status: "failed",
            failure: Some("hash_mismatch".to_owned()),
            ..ArtSummary::default()
        };
        let text = json(&game, &ArenaView::of(&battle), HostStatus::default(), &art);
        assert!(text.contains("\"art\":{\"status\":\"failed\",\"visible\":false,\"sheets\":0,\"failure\":\"hash_mismatch\"}"));
        assert!(text.is_ascii());
        assert!(text.starts_with("{\"v\":2,"));
        assert!(text.contains("\"mode\":\"boss\""));
        assert!(text.contains("\"outcome\":null"));
        assert!(text.contains("\"phase\":1"));
        assert!(text.contains("\"hp\":"));
        assert_eq!(
            text.matches('{').count(),
            text.matches('}').count(),
            "ngoặc cân bằng"
        );
        assert!(!text.contains("NaN"));
    }
}
