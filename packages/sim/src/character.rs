//! Shared presentation projection for web and Bevy. Never part of Kit/World/replay hashes.
//! Names and inheritance belong to the domain catalog; these details add no roster or lore.

use crate::catalog::{KitHierarchy, KitId, KitPlacement, KitRecord, ResolveError, valid_slug};
use crate::registry::{
    KitEntry, KitRegistry, KitRequest, LoadoutBinding, LoadoutRequest, LockedLoadout, RegistryError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterIcon {
    /// A text marker, not an approved portrait or religious symbol. No image I/O.
    Placeholder(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterDetails {
    pub kit: KitId,
    pub description: &'static str,
    pub icon: CharacterIcon,
    /// Presentation labels, independent of legacy hashed ActionSpec.name.
    pub skills: [&'static str; 3],
}

pub const CHARACTER_DETAILS: &[CharacterDetails] = &[CharacterDetails {
    kit: KitId::LONG_LUU,
    description: "Kit thử nghiệm giữ trung cự và phản công. Chưa gán thế lực, nguồn truyền thừa, trực hệ hoặc nhánh; chỉ chơi fixture offline.",
    icon: CharacterIcon::Placeholder("LL"),
    skills: ["Lưu Tiễn", "Hồi Thế", "Triều Dâng"],
}];

#[derive(Debug, PartialEq, Eq)]
pub enum CharacterError {
    Registry(RegistryError),
    Inheritance(ResolveError),
    InvalidId(String),
    UnknownKit(String),
    MissingPresentation(KitId),
    DuplicatePresentation(KitId),
    OrphanPresentation(KitId),
    InvalidPresentation(KitId),
}

impl std::fmt::Display for CharacterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "character catalog: {self:?}")
    }
}
impl std::error::Error for CharacterError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inheritance {
    Mapped(KitHierarchy),
    UnmappedFixture,
}

impl Inheritance {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mapped(_) => "Truyền thừa theo catalog; trạng thái duyệt riêng từng tầng",
            Self::UnmappedFixture => "Fixture chưa ánh xạ truyền thừa · chưa duyệt lore",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CharacterView<'a> {
    pub record: &'a KitRecord,
    pub details: &'a CharacterDetails,
    pub inheritance: Inheritance,
    pub implementation: Option<&'a KitEntry>,
}

#[derive(Debug)]
pub struct CharacterCatalog<'a> {
    registry: KitRegistry<'a>,
    details: &'a [CharacterDetails],
}

impl<'a> CharacterCatalog<'a> {
    pub fn new(
        registry: KitRegistry<'a>,
        details: &'a [CharacterDetails],
    ) -> Result<Self, CharacterError> {
        let mut seen = std::collections::BTreeSet::new();
        for detail in details {
            if !seen.insert(detail.kit) {
                return Err(CharacterError::DuplicatePresentation(detail.kit));
            }
            if !registry.catalog().kits.iter().any(|k| k.id == detail.kit) {
                return Err(CharacterError::OrphanPresentation(detail.kit));
            }
            let CharacterIcon::Placeholder(icon) = detail.icon;
            if detail.description.trim().is_empty()
                || icon.trim().is_empty()
                || detail.skills.iter().any(|s| s.trim().is_empty())
            {
                return Err(CharacterError::InvalidPresentation(detail.kit));
            }
        }
        Ok(Self { registry, details })
    }

    pub fn baseline() -> Result<CharacterCatalog<'static>, CharacterError> {
        CharacterCatalog::new(
            KitRegistry::baseline().map_err(CharacterError::Registry)?,
            CHARACTER_DETAILS,
        )
    }

    pub fn registry(&self) -> &KitRegistry<'a> {
        &self.registry
    }

    /// Runtime strings are compared to stable IDs, never labels, indices or map/team identity.
    pub fn lookup(&self, id: &str) -> Result<CharacterView<'_>, CharacterError> {
        if !valid_slug(id) {
            return Err(CharacterError::InvalidId(id.to_owned()));
        }
        let record = self
            .registry
            .catalog()
            .kits
            .iter()
            .find(|k| k.id.as_str() == id)
            .ok_or_else(|| CharacterError::UnknownKit(id.to_owned()))?;
        let details = self
            .details
            .iter()
            .find(|d| d.kit == record.id)
            .ok_or(CharacterError::MissingPresentation(record.id))?;
        let inheritance = match record.placement {
            KitPlacement::UnmappedFixture => Inheritance::UnmappedFixture,
            KitPlacement::Branch(_) => Inheritance::Mapped(
                self.registry
                    .catalog()
                    .hierarchy(record.id)
                    .map_err(CharacterError::Inheritance)?,
            ),
        };
        Ok(CharacterView {
            record,
            details,
            inheritance,
            implementation: self.registry.entries().iter().find(|e| e.kit.id == id),
        })
    }

    /// Selection is before the round. Exact pin and all registry/equip/approval guards apply.
    /// Callers expose the fixture status before offering this offline selection.
    pub fn select(&self, request: KitRequest<'_>) -> Result<LockedLoadout, CharacterError> {
        let view = self.lookup(request.id)?;
        let binding = match view.record.placement {
            KitPlacement::UnmappedFixture => LoadoutBinding::UnresolvedLegacyFixture,
            KitPlacement::Branch(branch) => LoadoutBinding::Branch(branch),
        };
        self.registry
            .resolve_loadout(LoadoutRequest {
                kit: request,
                binding,
                equipment: &[],
            })
            .map_err(CharacterError::Registry)
    }
}
