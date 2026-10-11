//! Lệnh từ client và người chơi giả lập vào cổng nhận lệnh của phiên.
//!
//! Thiết bị (bàn phím, gamepad, touch) chạy theo frame ở `Update` và chỉ ghi vào [`LocalInput`].
//! Mỗi tick cố định lấy đúng một lệnh: hướng/đỡ là trạng thái đang giữ, nút bấm được chốt từ lúc
//! bấm tới tick kế tiếp. Nhờ vậy 30 FPS không bấm đôi một hành động qua hai tick, còn 144 FPS không
//! làm mất cú bấm nằm giữa hai tick.

use bevy_ecs::prelude::*;
use myva_sim::fixture::Brawler;
use myva_sim::protocol::{Action, CommandEnvelope, CommandFrame, Rejection};
use myva_sim::tick::Tick;
use myva_sim::{Buttons, FighterId, InputFrame, SessionEpoch};

use crate::SimSession;

/// Lệnh bị cổng từ chối, để UI/log biết; lõi không chạy lệnh này.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandRejected {
    pub actor: FighterId,
    pub seq: u32,
    pub reason: Rejection,
}

/// Bộ gom input của người chơi cục bộ giữa các tick.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct LocalInput {
    actor: FighterId,
    session: SessionEpoch,
    seq: u32,
    move_x: i8,
    guard: bool,
    pressed: Buttons,
}

impl LocalInput {
    /// `last_seq` lấy từ snapshot khi vào lại phiên, để `seq` tiếp tục tăng.
    pub fn new(actor: FighterId, session: SessionEpoch, last_seq: Option<u32>) -> Self {
        Self {
            actor,
            session,
            seq: last_seq.unwrap_or(0),
            move_x: 0,
            guard: false,
            pressed: Buttons::NONE,
        }
    }

    pub fn actor(&self) -> FighterId {
        self.actor
    }

    pub fn set_move(&mut self, move_x: i8) {
        self.move_x = move_x.signum();
    }

    pub fn set_guard(&mut self, held: bool) {
        self.guard = held;
    }

    /// Chốt một cú bấm tới tick kế tiếp; nhiều cú bấm trong một tick chọn theo ưu tiên của lõi.
    pub fn press(&mut self, action: Action) {
        self.pressed |= CommandFrame {
            action: Some(action),
            ..CommandFrame::IDLE
        }
        .to_input(0)
        .pressed;
    }

    /// Thay toàn bộ ý định của tick kế tiếp bằng `frame` (bot lái hộ), bỏ cú bấm đang chốt.
    pub fn set_frame(&mut self, frame: CommandFrame) {
        self.release_all();
        self.set_move(frame.move_x);
        self.set_guard(frame.guard);
        if let Some(action) = frame.action {
            self.press(action);
        }
    }

    /// Có cú bấm đang chờ vào tick kế tiếp.
    pub fn has_press(&self) -> bool {
        !self.pressed.is_empty()
    }

    /// Mất focus hoặc mở bàn phím chat: thả mọi phím để nhân vật không tự đi/đánh.
    pub fn release_all(&mut self) {
        self.move_x = 0;
        self.guard = false;
        self.pressed = Buttons::NONE;
    }

    pub(crate) fn take(&mut self, tick: Tick) -> CommandEnvelope {
        self.seq += 1;
        let frame = CommandFrame::from_input(&InputFrame {
            seq: self.seq,
            move_x: self.move_x,
            held: if self.guard {
                Buttons::GUARD
            } else {
                Buttons::NONE
            },
            pressed: self.pressed,
        });
        self.pressed = Buttons::NONE;
        CommandEnvelope::new(self.session, self.actor, self.seq, tick, frame)
    }
}

/// Người chơi giả lập của fixture, chạy trong tick cố định như một client đọc snapshot.
#[derive(Resource, Clone, Debug)]
pub struct ScriptedPlayer(pub Brawler);

pub(crate) fn submit_local_input(
    input: Option<ResMut<LocalInput>>,
    mut session: ResMut<SimSession>,
    mut rejected: MessageWriter<CommandRejected>,
) {
    let Some(mut input) = input else {
        return;
    };
    let tick = session.get().world().tick();
    submit(&mut session, input.take(tick), &mut rejected);
}

pub(crate) fn submit_scripted_player(
    player: Option<ResMut<ScriptedPlayer>>,
    mut session: ResMut<SimSession>,
    mut rejected: MessageWriter<CommandRejected>,
) {
    let Some(mut player) = player else {
        return;
    };
    let command = player.0.command(session.get().world());
    submit(&mut session, command, &mut rejected);
}

fn submit(
    session: &mut SimSession,
    command: CommandEnvelope,
    rejected: &mut MessageWriter<CommandRejected>,
) {
    if let Err(reason) = session.session_mut().submit(command) {
        rejected.write(CommandRejected {
            actor: command.actor,
            seq: command.seq,
            reason,
        });
    }
}
