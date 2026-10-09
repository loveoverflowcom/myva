//! Nhân vật trong phòng thử: trạng thái, xử lý input và vật lý số nguyên
//! (combat.md §2–§5, §11).

use crate::input::{Buttons, InputFrame, Intent};
use crate::kit::{ActionKind, ActionSpec, HUMAN, Kit, PX, Phase, Rect};
use crate::meter::Meter;
use crate::tick::ms_to_ticks;
use crate::world::Event;

/// Sinh lực chuẩn hóa ở phòng thử.
pub const MAX_HP: u32 = HUMAN.max_hp;

pub const ARENA_WIDTH: i32 = 1_600 * PX;
/// Giới hạn vị trí của cơ thể chuẩn; cơ thể lớn hơn bị giới hạn theo bề rộng của nó.
pub const MIN_X: i32 = HUMAN.half_width;
pub const MAX_X: i32 = ARENA_WIDTH - HUMAN.half_width;

// Vật lý graybox (GT), mili-pixel mỗi tick.
pub const JUMP_VELOCITY: i32 = 14 * PX;
pub const GRAVITY: i32 = 800;

pub const GUARD_STARTUP: u32 = ms_to_ticks(100);
/// 90 ms đầu sau khi đỡ active, không tính từ lúc nhấn nút.
pub const PERFECT_GUARD: u32 = ms_to_ticks(90);
pub const GUARD_DRAIN_PER_SEC: u32 = 12;
pub const GUARD_BREAK: u32 = ms_to_ticks(500);

pub const DASH_STAMINA: u32 = 25;
pub const DASH_STARTUP: u32 = ms_to_ticks(50);
pub const DASH_MOVE: u32 = ms_to_ticks(220);
pub const DASH_RECOVERY: u32 = ms_to_ticks(140);
pub const DASH_TOTAL: u32 = DASH_STARTUP + DASH_MOVE + DASH_RECOVERY;
/// Khoảng bất tử nằm ở đầu đoạn di chuyển.
pub const DASH_IFRAMES: u32 = ms_to_ticks(100);
pub const DASH_SPEED: i32 = 12 * PX;

/// Buffer hành động ở cuối recovery.
pub const ACTION_BUFFER: u32 = ms_to_ticks(120);
/// Nhẹ 1 → 2 → 3 trong 100 ms cuối recovery.
pub const LIGHT_CHAIN_WINDOW: u32 = ms_to_ticks(100);
/// Nhẹ xác nhận trúng → thuật trong 80 ms sau hit.
pub const HIT_CANCEL_WINDOW: u32 = ms_to_ticks(80);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FighterId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    /// Đứng, đi hoặc đang nhảy/rơi mà không làm hành động khác.
    Neutral,
    Attack {
        action: ActionKind,
        elapsed: u32,
        /// Mỗi lần ra đòn có ID riêng; tập mục tiêu đã trúng gắn với nó.
        instance: u32,
        /// `elapsed` tại hit đầu tiên được server xác nhận.
        confirmed_at: Option<u32>,
    },
    Guard {
        elapsed: u32,
    },
    Dash {
        elapsed: u32,
        dir: i8,
    },
    Hitstun {
        remaining: u32,
    },
    GuardBreak {
        remaining: u32,
    },
    Downed,
}

enum Outcome {
    Started(Option<ActionKind>),
    /// Chưa tới cửa sổ hợp lệ; có thể buffer.
    NotYet,
    Refused,
}

#[derive(Clone, Debug, Hash)]
pub struct Fighter {
    pub id: FighterId,
    pub kit: &'static Kit,
    pub x: i32,
    pub y: i32,
    pub vy: i32,
    pub facing: i8,
    pub hp: u32,
    pub stamina: Meter,
    pub energy: Meter,
    pub mach: Meter,
    pub state: State,
    pub cooldowns: [u32; 3],
    pub(crate) move_x: i8,
    pub(crate) last_seq: Option<u32>,
    pub(crate) buffered: Option<(Intent, u32)>,
    pub(crate) hit_set: Vec<FighterId>,
}

impl Fighter {
    pub(crate) fn new(id: FighterId, kit: &'static Kit, x: i32, facing: i8) -> Self {
        Self {
            id,
            kit,
            x: clamp_x(kit, x),
            y: 0,
            vy: 0,
            facing: if facing < 0 { -1 } else { 1 },
            hp: kit.body.max_hp,
            stamina: Meter::stamina(),
            energy: Meter::energy(),
            mach: Meter::mach(),
            state: State::Neutral,
            cooldowns: [0; 3],
            move_x: 0,
            last_seq: None,
            buffered: None,
            hit_set: Vec::new(),
        }
    }

    pub fn is_grounded(&self) -> bool {
        self.y == 0 && self.vy == 0
    }

    /// Đòn đang thực hiện cùng pha hiện tại.
    pub fn action(&self) -> Option<(ActionKind, &'static ActionSpec, Phase)> {
        let kit: &'static Kit = self.kit;
        match self.state {
            State::Attack {
                action, elapsed, ..
            } => {
                let spec = kit.spec(action);
                Some((action, spec, spec.phase(elapsed)))
            }
            _ => None,
        }
    }

    pub fn hurtbox(&self) -> Rect {
        Rect {
            x0: self.x - self.kit.body.half_width,
            x1: self.x + self.kit.body.half_width,
            y0: self.y,
            y1: self.y + self.kit.body.height,
        }
    }

    /// Hộp đánh cận chiến khi đòn có damage đang ở pha active.
    pub fn attack_box(&self) -> Option<Rect> {
        let (_, spec, phase) = self.action()?;
        if phase != Phase::Active || spec.damage == 0 || spec.projectile.is_some() {
            return None;
        }
        Some(spec.hitbox.place(self.x, self.y, self.facing))
    }

    pub fn is_invulnerable(&self) -> bool {
        match self.state {
            State::Dash { elapsed, .. } => {
                (DASH_STARTUP..DASH_STARTUP + DASH_IFRAMES).contains(&elapsed)
            }
            State::Downed => true,
            _ => false,
        }
    }

    /// `Some(perfect)` khi đỡ đã qua startup và nhân vật đứng trên nền.
    pub fn guard_active(&self) -> Option<bool> {
        match self.state {
            State::Guard { elapsed } if elapsed >= GUARD_STARTUP && self.is_grounded() => {
                Some(elapsed < GUARD_STARTUP + PERFECT_GUARD)
            }
            _ => None,
        }
    }

    /// `(damage, hitstun)` nếu đang ở active của tư thế phản công và chưa bắt đòn nào.
    pub(crate) fn counter_ready(&self) -> Option<(u32, u32)> {
        let (_, spec, phase) = self.action()?;
        let damage = spec.counter_damage?;
        (phase == Phase::Active && self.hit_set.is_empty()).then_some((damage, spec.hitstun))
    }

    /// Đỡ và phản công chỉ chặn đòn từ phía trước.
    pub(crate) fn faces(&self, x: i32) -> bool {
        let dx = x - self.x;
        dx == 0 || (dx > 0) == (self.facing > 0)
    }

    pub(crate) fn accept_seq(&mut self, seq: u32) -> bool {
        if self.last_seq.is_some_and(|last| seq <= last) {
            return false;
        }
        self.last_seq = Some(seq);
        true
    }

    pub(crate) fn take_hit(
        &mut self,
        damage: u32,
        hitstun: u32,
        knockback: i32,
        events: &mut Vec<Event>,
    ) {
        self.hp = self.hp.saturating_sub(damage);
        if self.hp == 0 {
            self.state = State::Downed;
            self.buffered = None;
            events.push(Event::Downed { fighter: self.id });
            return;
        }
        if self.kit.body.armored {
            return;
        }
        self.x = clamp_x(self.kit, self.x + knockback);
        // Bị trúng không phát lại thao tác cũ trong buffer.
        self.buffered = None;
        self.state = State::Hitstun {
            remaining: hitstun.max(1),
        };
    }

    pub(crate) fn apply_input(
        &mut self,
        frame: Option<InputFrame>,
        next_instance: &mut u32,
        events: &mut Vec<Event>,
    ) {
        if self.state == State::Downed {
            self.move_x = 0;
            self.buffered = None;
            return;
        }
        let (move_x, held, intent) = frame.map_or((0, Buttons::NONE, None), |f| {
            (f.move_x.signum(), f.held, f.intent())
        });
        self.move_x = move_x;
        // Hướng chỉ đổi ở trạng thái trung tính, trước khi bắt đầu startup.
        if self.state == State::Neutral && move_x != 0 {
            self.facing = move_x;
        }
        if matches!(self.state, State::Guard { .. }) && !held.contains(Buttons::GUARD) {
            self.state = State::Neutral;
        }

        if let Some(want) = intent.or(self.buffered.map(|(buffered, _)| buffered)) {
            match self.try_start(want, next_instance) {
                Outcome::Started(action) => {
                    self.buffered = None;
                    if let Some(action) = action {
                        events.push(Event::ActionStarted {
                            fighter: self.id,
                            action,
                        });
                    }
                }
                // Input ngoài cửa sổ không bị biến thành combo bí mật.
                Outcome::NotYet if intent.is_some() => {
                    self.buffered = self.can_buffer(want).then_some((want, ACTION_BUFFER));
                }
                Outcome::NotYet => {}
                Outcome::Refused => self.buffered = None,
            }
        }

        if self.state == State::Neutral && self.is_grounded() && held.contains(Buttons::GUARD) {
            self.state = State::Guard { elapsed: 0 };
        }
    }

    fn try_start(&mut self, want: Intent, next_instance: &mut u32) -> Outcome {
        match self.state {
            State::Neutral => self.start_from_neutral(want, next_instance),
            State::Attack {
                action: ActionKind::Light(step),
                elapsed,
                confirmed_at,
                ..
            } => {
                let remaining = self.kit.light[usize::from(step)].total() - elapsed;
                match want {
                    Intent::Light if step < 2 && remaining <= LIGHT_CHAIN_WINDOW => {
                        self.start_attack(ActionKind::Light(step + 1), next_instance)
                    }
                    Intent::Skill(slot)
                        if confirmed_at.is_some_and(|hit| elapsed - hit <= HIT_CANCEL_WINDOW) =>
                    {
                        self.start_attack(ActionKind::Skill(slot), next_instance)
                    }
                    _ => Outcome::NotYet,
                }
            }
            State::Attack { .. } | State::Dash { .. } => Outcome::NotYet,
            State::Guard { .. }
            | State::Hitstun { .. }
            | State::GuardBreak { .. }
            | State::Downed => Outcome::Refused,
        }
    }

    fn can_buffer(&self, want: Intent) -> bool {
        match self.state {
            State::Attack {
                action, elapsed, ..
            } => {
                let remaining = self.kit.spec(action).total() - elapsed;
                // Ý định nối thuật được giữ trước hit để chờ server xác nhận trúng.
                remaining <= ACTION_BUFFER
                    || matches!((action, want), (ActionKind::Light(_), Intent::Skill(_)))
            }
            State::Dash { elapsed, .. } => DASH_TOTAL - elapsed <= ACTION_BUFFER,
            _ => false,
        }
    }

    fn start_from_neutral(&mut self, want: Intent, next_instance: &mut u32) -> Outcome {
        let grounded = self.is_grounded();
        match want {
            Intent::Jump if grounded => {
                self.vy = JUMP_VELOCITY;
                Outcome::Started(None)
            }
            Intent::Dash if grounded => {
                if !self.stamina.try_spend(DASH_STAMINA) {
                    return Outcome::Refused;
                }
                let dir = if self.move_x != 0 {
                    self.move_x
                } else {
                    self.facing
                };
                self.facing = dir;
                self.state = State::Dash { elapsed: 0, dir };
                Outcome::Started(None)
            }
            Intent::Light => self.start_attack(ActionKind::Light(0), next_instance),
            Intent::Heavy if grounded => self.start_attack(ActionKind::Heavy, next_instance),
            Intent::Skill(slot) if grounded => {
                self.start_attack(ActionKind::Skill(slot), next_instance)
            }
            _ => Outcome::Refused,
        }
    }

    fn start_attack(&mut self, action: ActionKind, next_instance: &mut u32) -> Outcome {
        let kit: &'static Kit = self.kit;
        let spec = kit.spec(action);
        if let ActionKind::Skill(slot) = action
            && self.cooldowns[usize::from(slot)] > 0
        {
            return Outcome::Refused;
        }
        if !self.stamina.can_spend(spec.stamina_cost) || !self.energy.can_spend(spec.energy_cost) {
            return Outcome::Refused;
        }
        // Chỉ trừ sau khi đủ mọi tài nguyên, để không tiêu nửa vời.
        if spec.stamina_cost > 0 {
            self.stamina.try_spend(spec.stamina_cost);
        }
        if spec.energy_cost > 0 {
            self.energy.try_spend(spec.energy_cost);
        }
        if let ActionKind::Skill(slot) = action {
            self.cooldowns[usize::from(slot)] = spec.cooldown;
        }
        self.state = State::Attack {
            action,
            elapsed: 0,
            instance: *next_instance,
            confirmed_at: None,
        };
        *next_instance = next_instance.wrapping_add(1);
        self.hit_set.clear();
        Outcome::Started(Some(action))
    }

    pub(crate) fn integrate(&mut self) {
        let grounded = self.is_grounded();
        let move_x = i32::from(self.move_x);
        let walk = self.kit.body.walk_speed;
        let vx = match self.state {
            State::Neutral => move_x * walk,
            // Đỡ làm chậm di chuyển; đòn trên không chỉ trôi nhẹ.
            State::Guard { .. } => move_x * walk / 3,
            State::Attack { .. } if !grounded => move_x * walk / 2,
            State::Dash { elapsed, dir }
                if (DASH_STARTUP..DASH_STARTUP + DASH_MOVE).contains(&elapsed) =>
            {
                i32::from(dir) * DASH_SPEED
            }
            _ => 0,
        };
        self.x = clamp_x(self.kit, self.x + vx);
        if !grounded {
            self.y += self.vy;
            self.vy -= GRAVITY;
            if self.y <= 0 {
                self.y = 0;
                self.vy = 0;
            }
        }
    }

    /// Tiến bộ đếm ở cuối tick; chuyển trạng thái khi hết thời gian.
    pub(crate) fn advance(&mut self, events: &mut Vec<Event>) {
        let kit: &'static Kit = self.kit;
        self.state = match self.state {
            State::Attack {
                action,
                elapsed,
                instance,
                confirmed_at,
            } => {
                if elapsed + 1 >= kit.spec(action).total() {
                    State::Neutral
                } else {
                    State::Attack {
                        action,
                        elapsed: elapsed + 1,
                        instance,
                        confirmed_at,
                    }
                }
            }
            State::Guard { elapsed } => {
                if self.stamina.drain_per_sec(GUARD_DRAIN_PER_SEC) {
                    events.push(Event::GuardBroken { fighter: self.id });
                    State::GuardBreak {
                        remaining: GUARD_BREAK,
                    }
                } else {
                    State::Guard {
                        elapsed: elapsed.saturating_add(1),
                    }
                }
            }
            State::Dash { elapsed, dir } => {
                if elapsed + 1 >= DASH_TOTAL {
                    State::Neutral
                } else {
                    State::Dash {
                        elapsed: elapsed + 1,
                        dir,
                    }
                }
            }
            State::Hitstun { remaining } if remaining > 1 => State::Hitstun {
                remaining: remaining - 1,
            },
            State::GuardBreak { remaining } if remaining > 1 => State::GuardBreak {
                remaining: remaining - 1,
            },
            State::Hitstun { .. } | State::GuardBreak { .. } => State::Neutral,
            state @ (State::Neutral | State::Downed) => state,
        };

        // Không hồi sức bền khi đỡ.
        self.stamina
            .tick(!matches!(self.state, State::Guard { .. }));
        self.energy.tick(true);
        for cooldown in &mut self.cooldowns {
            *cooldown = cooldown.saturating_sub(1);
        }
        self.buffered = self
            .buffered
            .and_then(|(intent, ttl)| (ttl > 1).then_some((intent, ttl - 1)));
    }
}

/// Giữ cả thân nhân vật trong arena.
fn clamp_x(kit: &Kit, x: i32) -> i32 {
    x.clamp(kit.body.half_width, ARENA_WIDTH - kit.body.half_width)
}
