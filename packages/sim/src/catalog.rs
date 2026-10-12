//! Catalog metadata thuần Rust theo ADR 0004. Không nằm trong `World`, `Kit`, protocol
//! hoặc replay: đổi tên/provenance không đổi combat hash. Không chứa luật hay registry kit.
//! Catalog mặc định chỉ xác nhận năm nền tảng; roster nguồn/trực hệ/nhánh còn chờ duyệt.

use std::collections::BTreeSet;

/// ID nền tảng đã chốt, không phải tên tổ chức trong game hay đội PvP.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FactionId {
    Christianity,
    Islam,
    Hinduism,
    Buddhism,
    Taoism,
}

impl FactionId {
    pub const ALL: [Self; 5] = [
        Self::Christianity,
        Self::Islam,
        Self::Hinduism,
        Self::Buddhism,
        Self::Taoism,
    ];

    /// Stable ASCII identity; never serialize enum discriminants or display labels.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Christianity => "christianity",
            Self::Islam => "islam",
            Self::Hinduism => "hinduism",
            Self::Buddhism => "buddhism",
            Self::Taoism => "taoism",
        }
    }

    pub fn parse(value: &str) -> Result<Self, IdError> {
        Self::ALL
            .into_iter()
            .find(|id| id.as_str() == value)
            .ok_or(IdError)
    }
}

/// Không chuẩn hóa tên hiển thị thành ID, không fallback về một ID hợp lệ khác.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdError;

macro_rules! slug_id {
    ($($name:ident),+ $(,)?) => {$ (
        /// ID authored cho catalog tĩnh; đổi metadata không đổi ID. Namespace theo kiểu.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(&'static str);

        impl $name {
            /// ASCII lowercase kebab-case, tối đa 64 byte; không tạo ID từ index catalog.
            pub fn new(value: &'static str) -> Result<Self, IdError> {
                let valid = !value.is_empty() && value.len() <= 64
                    && value.split('-').all(|part| !part.is_empty()
                        && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()));
                if valid { Ok(Self(value)) } else { Err(IdError) }
            }

            pub const fn as_str(self) -> &'static str { self.0 }
        }
    )+};
}
slug_id!(SourceId, LineageId, BranchId, KitId);

impl KitId {
    /// Giá trị replay legacy được giữ nguyên. Không suy ra faction/trực hệ từ tên này.
    pub const LONG_LUU: Self = Self("long-luu");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Approval {
    pub decision: &'static str,
    pub decided_by: &'static str,
    pub date: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewStatus {
    Proposed,
    Approved(Approval),
}

impl ReviewStatus {
    fn approved(self) -> bool {
        matches!(self, Self::Approved(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceKind {
    SourceFact,
    GameFiction,
    GameplayHypothesis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Provenance {
    /// Hồ sơ/decision có phạm vi nguồn và phân biệt DK/ST/GT; không phải lời xác nhận giáo lý.
    pub reference: &'static str,
    pub kind: ProvenanceKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub display_name: &'static str,
    /// Duyệt riêng mỗi tầng; duyệt nền tảng không duyệt roster hoặc balance.
    pub review: ReviewStatus,
    pub provenance: &'static [Provenance],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Faction {
    pub id: FactionId,
    pub metadata: Metadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceRole {
    MonotheisticGod,
    Deity,
    Avatar,
    Angel,
    Saint,
    Prophet,
    Buddha,
    Bodhisattva,
    Immortal,
    TeachingTradition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transmission {
    Blessing,
    Chosen,
    DharmaTransmission,
    TeacherStudent,
    Bloodline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub id: SourceId,
    pub faction: FactionId,
    pub role: SourceRole,
    /// Quan hệ ST được hồ sơ cụ thể cho phép; không tự coi tất cả nguồn là huyết thống.
    pub transmissions: &'static [Transmission],
    pub metadata: Metadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lineage {
    pub id: LineageId,
    pub source: SourceId,
    /// Tuyến bảo trợ/truyền pháp trung gian, nếu có; cùng nền tảng với nguồn.
    pub via: Option<SourceId>,
    pub transmission: Transmission,
    pub metadata: Metadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Branch {
    pub id: BranchId,
    pub lineage: LineageId,
    pub metadata: Metadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KitPlacement {
    Branch(BranchId),
    /// Fixture gameplay có trước ADR 0004; ánh xạ lore chưa được quyết định.
    UnmappedFixture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitRecord {
    pub id: KitId,
    pub placement: KitPlacement,
    pub metadata: Metadata,
}

/// Slices để tác giả/test xây catalog nhỏ; validate trước khi nhận vào consumer.
/// Không có địa lý, quê hương, chính trị, Team hay thông số chiến đấu trong schema này.
#[derive(Clone, Copy, Debug)]
pub struct Catalog<'a> {
    pub factions: &'a [Faction],
    pub sources: &'a [Source],
    pub lineages: &'a [Lineage],
    pub branches: &'a [Branch],
    pub kits: &'a [KitRecord],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeId {
    Faction(FactionId),
    Source(SourceId),
    Lineage(LineageId),
    Branch(BranchId),
    Kit(KitId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationError {
    DuplicateId(NodeId),
    MissingFaction(FactionId),
    MissingReference { node: NodeId, target: NodeId },
    MissingMetadata(NodeId),
    MissingSourceFact(SourceId),
    MissingGameFiction(LineageId),
    InvalidSourceRole(SourceId),
    InvalidTransmission { node: NodeId, source: SourceId },
    InvalidMediator(LineageId),
    UnapprovedParent { node: NodeId, parent: NodeId },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResolveError {
    InvalidCatalog(Vec<ValidationError>),
    UnknownKit(KitId),
    UnmappedFixture(KitId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitHierarchy {
    pub faction: FactionId,
    pub source: SourceId,
    pub via: Option<SourceId>,
    pub lineage: LineageId,
    pub branch: BranchId,
    pub kit: KitId,
}

impl Catalog<'_> {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let records = self
            .factions
            .iter()
            .map(|r| (NodeId::Faction(r.id), r.metadata))
            .chain(
                self.sources
                    .iter()
                    .map(|r| (NodeId::Source(r.id), r.metadata)),
            )
            .chain(
                self.lineages
                    .iter()
                    .map(|r| (NodeId::Lineage(r.id), r.metadata)),
            )
            .chain(
                self.branches
                    .iter()
                    .map(|r| (NodeId::Branch(r.id), r.metadata)),
            )
            .chain(self.kits.iter().map(|r| (NodeId::Kit(r.id), r.metadata)));
        let mut seen = BTreeSet::new();
        for (id, metadata) in records {
            if !seen.insert(id) {
                errors.push(ValidationError::DuplicateId(id));
            }
            let approval_valid = match metadata.review {
                ReviewStatus::Proposed => true,
                ReviewStatus::Approved(a) => [a.decision, a.decided_by, a.date]
                    .iter()
                    .all(|v| !v.trim().is_empty()),
            };
            if metadata.display_name.trim().is_empty()
                || metadata.provenance.is_empty()
                || metadata
                    .provenance
                    .iter()
                    .any(|p| p.reference.trim().is_empty())
                || !approval_valid
            {
                errors.push(ValidationError::MissingMetadata(id));
            }
        }
        for faction in FactionId::ALL {
            if !self.factions.iter().any(|r| r.id == faction) {
                errors.push(ValidationError::MissingFaction(faction));
            }
        }
        for source in self.sources {
            let node = NodeId::Source(source.id);
            check_parent(
                node,
                source.metadata,
                NodeId::Faction(source.faction),
                self.factions
                    .iter()
                    .find(|r| r.id == source.faction)
                    .map(|r| r.metadata),
                &mut errors,
            );
            if !has_provenance(source.metadata, ProvenanceKind::SourceFact) {
                errors.push(ValidationError::MissingSourceFact(source.id));
            }
            if source.role == SourceRole::MonotheisticGod
                && !matches!(source.faction, FactionId::Christianity | FactionId::Islam)
            {
                errors.push(ValidationError::InvalidSourceRole(source.id));
            }
            if source.transmissions.is_empty() {
                errors.push(ValidationError::InvalidTransmission {
                    node,
                    source: source.id,
                });
            }
            for &transmission in source.transmissions {
                if !source.permits(transmission) {
                    errors.push(ValidationError::InvalidTransmission {
                        node,
                        source: source.id,
                    });
                }
            }
        }
        for lineage in self.lineages {
            let node = NodeId::Lineage(lineage.id);
            let source = self.sources.iter().find(|r| r.id == lineage.source);
            check_parent(
                node,
                lineage.metadata,
                NodeId::Source(lineage.source),
                source.map(|r| r.metadata),
                &mut errors,
            );
            if !has_provenance(lineage.metadata, ProvenanceKind::GameFiction) {
                errors.push(ValidationError::MissingGameFiction(lineage.id));
            }
            if let Some(source) = source
                && !source.permits(lineage.transmission)
            {
                errors.push(ValidationError::InvalidTransmission {
                    node,
                    source: source.id,
                });
            }
            if let Some(via) = lineage.via {
                let mediator = self.sources.iter().find(|r| r.id == via);
                check_parent(
                    node,
                    lineage.metadata,
                    NodeId::Source(via),
                    mediator.map(|r| r.metadata),
                    &mut errors,
                );
                if via == lineage.source
                    || source
                        .zip(mediator)
                        .is_some_and(|(s, m)| s.faction != m.faction)
                {
                    errors.push(ValidationError::InvalidMediator(lineage.id));
                }
                if let Some(mediator) = mediator
                    && !mediator.permits(lineage.transmission)
                {
                    errors.push(ValidationError::InvalidTransmission { node, source: via });
                }
            }
        }
        for branch in self.branches {
            check_parent(
                NodeId::Branch(branch.id),
                branch.metadata,
                NodeId::Lineage(branch.lineage),
                self.lineages
                    .iter()
                    .find(|r| r.id == branch.lineage)
                    .map(|r| r.metadata),
                &mut errors,
            );
        }
        for kit in self.kits {
            if let KitPlacement::Branch(branch) = kit.placement {
                check_parent(
                    NodeId::Kit(kit.id),
                    kit.metadata,
                    NodeId::Branch(branch),
                    self.branches
                        .iter()
                        .find(|r| r.id == branch)
                        .map(|r| r.metadata),
                    &mut errors,
                );
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Fail tại boundary cho fixture chưa ánh xạ hoặc ID không tồn tại; không chọn lore mặc định.
    /// Việc phân giải metadata không xác nhận một kit đã có implementation/loadout hợp lệ.
    pub fn hierarchy(&self, id: KitId) -> Result<KitHierarchy, ResolveError> {
        self.validate().map_err(ResolveError::InvalidCatalog)?;
        let kit = self
            .kits
            .iter()
            .find(|r| r.id == id)
            .ok_or(ResolveError::UnknownKit(id))?;
        let KitPlacement::Branch(branch_id) = kit.placement else {
            return Err(ResolveError::UnmappedFixture(id));
        };
        // Không expect/panic, kể cả tại consumer boundary. validate đã kiểm mọi cạnh.
        let missing = || {
            ResolveError::InvalidCatalog(vec![ValidationError::MissingReference {
                node: NodeId::Kit(id),
                target: NodeId::Branch(branch_id),
            }])
        };
        let branch = self
            .branches
            .iter()
            .find(|r| r.id == branch_id)
            .ok_or_else(missing)?;
        let lineage = self
            .lineages
            .iter()
            .find(|r| r.id == branch.lineage)
            .ok_or_else(missing)?;
        let source = self
            .sources
            .iter()
            .find(|r| r.id == lineage.source)
            .ok_or_else(missing)?;
        Ok(KitHierarchy {
            faction: source.faction,
            source: source.id,
            via: lineage.via,
            lineage: lineage.id,
            branch: branch.id,
            kit: id,
        })
    }
}

impl Source {
    fn permits(self, transmission: Transmission) -> bool {
        self.transmissions.contains(&transmission)
            && match transmission {
                Transmission::Bloodline => !matches!(
                    self.role,
                    SourceRole::MonotheisticGod
                        | SourceRole::Angel
                        | SourceRole::Buddha
                        | SourceRole::Bodhisattva
                        | SourceRole::TeachingTradition
                ),
                Transmission::DharmaTransmission => matches!(
                    self.role,
                    SourceRole::Buddha | SourceRole::Bodhisattva | SourceRole::TeachingTradition
                ),
                _ => true,
            }
    }
}

fn has_provenance(metadata: Metadata, kind: ProvenanceKind) -> bool {
    metadata
        .provenance
        .iter()
        .any(|p| p.kind == kind && !p.reference.trim().is_empty())
}

fn check_parent(
    node: NodeId,
    child: Metadata,
    parent: NodeId,
    metadata: Option<Metadata>,
    errors: &mut Vec<ValidationError>,
) {
    match metadata {
        None => errors.push(ValidationError::MissingReference {
            node,
            target: parent,
        }),
        Some(metadata) if child.review.approved() && !metadata.review.approved() => {
            errors.push(ValidationError::UnapprovedParent { node, parent });
        }
        _ => {}
    }
}

const FOUNDATION_PROVENANCE: &[Provenance] = &[Provenance {
    reference: "docs/decisions/0004-five-factions-lineage-hierarchy.md",
    kind: ProvenanceKind::GameFiction,
}];
const FOUNDATION_APPROVAL: ReviewStatus = ReviewStatus::Approved(Approval {
    decision: "https://github.com/loveoverflowcom/myva/issues/18",
    decided_by: "project owner",
    date: "2026-10-11",
});

const fn foundation(id: FactionId, name: &'static str) -> Faction {
    Faction {
        id,
        metadata: Metadata {
            display_name: name,
            review: FOUNDATION_APPROVAL,
            provenance: FOUNDATION_PROVENANCE,
        },
    }
}

pub const FACTIONS: [Faction; 5] = [
    foundation(FactionId::Christianity, "Kitô giáo"),
    foundation(FactionId::Islam, "Hồi giáo"),
    foundation(FactionId::Hinduism, "Ấn Độ giáo"),
    foundation(FactionId::Buddhism, "Phật giáo"),
    foundation(FactionId::Taoism, "Đạo giáo"),
];

pub const BASELINE_CATALOG: Catalog<'static> = Catalog {
    factions: &FACTIONS,
    sources: &[],
    lineages: &[],
    branches: &[],
    kits: &[KitRecord {
        id: KitId::LONG_LUU,
        placement: KitPlacement::UnmappedFixture,
        metadata: Metadata {
            display_name: "Long Lưu",
            review: ReviewStatus::Proposed,
            provenance: &[Provenance {
                reference: "docs/design/lineages.md#6-giữ-kit-hiện-tại-và-mở-rộng-theo-gate",
                kind: ProvenanceKind::GameplayHypothesis,
            }],
        },
    }],
};
