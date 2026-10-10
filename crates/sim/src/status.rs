//! Hiệu ứng trạng thái do luật áp lên nhân vật (combat.md §6).
//!
//! Graybox chỉ có Slow: không cộng dồn, chỉ hiệu lực mạnh nhất đang còn tác dụng, giới hạn 25% như
//! PvP prototype. Slow yếu hơn đến khi đang có Slow mạnh hơn bị bỏ qua thay vì xếp hàng chờ (GT
//! đơn giản hóa). Root, stun, launch và DR khống chế mạnh chưa làm.
//!
//! Thời lượng tính như hitstun: `remaining` đếm từ tick trúng, bước status cuối tick trừ dần.

use crate::fighter::FighterId;

/// Trần Slow ở PvP prototype (combat.md §6).
pub const SLOW_CAP_PERCENT: u8 = 25;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StatusKind {
    /// Giảm tốc độ đi bộ theo phần trăm; không ảnh hưởng lướt.
    Slow,
}

/// Hiệu ứng một đòn gây ra khi trúng; không áp khi bị đỡ.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StatusSpec {
    pub kind: StatusKind,
    pub percent: u8,
    /// Số tick, đã lượng tử hóa.
    pub duration: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StatusEffect {
    pub kind: StatusKind,
    pub percent: u8,
    pub remaining: u32,
    pub source: FighterId,
}

/// Tối đa một hiệu ứng mỗi loại, sắp theo loại để hash và snapshot ổn định.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Statuses(Vec<StatusEffect>);

impl Statuses {
    /// Áp hiệu ứng theo luật "mạnh nhất thắng"; trả hiệu ứng còn hiệu lực nếu có thay đổi.
    pub fn apply(&mut self, spec: StatusSpec, source: FighterId) -> Option<StatusEffect> {
        let percent = match spec.kind {
            StatusKind::Slow => spec.percent.min(SLOW_CAP_PERCENT),
        };
        if spec.duration == 0 || percent == 0 {
            return None;
        }
        let incoming = StatusEffect {
            kind: spec.kind,
            percent,
            remaining: spec.duration,
            source,
        };
        match self.0.binary_search_by_key(&spec.kind, |s| s.kind) {
            Err(index) => {
                self.0.insert(index, incoming);
                Some(incoming)
            }
            Ok(index) => {
                let current = &mut self.0[index];
                if percent > current.percent {
                    *current = incoming;
                } else if percent == current.percent && spec.duration > current.remaining {
                    // Cùng độ mạnh: làm mới thời lượng, không cộng dồn.
                    current.remaining = spec.duration;
                    current.source = source;
                } else {
                    return None;
                }
                Some(*current)
            }
        }
    }

    /// Trừ một tick; gọi `ended` cho mỗi hiệu ứng vừa hết.
    pub fn tick(&mut self, mut ended: impl FnMut(StatusKind)) {
        self.0.retain_mut(|status| {
            status.remaining = status.remaining.saturating_sub(1);
            if status.remaining == 0 {
                ended(status.kind);
            }
            status.remaining > 0
        });
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn get(&self, kind: StatusKind) -> Option<&StatusEffect> {
        self.0.iter().find(|s| s.kind == kind)
    }

    pub fn as_slice(&self) -> &[StatusEffect] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn slow_percent(&self) -> u8 {
        self.get(StatusKind::Slow).map_or(0, |s| s.percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: FighterId = FighterId(0);
    const B: FighterId = FighterId(1);

    fn slow(percent: u8, duration: u32) -> StatusSpec {
        StatusSpec {
            kind: StatusKind::Slow,
            percent,
            duration,
        }
    }

    #[test]
    fn slow_is_capped_and_never_stacks() {
        let mut statuses = Statuses::default();
        assert_eq!(statuses.apply(slow(40, 30), A).unwrap().percent, 25);
        statuses.apply(slow(25, 30), B);
        assert_eq!(statuses.as_slice().len(), 1);
        assert_eq!(statuses.slow_percent(), 25);
    }

    #[test]
    fn strongest_wins_and_equal_strength_refreshes() {
        let mut statuses = Statuses::default();
        statuses.apply(slow(10, 60), A);
        assert!(
            statuses.apply(slow(5, 600), B).is_none(),
            "yếu hơn bị bỏ qua"
        );
        let stronger = statuses.apply(slow(20, 10), B).unwrap();
        assert_eq!(
            (stronger.percent, stronger.remaining, stronger.source),
            (20, 10, B)
        );
        let refreshed = statuses.apply(slow(20, 40), A).unwrap();
        assert_eq!((refreshed.remaining, refreshed.source), (40, A));
        assert!(
            statuses.apply(slow(20, 5), B).is_none(),
            "không rút ngắn hiệu ứng đang có"
        );
    }

    #[test]
    fn tick_expires_once() {
        let mut statuses = Statuses::default();
        statuses.apply(slow(20, 2), A);
        let mut ended = Vec::new();
        for _ in 0..5 {
            statuses.tick(|kind| ended.push(kind));
        }
        assert_eq!(ended, [StatusKind::Slow]);
        assert!(statuses.is_empty());
    }

    #[test]
    fn zero_duration_or_strength_is_ignored() {
        let mut statuses = Statuses::default();
        assert!(statuses.apply(slow(20, 0), A).is_none());
        assert!(statuses.apply(slow(0, 20), A).is_none());
        assert!(statuses.is_empty());
    }
}
