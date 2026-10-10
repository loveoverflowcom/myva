//! Ảnh chụp trạng thái cho host (bridge web, test trình duyệt). Chỉ đọc; không có lệnh nào ở
//! đây đổi được trận. Không serialize ECS world hay state nội bộ của lõi.

use myva_sim::battle::Tally;
use myva_sim::{Fighter, State};

use crate::scene::px;
use crate::session::Session;
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

fn state_kind(fighter: &Fighter) -> &'static str {
    match fighter.state {
        State::Neutral if fighter.is_grounded() => "neutral",
        State::Neutral => "airborne",
        State::Attack { .. } => "attack",
        State::Guard { .. } => "guard",
        State::Dash { .. } => "dash",
        State::Hitstun { .. } => "hitstun",
        State::GuardBreak { .. } => "guard_break",
        State::Downed => "downed",
    }
}

fn fighter_json(fighter: &Fighter) -> String {
    let action = fighter
        .action()
        .map_or("null".to_owned(), |(_, spec, phase)| {
            format!(
                "{{\"name\":\"{}\",\"phase\":\"{phase:?}\"}}",
                ascii(spec.name)
            )
        });
    format!(
        "{{\"x\":{:.1},\"y\":{:.1},\"facing\":{},\"hp\":{},\"max_hp\":{},\"stamina\":{},\"energy\":{},\"mach\":{},\"cooldowns\":[{},{},{}],\"state\":\"{}\",\"action\":{action}}}",
        px(fighter.x),
        px(fighter.y),
        fighter.facing,
        fighter.hp,
        fighter.kit.body.max_hp,
        fighter.stamina.points(),
        fighter.energy.points(),
        fighter.mach.points(),
        fighter.cooldowns[0],
        fighter.cooldowns[1],
        fighter.cooldowns[2],
        state_kind(fighter),
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

/// JSON một dòng, chỉ gồm số, boolean và chuỗi ASCII không cần escape.
pub fn json(session: &Session, host: HostStatus) -> String {
    let battle = &session.battle;
    let world = battle.world();
    let (player, rival) = (battle.player(), battle.rival());
    let outcome = battle
        .outcome()
        .map_or("null".to_owned(), |o| format!("\"{}\"", o.id()));
    let (verified, verified_ticks) = match &session.verification {
        None => ("null", 0),
        Some(Ok(v)) => ("true", v.ticks),
        Some(Err(_)) => ("false", 0),
    };
    let phase = battle.boss_phase().map_or(0, |phase| phase as u8 + 1);
    let sparring = battle
        .sparring_bot()
        .map_or("null".to_owned(), |on| on.to_string());
    format!(
        "{{\"v\":2,\"ready\":{},\"paused\":{},\"updates\":{},\"touch\":{},\"gamepads\":{},\"mode\":\"{}\",\"round\":{},\"tick\":{},\"autopilot\":{},\"sparring\":{sparring},\"hitboxes\":{},\"phase\":{phase},\"projectiles\":{},\"outcome\":{outcome},\"settled\":{},\"verified\":{verified},\"verified_ticks\":{verified_ticks},\"hash\":\"{:016x}\",\"presses\":{},\"player\":{},\"rival\":{},\"tally\":{{\"player\":{},\"rival\":{}}}}}",
        host.ready,
        host.paused,
        host.updates,
        host.touch,
        host.gamepads,
        battle.mode().id(),
        battle.round(),
        world.tick(),
        battle.autopilot(),
        session.show_boxes,
        world.projectiles().len(),
        battle.is_settled(),
        world.state_hash(),
        session.applied_presses,
        fighter_json(world.fighter(player)),
        fighter_json(world.fighter(rival)),
        tally_json(battle.tally(player)),
        tally_json(battle.tally(rival)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use myva_sim::battle::Mode;

    #[test]
    fn telemetry_is_plain_ascii_json() {
        let session = Session::new(Mode::Boss, false);
        let text = json(&session, HostStatus::default());
        assert!(text.is_ascii());
        assert!(text.starts_with("{\"v\":2,"));
        assert!(text.contains("\"mode\":\"boss\""));
        assert!(text.contains("\"outcome\":null"));
        assert!(text.contains("\"phase\":1"));
        assert_eq!(
            text.matches('{').count(),
            text.matches('}').count(),
            "ngoặc cân bằng"
        );
        assert!(!text.contains("NaN"));
    }
}
