//! Bounded asset-streaming lifecycle planner; no I/O, decoder, renderer or gameplay rules.
//!
//! D05-A assets and a Bevy backend are prerequisites for runtime integration. The backend must
//! return strong handles as `H`, consume cancellation actions, and acknowledge cancellation only
//! after the request has stopped. Cancelled requests keep their global concurrency slots until
//! acknowledged or completed. Dropping planner ownership neither proves transport cancellation nor
//! GPU memory release: sprite/material owners and render-world allocations must be audited separately.
//! Encoded bytes in RAM, decoded/staging bytes and GPU upload/allocation are separate backend metrics.

use std::collections::BTreeMap;

/// The bounded content units planned by D05-B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BundleKind {
    Character,
    Map,
    Boss,
    Vfx,
}

/// Exact immutable content identity. `id` must include its revision/hash when content changes.
/// The planner does not validate paths, hashes, provenance or bundle byte budgets.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BundleKey {
    pub kind: BundleKind,
    pub id: String,
}

/// Configurable planner bounds; attempts include the initial request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_live_bundles: usize,
    pub max_in_flight: usize,
    pub max_attempts: u32,
    pub request_timeout_ms: u64,
    /// Linear retry delay: this value times the number of completed attempts.
    pub retry_backoff_ms: u64,
}

/// Configuration or scene-plan rejection; a rejected scene change preserves current ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannerError {
    InvalidLimits,
    TooManyRequired { limit: usize },
    GenerationExhausted,
}

/// Backend-classified failure; essential content never silently falls back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadFailure {
    Missing,
    Corrupt,
    TimedOut,
    Backend(String),
}

/// Current bundle state; fallback requires a separately verified cosmetic substitute in the backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BundleState {
    Queued { not_before_ms: u64 },
    Loading { attempt: u32 },
    Ready,
    RequiredBlocked(LoadFailure),
    OptionalFallback(LoadFailure),
}

/// Scene-entry gate. Optional prefetch cannot delay `Ready` or bypass `Blocked`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneState {
    Idle,
    Preloading,
    Ready,
    Blocked,
}

/// Unique request identity across scene changes and retries. It owns no asset handle.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LoadTicket {
    pub generation: u64,
    pub bundle: BundleKey,
    pub attempt: u32,
}

/// Backend work to perform after a planner update; cancellation never frees a slot by itself.
#[derive(Debug, Default)]
pub struct Actions {
    pub loads: Vec<LoadTicket>,
    pub cancellations: Vec<LoadTicket>,
}

/// Scene-change outcome; required entries reserve capacity before optional adjacent entries.
#[derive(Debug)]
pub struct SceneChange {
    pub generation: u64,
    pub cancellations: Vec<LoadTicket>,
    /// Adjacent entries omitted because the bounded plan was full.
    pub skipped_prefetch: usize,
}

/// Whether completion changed current ownership. Stale and late successful handles are dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    Applied,
    Stale,
    TimedOut,
}

struct Bundle<H> {
    required: bool,
    state: BundleState,
    attempts: u32,
    // The only planner-owned strong handle. H intentionally has no Clone bound.
    handle: Option<H>,
}

struct Flight {
    deadline_ms: u64,
    cancelled: bool,
}

/// Single explicit owner of retained bundle handles and outstanding backend tickets.
/// Callers supply a monotonic millisecond clock; this clock never affects combat simulation.
pub struct StreamingPlanner<H> {
    limits: Limits,
    generation: u64,
    active_scene: bool,
    bundles: BTreeMap<BundleKey, Bundle<H>>,
    flights: BTreeMap<LoadTicket, Flight>,
}

impl<H> StreamingPlanner<H> {
    /// Reject zero bounds/timeouts; zero retry delay is allowed for controlled tests.
    pub fn new(limits: Limits) -> Result<Self, PlannerError> {
        if limits.max_live_bundles == 0
            || limits.max_in_flight == 0
            || limits.max_attempts == 0
            || limits.request_timeout_ms == 0
        {
            return Err(PlannerError::InvalidLimits);
        }
        Ok(Self {
            limits,
            generation: 0,
            active_scene: false,
            bundles: BTreeMap::new(),
            flights: BTreeMap::new(),
        })
    }

    /// Reserve required preload capacity, then admit adjacent prefetch in caller order.
    /// Already-ready immutable bundles transfer ownership, including prefetch promoted to required.
    /// Other old handles drop and old requests are cancelled; no hidden cache retains clones.
    pub fn enter_scene(
        &mut self,
        required: &[BundleKey],
        adjacent: &[BundleKey],
        now_ms: u64,
    ) -> Result<SceneChange, PlannerError> {
        let generation = self.next_generation()?;
        let mut requested = BTreeMap::new();
        for key in required {
            if !requested.contains_key(key) && requested.len() == self.limits.max_live_bundles {
                return Err(PlannerError::TooManyRequired {
                    limit: self.limits.max_live_bundles,
                });
            }
            requested.insert(key.clone(), true);
        }
        let mut skipped_prefetch = 0;
        for key in adjacent {
            if requested.contains_key(key) {
                continue;
            }
            if requested.len() < self.limits.max_live_bundles {
                requested.insert(key.clone(), false);
            } else {
                skipped_prefetch += 1;
            }
        }
        let cancellations = self.cancel_outstanding();
        let mut previous = std::mem::take(&mut self.bundles);
        for (key, required) in requested {
            let mut bundle = match previous.remove(&key) {
                Some(bundle) if bundle.state == BundleState::Ready && bundle.handle.is_some() => {
                    bundle
                }
                _ => Bundle {
                    required,
                    state: BundleState::Queued {
                        not_before_ms: now_ms,
                    },
                    attempts: 0,
                    handle: None,
                },
            };
            bundle.required = required;
            self.bundles.insert(key, bundle);
        }
        self.generation = generation;
        self.active_scene = true;
        Ok(SceneChange {
            generation,
            cancellations,
            skipped_prefetch,
        })
    }

    /// Drop every planner-owned handle on exit and return outstanding cancellation requests.
    pub fn exit_scene(&mut self) -> Result<Vec<LoadTicket>, PlannerError> {
        let generation = self.next_generation()?;
        let cancellations = self.cancel_outstanding();
        self.bundles.clear();
        self.generation = generation;
        self.active_scene = false;
        Ok(cancellations)
    }

    /// Schedule due requests in required-first order and report each timeout cancellation once.
    pub fn poll(&mut self, now_ms: u64) -> Actions {
        let mut actions = Actions::default();
        let expired: Vec<_> = self
            .flights
            .iter()
            .filter(|(ticket, flight)| {
                ticket.generation == self.generation
                    && !flight.cancelled
                    && now_ms >= flight.deadline_ms
            })
            .map(|(ticket, _)| ticket.clone())
            .collect();
        for ticket in expired {
            if let Some(flight) = self.flights.get_mut(&ticket) {
                flight.cancelled = true;
            }
            if let Some(bundle) = self.bundles.get_mut(&ticket.bundle) {
                Self::fail_bundle(self.limits, bundle, LoadFailure::TimedOut, now_ms);
            }
            actions.cancellations.push(ticket);
        }
        for required in [true, false] {
            while self.flights.len() < self.limits.max_in_flight {
                let next = self.bundles.iter().find_map(|(key, bundle)| {
                    (bundle.required == required
                        && matches!(bundle.state, BundleState::Queued { not_before_ms } if now_ms >= not_before_ms))
                    .then(|| key.clone())
                });
                let Some(key) = next else { break };
                let Some(bundle) = self.bundles.get_mut(&key) else {
                    break;
                };
                bundle.attempts += 1;
                bundle.state = BundleState::Loading {
                    attempt: bundle.attempts,
                };
                let ticket = LoadTicket {
                    generation: self.generation,
                    bundle: key,
                    attempt: bundle.attempts,
                };
                self.flights.insert(
                    ticket.clone(),
                    Flight {
                        deadline_ms: now_ms.saturating_add(self.limits.request_timeout_ms),
                        cancelled: false,
                    },
                );
                actions.loads.push(ticket);
            }
        }
        actions
    }

    /// Consume backend completion. Unknown, cancelled or old-generation success drops `H`.
    /// A completion arriving at/after its deadline is rejected even before the next `poll`.
    pub fn complete(
        &mut self,
        ticket: LoadTicket,
        result: Result<H, LoadFailure>,
        now_ms: u64,
    ) -> Completion {
        let Some(flight) = self.flights.remove(&ticket) else {
            return Completion::Stale;
        };
        if flight.cancelled || ticket.generation != self.generation || !self.active_scene {
            return Completion::Stale;
        }
        let Some(bundle) = self.bundles.get_mut(&ticket.bundle) else {
            return Completion::Stale;
        };
        if bundle.state
            != (BundleState::Loading {
                attempt: ticket.attempt,
            })
        {
            return Completion::Stale;
        }
        if now_ms >= flight.deadline_ms {
            Self::fail_bundle(self.limits, bundle, LoadFailure::TimedOut, now_ms);
            return Completion::TimedOut;
        }
        match result {
            Ok(handle) => {
                bundle.handle = Some(handle);
                bundle.state = BundleState::Ready;
            }
            Err(failure) => Self::fail_bundle(self.limits, bundle, failure, now_ms),
        }
        Completion::Applied
    }

    /// Backend confirmation that cancelled work has stopped; only then release its global slot.
    pub fn acknowledge_cancellation(&mut self, ticket: &LoadTicket) -> bool {
        if self
            .flights
            .get(ticket)
            .is_some_and(|flight| flight.cancelled)
        {
            self.flights.remove(ticket);
            true
        } else {
            false
        }
    }

    /// Required content gate; zero required bundles permits immediate scene entry.
    pub fn scene_state(&self) -> SceneState {
        if !self.active_scene {
            return SceneState::Idle;
        }
        let required = || self.bundles.values().filter(|bundle| bundle.required);
        if required().any(|bundle| matches!(bundle.state, BundleState::RequiredBlocked(_))) {
            SceneState::Blocked
        } else if required().all(|bundle| bundle.state == BundleState::Ready) {
            SceneState::Ready
        } else {
            SceneState::Preloading
        }
    }

    /// Inspect progress/failure without taking or cloning ownership.
    pub fn bundle_state(&self, key: &BundleKey) -> Option<&BundleState> {
        self.bundles.get(key).map(|bundle| &bundle.state)
    }

    /// Borrow a planner-owned handle. Any caller-created clones require their own lifetime audit.
    pub fn owned_handle(&self, key: &BundleKey) -> Option<&H> {
        self.bundles
            .get(key)
            .and_then(|bundle| bundle.handle.as_ref())
    }

    /// Current planned bundles, including queued/blocked/fallback entries.
    pub fn live_bundle_count(&self) -> usize {
        self.bundles.len()
    }

    /// All backend requests still outstanding, including cancelled requests awaiting acknowledgement.
    pub fn in_flight_count(&self) -> usize {
        self.flights.len()
    }

    /// Planner-owned handles only; neither total Bevy strong references nor GPU allocations.
    pub fn owned_handle_count(&self) -> usize {
        self.bundles
            .values()
            .filter(|bundle| bundle.handle.is_some())
            .count()
    }

    fn next_generation(&self) -> Result<u64, PlannerError> {
        self.generation
            .checked_add(1)
            .ok_or(PlannerError::GenerationExhausted)
    }

    fn cancel_outstanding(&mut self) -> Vec<LoadTicket> {
        self.flights
            .iter_mut()
            .filter_map(|(ticket, flight)| {
                if flight.cancelled {
                    None
                } else {
                    flight.cancelled = true;
                    Some(ticket.clone())
                }
            })
            .collect()
    }

    fn fail_bundle(limits: Limits, bundle: &mut Bundle<H>, failure: LoadFailure, now_ms: u64) {
        bundle.handle = None;
        bundle.state = if bundle.attempts < limits.max_attempts {
            BundleState::Queued {
                not_before_ms: now_ms.saturating_add(
                    limits
                        .retry_backoff_ms
                        .saturating_mul(u64::from(bundle.attempts)),
                ),
            }
        } else if bundle.required {
            BundleState::RequiredBlocked(failure)
        } else {
            BundleState::OptionalFallback(failure)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    // Deliberately non-Clone: the planner must work without hidden owner copies.
    struct HandleDrop(Rc<Cell<usize>>);
    impl Drop for HandleDrop {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    fn key(kind: BundleKind, id: &str) -> BundleKey {
        BundleKey {
            kind,
            id: id.to_owned(),
        }
    }

    fn limits() -> Limits {
        Limits {
            max_live_bundles: 3,
            max_in_flight: 2,
            max_attempts: 3,
            request_timeout_ms: 10,
            retry_backoff_ms: 5,
        }
    }

    #[test]
    fn required_capacity_and_request_priority_are_bounded() {
        let mut planner = StreamingPlanner::<()>::new(Limits {
            max_in_flight: 1,
            ..limits()
        })
        .unwrap();
        let map = key(BundleKind::Map, "map-v1");
        let boss = key(BundleKind::Boss, "boss-v1");
        let character = key(BundleKind::Character, "adjacent-v1");
        let vfx = key(BundleKind::Vfx, "optional-v1");
        let change = planner
            .enter_scene(&[map.clone(), boss.clone()], &[character.clone(), vfx], 0)
            .unwrap();
        assert_eq!(change.skipped_prefetch, 1);
        assert_eq!(planner.live_bundle_count(), 3);
        let first = planner.poll(0).loads.remove(0);
        assert_eq!(first.bundle, map);
        assert!(planner.poll(1).loads.is_empty());
        planner.complete(first, Ok(()), 1);
        let second = planner.poll(1).loads.remove(0);
        assert_eq!(second.bundle, boss);
        planner.complete(second, Ok(()), 2);
        assert_eq!(planner.scene_state(), SceneState::Ready);
        assert_eq!(planner.poll(2).loads.remove(0).bundle, character);
        assert_eq!(planner.in_flight_count(), 1);
    }

    #[test]
    fn oversized_required_plan_preserves_current_handle() {
        let drops = Rc::new(Cell::new(0));
        let mut planner = StreamingPlanner::new(Limits {
            max_live_bundles: 1,
            ..limits()
        })
        .unwrap();
        let current = key(BundleKind::Map, "current-v1");
        planner
            .enter_scene(std::slice::from_ref(&current), &[], 0)
            .unwrap();
        let ticket = planner.poll(0).loads.remove(0);
        planner.complete(ticket, Ok(HandleDrop(drops.clone())), 1);
        let result =
            planner.enter_scene(&[current.clone(), key(BundleKind::Boss, "boss-v1")], &[], 2);
        assert!(matches!(
            result,
            Err(PlannerError::TooManyRequired { limit: 1 })
        ));
        assert!(planner.owned_handle(&current).is_some());
        assert_eq!(planner.scene_state(), SceneState::Ready);
        assert_eq!(drops.get(), 0);
    }

    #[test]
    fn ready_prefetch_promotes_without_reload_or_hidden_handle_clone() {
        let drops = Rc::new(Cell::new(0));
        let mut planner = StreamingPlanner::new(limits()).unwrap();
        let next = key(BundleKind::Map, "next-v1");
        planner
            .enter_scene(&[], std::slice::from_ref(&next), 0)
            .unwrap();
        let ticket = planner.poll(0).loads.remove(0);
        planner.complete(ticket, Ok(HandleDrop(drops.clone())), 1);
        planner
            .enter_scene(std::slice::from_ref(&next), std::slice::from_ref(&next), 2)
            .unwrap();
        assert_eq!(planner.scene_state(), SceneState::Ready);
        assert!(planner.poll(2).loads.is_empty());
        assert_eq!(planner.owned_handle_count(), 1);
        planner.exit_scene().unwrap();
        assert_eq!(drops.get(), 1);
        assert_eq!(planner.owned_handle_count(), 0);
    }

    #[test]
    fn rapid_scene_changes_keep_global_slots_until_backend_stops() {
        let drops = Rc::new(Cell::new(0));
        let mut planner = StreamingPlanner::new(limits()).unwrap();
        planner
            .enter_scene(&[key(BundleKind::Map, "a-v1")], &[], 0)
            .unwrap();
        let a = planner.poll(0).loads.remove(0);
        assert!(!planner.acknowledge_cancellation(&a));
        let b_change = planner
            .enter_scene(&[key(BundleKind::Map, "b-v1")], &[], 1)
            .unwrap();
        assert_eq!(b_change.cancellations, vec![a.clone()]);
        let b = planner.poll(1).loads.remove(0);
        let c_change = planner
            .enter_scene(&[key(BundleKind::Map, "c-v1")], &[], 2)
            .unwrap();
        assert_eq!(c_change.cancellations, vec![b.clone()]);
        assert!(planner.poll(2).loads.is_empty());
        assert_eq!(planner.in_flight_count(), 2);
        assert_eq!(
            planner.complete(a.clone(), Ok(HandleDrop(drops.clone())), 3),
            Completion::Stale
        );
        assert!(planner.acknowledge_cancellation(&b));
        let c = planner.poll(3).loads.remove(0);
        assert_eq!(
            planner.complete(c, Ok(HandleDrop(drops.clone())), 4),
            Completion::Applied
        );
        assert_eq!(
            planner.complete(a, Ok(HandleDrop(drops.clone())), 4),
            Completion::Stale
        );
        assert_eq!(drops.get(), 2);
        assert_eq!(planner.owned_handle_count(), 1);
        assert_eq!(planner.scene_state(), SceneState::Ready);
    }

    #[test]
    fn timeout_cancellation_retains_slot_and_retry_waits_for_backoff() {
        let mut planner = StreamingPlanner::<()>::new(Limits {
            max_in_flight: 1,
            max_attempts: 2,
            ..limits()
        })
        .unwrap();
        let map = key(BundleKind::Map, "map-v1");
        planner
            .enter_scene(std::slice::from_ref(&map), &[], 0)
            .unwrap();
        let first = planner.poll(0).loads.remove(0);
        assert_eq!(planner.poll(10).cancellations, vec![first.clone()]);
        assert!(planner.poll(14).loads.is_empty());
        assert!(planner.poll(14).cancellations.is_empty());
        assert!(planner.acknowledge_cancellation(&first));
        assert!(planner.poll(14).loads.is_empty());
        let retry = planner.poll(15).loads.remove(0);
        assert_eq!(retry.attempt, 2);
        planner.complete(retry, Err(LoadFailure::Missing), 16);
        assert_eq!(
            planner.bundle_state(&map),
            Some(&BundleState::RequiredBlocked(LoadFailure::Missing))
        );
        assert_eq!(planner.scene_state(), SceneState::Blocked);
        assert!(planner.poll(100).loads.is_empty());
    }

    #[test]
    fn exit_cancellation_cannot_fill_a_new_scene_with_the_same_bundle() {
        let drops = Rc::new(Cell::new(0));
        let mut planner = StreamingPlanner::new(limits()).unwrap();
        let map = key(BundleKind::Map, "map-v1");
        planner
            .enter_scene(std::slice::from_ref(&map), &[], 0)
            .unwrap();
        let old = planner.poll(0).loads.remove(0);
        assert_eq!(planner.exit_scene().unwrap(), vec![old.clone()]);
        assert_eq!(planner.scene_state(), SceneState::Idle);
        planner
            .enter_scene(std::slice::from_ref(&map), &[], 1)
            .unwrap();
        let current = planner.poll(1).loads.remove(0);
        assert_ne!(old.generation, current.generation);
        assert_eq!(
            planner.complete(old, Ok(HandleDrop(drops.clone())), 2),
            Completion::Stale
        );
        assert!(planner.owned_handle(&map).is_none());
        assert_eq!(
            planner.complete(current, Ok(HandleDrop(drops.clone())), 2),
            Completion::Applied
        );
        planner.exit_scene().unwrap();
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn late_success_is_rejected_before_timeout_poll() {
        let drops = Rc::new(Cell::new(0));
        let mut planner = StreamingPlanner::new(limits()).unwrap();
        planner
            .enter_scene(&[key(BundleKind::Map, "map-v1")], &[], 0)
            .unwrap();
        let ticket = planner.poll(0).loads.remove(0);
        assert_eq!(
            planner.complete(ticket, Ok(HandleDrop(drops.clone())), 10),
            Completion::TimedOut
        );
        assert_eq!(drops.get(), 1);
        assert_eq!(planner.owned_handle_count(), 0);
        assert_eq!(planner.scene_state(), SceneState::Preloading);
        assert!(planner.poll(14).loads.is_empty());
        assert_eq!(planner.poll(15).loads.len(), 1);
    }

    #[test]
    fn missing_required_blocks_and_corrupt_optional_selects_fallback() {
        let mut planner = StreamingPlanner::<()>::new(Limits {
            max_attempts: 1,
            ..limits()
        })
        .unwrap();
        let boss = key(BundleKind::Boss, "boss-v1");
        let vfx = key(BundleKind::Vfx, "vfx-v1");
        planner
            .enter_scene(
                &[boss.clone(), boss.clone()],
                &[vfx.clone(), boss.clone()],
                0,
            )
            .unwrap();
        let tickets = planner.poll(0).loads;
        assert_eq!(tickets.len(), 2);
        for ticket in tickets {
            let failure = if ticket.bundle == boss {
                LoadFailure::Missing
            } else {
                LoadFailure::Corrupt
            };
            planner.complete(ticket, Err(failure), 1);
        }
        assert_eq!(planner.live_bundle_count(), 2);
        assert_eq!(
            planner.bundle_state(&boss),
            Some(&BundleState::RequiredBlocked(LoadFailure::Missing))
        );
        assert_eq!(
            planner.bundle_state(&vfx),
            Some(&BundleState::OptionalFallback(LoadFailure::Corrupt))
        );
        assert_eq!(planner.scene_state(), SceneState::Blocked);
        assert!(planner.poll(100).loads.is_empty());
    }

    #[test]
    fn repeated_failures_have_bounded_attempts_and_linear_backoff() {
        let mut planner = StreamingPlanner::<()>::new(limits()).unwrap();
        planner
            .enter_scene(&[key(BundleKind::Character, "player-v1")], &[], 0)
            .unwrap();
        for (attempt, start, finish, next_due) in [(1, 0, 1, 6), (2, 6, 7, 17), (3, 17, 18, 33)] {
            let ticket = planner.poll(start).loads.remove(0);
            assert_eq!(ticket.attempt, attempt);
            planner.complete(ticket, Err(LoadFailure::Corrupt), finish);
            assert!(planner.poll(next_due - 1).loads.is_empty());
        }
        assert_eq!(planner.scene_state(), SceneState::Blocked);
        assert!(planner.poll(1_000).loads.is_empty());
    }

    #[test]
    fn thirty_planner_cycles_release_owned_handles_not_gpu_evidence() {
        let drops = Rc::new(Cell::new(0));
        let mut planner = StreamingPlanner::new(limits()).unwrap();
        let map = key(BundleKind::Map, "map-v1");
        for cycle in 0..30 {
            let now = cycle * 100;
            planner
                .enter_scene(std::slice::from_ref(&map), &[], now)
                .unwrap();
            let ticket = planner.poll(now).loads.remove(0);
            planner.complete(ticket, Ok(HandleDrop(drops.clone())), now + 1);
            assert_eq!(planner.scene_state(), SceneState::Ready);
            assert!(planner.exit_scene().unwrap().is_empty());
            assert_eq!(planner.scene_state(), SceneState::Idle);
            assert_eq!(planner.live_bundle_count(), 0);
            assert_eq!(planner.in_flight_count(), 0);
            assert_eq!(planner.owned_handle_count(), 0);
            assert_eq!(drops.get(), cycle as usize + 1);
        }
    }

    #[test]
    fn invalid_limits_and_generation_exhaustion_are_safe() {
        assert!(matches!(
            StreamingPlanner::<()>::new(Limits {
                max_attempts: 0,
                ..limits()
            }),
            Err(PlannerError::InvalidLimits)
        ));
        let mut planner = StreamingPlanner::<()>::new(limits()).unwrap();
        planner.generation = u64::MAX;
        assert!(matches!(
            planner.enter_scene(&[], &[], 0),
            Err(PlannerError::GenerationExhausted)
        ));
        assert_eq!(planner.scene_state(), SceneState::Idle);
    }
}
