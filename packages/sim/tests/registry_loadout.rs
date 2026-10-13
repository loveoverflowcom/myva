//! Synthetic branch/approval data below is test-only; it is not an authored lore binding.
use myva_sim::battle::{Battle, Mode};
use myva_sim::catalog::*;
use myva_sim::registry::*;
use myva_sim::replay::{PINNED_HEADER, Replay, ReplayError};
use myva_sim::{CommandFrame, LONG_LUU};

fn req(id: &str) -> LoadoutRequest<'_> {
    LoadoutRequest {
        kit: KitRequest {
            id,
            revision: LONG_LUU_REVISION,
        },
        ..legacy_fixture_loadout()
    }
}

#[test]
fn dynamic_ids_and_registry_order_resolve_the_same_locked_fixture() {
    let registry = KitRegistry::baseline().unwrap();
    let mut entries = IMPLEMENTED_KITS;
    entries.reverse();
    let reordered = KitRegistry::new(BASELINE_CATALOG, &entries).unwrap();
    let dynamic = String::from("long-luu");
    let a = registry.resolve_loadout(req(&dynamic)).unwrap();
    assert_eq!(a, reordered.resolve_loadout(req(&dynamic)).unwrap());
    assert_eq!(a.pin().id, KitId::LONG_LUU);
    assert_eq!(a.kit(), &LONG_LUU);
    assert_eq!(a.binding(), LoadoutBinding::UnresolvedLegacyFixture);
    assert!(registry.catalog().sources.is_empty());
    for invalid in [
        "",
        "Long Lưu",
        "long_luu",
        "LONG-LUU",
        "long-luu ",
        "kit--id",
        "-kit",
    ] {
        assert!(matches!(
            registry.entry(invalid),
            Err(RegistryError::InvalidId(_))
        ));
    }
    assert!(matches!(
        registry.entry(&"a".repeat(65)),
        Err(RegistryError::InvalidId(_))
    ));
    assert!(matches!(
        registry.resolve_loadout(req("son-cot")),
        Err(RegistryError::UnknownKit(_))
    ));
}

#[test]
fn loadout_rejects_version_hash_equipment_and_nonplayer_kits() {
    let registry = KitRegistry::baseline().unwrap();
    let mut request = legacy_fixture_loadout();
    request.kit.revision.version += 1;
    assert!(matches!(
        registry.resolve_loadout(request),
        Err(RegistryError::VersionMismatch { .. })
    ));
    request = legacy_fixture_loadout();
    request.kit.revision.config_hash ^= 1;
    assert!(matches!(
        registry.resolve_loadout(request),
        Err(RegistryError::HashMismatch { .. })
    ));
    request = legacy_fixture_loadout();
    request.equipment = &["unknown-item"];
    assert_eq!(
        registry.resolve_loadout(request),
        Err(RegistryError::Equipment(
            myva_sim::equipment::EquipmentError::UnknownItem("unknown-item".to_owned())
        ))
    );
    for id in ["ke-giu-dap", "quai-bun"] {
        let entry = registry.entry(id).unwrap();
        let request = LoadoutRequest {
            kit: KitRequest {
                id,
                revision: entry.revision,
            },
            ..legacy_fixture_loadout()
        };
        assert!(matches!(
            registry.resolve_loadout(request),
            Err(RegistryError::NotEquippable(_))
        ));
        // Replay implementation does not need a mythology record or equip permission.
        assert_eq!(registry.resolve(request.kit).unwrap().kit, entry.kit);
    }
    let mut entries = IMPLEMENTED_KITS;
    entries[0].equip = EquipPermission::Disabled;
    assert!(matches!(
        KitRegistry::new(BASELINE_CATALOG, &entries)
            .unwrap()
            .resolve_loadout(legacy_fixture_loadout()),
        Err(RegistryError::NotEquippable(_))
    ));
}

#[test]
fn invalid_registry_data_is_rejected_before_lookup() {
    let duplicate = [IMPLEMENTED_KITS[0]; 2];
    assert!(
        matches!(KitRegistry::new(BASELINE_CATALOG, &duplicate), Err(RegistryError::InvalidRegistry(e)) if e.contains(&RegistryValidationError::DuplicateId(KitId::LONG_LUU)))
    );
    let mut entry = IMPLEMENTED_KITS[0];
    entry.revision.version = 0;
    entry.revision.config_hash ^= 1;
    assert!(
        matches!(KitRegistry::new(BASELINE_CATALOG, &[entry]), Err(RegistryError::InvalidRegistry(e)) if e.contains(&RegistryValidationError::InvalidRevision(KitId::LONG_LUU)) && e.contains(&RegistryValidationError::ConfigHashMismatch(KitId::LONG_LUU)))
    );
    let missing = Catalog {
        kits: &[],
        ..BASELINE_CATALOG
    };
    assert!(
        matches!(KitRegistry::new(missing, &IMPLEMENTED_KITS), Err(RegistryError::InvalidRegistry(e)) if e.contains(&RegistryValidationError::MissingCatalogKit(KitId::LONG_LUU)))
    );
    let invalid = Catalog {
        factions: &[],
        ..BASELINE_CATALOG
    };
    assert!(matches!(
        KitRegistry::new(invalid, &IMPLEMENTED_KITS),
        Err(RegistryError::InvalidRegistry(_))
    ));
    const BAD_SLOT: myva_sim::Kit = myva_sim::Kit {
        ultimate: Some(myva_sim::kit::UltimateSpec {
            slot: 3,
            mach_cost: 100,
            action: LONG_LUU.skills[2],
        }),
        ..LONG_LUU
    };
    entry = IMPLEMENTED_KITS[0];
    entry.kit = &BAD_SLOT;
    entry.revision.config_hash = kit_config_hash(&BAD_SLOT);
    assert!(
        matches!(KitRegistry::new(BASELINE_CATALOG, &[entry]), Err(RegistryError::InvalidRegistry(e)) if e.contains(&RegistryValidationError::InvalidSpec(KitId::LONG_LUU)))
    );
    const BAD_ID: myva_sim::Kit = myva_sim::Kit {
        id: "Invalid ID",
        ..LONG_LUU
    };
    entry.kit = &BAD_ID;
    entry.revision.config_hash = kit_config_hash(&BAD_ID);
    assert!(
        matches!(KitRegistry::new(BASELINE_CATALOG, &[entry]), Err(RegistryError::InvalidRegistry(e)) if e.iter().any(|e| matches!(e, RegistryValidationError::InvalidId(_))))
    );
}

#[test]
fn metadata_proposal_without_implementation_never_becomes_equippable() {
    let kits = [
        BASELINE_CATALOG.kits[0],
        KitRecord {
            id: KitId::new("proposed-kit").unwrap(),
            ..BASELINE_CATALOG.kits[0]
        },
    ];
    let c = Catalog {
        kits: &kits,
        ..BASELINE_CATALOG
    };
    let registry = KitRegistry::new(c, &IMPLEMENTED_KITS).unwrap();
    assert!(matches!(
        registry.resolve_loadout(req("proposed-kit")),
        Err(RegistryError::UnknownKit(_))
    ));
}

#[test]
fn branch_binding_requires_exact_placement_and_approved_parent_chain() {
    let branch_id = BranchId::new("synthetic-branch").unwrap();
    let proposed = Metadata {
        display_name: "Synthetic test only",
        review: ReviewStatus::Proposed,
        provenance: &[
            Provenance {
                reference: "synthetic source",
                kind: ProvenanceKind::SourceFact,
            },
            Provenance {
                reference: "synthetic relation",
                kind: ProvenanceKind::GameFiction,
            },
        ],
    };
    let approved = Metadata {
        review: ReviewStatus::Approved(Approval {
            decision: "test only",
            decided_by: "test author",
            date: "2026-10-12",
        }),
        ..proposed
    };
    let sources = [Source {
        id: SourceId::new("synthetic-source").unwrap(),
        faction: FactionId::Buddhism,
        role: SourceRole::TeachingTradition,
        transmissions: &[Transmission::TeacherStudent],
        metadata: approved,
    }];
    let lineages = [Lineage {
        id: LineageId::new("synthetic-lineage").unwrap(),
        source: sources[0].id,
        via: None,
        transmission: Transmission::TeacherStudent,
        metadata: approved,
    }];
    let mut branches = [Branch {
        id: branch_id,
        lineage: lineages[0].id,
        metadata: approved,
    }];
    let mut kits = [KitRecord {
        placement: KitPlacement::Branch(branch_id),
        metadata: proposed,
        ..BASELINE_CATALOG.kits[0]
    }];
    let c = Catalog {
        sources: &sources,
        lineages: &lineages,
        branches: &branches,
        kits: &kits,
        ..BASELINE_CATALOG
    };
    let request = LoadoutRequest {
        binding: LoadoutBinding::Branch(branch_id),
        ..legacy_fixture_loadout()
    };
    let registry = KitRegistry::new(c, &IMPLEMENTED_KITS).unwrap();
    assert_eq!(
        registry.resolve_loadout(request),
        Err(RegistryError::UnapprovedKit(KitId::LONG_LUU))
    );
    assert!(matches!(
        registry.resolve_loadout(legacy_fixture_loadout()),
        Err(RegistryError::BranchMismatch { .. })
    ));
    kits[0].metadata = approved;
    let c = Catalog {
        sources: &sources,
        lineages: &lineages,
        branches: &branches,
        kits: &kits,
        ..BASELINE_CATALOG
    };
    let registry = KitRegistry::new(c, &IMPLEMENTED_KITS).unwrap();
    assert_eq!(
        registry.resolve_loadout(request).unwrap().binding(),
        request.binding
    );
    let wrong = LoadoutRequest {
        binding: LoadoutBinding::Branch(BranchId::new("another-branch").unwrap()),
        ..request
    };
    assert!(matches!(
        registry.resolve_loadout(wrong),
        Err(RegistryError::BranchMismatch { .. })
    ));
    assert!(matches!(
        KitRegistry::baseline().unwrap().resolve_loadout(request),
        Err(RegistryError::BranchMismatch { .. })
    ));
    branches[0].metadata = proposed;
    let c = Catalog {
        sources: &sources,
        lineages: &lineages,
        branches: &branches,
        kits: &kits,
        ..BASELINE_CATALOG
    };
    assert!(matches!(
        KitRegistry::new(c, &IMPLEMENTED_KITS),
        Err(RegistryError::InvalidRegistry(_))
    ));
}

#[test]
fn fingerprint_covers_all_stats_and_preserves_legacy_labels() {
    assert_eq!(kit_config_hash(&LONG_LUU), LONG_LUU_REVISION.config_hash);
    assert_eq!(rules_config_hash(), LEGACY_RULES_HASH);
    let base = kit_config_hash(&LONG_LUU);
    let mut changed = LONG_LUU;
    changed.lineage = "Renamed legacy label";
    assert_ne!(kit_config_hash(&changed), base);
    changed = LONG_LUU;
    changed.skills[0].projectile.as_mut().unwrap().lifetime += 1;
    assert_ne!(kit_config_hash(&changed), base);
    changed = LONG_LUU;
    changed.ultimate.as_mut().unwrap().mach_cost += 1;
    assert_ne!(kit_config_hash(&changed), base);
    changed = LONG_LUU;
    changed.light[0].name = "Renamed legacy action";
    assert_ne!(kit_config_hash(&changed), base);
    changed = LONG_LUU;
    changed.body.walk_speed += 1;
    assert_ne!(kit_config_hash(&changed), base);
    for entry in IMPLEMENTED_KITS {
        assert_eq!(kit_config_hash(entry.kit), entry.revision.config_hash);
        assert_eq!(
            legacy_replay_request(entry.kit.id).unwrap().revision,
            entry.revision
        );
    }
}

#[test]
fn metadata_edits_preserve_locked_kit_fingerprint_and_combat_state() {
    let mut kits = [BASELINE_CATALOG.kits[0]];
    kits[0].metadata.display_name = "New UI label";
    kits[0].metadata.provenance = &[Provenance {
        reference: "New reference",
        kind: ProvenanceKind::GameplayHypothesis,
    }];
    let c = Catalog {
        kits: &kits,
        ..BASELINE_CATALOG
    };
    let original = KitRegistry::baseline()
        .unwrap()
        .resolve_loadout(legacy_fixture_loadout())
        .unwrap();
    let changed = KitRegistry::new(c, &IMPLEMENTED_KITS)
        .unwrap()
        .resolve_loadout(legacy_fixture_loadout())
        .unwrap();
    assert_eq!(original, changed);
    let mut a = myva_sim::World::new();
    let mut b = myva_sim::World::new();
    a.spawn(original.kit(), 640_000, 1);
    b.spawn(changed.kit(), 640_000, 1);
    assert_eq!(a.state_hash(), b.state_hash());
}

fn pinned_replay() -> Replay {
    let mut battle = Battle::with_loadout(Mode::Boss, 1, legacy_fixture_loadout()).unwrap();
    battle.bout_mut().set_autopilot(true);
    for _ in 0..600 {
        battle.step(CommandFrame::IDLE);
    }
    battle.replay()
}

#[test]
fn selected_loadout_matches_legacy_each_tick_and_records_v4() {
    for mode in [Mode::Boss, Mode::Duel] {
        for round in [1, 7, 12] {
            let mut legacy = Battle::new(mode, round).unwrap();
            let mut selected = Battle::with_loadout(mode, round, legacy_fixture_loadout()).unwrap();
            legacy.bout_mut().set_autopilot(true);
            selected.bout_mut().set_autopilot(true);
            for _ in 0..3_600 {
                assert_eq!(
                    legacy.step(CommandFrame::IDLE),
                    selected.step(CommandFrame::IDLE)
                );
                assert_eq!(legacy.world().state_hash(), selected.world().state_hash());
            }
            let a = legacy.replay();
            let b = selected.replay();
            assert!(a.to_text().starts_with("myva-replay 3\n"));
            assert!(b.to_text().starts_with(&format!("{PINNED_HEADER}\n")));
            let parsed = Replay::parse(&b.to_text()).unwrap();
            assert_eq!(parsed, b);
            assert_eq!(
                parsed.play().unwrap().world.state_hash(),
                selected.world().state_hash()
            );
            let mut unpinned = b;
            unpinned.contract = None;
            assert_eq!(unpinned.to_text(), a.to_text());
        }
    }
}

#[test]
fn rematch_keeps_locked_loadout_and_contract_after_mode_switch() {
    let mut battle = Battle::with_loadout(Mode::Boss, 7, legacy_fixture_loadout()).unwrap();
    battle.bout_mut().set_autopilot(true);
    battle.step(CommandFrame::IDLE);
    let next = battle.rematch(Mode::Duel).unwrap();
    assert_eq!(next.bout().round(), 8);
    assert_eq!(next.bout().loadout(), battle.bout().loadout());
    assert!(next.bout().autopilot());
    assert_eq!(next.world().tick(), 0);
    assert_eq!(next.replay().contract.as_ref().unwrap().kits.len(), 1);
    assert!(next.replay().play().is_ok());
}

#[test]
fn replay_text_rejects_unknown_versions_hashes_and_incomplete_contracts() {
    let replay = pinned_replay();
    let text = replay.to_text();
    let kit = format!("kit long-luu 1 {:016x}", LONG_LUU_REVISION.config_hash);
    let rules = format!("rules {RULES_VERSION} {LEGACY_RULES_HASH:016x}");
    for bad in [
        text.replace(&kit, "kit long-luu 99 0000000000000000"),
        text.replace(&kit, "kit long-luu 1 0000000000000000"),
        text.replace(&kit, "kit unknown-kit 1 0000000000000000"),
        text.replace(&rules, &format!("rules 99 {LEGACY_RULES_HASH:016x}")),
        text.replace(&rules, "rules 3 0000000000000000"),
        text.replace(&format!("{rules}\n"), ""),
        text.replace(&format!("{kit}\n"), ""),
        text.replace(&kit, &format!("{kit}\n{kit}")),
        text.replace(&rules, &format!("{rules}\n{rules}")),
        text.replace(
            "seed 1",
            &format!(
                "kit quai-bun 1 {:016x}\nseed 1",
                MONSTER_REVISION.config_hash
            ),
        ),
    ] {
        assert!(
            Replay::parse(&bad).is_err(),
            "accepted invalid contract: {}",
            bad.lines().take(6).collect::<Vec<_>>().join("\n")
        );
    }
    assert!(matches!(
        Replay::parse(&text.replace("fighter long-luu", "fighter unknown-kit")),
        Err(ReplayError::Parse { .. })
    ));
}

#[test]
fn playback_checks_contract_even_when_parser_is_bypassed() {
    let base = pinned_replay();
    let mut changed = base.clone();
    changed.contract.as_mut().unwrap().rules_version += 1;
    assert!(matches!(
        changed.play(),
        Err(ReplayError::Contract(
            RegistryError::RulesVersionMismatch { .. }
        ))
    ));
    changed = base.clone();
    changed.contract.as_mut().unwrap().rules_hash ^= 1;
    assert!(matches!(
        changed.world(),
        Err(ReplayError::Contract(
            RegistryError::RulesHashMismatch { .. }
        ))
    ));
    changed = base.clone();
    changed.contract.as_mut().unwrap().kits[0].revision.version += 1;
    assert!(matches!(
        changed.play(),
        Err(ReplayError::Contract(RegistryError::VersionMismatch { .. }))
    ));
    changed = base.clone();
    changed.contract.as_mut().unwrap().kits[0]
        .revision
        .config_hash ^= 1;
    assert!(matches!(
        changed.play(),
        Err(ReplayError::Contract(RegistryError::HashMismatch { .. }))
    ));
    changed = base.clone();
    changed.contract.as_mut().unwrap().kits.clear();
    assert!(matches!(
        changed.play(),
        Err(ReplayError::Contract(RegistryError::MissingReplayPin(_)))
    ));
    const ALTERED: myva_sim::Kit = myva_sim::Kit {
        body: myva_sim::kit::Body {
            max_hp: 999,
            ..LONG_LUU.body
        },
        ..LONG_LUU
    };
    changed = base;
    changed.spawns[0].kit = &ALTERED;
    assert!(matches!(
        changed.play(),
        Err(ReplayError::Contract(RegistryError::UnregisteredSpec(_)))
    ));
    changed.contract = None;
    assert!(matches!(
        changed.play(),
        Err(ReplayError::Contract(RegistryError::UnregisteredSpec(_)))
    ));
}

#[test]
fn battle_boundary_rejects_invalid_selection_without_spawning() {
    assert!(matches!(
        Battle::with_loadout(Mode::Boss, 1, req("unknown-kit")),
        Err(RegistryError::UnknownKit(_))
    ));
    let r = LoadoutRequest {
        binding: LoadoutBinding::Branch(BranchId::new("synthetic-branch").unwrap()),
        ..legacy_fixture_loadout()
    };
    assert!(matches!(
        Battle::with_loadout(Mode::Boss, 1, r),
        Err(RegistryError::BranchMismatch { .. })
    ));
}

#[test]
fn registry_rejects_specs_that_overflow_or_exceed_arena_geometry() {
    // Correct fingerprints must not turn structurally unsafe authored data into a loadout.
    const BAD: &[myva_sim::Kit] = &[
        {
            let mut k = LONG_LUU;
            k.body.half_width = 800_001;
            k
        },
        {
            let mut k = LONG_LUU;
            k.body.walk_speed = i32::MAX / 100 + 1;
            k
        },
        {
            let mut k = LONG_LUU;
            k.body.height = i32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.body.max_hp = u32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.stamina_cost = u32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.skills[0].energy_cost = u32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.guard_pressure = u32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.damage = u32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.skills[1].counter_damage = Some(u32::MAX);
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.knockback = i32::MIN;
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.hitbox.front = i32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.hitbox.width = i32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.hitbox.bottom = i32::MIN;
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.hitbox.height = i32::MAX;
            k
        },
        {
            let mut k = LONG_LUU;
            k.skills[0].projectile = Some(myva_sim::kit::ProjectileSpec {
                speed: i32::MIN,
                ..LONG_LUU.skills[0].projectile.unwrap()
            });
            k
        },
        {
            let mut k = LONG_LUU;
            k.skills[0].projectile = Some(myva_sim::kit::ProjectileSpec {
                hitbox: myva_sim::kit::Hitbox {
                    height: i32::MAX,
                    ..LONG_LUU.skills[0].projectile.unwrap().hitbox
                },
                ..LONG_LUU.skills[0].projectile.unwrap()
            });
            k
        },
        {
            let mut k = LONG_LUU;
            k.ultimate = Some(myva_sim::kit::UltimateSpec {
                mach_cost: u32::MAX,
                ..LONG_LUU.ultimate.unwrap()
            });
            k
        },
        {
            let mut k = LONG_LUU;
            k.heavy.startup = u32::MAX;
            k
        },
    ];
    for kit in BAD {
        let entry = KitEntry {
            kit,
            revision: KitRevision {
                version: 1,
                config_hash: kit_config_hash(kit),
            },
            equip: EquipPermission::Player,
        };
        assert!(
            matches!(KitRegistry::new(BASELINE_CATALOG, &[entry]), Err(RegistryError::InvalidRegistry(e)) if e.contains(&RegistryValidationError::InvalidSpec(KitId::LONG_LUU))),
            "unsafe spec was accepted: {kit:?}"
        );
    }
}

#[test]
fn pinned_session_includes_actors_added_before_first_tick() {
    let (_, mut session) =
        myva_sim::battle::Bout::start_with_loadout(Mode::Boss, 1, legacy_fixture_loadout())
            .unwrap();
    session.spawn_monster(&myva_sim::monster::QUAI_BUN, 800_000, -1, None);
    let initial = session.replay();
    assert_eq!(initial.contract.as_ref().unwrap().kits.len(), 3);
    assert!(Replay::parse(&initial.to_text()).unwrap().play().is_ok());
    session.step();
    let replay = Replay::parse(&session.replay().to_text()).unwrap();
    assert_eq!(
        replay.play().unwrap().world.state_hash(),
        session.world().state_hash()
    );
}
