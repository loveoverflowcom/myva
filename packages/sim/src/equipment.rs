//! Offline equipment/build rules for #20. Immutable authored data, exact pins and pure
//! effect prototypes; these rules do not mutate World or run in a combat round yet.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::catalog::{IdError, valid_slug};
use crate::fighter::FighterId;

pub const MAX_EQUIPMENT_SLOTS: usize = 3;
pub const BASIS_POINTS: u16 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId(&'static str);

impl ItemId {
    pub const AN_PHAN_CHAN: Self = Self("an-phan-chan");
    pub const GIAP_TRAN_MACH: Self = Self("giap-tran-mach");
    pub const PHU_VU_BO: Self = Self("phu-vu-bo");

    pub fn new(value: &'static str) -> Result<Self, IdError> {
        if valid_slug(value) {
            Ok(Self(value))
        } else {
            Err(IdError)
        }
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectGroup {
    GuardRecovery,
    GuardPressure,
    DashCost,
}

impl EffectGroup {
    fn index(self) -> usize {
        match self {
            Self::GuardRecovery => 0,
            Self::GuardPressure => 1,
            Self::DashCost => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectSpec {
    PerfectGuardRecovery {
        points: u32,
        cooldown_ticks: u32,
    },
    GuardPressureReduction {
        basis_points: u16,
    },
    DashCostReduction {
        basis_points: u16,
    },
}

impl EffectSpec {
    pub fn group(self) -> EffectGroup {
        match self {
            Self::PerfectGuardRecovery { .. } => EffectGroup::GuardRecovery,
            Self::GuardPressureReduction { .. } => EffectGroup::GuardPressure,
            Self::DashCostReduction { .. } => EffectGroup::DashCost,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemDefinition {
    pub id: ItemId,
    /// Presentation is excluded from the gameplay fingerprint.
    pub display_name: &'static str,
    pub budget_cost: u32,
    pub effect: EffectSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EquipmentRevision {
    pub version: u32,
    pub config_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemPin {
    pub id: ItemId,
    pub revision: EquipmentRevision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemEntry {
    pub definition: ItemDefinition,
    pub revision: EquipmentRevision,
}

/// GT offline budget, not production entitlement, inventory or a balance claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildRules {
    pub max_slots: u8,
    pub budget: u32,
    /// Indexed by GuardRecovery / GuardPressure / DashCost. Zero bans that group.
    pub group_limits: [u8; 3],
    pub max_recovery_points: u32,
    pub min_recovery_cooldown_ticks: u32,
    pub max_guard_reduction_bps: u16,
    pub max_dash_reduction_bps: u16,
    pub dash_cost_floor: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildRulesEntry {
    pub rules: BuildRules,
    pub revision: EquipmentRevision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EquipmentValidationError {
    InvalidRules,
    InvalidRulesRevision,
    RulesHashMismatch,
    DuplicateId(ItemId),
    InvalidDefinition(ItemId),
    InvalidRevision(ItemId),
    ConfigHashMismatch(ItemId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EquipmentError {
    InvalidRegistry(Vec<EquipmentValidationError>),
    InvalidId(String),
    UnknownItem(String),
    DuplicateItem(ItemId),
    TooManySlots {
        limit: u8,
        actual: usize,
    },
    BudgetExceeded {
        limit: u32,
        actual: u64,
    },
    GroupLimitExceeded {
        group: EffectGroup,
        limit: u8,
    },
    EffectCapExceeded(EffectGroup),
    VersionMismatch {
        id: ItemId,
        expected: u32,
        actual: u32,
    },
    HashMismatch {
        id: ItemId,
        expected: u64,
        actual: u64,
    },
    RulesRevisionMismatch {
        expected: EquipmentRevision,
        actual: EquipmentRevision,
    },
    /// World/replay v3/v4 have no equipment/proc state. Never silently ignore a valid build.
    RuntimeNotImplemented,
}

impl fmt::Display for EquipmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "equipment/build contract: {self:?}")
    }
}
impl std::error::Error for EquipmentError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BuildEffects {
    recovery_points: u32,
    recovery_cooldown: u32,
    guard_reduction_bps: u16,
    dash_reduction_bps: u16,
}

/// Private fields; canonical ID order, validated limits and exact data/rules pins.
/// Copying a build cannot change any actor/resource/cooldown or grant ownership of items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockedEquipment {
    ids: [&'static str; MAX_EQUIPMENT_SLOTS],
    pins: [Option<ItemPin>; MAX_EQUIPMENT_SLOTS],
    len: usize,
    budget_used: u32,
    rules: BuildRulesEntry,
    effects: BuildEffects,
}

impl LockedEquipment {
    pub fn ids(&self) -> &[&'static str] {
        &self.ids[..self.len]
    }

    pub fn pins(&self) -> impl Iterator<Item = ItemPin> + '_ {
        self.pins[..self.len].iter().flatten().copied()
    }

    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    pub fn budget_used(self) -> u32 {
        self.budget_used
    }

    pub fn rules_revision(self) -> EquipmentRevision {
        self.rules.revision
    }

    pub fn fingerprint(self) -> u64 {
        let mut h = Fingerprint::new("myva-locked-equipment-1");
        h.revision(self.rules.revision);
        h.u32(self.len as u32);
        for pin in self.pins() {
            h.string(pin.id.as_str());
            h.revision(pin.revision);
        }
        h.0
    }

    /// Pure prototype: positive costs stay positive, round up, never increase small costs.
    /// Does not reset dash, change i-frames, or mutate a fighter.
    pub fn dash_cost(self, base_cost: u32) -> u32 {
        reduced(base_cost, self.effects.dash_reduction_bps)
            .max(self.rules.rules.dash_cost_floor.min(base_cost))
    }

    /// Pure prototype: only front/blockable guard pressure; never changes HP damage.
    pub fn guard_pressure(self, pressure: u32, front: bool, blockable: bool) -> u32 {
        if front && blockable {
            reduced(pressure, self.effects.guard_reduction_bps)
        } else {
            pressure
        }
    }
}

/// Immutable borrowed catalog. Ordering is irrelevant to lookup and locked build hashes.
#[derive(Debug)]
pub struct EquipmentRegistry<'a> {
    entries: &'a [ItemEntry],
    rules: BuildRulesEntry,
}

impl<'a> EquipmentRegistry<'a> {
    pub fn new(entries: &'a [ItemEntry], rules: BuildRulesEntry) -> Result<Self, EquipmentError> {
        let mut errors = Vec::new();
        let r = rules.rules;
        if r.max_slots == 0
            || usize::from(r.max_slots) > MAX_EQUIPMENT_SLOTS
            || r.budget == 0
            || r.group_limits.iter().any(|&n| n > r.max_slots)
            || r.max_recovery_points == 0
            || r.max_recovery_points > 100
            || r.min_recovery_cooldown_ticks == 0
            || r.max_guard_reduction_bps >= BASIS_POINTS
            || r.max_dash_reduction_bps >= BASIS_POINTS
            || r.dash_cost_floor == 0
        {
            errors.push(EquipmentValidationError::InvalidRules);
        }
        if rules.revision.version == 0 {
            errors.push(EquipmentValidationError::InvalidRulesRevision);
        }
        if build_rules_hash(r) != rules.revision.config_hash {
            errors.push(EquipmentValidationError::RulesHashMismatch);
        }
        let mut seen = BTreeSet::new();
        for entry in entries {
            let d = entry.definition;
            if !seen.insert(d.id) {
                errors.push(EquipmentValidationError::DuplicateId(d.id));
            }
            let valid_effect = match d.effect {
                EffectSpec::PerfectGuardRecovery {
                    points,
                    cooldown_ticks,
                } => {
                    points > 0
                        && points <= r.max_recovery_points
                        && cooldown_ticks >= r.min_recovery_cooldown_ticks
                }
                EffectSpec::GuardPressureReduction { basis_points } => {
                    basis_points > 0 && basis_points <= r.max_guard_reduction_bps
                }
                EffectSpec::DashCostReduction { basis_points } => {
                    basis_points > 0 && basis_points <= r.max_dash_reduction_bps
                }
            };
            if !valid_slug(d.id.as_str())
                || d.display_name.trim().is_empty()
                || d.budget_cost == 0
                || d.budget_cost > r.budget
                || !valid_effect
            {
                errors.push(EquipmentValidationError::InvalidDefinition(d.id));
            }
            if entry.revision.version == 0 {
                errors.push(EquipmentValidationError::InvalidRevision(d.id));
            }
            if item_config_hash(d) != entry.revision.config_hash {
                errors.push(EquipmentValidationError::ConfigHashMismatch(d.id));
            }
        }
        if !errors.is_empty() {
            return Err(EquipmentError::InvalidRegistry(errors));
        }
        Ok(Self { entries, rules })
    }

    pub fn baseline() -> Result<EquipmentRegistry<'static>, EquipmentError> {
        EquipmentRegistry::new(&PROTOTYPE_ITEMS, PROTOTYPE_BUILD_RULES)
    }

    pub fn entries(&self) -> &[ItemEntry] {
        self.entries
    }

    pub fn rules(&self) -> BuildRulesEntry {
        self.rules
    }

    pub fn entry(&self, id: &str) -> Result<&ItemEntry, EquipmentError> {
        if !valid_slug(id) {
            return Err(EquipmentError::InvalidId(id.to_owned()));
        }
        self.entries
            .iter()
            .find(|e| e.definition.id.as_str() == id)
            .ok_or_else(|| EquipmentError::UnknownItem(id.to_owned()))
    }

    pub fn resolve(&self, pin: ItemPin) -> Result<&ItemEntry, EquipmentError> {
        let entry = self.entry(pin.id.as_str())?;
        if pin.revision.version != entry.revision.version {
            return Err(EquipmentError::VersionMismatch {
                id: pin.id,
                expected: entry.revision.version,
                actual: pin.revision.version,
            });
        }
        if pin.revision.config_hash != entry.revision.config_hash {
            return Err(EquipmentError::HashMismatch {
                id: pin.id,
                expected: entry.revision.config_hash,
                actual: pin.revision.config_hash,
            });
        }
        Ok(entry)
    }

    pub fn resolve_build(&self, ids: &[&str]) -> Result<LockedEquipment, EquipmentError> {
        if ids.len() > usize::from(self.rules.rules.max_slots) {
            return Err(EquipmentError::TooManySlots {
                limit: self.rules.rules.max_slots,
                actual: ids.len(),
            });
        }
        let mut selected = Vec::with_capacity(ids.len());
        let mut seen = BTreeSet::new();
        for &id in ids {
            let entry = self.entry(id)?;
            if !seen.insert(entry.definition.id) {
                return Err(EquipmentError::DuplicateItem(entry.definition.id));
            }
            selected.push(entry);
        }
        selected.sort_by_key(|e| e.definition.id);
        let budget: u64 = selected
            .iter()
            .map(|entry| u64::from(entry.definition.budget_cost))
            .sum();
        if budget > u64::from(self.rules.rules.budget) {
            return Err(EquipmentError::BudgetExceeded {
                limit: self.rules.rules.budget,
                actual: budget,
            });
        }
        let mut locked = LockedEquipment {
            ids: [""; MAX_EQUIPMENT_SLOTS],
            pins: [None; MAX_EQUIPMENT_SLOTS],
            len: selected.len(),
            budget_used: 0,
            rules: self.rules,
            effects: BuildEffects::default(),
        };
        let mut groups = [0_u8; 3];
        for (slot, entry) in selected.iter().enumerate() {
            let d = entry.definition;
            let group = d.effect.group();
            let index = group.index();
            groups[index] += 1;
            if groups[index] > self.rules.rules.group_limits[index] {
                return Err(EquipmentError::GroupLimitExceeded {
                    group,
                    limit: self.rules.rules.group_limits[index],
                });
            }
            let e = &mut locked.effects;
            match d.effect {
                EffectSpec::PerfectGuardRecovery {
                    points,
                    cooldown_ticks,
                } => {
                    e.recovery_points += points;
                    // Additive refund, slowest cooldown; never multiply/reset on stacking.
                    e.recovery_cooldown = e.recovery_cooldown.max(cooldown_ticks);
                    if e.recovery_points > self.rules.rules.max_recovery_points {
                        return Err(EquipmentError::EffectCapExceeded(group));
                    }
                }
                EffectSpec::GuardPressureReduction { basis_points } => {
                    e.guard_reduction_bps = e.guard_reduction_bps.saturating_add(basis_points);
                    if e.guard_reduction_bps > self.rules.rules.max_guard_reduction_bps {
                        return Err(EquipmentError::EffectCapExceeded(group));
                    }
                }
                EffectSpec::DashCostReduction { basis_points } => {
                    e.dash_reduction_bps = e.dash_reduction_bps.saturating_add(basis_points);
                    if e.dash_reduction_bps > self.rules.rules.max_dash_reduction_bps {
                        return Err(EquipmentError::EffectCapExceeded(group));
                    }
                }
            }
            locked.ids[slot] = d.id.as_str();
            locked.pins[slot] = Some(ItemPin {
                id: d.id,
                revision: entry.revision,
            });
        }
        locked.budget_used = budget as u32; // bounded by the u32 rules budget above
        Ok(locked)
    }

    /// Exact revalidation; a rematch must not silently upgrade item or build-rule revisions.
    pub fn revalidate(&self, build: LockedEquipment) -> Result<LockedEquipment, EquipmentError> {
        if build.rules_revision() != self.rules.revision {
            return Err(EquipmentError::RulesRevisionMismatch {
                expected: self.rules.revision,
                actual: build.rules_revision(),
            });
        }
        for pin in build.pins() {
            self.resolve(pin)?;
        }
        self.resolve_build(build.ids())
    }
}

fn reduced(value: u32, basis_points: u16) -> u32 {
    let numerator = u64::from(value) * u64::from(BASIS_POINTS - basis_points);
    numerator.div_ceil(u64::from(BASIS_POINTS)) as u32
}

/// A verified core contact must supply its source/action/hit identity. This prototype
/// conservatively allows at most one refund per source action, including multi-hit actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuardContact {
    pub tick: u32,
    pub source: FighterId,
    pub action_instance: u32,
    pub hit_index: u16,
    pub target: FighterId,
    pub perfect: bool,
    pub front: bool,
    pub blockable: bool,
    pub origin: ProcOrigin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcOrigin {
    Combat,
    Equipment,
}

/// Owned state-in/state-out for headless experiments, not World/replay state. Per-source
/// action watermarks reject stale/repeated hits without storing an unbounded hit history.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuardRecoveryState {
    build_fingerprint: Option<u64>,
    target: Option<FighterId>,
    observed_tick: Option<u32>,
    seen_actions: BTreeMap<FighterId, u32>,
    ready_at: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardRecoveryOutcome {
    pub state: GuardRecoveryState,
    pub stamina_points: u32,
    pub restored_points: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectError {
    TimeReversed,
    TargetMismatch,
    BuildMismatch,
    InvalidResource,
}

impl fmt::Display for EffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "equipment effect prototype: {self:?}")
    }
}
impl std::error::Error for EffectError {}

/// Pure prototype: no effect can trigger equipment effects, refill beyond the resource
/// cap, reuse a source action, or proc during the item cooldown. No guard-hold/whiff path.
/// Core integration needs authoritative verified contacts, not client-supplied booleans.
pub fn guard_recovery(
    build: LockedEquipment,
    previous: &GuardRecoveryState,
    contact: GuardContact,
    stamina_points: u32,
    max_stamina_points: u32,
) -> Result<GuardRecoveryOutcome, EffectError> {
    if stamina_points > max_stamina_points {
        return Err(EffectError::InvalidResource);
    }
    if previous.observed_tick.is_some_and(|t| contact.tick < t) {
        return Err(EffectError::TimeReversed);
    }
    if previous.target.is_some_and(|id| id != contact.target) {
        return Err(EffectError::TargetMismatch);
    }
    let fingerprint = build.fingerprint();
    if previous
        .build_fingerprint
        .is_some_and(|pin| pin != fingerprint)
    {
        return Err(EffectError::BuildMismatch);
    }
    let mut state = previous.clone();
    state.build_fingerprint = Some(fingerprint);
    state.target = Some(contact.target);
    state.observed_tick = Some(contact.tick);
    let mut restored_points = 0;
    if contact.origin == ProcOrigin::Combat
        && contact.perfect
        && contact.front
        && contact.blockable
        && build.effects.recovery_points > 0
    {
        let fresh = state
            .seen_actions
            .get(&contact.source)
            .is_none_or(|&instance| contact.action_instance > instance);
        if fresh {
            state.seen_actions.insert(contact.source, contact.action_instance);
            if u64::from(contact.tick) >= state.ready_at {
                restored_points = build
                    .effects
                    .recovery_points
                    .min(max_stamina_points - stamina_points);
                state.ready_at =
                    u64::from(contact.tick) + u64::from(build.effects.recovery_cooldown);
            }
        }
    }
    Ok(GuardRecoveryOutcome {
        state,
        stamina_points: stamina_points + restored_points,
        restored_points,
    })
}

/// Canonical FNV-1a v1: fixed little-endian integers, length-prefixed UTF-8 and explicit tags.
/// Identity/budget/effects are gameplay data; display names are not.
pub fn item_config_hash(definition: ItemDefinition) -> u64 {
    let mut h = Fingerprint::new("myva-equipment-item-1");
    h.string(definition.id.as_str());
    h.u32(definition.budget_cost);
    match definition.effect {
        EffectSpec::PerfectGuardRecovery {
            points,
            cooldown_ticks,
        } => {
            h.byte(0);
            h.u32(points);
            h.u32(cooldown_ticks);
        }
        EffectSpec::GuardPressureReduction { basis_points } => {
            h.byte(1);
            h.u16(basis_points);
        }
        EffectSpec::DashCostReduction { basis_points } => {
            h.byte(2);
            h.u16(basis_points);
        }
    }
    h.0
}

pub fn build_rules_hash(rules: BuildRules) -> u64 {
    let mut h = Fingerprint::new("myva-equipment-build-rules-1");
    h.byte(rules.max_slots);
    h.u32(rules.budget);
    for limit in rules.group_limits {
        h.byte(limit);
    }
    h.u32(rules.max_recovery_points);
    h.u32(rules.min_recovery_cooldown_ticks);
    h.u16(rules.max_guard_reduction_bps);
    h.u16(rules.max_dash_reduction_bps);
    h.u32(rules.dash_cost_floor);
    h.0
}

struct Fingerprint(u64);
impl Fingerprint {
    fn new(domain: &str) -> Self {
        let mut h = Self(0xcbf29ce484222325);
        h.string(domain);
        h
    }
    fn byte(&mut self, byte: u8) {
        self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    fn bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.byte(byte);
        }
    }
    fn u16(&mut self, n: u16) {
        self.bytes(&n.to_le_bytes());
    }
    fn u32(&mut self, n: u32) {
        self.bytes(&n.to_le_bytes());
    }
    fn string(&mut self, value: &str) {
        self.bytes(&(value.len() as u64).to_le_bytes());
        self.bytes(value.as_bytes());
    }
    fn revision(&mut self, revision: EquipmentRevision) {
        self.u32(revision.version);
        self.bytes(&revision.config_hash.to_le_bytes());
    }
}

// Frozen GT prototype pins. Author new versions explicitly when any gameplay data changes.
pub const PROTOTYPE_BUILD_RULES: BuildRulesEntry = BuildRulesEntry {
    rules: BuildRules {
        max_slots: 3,
        budget: 6,
        group_limits: [1, 1, 1],
        max_recovery_points: 10,
        min_recovery_cooldown_ticks: 180,
        max_guard_reduction_bps: 2_000,
        max_dash_reduction_bps: 2_000,
        dash_cost_floor: 15,
    },
    revision: EquipmentRevision {
        version: 1,
        config_hash: 0x9dc0378661ca5573,
    },
};

pub const PROTOTYPE_ITEMS: [ItemEntry; 3] = [
    ItemEntry {
        definition: ItemDefinition {
            id: ItemId::AN_PHAN_CHAN,
            display_name: "Ấn Phản Chấn",
            budget_cost: 3,
            effect: EffectSpec::PerfectGuardRecovery {
                points: 8,
                cooldown_ticks: 300,
            },
        },
        revision: EquipmentRevision {
            version: 1,
            config_hash: 0x67bf57e86c58af0e,
        },
    },
    ItemEntry {
        definition: ItemDefinition {
            id: ItemId::GIAP_TRAN_MACH,
            display_name: "Giáp Trấn Mạch",
            budget_cost: 3,
            effect: EffectSpec::GuardPressureReduction {
                basis_points: 1_500,
            },
        },
        revision: EquipmentRevision {
            version: 1,
            config_hash: 0x8683fcf7fed50f68,
        },
    },
    ItemEntry {
        definition: ItemDefinition {
            id: ItemId::PHU_VU_BO,
            display_name: "Phù Vũ Bộ",
            budget_cost: 2,
            effect: EffectSpec::DashCostReduction {
                basis_points: 2_000,
            },
        },
        revision: EquipmentRevision {
            version: 1,
            config_hash: 0x782d9b33f5a4124b,
        },
    },
];
