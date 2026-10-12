//! Synthetic metadata fixtures, không duyệt roster trong docs hoặc ánh xạ Long Lưu.
use myva_sim::catalog::*;
use myva_sim::{LONG_LUU, Team};

const PROPOSED: Metadata = Metadata {
    display_name: "Synthetic test only",
    review: ReviewStatus::Proposed,
    provenance: &[
        Provenance {
            reference: "synthetic source profile",
            kind: ProvenanceKind::SourceFact,
        },
        Provenance {
            reference: "synthetic MyVa relation",
            kind: ProvenanceKind::GameFiction,
        },
    ],
};
const APPROVED: Metadata = Metadata {
    review: ReviewStatus::Approved(Approval {
        decision: "test decision",
        decided_by: "test author",
        date: "2026-10-12",
    }),
    ..PROPOSED
};

struct Fixture {
    sources: [Source; 2],
    lineages: [Lineage; 1],
    branches: [Branch; 1],
    kits: [KitRecord; 1],
}

impl Fixture {
    fn new() -> Self {
        Self {
            sources: [
                Source {
                    id: SourceId::new("test-source").unwrap(),
                    faction: FactionId::Christianity,
                    role: SourceRole::MonotheisticGod,
                    transmissions: &[Transmission::Blessing, Transmission::Chosen],
                    metadata: PROPOSED,
                },
                Source {
                    id: SourceId::new("test-mediator").unwrap(),
                    faction: FactionId::Christianity,
                    role: SourceRole::Angel,
                    transmissions: &[Transmission::Blessing],
                    metadata: PROPOSED,
                },
            ],
            lineages: [Lineage {
                id: LineageId::new("test-lineage").unwrap(),
                source: SourceId::new("test-source").unwrap(),
                via: Some(SourceId::new("test-mediator").unwrap()),
                transmission: Transmission::Blessing,
                metadata: PROPOSED,
            }],
            branches: [Branch {
                id: BranchId::new("test-branch").unwrap(),
                lineage: LineageId::new("test-lineage").unwrap(),
                metadata: PROPOSED,
            }],
            kits: [KitRecord {
                id: KitId::new("test-kit").unwrap(),
                placement: KitPlacement::Branch(BranchId::new("test-branch").unwrap()),
                metadata: PROPOSED,
            }],
        }
    }

    fn catalog(&self) -> Catalog<'_> {
        Catalog {
            sources: &self.sources,
            lineages: &self.lineages,
            branches: &self.branches,
            kits: &self.kits,
            ..BASELINE_CATALOG
        }
    }
}

#[test]
fn foundation_and_legacy_identities_are_stable() {
    assert_eq!(
        FactionId::ALL.map(FactionId::as_str),
        ["christianity", "islam", "hinduism", "buddhism", "taoism"]
    );
    for id in FactionId::ALL {
        assert_eq!(FactionId::parse(id.as_str()), Ok(id));
    }
    assert!(FactionId::parse("Kitô giáo").is_err());
    assert_eq!(LONG_LUU.catalog_id(), Ok(KitId::LONG_LUU));
    assert_eq!(KitId::LONG_LUU.as_str(), "long-luu");
    assert_eq!(LONG_LUU.lineage, "Long Lưu");
    for bad in [
        "",
        "Long Lưu",
        "long_luu",
        "long-luu ",
        "-kit",
        "kit-",
        "kit--id",
        "KIT",
        "kit.id",
        "💧",
    ] {
        assert!(KitId::new(bad).is_err(), "{bad:?}");
        assert!(SourceId::new(bad).is_err());
        assert!(LineageId::new(bad).is_err());
        assert!(BranchId::new(bad).is_err());
    }
    assert!(
        KitId::new("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").is_err()
    );
    assert!(KitId::new("kit-v1").is_ok());
}

#[test]
fn baseline_does_not_promote_proposals_or_invent_fixture_lore() {
    assert_eq!(BASELINE_CATALOG.validate(), Ok(()));
    assert!(BASELINE_CATALOG.sources.is_empty());
    assert!(BASELINE_CATALOG.lineages.is_empty());
    assert!(BASELINE_CATALOG.branches.is_empty());
    assert_eq!(
        BASELINE_CATALOG.hierarchy(KitId::LONG_LUU),
        Err(ResolveError::UnmappedFixture(KitId::LONG_LUU))
    );
    let unknown = KitId::new("not-supported").unwrap();
    assert_eq!(
        BASELINE_CATALOG.hierarchy(unknown),
        Err(ResolveError::UnknownKit(unknown))
    );
}

#[test]
fn hierarchy_uses_references_not_names_or_catalog_order() {
    let mut f = Fixture::new();
    let expected = KitHierarchy {
        faction: FactionId::Christianity,
        source: f.sources[0].id,
        via: Some(f.sources[1].id),
        lineage: f.lineages[0].id,
        branch: f.branches[0].id,
        kit: f.kits[0].id,
    };
    assert_eq!(f.catalog().hierarchy(expected.kit), Ok(expected));
    f.sources.reverse();
    f.lineages[0].metadata.display_name = "Renamed presentation";
    f.kits[0].metadata.display_name = "Another label";
    assert_eq!(f.catalog().hierarchy(expected.kit), Ok(expected));
}

#[test]
fn rejects_duplicate_ids_at_every_layer() {
    let f = Fixture::new();
    let mut factions = FACTIONS.to_vec();
    factions.push(FACTIONS[0]);
    let sources = [f.sources[0], f.sources[0], f.sources[1]];
    let lineages = [f.lineages[0]; 2];
    let branches = [f.branches[0]; 2];
    let kits = [f.kits[0]; 2];
    let c = Catalog {
        factions: &factions,
        sources: &sources,
        lineages: &lineages,
        branches: &branches,
        kits: &kits,
    };
    let errors = c.validate().unwrap_err();
    for id in [
        NodeId::Faction(FACTIONS[0].id),
        NodeId::Source(f.sources[0].id),
        NodeId::Lineage(f.lineages[0].id),
        NodeId::Branch(f.branches[0].id),
        NodeId::Kit(f.kits[0].id),
    ] {
        assert!(errors.contains(&ValidationError::DuplicateId(id)));
    }
    assert!(matches!(
        c.hierarchy(f.kits[0].id),
        Err(ResolveError::InvalidCatalog(_))
    ));
}

#[test]
fn requires_all_five_foundations_and_checks_all_references() {
    let f = Fixture::new();
    let c = Catalog {
        factions: &FACTIONS[1..],
        ..f.catalog()
    };
    let errors = c.validate().unwrap_err();
    assert!(errors.contains(&ValidationError::MissingFaction(FactionId::Christianity)));
    assert!(errors.contains(&ValidationError::MissingReference {
        node: NodeId::Source(f.sources[0].id),
        target: NodeId::Faction(FactionId::Christianity)
    }));
    let mut f = Fixture::new();
    f.lineages[0].source = SourceId::new("absent-source").unwrap();
    f.lineages[0].via = Some(SourceId::new("absent-mediator").unwrap());
    f.branches[0].lineage = LineageId::new("absent-lineage").unwrap();
    f.kits[0].placement = KitPlacement::Branch(BranchId::new("absent-branch").unwrap());
    let errors = f.catalog().validate().unwrap_err();
    assert_eq!(
        errors
            .iter()
            .filter(|e| matches!(e, ValidationError::MissingReference { .. }))
            .count(),
        4
    );
    assert!(matches!(
        f.catalog().hierarchy(f.kits[0].id),
        Err(ResolveError::InvalidCatalog(_))
    ));
}

#[test]
fn rejects_wrong_or_self_mediator_and_disallowed_transmission() {
    let mut f = Fixture::new();
    f.sources[1].faction = FactionId::Islam;
    assert!(
        f.catalog()
            .validate()
            .unwrap_err()
            .contains(&ValidationError::InvalidMediator(f.lineages[0].id))
    );
    f.sources[1].faction = FactionId::Christianity;
    f.lineages[0].via = Some(f.sources[0].id);
    assert!(
        f.catalog()
            .validate()
            .unwrap_err()
            .contains(&ValidationError::InvalidMediator(f.lineages[0].id))
    );
    f.lineages[0].via = Some(f.sources[1].id);
    f.lineages[0].transmission = Transmission::Chosen;
    assert!(
        f.catalog()
            .validate()
            .unwrap_err()
            .contains(&ValidationError::InvalidTransmission {
                node: NodeId::Lineage(f.lineages[0].id),
                source: f.sources[1].id
            })
    );
}

#[test]
fn relation_guards_keep_roles_distinct_even_if_authored_allow_list_is_wrong() {
    for role in [
        SourceRole::MonotheisticGod,
        SourceRole::Angel,
        SourceRole::Buddha,
        SourceRole::Bodhisattva,
        SourceRole::TeachingTradition,
    ] {
        let mut f = Fixture::new();
        f.sources[0].role = role;
        if matches!(
            role,
            SourceRole::Buddha | SourceRole::Bodhisattva | SourceRole::TeachingTradition
        ) {
            f.sources[0].faction = FactionId::Buddhism;
        }
        f.sources[0].transmissions = &[Transmission::Bloodline];
        f.lineages[0].transmission = Transmission::Bloodline;
        f.lineages[0].via = None;
        assert!(f.catalog().validate().unwrap_err().contains(
            &ValidationError::InvalidTransmission {
                node: NodeId::Lineage(f.lineages[0].id),
                source: f.sources[0].id
            }
        ));
    }
    let mut f = Fixture::new();
    f.sources[0].faction = FactionId::Buddhism;
    assert!(
        f.catalog()
            .validate()
            .unwrap_err()
            .contains(&ValidationError::InvalidSourceRole(f.sources[0].id))
    );
    f.sources[0].role = SourceRole::Bodhisattva;
    f.sources[0].transmissions = &[Transmission::DharmaTransmission];
    f.lineages[0].via = None;
    f.lineages[0].transmission = Transmission::DharmaTransmission;
    assert_eq!(f.catalog().validate(), Ok(()));
    f.sources[0].role = SourceRole::Prophet;
    assert!(f.catalog().validate().is_err());
    f.sources[0].role = SourceRole::Deity;
    f.sources[0].faction = FactionId::Hinduism;
    f.sources[0].transmissions = &[Transmission::Bloodline];
    f.lineages[0].transmission = Transmission::Bloodline;
    assert_eq!(f.catalog().validate(), Ok(()));
}

#[test]
fn metadata_requires_evidence_and_separates_source_fact_from_game_fiction() {
    let mut f = Fixture::new();
    f.sources[0].metadata.provenance = &[Provenance {
        reference: "fiction only",
        kind: ProvenanceKind::GameFiction,
    }];
    f.lineages[0].metadata.provenance = &[Provenance {
        reference: "fact only",
        kind: ProvenanceKind::SourceFact,
    }];
    f.branches[0].metadata.display_name = " ";
    f.kits[0].metadata.review = ReviewStatus::Approved(Approval {
        decision: "",
        decided_by: "",
        date: "",
    });
    let errors = f.catalog().validate().unwrap_err();
    assert!(errors.contains(&ValidationError::MissingSourceFact(f.sources[0].id)));
    assert!(errors.contains(&ValidationError::MissingGameFiction(f.lineages[0].id)));
    assert!(
        errors.contains(&ValidationError::MissingMetadata(NodeId::Branch(
            f.branches[0].id
        )))
    );
    assert!(errors.contains(&ValidationError::MissingMetadata(NodeId::Kit(f.kits[0].id))));
}

#[test]
fn approval_cannot_skip_unapproved_ancestors() {
    let mut f = Fixture::new();
    f.kits[0].metadata = APPROVED;
    assert!(
        f.catalog()
            .validate()
            .unwrap_err()
            .contains(&ValidationError::UnapprovedParent {
                node: NodeId::Kit(f.kits[0].id),
                parent: NodeId::Branch(f.branches[0].id)
            })
    );
    f.branches[0].metadata = APPROVED;
    assert!(
        f.catalog()
            .validate()
            .unwrap_err()
            .contains(&ValidationError::UnapprovedParent {
                node: NodeId::Branch(f.branches[0].id),
                parent: NodeId::Lineage(f.lineages[0].id)
            })
    );
    f.lineages[0].metadata = APPROVED;
    assert!(
        f.catalog()
            .validate()
            .unwrap_err()
            .contains(&ValidationError::UnapprovedParent {
                node: NodeId::Lineage(f.lineages[0].id),
                parent: NodeId::Source(f.sources[1].id)
            })
    );
    for source in &mut f.sources {
        source.metadata = APPROVED;
    }
    assert_eq!(f.catalog().validate(), Ok(()));
    let mut factions = FACTIONS;
    factions[0].metadata = PROPOSED;
    assert!(
        Catalog {
            factions: &factions,
            ..f.catalog()
        }
        .validate()
        .unwrap_err()
        .contains(&ValidationError::UnapprovedParent {
            node: NodeId::Source(f.sources[0].id),
            parent: NodeId::Faction(FactionId::Christianity)
        })
    );
}

#[test]
fn team_alliance_depends_on_combat_team_not_catalog_identity() {
    let mut world = myva_sim::World::new();
    let a = world.spawn_in_team(&LONG_LUU, 400 * myva_sim::PX, 1, Some(Team(7)));
    let ally = world.spawn_in_team(&LONG_LUU, 500 * myva_sim::PX, -1, Some(Team(7)));
    let rival = world.spawn_in_team(&LONG_LUU, 600 * myva_sim::PX, -1, Some(Team(8)));
    assert!(world.fighter(a).is_ally(world.fighter(ally)));
    assert!(!world.fighter(a).is_ally(world.fighter(rival)));
    // Cùng kit ID và legacy label vẫn đánh được khi khác Team; không ngầm gán faction.
    assert_eq!(
        world.fighter(a).kit.catalog_id(),
        world.fighter(rival).kit.catalog_id()
    );
}
