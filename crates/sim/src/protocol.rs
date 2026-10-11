//! Schema lệnh/sự kiện có version giữa client, server và adapter ECS (architecture.md §6).
//!
//! Lệnh là **ý định**: hướng di chuyển, giữ đỡ và tối đa một hành động mỗi tick. Không có trường
//! damage, kết quả trúng, phần thưởng hay số dư; những thứ đó chỉ do mô phỏng và server quyết
//! định (combat.md §4). Định dạng truyền tải chưa chọn; các kiểu ở đây là hợp đồng logic.
//!
//! **Định danh.** [`EntityRef`] là ID miền ổn định trong một `World` và không bao giờ tái sử
//! dụng. Handle `Entity` của Bevy chỉ sống trong một process, nên không xuất hiện ở đây.
//!
//! **Tick nhận lệnh.** Server tự gán tick áp dụng khi nhận lệnh, không lùi về tick client đề xuất
//! (combat.md §6). `CommandEnvelope::tick` là tick client đã dự đoán, chỉ dùng để loại lệnh quá
//! cũ/quá sớm và để client hòa giải qua [`CommandAck`].

use std::collections::BTreeMap;

use crate::fighter::FighterId;
use crate::input::{Buttons, InputFrame, Intent};
use crate::npc::NpcId;
use crate::projectile::ProjectileId;
use crate::tick::Tick;
use crate::world::Event;

pub const PROTOCOL_VERSION: u16 = 1;

/// Lệnh có tick dự đoán lệch quá ngần này so với server bị từ chối (500 ms ở 60 Hz, GT).
pub const INPUT_WINDOW: Tick = 30;

/// Khi nhiều lệnh của một tác nhân đến cùng lúc, chúng được xếp mỗi tick một lệnh, tối đa ngần
/// này tick phía trước; quá nữa thì từ chối để không tích độ trễ.
pub const MAX_BACKLOG: Tick = 8;

/// Phiên điều khiển một tác nhân. Reconnect cấp epoch mới; lệnh của epoch cũ bị từ chối.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionEpoch(pub u32);

impl SessionEpoch {
    /// Bộ não quái/boss chạy trên server; người chơi dùng epoch khác 0.
    pub const AI: Self = Self(0);
}

/// ID miền của mọi thực thể mô phỏng, dùng cho mạng, replay và ánh xạ sang ECS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityRef {
    Fighter(FighterId),
    Projectile(ProjectileId),
    Npc(NpcId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Attack {
    Light,
    Heavy,
}

/// Hành động bấm một lần. `Move` và `Guard` là trạng thái giữ nên nằm trong [`CommandFrame`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Jump,
    Dash,
    Attack(Attack),
    /// Ô thuật `0..=2`.
    Skill(u8),
    Interact,
}

/// Ý định của một tác nhân trong một tick: Move, Guard và tối đa một Action.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CommandFrame {
    /// -1 trái, 0 đứng, 1 phải.
    pub move_x: i8,
    pub guard: bool,
    pub action: Option<Action>,
}

impl CommandFrame {
    pub const IDLE: Self = Self {
        move_x: 0,
        guard: false,
        action: None,
    };

    /// Khung input lõi tương đương; `seq` do cổng nhận lệnh giữ.
    pub fn to_input(self, seq: u32) -> InputFrame {
        InputFrame {
            seq,
            move_x: self.move_x,
            held: if self.guard {
                Buttons::GUARD
            } else {
                Buttons::NONE
            },
            pressed: self.action.map_or(Buttons::NONE, |action| match action {
                Action::Jump => Buttons::JUMP,
                Action::Dash => Buttons::DASH,
                Action::Attack(Attack::Light) => Buttons::LIGHT,
                Action::Attack(Attack::Heavy) => Buttons::HEAVY,
                Action::Skill(0) => Buttons::SKILL1,
                Action::Skill(1) => Buttons::SKILL2,
                Action::Skill(_) => Buttons::SKILL3,
                Action::Interact => Buttons::INTERACT,
            }),
        }
    }

    /// Rút gọn một khung input lõi; lõi chỉ đọc hướng, nút đỡ đang giữ và một ý định theo ưu
    /// tiên, nên kết quả mô phỏng không đổi.
    pub fn from_input(frame: &InputFrame) -> Self {
        Self {
            move_x: frame.move_x.signum(),
            guard: frame.held.contains(Buttons::GUARD),
            action: frame.intent().map(|intent| match intent {
                Intent::Skill(slot) => Action::Skill(slot),
                Intent::Heavy => Action::Attack(Attack::Heavy),
                Intent::Light => Action::Attack(Attack::Light),
                Intent::Dash => Action::Dash,
                Intent::Jump => Action::Jump,
                Intent::Interact => Action::Interact,
            }),
        }
    }

    fn validate(&self) -> Result<(), Rejection> {
        if !(-1..=1).contains(&self.move_x) {
            return Err(Rejection::Malformed("move_x ngoài -1..=1"));
        }
        if let Some(Action::Skill(slot)) = self.action
            && slot > 2
        {
            return Err(Rejection::Malformed("ô thuật ngoài 0..=2"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CommandEnvelope {
    pub protocol: u16,
    pub session: SessionEpoch,
    pub actor: FighterId,
    /// Tăng dần theo tác nhân suốt đời `World`; reconnect tiếp tục sau `last_seq` trong snapshot.
    pub seq: u32,
    /// Tick client dự đoán lệnh được áp dụng.
    pub tick: Tick,
    pub frame: CommandFrame,
}

impl CommandEnvelope {
    pub fn new(
        session: SessionEpoch,
        actor: FighterId,
        seq: u32,
        tick: Tick,
        frame: CommandFrame,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            session,
            actor,
            seq,
            tick,
            frame,
        }
    }
}

/// Server nhận lệnh `seq` và sẽ áp dụng ở `accepted_tick`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CommandAck {
    pub actor: FighterId,
    pub seq: u32,
    pub accepted_tick: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rejection {
    UnsupportedProtocol(u16),
    /// Tác nhân chưa được gắn phiên điều khiển nào.
    UnknownActor(FighterId),
    /// Phiên không điều khiển tác nhân này, gồm cả epoch cũ sau reconnect.
    NotController {
        actor: FighterId,
        session: SessionEpoch,
    },
    StaleSeq {
        seq: u32,
        last: u32,
    },
    Late {
        tick: Tick,
        now: Tick,
    },
    TooEarly {
        tick: Tick,
        now: Tick,
    },
    Backlog {
        actor: FighterId,
    },
    Malformed(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Binding {
    session: SessionEpoch,
    last_seq: Option<u32>,
    next_free: Tick,
}

/// Cổng nhận lệnh có kiểm tra phiên và thứ tự. Lệnh của một tick được trả theo `FighterId`, nên
/// thứ tự đến của các tác nhân khác nhau không đổi kết quả.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandQueue {
    bindings: BTreeMap<FighterId, Binding>,
    pending: BTreeMap<(Tick, FighterId), (u32, CommandFrame)>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Giao quyền điều khiển `actor` cho `session`. Đổi phiên hủy mọi lệnh đang chờ của phiên cũ;
    /// `seq` vẫn phải tăng tiếp.
    pub fn bind(&mut self, actor: FighterId, session: SessionEpoch) {
        let binding = self.bindings.entry(actor).or_insert(Binding {
            session,
            last_seq: None,
            next_free: 0,
        });
        if binding.session != session {
            binding.session = session;
            binding.next_free = 0;
            self.pending.retain(|&(_, id), _| id != actor);
        }
    }

    pub fn controller(&self, actor: FighterId) -> Option<SessionEpoch> {
        self.bindings.get(&actor).map(|b| b.session)
    }

    pub fn submit(
        &mut self,
        envelope: CommandEnvelope,
        now: Tick,
    ) -> Result<CommandAck, Rejection> {
        if envelope.protocol != PROTOCOL_VERSION {
            return Err(Rejection::UnsupportedProtocol(envelope.protocol));
        }
        envelope.frame.validate()?;
        let actor = envelope.actor;
        let binding = self
            .bindings
            .get_mut(&actor)
            .ok_or(Rejection::UnknownActor(actor))?;
        if binding.session != envelope.session {
            return Err(Rejection::NotController {
                actor,
                session: envelope.session,
            });
        }
        if let Some(last) = binding.last_seq
            && envelope.seq <= last
        {
            return Err(Rejection::StaleSeq {
                seq: envelope.seq,
                last,
            });
        }
        if envelope.tick.saturating_add(INPUT_WINDOW) < now {
            return Err(Rejection::Late {
                tick: envelope.tick,
                now,
            });
        }
        if envelope.tick > now.saturating_add(INPUT_WINDOW) {
            return Err(Rejection::TooEarly {
                tick: envelope.tick,
                now,
            });
        }
        let accepted_tick = now.max(binding.next_free);
        if accepted_tick > now + MAX_BACKLOG {
            return Err(Rejection::Backlog { actor });
        }
        binding.last_seq = Some(envelope.seq);
        binding.next_free = accepted_tick + 1;
        self.pending
            .insert((accepted_tick, actor), (envelope.seq, envelope.frame));
        Ok(CommandAck {
            actor,
            seq: envelope.seq,
            accepted_tick,
        })
    }

    /// Lấy lệnh áp dụng ở `tick` (và mọi lệnh còn sót của tick cũ), theo thứ tự tick rồi
    /// `FighterId`.
    pub fn drain(&mut self, tick: Tick) -> Vec<(FighterId, InputFrame)> {
        let later = self.pending.split_off(&(tick + 1, FighterId(0)));
        let due = std::mem::replace(&mut self.pending, later);
        due.into_iter()
            .map(|((_, actor), (seq, frame))| (actor, frame.to_input(seq)))
            .collect()
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
}

/// ID duy nhất của một sự kiện: tick và vị trí trong tick. Client dùng để bỏ sự kiện nhận lặp
/// qua snapshot hoặc retransmit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId {
    pub tick: Tick,
    pub index: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EventRecord {
    pub id: EventId,
    pub event: Event,
}

/// Kết quả một tick authoritative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickReport {
    pub protocol: u16,
    /// Tick vừa chạy.
    pub tick: Tick,
    pub events: Vec<EventRecord>,
    /// Hash trạng thái sau tick, chỉ so được trong cùng build/target.
    pub hash: u64,
}

impl TickReport {
    pub fn new(tick: Tick, events: Vec<Event>, hash: u64) -> Self {
        let events = events
            .into_iter()
            .enumerate()
            .map(|(index, event)| EventRecord {
                id: EventId {
                    tick,
                    index: u16::try_from(index).expect("quá nhiều sự kiện trong một tick"),
                },
                event,
            })
            .collect();
        Self {
            protocol: PROTOCOL_VERSION,
            tick,
            events,
            hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: FighterId = FighterId(0);
    const B: FighterId = FighterId(1);
    const ONE: SessionEpoch = SessionEpoch(1);
    const TWO: SessionEpoch = SessionEpoch(2);

    fn light() -> CommandFrame {
        CommandFrame {
            action: Some(Action::Attack(Attack::Light)),
            ..CommandFrame::IDLE
        }
    }

    fn queue() -> CommandQueue {
        let mut queue = CommandQueue::new();
        queue.bind(A, ONE);
        queue.bind(B, TWO);
        queue
    }

    #[test]
    fn rejects_bad_headers_and_payloads() {
        let mut q = queue();
        let ok = CommandEnvelope::new(ONE, A, 1, 10, light());
        let wrong_version = CommandEnvelope { protocol: 99, ..ok };
        assert_eq!(
            q.submit(wrong_version, 10),
            Err(Rejection::UnsupportedProtocol(99))
        );
        assert_eq!(
            q.submit(CommandEnvelope { actor: B, ..ok }, 10),
            Err(Rejection::NotController {
                actor: B,
                session: ONE
            }),
            "phiên không được điều khiển nhân vật của người khác"
        );
        assert_eq!(
            q.submit(
                CommandEnvelope {
                    actor: FighterId(7),
                    ..ok
                },
                10
            ),
            Err(Rejection::UnknownActor(FighterId(7)))
        );
        let far = CommandFrame {
            move_x: 5,
            ..light()
        };
        assert!(matches!(
            q.submit(CommandEnvelope { frame: far, ..ok }, 10),
            Err(Rejection::Malformed(_))
        ));
        let slot = CommandFrame {
            action: Some(Action::Skill(3)),
            ..CommandFrame::IDLE
        };
        assert!(matches!(
            q.submit(CommandEnvelope { frame: slot, ..ok }, 10),
            Err(Rejection::Malformed(_))
        ));
        assert_eq!(q.pending_len(), 0, "lệnh bị từ chối không chiếm seq");
        assert!(q.submit(ok, 10).is_ok());
    }

    #[test]
    fn seq_must_increase_and_ticks_stay_in_window() {
        let mut q = queue();
        q.submit(CommandEnvelope::new(ONE, A, 5, 100, light()), 100)
            .unwrap();
        assert_eq!(
            q.submit(CommandEnvelope::new(ONE, A, 5, 100, light()), 100),
            Err(Rejection::StaleSeq { seq: 5, last: 5 })
        );
        assert_eq!(
            q.submit(CommandEnvelope::new(ONE, A, 6, 69, light()), 100),
            Err(Rejection::Late { tick: 69, now: 100 })
        );
        assert_eq!(
            q.submit(CommandEnvelope::new(ONE, A, 6, 131, light()), 100),
            Err(Rejection::TooEarly {
                tick: 131,
                now: 100
            })
        );
    }

    #[test]
    fn server_assigns_tick_and_spreads_bursts() {
        let mut q = queue();
        // Client đề xuất tick cũ hơn; server vẫn áp dụng từ tick hiện tại, không lùi lại.
        let ack = q
            .submit(CommandEnvelope::new(ONE, A, 1, 95, light()), 100)
            .unwrap();
        assert_eq!(ack.accepted_tick, 100);
        for seq in 2..=9 {
            let ack = q
                .submit(
                    CommandEnvelope::new(ONE, A, seq, 100, CommandFrame::IDLE),
                    100,
                )
                .unwrap();
            assert_eq!(ack.accepted_tick, 99 + seq);
        }
        assert_eq!(
            q.submit(
                CommandEnvelope::new(ONE, A, 10, 100, CommandFrame::IDLE),
                100
            ),
            Err(Rejection::Backlog { actor: A })
        );
        assert_eq!(q.drain(100), [(A, light().to_input(1))]);
        assert_eq!(q.drain(101).len(), 1);
    }

    #[test]
    fn drain_order_ignores_arrival_order() {
        let a = CommandEnvelope::new(ONE, A, 1, 0, light());
        let b = CommandEnvelope::new(TWO, B, 1, 0, CommandFrame::IDLE);
        let mut first = queue();
        first.submit(a, 0).unwrap();
        first.submit(b, 0).unwrap();
        let mut second = queue();
        second.submit(b, 0).unwrap();
        second.submit(a, 0).unwrap();
        assert_eq!(first.drain(0), second.drain(0));
    }

    #[test]
    fn reconnect_invalidates_old_epoch() {
        let mut q = queue();
        q.submit(CommandEnvelope::new(ONE, A, 1, 0, light()), 0)
            .unwrap();
        q.submit(CommandEnvelope::new(ONE, A, 2, 0, light()), 0)
            .unwrap();
        let three = SessionEpoch(3);
        q.bind(A, three);
        assert_eq!(q.pending_len(), 0, "lệnh chưa chạy của phiên cũ bị hủy");
        assert_eq!(
            q.submit(CommandEnvelope::new(ONE, A, 3, 0, light()), 0),
            Err(Rejection::NotController {
                actor: A,
                session: ONE
            })
        );
        assert_eq!(
            q.submit(CommandEnvelope::new(three, A, 2, 0, light()), 0),
            Err(Rejection::StaleSeq { seq: 2, last: 2 }),
            "phiên mới tiếp tục seq, không phát lại seq đã nhận"
        );
        assert!(
            q.submit(CommandEnvelope::new(three, A, 3, 0, light()), 0)
                .is_ok()
        );
    }

    #[test]
    fn command_frame_roundtrips_through_input() {
        let actions = [
            None,
            Some(Action::Jump),
            Some(Action::Dash),
            Some(Action::Attack(Attack::Light)),
            Some(Action::Attack(Attack::Heavy)),
            Some(Action::Skill(0)),
            Some(Action::Skill(1)),
            Some(Action::Skill(2)),
            Some(Action::Interact),
        ];
        for action in actions {
            for (move_x, guard) in [(-1, false), (0, true), (1, false)] {
                let frame = CommandFrame {
                    move_x,
                    guard,
                    action,
                };
                assert_eq!(CommandFrame::from_input(&frame.to_input(1)), frame);
            }
        }
    }

    #[test]
    fn event_ids_are_unique_within_a_tick() {
        let report = TickReport::new(
            7,
            vec![Event::Downed { fighter: A }, Event::Downed { fighter: B }],
            0,
        );
        let ids: Vec<_> = report.events.iter().map(|e| e.id).collect();
        assert_eq!(
            ids,
            [EventId { tick: 7, index: 0 }, EventId { tick: 7, index: 1 }]
        );
    }
}
