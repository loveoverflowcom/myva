//! Validated implemented kits and round loadouts. Lore, equipment and gameplay approval
//! are separate: the only equip permission today is the explicit offline legacy fixture.
//! Fingerprints detect incompatible configuration; they are not security signatures.

use std::collections::BTreeSet;
use std::fmt;

use crate::boss::KE_GIU_DAP;
use crate::catalog::{
    BASELINE_CATALOG, BranchId, Catalog, KitId, KitPlacement, ReviewStatus, valid_slug,
};
use crate::equipment::{EquipmentError, EquipmentRegistry, LockedEquipment};
use crate::kit::{ActionSpec, Hitbox, Kit, LONG_LUU};
use crate::monster::QUAI_BUN;
use crate::status::StatusKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitRevision {
    pub version: u32,
    pub config_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitPin {
    pub id: KitId,
    pub revision: KitRevision,
}

impl KitPin {
    pub fn request(&self) -> KitRequest<'_> {
        KitRequest {
            id: self.id.as_str(),
            revision: self.revision,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipPermission {
    Player,
    ReplayOnly,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitEntry {
    pub kit: &'static Kit,
    pub revision: KitRevision,
    pub equip: EquipPermission,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitRequest<'a> {
    pub id: &'a str,
    pub revision: KitRevision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadoutBinding {
    Branch(BranchId),
    /// Explicit opt-in to the offline fixture, never an approved worldbuilding binding.
    UnresolvedLegacyFixture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadoutRequest<'a> {
    pub kit: KitRequest<'a>,
    pub binding: LoadoutBinding,
    /// Offline build IDs. Registry validates them; combat rejects effects not yet integrated.
    pub equipment: &'a [&'a str],
}

/// Constructed only by a validated registry. No API to change a kit during a round.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockedLoadout {
    kit: &'static Kit,
    pin: KitPin,
    binding: LoadoutBinding,
    equipment: LockedEquipment,
}

impl LockedLoadout {
    pub fn kit(self) -> &'static Kit {
        self.kit
    }
    pub fn pin(self) -> KitPin {
        self.pin
    }
    pub fn binding(self) -> LoadoutBinding {
        self.binding
    }
    pub fn equipment(self) -> LockedEquipment {
        self.equipment
    }
    /// ID request only. Use revalidate_loadout to preserve exact item/rules pins on rematch.
    pub fn request(&self) -> LoadoutRequest<'_> {
        LoadoutRequest {
            kit: self.pin.request(),
            binding: self.binding,
            equipment: self.equipment.ids(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryValidationError {
    Catalog(Vec<crate::catalog::ValidationError>),
    InvalidId(String),
    DuplicateId(KitId),
    InvalidRevision(KitId),
    ConfigHashMismatch(KitId),
    InvalidSpec(KitId),
    MissingCatalogKit(KitId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    InvalidRegistry(Vec<RegistryValidationError>),
    InvalidId(String),
    UnknownKit(String),
    VersionMismatch {
        id: KitId,
        expected: u32,
        actual: u32,
    },
    HashMismatch {
        id: KitId,
        expected: u64,
        actual: u64,
    },
    NotEquippable(KitId),
    BranchMismatch {
        id: KitId,
        requested: LoadoutBinding,
        actual: KitPlacement,
    },
    UnapprovedKit(KitId),
    Equipment(EquipmentError),
    RulesVersionMismatch {
        expected: u32,
        actual: u32,
    },
    RulesHashMismatch {
        expected: u64,
        actual: u64,
    },
    MissingReplayPin(String),
    UnexpectedReplayPin(KitId),
    DuplicateReplayPin(KitId),
    UnregisteredSpec(String),
    RoundAlreadyStarted,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kit/loadout contract: {self:?}")
    }
}
impl std::error::Error for RegistryError {}

/// Immutable borrowed data: successful construction validates catalog, implementations and
/// pinned revisions once. Proposed catalog records can exist without an implementation.
#[derive(Debug)]
pub struct KitRegistry<'a> {
    catalog: Catalog<'a>,
    entries: &'a [KitEntry],
}

impl<'a> KitRegistry<'a> {
    pub fn new(catalog: Catalog<'a>, entries: &'a [KitEntry]) -> Result<Self, RegistryError> {
        let mut errors = Vec::new();
        if let Err(e) = catalog.validate() {
            errors.push(RegistryValidationError::Catalog(e));
        }
        let mut seen = BTreeSet::new();
        for entry in entries {
            let Ok(id) = entry.kit.catalog_id() else {
                errors.push(RegistryValidationError::InvalidId(entry.kit.id.to_owned()));
                continue;
            };
            if !seen.insert(id) {
                errors.push(RegistryValidationError::DuplicateId(id));
            }
            if entry.revision.version == 0 {
                errors.push(RegistryValidationError::InvalidRevision(id));
            }
            if kit_config_hash(entry.kit) != entry.revision.config_hash {
                errors.push(RegistryValidationError::ConfigHashMismatch(id));
            }
            let kit = entry.kit;
            if kit.body.max_hp == 0
                || kit.body.max_hp > u32::MAX / crate::control::DAMAGE_PERCENT
                || kit.body.half_width <= 0
                || kit.body.half_width > crate::fighter::ARENA_WIDTH / 2
                || kit.body.height <= 0
                || kit.body.height > crate::fighter::ARENA_WIDTH
                || kit.body.walk_speed < 0
                || kit.body.walk_speed > i32::MAX / 100
                || kit.ultimate.is_some_and(|u| {
                    usize::from(u.slot) >= kit.skills.len() || u.mach_cost > MAX_RESOURCE_POINTS
                })
                || kit
                    .light
                    .iter()
                    .chain([&kit.heavy])
                    .chain(kit.skills.iter())
                    .chain(kit.ultimate.iter().map(|u| &u.action))
                    .any(|a| !valid_action(a))
            {
                errors.push(RegistryValidationError::InvalidSpec(id));
            }
            if entry.equip == EquipPermission::Player && !catalog.kits.iter().any(|k| k.id == id) {
                errors.push(RegistryValidationError::MissingCatalogKit(id));
            }
        }
        if !errors.is_empty() {
            return Err(RegistryError::InvalidRegistry(errors));
        }
        Ok(Self { catalog, entries })
    }

    pub fn baseline() -> Result<KitRegistry<'static>, RegistryError> {
        KitRegistry::new(BASELINE_CATALOG, &IMPLEMENTED_KITS)
    }

    pub fn entries(&self) -> &[KitEntry] {
        self.entries
    }
    pub fn catalog(&self) -> Catalog<'_> {
        self.catalog
    }

    pub fn entry(&self, id: &str) -> Result<&KitEntry, RegistryError> {
        if !valid_slug(id) {
            return Err(RegistryError::InvalidId(id.to_owned()));
        }
        self.entries
            .iter()
            .find(|entry| entry.kit.id == id)
            .ok_or_else(|| RegistryError::UnknownKit(id.to_owned()))
    }

    pub fn resolve(&self, request: KitRequest<'_>) -> Result<&KitEntry, RegistryError> {
        let entry = self.entry(request.id)?;
        let id = entry
            .kit
            .catalog_id()
            .map_err(|_| RegistryError::InvalidId(request.id.to_owned()))?;
        if request.revision.version != entry.revision.version {
            return Err(RegistryError::VersionMismatch {
                id,
                expected: entry.revision.version,
                actual: request.revision.version,
            });
        }
        if request.revision.config_hash != entry.revision.config_hash {
            return Err(RegistryError::HashMismatch {
                id,
                expected: entry.revision.config_hash,
                actual: request.revision.config_hash,
            });
        }
        Ok(entry)
    }

    pub fn resolve_loadout(
        &self,
        request: LoadoutRequest<'_>,
    ) -> Result<LockedLoadout, RegistryError> {
        let equipment = EquipmentRegistry::baseline().map_err(RegistryError::Equipment)?;
        self.resolve_loadout_with_equipment(request, &equipment)
    }

    /// Custom authored offline build rules share the same kit/lore boundary.
    pub fn resolve_loadout_with_equipment(
        &self,
        request: LoadoutRequest<'_>,
        equipment: &EquipmentRegistry<'_>,
    ) -> Result<LockedLoadout, RegistryError> {
        let entry = self.resolve(request.kit)?;
        let id = entry
            .kit
            .catalog_id()
            .map_err(|_| RegistryError::InvalidId(request.kit.id.to_owned()))?;
        if entry.equip != EquipPermission::Player {
            return Err(RegistryError::NotEquippable(id));
        }
        let record = self
            .catalog
            .kits
            .iter()
            .find(|k| k.id == id)
            .ok_or_else(|| RegistryError::UnknownKit(request.kit.id.to_owned()))?;
        match (request.binding, record.placement) {
            (LoadoutBinding::UnresolvedLegacyFixture, KitPlacement::UnmappedFixture) => {}
            (LoadoutBinding::Branch(requested), KitPlacement::Branch(actual))
                if requested == actual =>
            {
                // Catalog validator already verifies approval of every parent of an approved kit.
                if !matches!(record.metadata.review, ReviewStatus::Approved(_)) {
                    return Err(RegistryError::UnapprovedKit(id));
                }
            }
            _ => {
                return Err(RegistryError::BranchMismatch {
                    id,
                    requested: request.binding,
                    actual: record.placement,
                });
            }
        }
        Ok(LockedLoadout {
            kit: entry.kit,
            pin: KitPin {
                id,
                revision: entry.revision,
            },
            binding: request.binding,
            equipment: equipment
                .resolve_build(request.equipment)
                .map_err(RegistryError::Equipment)?,
        })
    }

    pub fn revalidate_loadout(
        &self,
        loadout: LockedLoadout,
    ) -> Result<LockedLoadout, RegistryError> {
        let equipment = EquipmentRegistry::baseline().map_err(RegistryError::Equipment)?;
        equipment
            .revalidate(loadout.equipment())
            .map_err(RegistryError::Equipment)?;
        self.resolve_loadout_with_equipment(loadout.request(), &equipment)
    }
}

fn valid_action(a: &ActionSpec) -> bool {
    !a.name.is_empty()
        && a.active > 0
        && a.startup
            .checked_add(a.active)
            .and_then(|n| n.checked_add(a.recovery))
            .is_some()
        && a.stamina_cost <= MAX_RESOURCE_POINTS
        && a.energy_cost <= MAX_RESOURCE_POINTS
        // World uses (16 + pressure) * SUB_PER_POINT before dividing by two.
        && a.guard_pressure <= MAX_RESOURCE_POINTS - 16
        // Meter::gain adds damage/10 points to a meter that can already hold 100 points.
        && a.damage <= MAX_MACH_DAMAGE
        && a.counter_damage.is_none_or(|n| n <= MAX_MACH_DAMAGE)
        && a.knockback.unsigned_abs() <= crate::fighter::ARENA_WIDTH as u32
        && valid_hitbox(a.hitbox)
        && a.projectile.is_none_or(|p| p.lifetime > 0 && p.hitbox.width > 0
            && p.hitbox.height > 0 && valid_hitbox(p.hitbox)
            && p.speed.unsigned_abs() <= crate::fighter::ARENA_WIDTH as u32)
}

// Arithmetic/domain bounds for the current arena, not balance caps. Geometry is bounded
// by arena extent so place/shift/center and facing multiplication remain representable.
const MAX_RESOURCE_POINTS: u32 = u32::MAX / crate::meter::SUB_PER_POINT;
const MAX_MACH_DAMAGE: u32 =
    ((u32::MAX - 100 * crate::meter::SUB_PER_POINT) / crate::meter::SUB_PER_POINT) * 10;
fn valid_hitbox(b: Hitbox) -> bool {
    let extent = crate::fighter::ARENA_WIDTH;
    b.front.unsigned_abs() <= extent as u32
        && b.bottom.unsigned_abs() <= extent as u32
        && (0..=extent).contains(&b.width)
        && (0..=extent).contains(&b.height)
}

// Frozen revision pins. A data change must publish a new version/hash explicitly.
pub const LONG_LUU_REVISION: KitRevision = KitRevision {
    version: 1,
    config_hash: 0xa624f529d63112f0,
};
pub const BOSS_REVISION: KitRevision = KitRevision {
    version: 1,
    config_hash: 0xf4c34792bf8d1ae4,
};
pub const MONSTER_REVISION: KitRevision = KitRevision {
    version: 1,
    config_hash: 0xe8aac515a7849264,
};
pub const IMPLEMENTED_KITS: [KitEntry; 3] = [
    KitEntry {
        kit: &LONG_LUU,
        revision: LONG_LUU_REVISION,
        equip: EquipPermission::Player,
    },
    KitEntry {
        kit: &KE_GIU_DAP,
        revision: BOSS_REVISION,
        equip: EquipPermission::ReplayOnly,
    },
    KitEntry {
        kit: &QUAI_BUN,
        revision: MONSTER_REVISION,
        equip: EquipPermission::ReplayOnly,
    },
];

pub fn legacy_fixture_loadout() -> LoadoutRequest<'static> {
    LoadoutRequest {
        kit: KitRequest {
            id: "long-luu",
            revision: LONG_LUU_REVISION,
        },
        binding: LoadoutBinding::UnresolvedLegacyFixture,
        equipment: &[],
    }
}

/// Canonical v1: FNV-1a, integers with fixed little-endian widths, UTF-8 strings with u64
/// byte lengths, explicit Option/enum tags. Includes legacy labels used by World::state_hash.
pub fn kit_config_hash(kit: &Kit) -> u64 {
    let mut h = Fingerprint::new();
    h.string("myva-kit-config-1");
    h.string(kit.id);
    h.string(kit.lineage);
    h.u32(kit.body.max_hp);
    h.i32(kit.body.half_width);
    h.i32(kit.body.height);
    h.i32(kit.body.walk_speed);
    h.byte(u8::from(kit.body.armored));
    for a in kit
        .light
        .iter()
        .chain([&kit.heavy])
        .chain(kit.skills.iter())
    {
        h.action(a);
    }
    match kit.ultimate {
        None => h.byte(0),
        Some(u) => {
            h.byte(1);
            h.byte(u.slot);
            h.u32(u.mach_cost);
            h.action(&u.action);
        }
    }
    h.0
}

pub const RULES_VERSION: u32 = 3;
/// Frozen at the v3 baseline. v3 has no pins, so it is accepted only with these exact laws.
pub const LEGACY_RULES_HASH: u64 = 0x93755e498de17ca3;

/// Conservative source fingerprint for playback laws/encoding, independent of metadata,
/// registry order, AI (recorded inputs), presentation and team/lore selection. Source-only
/// edits to these files can reject replay even if semantically harmless; use the original build.
pub fn rules_config_hash() -> u64 {
    let mut h = Fingerprint::new();
    h.string("myva-playback-laws-1");
    h.u32(RULES_VERSION);
    for source in [
        include_str!("control.rs"),
        include_str!("fighter.rs"),
        include_str!("input.rs"),
        include_str!("kit.rs"),
        include_str!("meter.rs"),
        include_str!("npc.rs"),
        include_str!("projectile.rs"),
        include_str!("status.rs"),
        include_str!("tick.rs"),
        include_str!("world.rs"),
    ] {
        h.string(source);
    }
    h.0
}

struct Fingerprint(u64);
impl Fingerprint {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
    fn byte(&mut self, b: u8) {
        self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x100000001b3);
    }
    fn bytes(&mut self, b: &[u8]) {
        for &byte in b {
            self.byte(byte);
        }
    }
    fn u32(&mut self, n: u32) {
        self.bytes(&n.to_le_bytes());
    }
    fn i32(&mut self, n: i32) {
        self.bytes(&n.to_le_bytes());
    }
    fn string(&mut self, s: &str) {
        self.bytes(&(s.len() as u64).to_le_bytes());
        self.bytes(s.as_bytes());
    }
    fn hitbox(&mut self, b: Hitbox) {
        self.i32(b.front);
        self.i32(b.width);
        self.i32(b.bottom);
        self.i32(b.height);
    }
    fn action(&mut self, a: &ActionSpec) {
        self.string(a.name);
        for n in [
            a.startup,
            a.active,
            a.recovery,
            a.stamina_cost,
            a.energy_cost,
            a.cooldown,
            a.damage,
            a.guard_pressure,
            a.hitstun,
        ] {
            self.u32(n);
        }
        self.i32(a.knockback);
        self.hitbox(a.hitbox);
        match a.counter_damage {
            None => self.byte(0),
            Some(n) => {
                self.byte(1);
                self.u32(n);
            }
        }
        match a.projectile {
            None => self.byte(0),
            Some(p) => {
                self.byte(1);
                self.hitbox(p.hitbox);
                self.i32(p.speed);
                self.u32(p.lifetime);
                self.byte(u8::from(p.pierce));
            }
        }
        match a.on_hit {
            None => self.byte(0),
            Some(s) => {
                self.byte(1);
                self.byte(match s.kind {
                    StatusKind::Slow => 0,
                });
                self.byte(s.percent);
                self.u32(s.duration);
            }
        }
    }
}

/// Immutable v3 revisions, independent of future registry revisions. Historical definitions
/// are not loaded automatically: mismatches require the original build.
pub fn legacy_replay_request(id: &str) -> Result<KitRequest<'_>, RegistryError> {
    let config_hash = match id {
        "long-luu" => 0xa624f529d63112f0,
        "ke-giu-dap" => 0xf4c34792bf8d1ae4,
        "quai-bun" => 0xe8aac515a7849264,
        _ => return Err(RegistryError::UnknownKit(id.to_owned())),
    };
    Ok(KitRequest {
        id,
        revision: KitRevision {
            version: 1,
            config_hash,
        },
    })
}
