use myva_sim::CommandFrame;
use myva_sim::battle::{Battle, Mode};
use myva_sim::catalog::{BASELINE_CATALOG, Catalog, KitId, KitRecord};
use myva_sim::character::{
    CHARACTER_DETAILS, CharacterCatalog, CharacterDetails, CharacterError, CharacterIcon,
    Inheritance,
};
use myva_sim::registry::{
    EquipPermission, IMPLEMENTED_KITS, KitRegistry, KitRequest, RegistryError,
};

#[test]
fn stable_lookup_never_infers_identity_from_names_teams_or_geography() {
    let catalog = CharacterCatalog::baseline().unwrap();
    let view = catalog.lookup("long-luu").unwrap();
    assert_eq!(view.record.id, KitId::LONG_LUU);
    assert_eq!(view.record.metadata.display_name, "Long Lưu");
    assert_eq!(view.inheritance, Inheritance::UnmappedFixture);
    for id in ["Long Lưu", "", "Long-Luu", "../long-luu"] {
        assert!(matches!(
            catalog.lookup(id),
            Err(CharacterError::InvalidId(_))
        ));
    }
    for id in ["van-thuy", "team-0", "christianity", "ke-giu-dap"] {
        assert!(matches!(
            catalog.lookup(id),
            Err(CharacterError::UnknownKit(_))
        ));
    }
}

#[test]
fn missing_duplicate_or_orphan_presentation_is_explicit() {
    let catalog = CharacterCatalog::new(KitRegistry::baseline().unwrap(), &[]).unwrap();
    assert!(matches!(
        catalog.lookup("long-luu"),
        Err(CharacterError::MissingPresentation(KitId::LONG_LUU))
    ));
    let duplicates = [CHARACTER_DETAILS[0], CHARACTER_DETAILS[0]];
    assert!(matches!(
        CharacterCatalog::new(KitRegistry::baseline().unwrap(), &duplicates),
        Err(CharacterError::DuplicatePresentation(_))
    ));
    let orphan = [CharacterDetails {
        kit: KitId::new("other-fixture").unwrap(),
        ..CHARACTER_DETAILS[0]
    }];
    assert!(matches!(
        CharacterCatalog::new(KitRegistry::baseline().unwrap(), &orphan),
        Err(CharacterError::OrphanPresentation(_))
    ));
}

#[test]
fn metadata_without_implementation_and_nonplayer_permission_cannot_be_selected() {
    // Synthetic fixture only, never installed in the baseline roster.
    let id = KitId::new("test-unimplemented").unwrap();
    let records = [
        BASELINE_CATALOG.kits[0],
        KitRecord {
            id,
            ..BASELINE_CATALOG.kits[0]
        },
    ];
    let details = [
        CHARACTER_DETAILS[0],
        CharacterDetails {
            kit: id,
            ..CHARACTER_DETAILS[0]
        },
    ];
    let registry = KitRegistry::new(
        Catalog {
            kits: &records,
            ..BASELINE_CATALOG
        },
        &IMPLEMENTED_KITS,
    )
    .unwrap();
    let catalog = CharacterCatalog::new(registry, &details).unwrap();
    assert!(
        catalog
            .lookup(id.as_str())
            .unwrap()
            .implementation
            .is_none()
    );
    assert!(matches!(
        catalog.select(KitRequest {
            id: id.as_str(),
            revision: IMPLEMENTED_KITS[0].revision
        }),
        Err(CharacterError::Registry(RegistryError::UnknownKit(_)))
    ));
    for permission in [EquipPermission::ReplayOnly, EquipPermission::Disabled] {
        let mut entries = IMPLEMENTED_KITS;
        entries[0].equip = permission;
        let catalog = CharacterCatalog::new(
            KitRegistry::new(BASELINE_CATALOG, &entries).unwrap(),
            CHARACTER_DETAILS,
        )
        .unwrap();
        assert!(matches!(
            catalog.select(KitRequest {
                id: "long-luu",
                revision: entries[0].revision
            }),
            Err(CharacterError::Registry(RegistryError::NotEquippable(_)))
        ));
    }
}

#[test]
fn selection_keeps_exact_pin_and_rejects_revision_mismatch() {
    let catalog = CharacterCatalog::baseline().unwrap();
    let revision = IMPLEMENTED_KITS[0].revision;
    let loadout = catalog
        .select(KitRequest {
            id: "long-luu",
            revision,
        })
        .unwrap();
    assert_eq!(loadout.pin().id, KitId::LONG_LUU);
    assert_eq!(loadout.pin().revision, revision);
    let mut mismatch = revision;
    mismatch.version += 1;
    assert!(matches!(
        catalog.select(KitRequest {
            id: "long-luu",
            revision: mismatch
        }),
        Err(CharacterError::Registry(
            RegistryError::VersionMismatch { .. }
        ))
    ));
    mismatch = revision;
    mismatch.config_hash ^= 1;
    assert!(matches!(
        catalog.select(KitRequest {
            id: "long-luu",
            revision: mismatch
        }),
        Err(CharacterError::Registry(RegistryError::HashMismatch { .. }))
    ));
}

fn selected_replay(catalog: &CharacterCatalog<'_>, mode: Mode) -> String {
    let loadout = catalog
        .select(KitRequest {
            id: "long-luu",
            revision: IMPLEMENTED_KITS[0].revision,
        })
        .unwrap();
    let mut battle = Battle::with_loadout(mode, 1, loadout.request()).unwrap();
    battle.bout_mut().set_autopilot(true);
    for _ in 0..3_600 {
        battle.step(CommandFrame::IDLE);
    }
    let replay = battle.replay();
    assert_eq!(
        replay.play().unwrap().world.state_hash(),
        battle.world().state_hash()
    );
    replay.to_text()
}

#[test]
fn changing_name_description_icon_and_skill_labels_preserves_pinned_replay_bytes() {
    let mut records = [BASELINE_CATALOG.kits[0]];
    records[0].metadata.display_name = "Presentation renamed";
    let details = [CharacterDetails {
        description: "New description",
        icon: CharacterIcon::Placeholder("XX"),
        skills: ["A", "B", "C"],
        ..CHARACTER_DETAILS[0]
    }];
    let changed = CharacterCatalog::new(
        KitRegistry::new(
            Catalog {
                kits: &records,
                ..BASELINE_CATALOG
            },
            &IMPLEMENTED_KITS,
        )
        .unwrap(),
        &details,
    )
    .unwrap();
    assert_eq!(
        changed
            .lookup("long-luu")
            .unwrap()
            .record
            .metadata
            .display_name,
        "Presentation renamed"
    );
    let baseline = CharacterCatalog::baseline().unwrap();
    for mode in [Mode::Boss, Mode::Duel] {
        assert_eq!(
            selected_replay(&baseline, mode),
            selected_replay(&changed, mode)
        );
    }
}
