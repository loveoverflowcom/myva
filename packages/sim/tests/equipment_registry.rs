//! Equipment is an offline build/prototype contract; it is not active combat or entitlement.
use myva_sim::battle::{Battle, Mode};
use myva_sim::equipment::*;
use myva_sim::registry::{KitRegistry, LoadoutRequest, RegistryError, legacy_fixture_loadout};
use myva_sim::replay::{PINNED_HEADER, Replay};
use myva_sim::{CommandFrame, FighterId};

fn registry() -> EquipmentRegistry<'static> {
    EquipmentRegistry::baseline().unwrap()
}

fn repin(mut entry: ItemEntry) -> ItemEntry {
    entry.revision.config_hash = item_config_hash(entry.definition);
    entry
}

fn rules(rules: BuildRules) -> BuildRulesEntry {
    BuildRulesEntry {
        revision: EquipmentRevision { version: 1, config_hash: build_rules_hash(rules) },
        rules,
    }
}

fn contact(tick: u32, action_instance: u32) -> GuardContact {
    GuardContact {
        tick, action_instance, source: FighterId(1), target: FighterId(0), hit_index: 0,
        perfect: true, front: true, blockable: true, origin: ProcOrigin::Combat,
    }
}

#[test]
fn frozen_pins_cover_gameplay_data_but_not_display_labels() {
    let r = registry();
    assert_eq!(r.entries().len(), 3);
    assert_eq!(build_rules_hash(PROTOTYPE_BUILD_RULES.rules), 0x9dc0378661ca5573);
    for entry in r.entries() {
        assert_eq!(item_config_hash(entry.definition), entry.revision.config_hash);
        let renamed = ItemDefinition { display_name: "Synthetic name", ..entry.definition };
        assert_eq!(item_config_hash(renamed), entry.revision.config_hash);
        let changed = ItemDefinition { budget_cost: entry.definition.budget_cost + 1, ..entry.definition };
        assert_ne!(item_config_hash(changed), entry.revision.config_hash);
    }
    let changed = BuildRules { dash_cost_floor: 16, ..PROTOTYPE_BUILD_RULES.rules };
    assert_ne!(build_rules_hash(changed), PROTOTYPE_BUILD_RULES.revision.config_hash);
}

#[test]
fn runtime_ids_and_catalog_order_canonicalize_without_fallback() {
    let mut entries = PROTOTYPE_ITEMS;
    entries.reverse();
    let reversed = EquipmentRegistry::new(&entries, PROTOTYPE_BUILD_RULES).unwrap();
    let dynamic = String::from("phu-vu-bo");
    let a = registry().resolve_build(&[&dynamic, "an-phan-chan"]).unwrap();
    let b = reversed.resolve_build(&["an-phan-chan", &dynamic]).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.fingerprint(), b.fingerprint());
    assert_eq!(a.ids(), ["an-phan-chan", "phu-vu-bo"]);
    assert_eq!(a.budget_used(), 5);
    assert_eq!(registry().revalidate(a).unwrap(), a);
    for id in ["", "Ấn Phản Chấn", "AN-PHAN-CHAN", "a--b", "_item", "item "] {
        assert!(matches!(registry().entry(id), Err(EquipmentError::InvalidId(_))));
    }
    assert!(matches!(registry().entry(&"a".repeat(65)), Err(EquipmentError::InvalidId(_))));
    assert_eq!(registry().resolve_build(&["unknown-item"]), Err(EquipmentError::UnknownItem("unknown-item".into())));
}

#[test]
fn slots_duplicates_and_budget_are_checked_before_locking() {
    let r = registry();
    assert!(r.resolve_build(&[]).unwrap().is_empty());
    assert_eq!(r.resolve_build(&["phu-vu-bo", "phu-vu-bo"]), Err(EquipmentError::DuplicateItem(ItemId::PHU_VU_BO)));
    assert_eq!(r.resolve_build(&["a", "b", "c", "d"]), Err(EquipmentError::TooManySlots { limit: 3, actual: 4 }));
    assert_eq!(r.resolve_build(&["an-phan-chan", "giap-tran-mach", "phu-vu-bo"]), Err(EquipmentError::BudgetExceeded { limit: 6, actual: 8 }));
    assert_eq!(r.resolve_build(&["an-phan-chan", "giap-tran-mach"]).unwrap().budget_used(), 6);
}

#[test]
fn invalid_registry_and_numeric_domain_fail_explicitly() {
    let duplicate = [PROTOTYPE_ITEMS[0]; 2];
    assert!(matches!(EquipmentRegistry::new(&duplicate, PROTOTYPE_BUILD_RULES), Err(EquipmentError::InvalidRegistry(e)) if e.contains(&EquipmentValidationError::DuplicateId(ItemId::AN_PHAN_CHAN))));
    let mut bad = PROTOTYPE_ITEMS[0];
    bad.revision.version = 0;
    bad.revision.config_hash ^= 1;
    assert!(matches!(EquipmentRegistry::new(&[bad], PROTOTYPE_BUILD_RULES), Err(EquipmentError::InvalidRegistry(e)) if e.contains(&EquipmentValidationError::InvalidRevision(ItemId::AN_PHAN_CHAN)) && e.contains(&EquipmentValidationError::ConfigHashMismatch(ItemId::AN_PHAN_CHAN))));
    for effect in [
        EffectSpec::PerfectGuardRecovery { points: u32::MAX, cooldown_ticks: 300 },
        EffectSpec::PerfectGuardRecovery { points: 8, cooldown_ticks: 0 },
        EffectSpec::GuardPressureReduction { basis_points: BASIS_POINTS },
        EffectSpec::DashCostReduction { basis_points: u16::MAX },
    ] {
        bad = repin(ItemEntry { definition: ItemDefinition { effect, ..PROTOTYPE_ITEMS[0].definition }, ..PROTOTYPE_ITEMS[0] });
        assert!(matches!(EquipmentRegistry::new(&[bad], PROTOTYPE_BUILD_RULES), Err(EquipmentError::InvalidRegistry(e)) if e.contains(&EquipmentValidationError::InvalidDefinition(ItemId::AN_PHAN_CHAN))));
    }
    for invalid in [
        BuildRules { max_slots: 4, ..PROTOTYPE_BUILD_RULES.rules },
        BuildRules { budget: 0, ..PROTOTYPE_BUILD_RULES.rules },
        BuildRules { dash_cost_floor: 0, ..PROTOTYPE_BUILD_RULES.rules },
        BuildRules { max_guard_reduction_bps: BASIS_POINTS, ..PROTOTYPE_BUILD_RULES.rules },
        BuildRules { max_dash_reduction_bps: BASIS_POINTS, ..PROTOTYPE_BUILD_RULES.rules },
        BuildRules { min_recovery_cooldown_ticks: 0, ..PROTOTYPE_BUILD_RULES.rules },
        BuildRules { max_recovery_points: u32::MAX, ..PROTOTYPE_BUILD_RULES.rules },
    ] {
        assert!(matches!(EquipmentRegistry::new(&[], rules(invalid)), Err(EquipmentError::InvalidRegistry(e)) if e.contains(&EquipmentValidationError::InvalidRules)));
    }
}

#[test]
fn stacking_is_additive_bounded_and_never_cooldown_resetting() {
    let mut second = PROTOTYPE_ITEMS[2];
    second.definition.id = ItemId::new("second-dash-item").unwrap();
    second.definition.budget_cost = 1;
    second.definition.effect = EffectSpec::DashCostReduction { basis_points: 500 };
    second = repin(second);
    let entries = [PROTOTYPE_ITEMS[2], second];
    let default = EquipmentRegistry::new(&entries, PROTOTYPE_BUILD_RULES).unwrap();
    assert_eq!(default.resolve_build(&["phu-vu-bo", "second-dash-item"]), Err(EquipmentError::GroupLimitExceeded { group: EffectGroup::DashCost, limit: 1 }));
    let r = EquipmentRegistry::new(&entries, rules(BuildRules { group_limits: [1, 1, 2], ..PROTOTYPE_BUILD_RULES.rules })).unwrap();
    assert_eq!(r.resolve_build(&["phu-vu-bo", "second-dash-item"]), Err(EquipmentError::EffectCapExceeded(EffectGroup::DashCost)));
    let r = EquipmentRegistry::new(&entries, rules(BuildRules { group_limits: [1, 1, 2], max_dash_reduction_bps: 2_500, ..PROTOTYPE_BUILD_RULES.rules })).unwrap();
    assert_eq!(r.resolve_build(&["phu-vu-bo", "second-dash-item"]).unwrap().dash_cost(100), 75);
    let mut second = PROTOTYPE_ITEMS[0];
    second.definition.id = ItemId::new("second-recovery-item").unwrap();
    second.definition.budget_cost = 1;
    second.definition.effect = EffectSpec::PerfectGuardRecovery { points: 2, cooldown_ticks: 600 };
    second = repin(second);
    let entries = [PROTOTYPE_ITEMS[0], second];
    let r = EquipmentRegistry::new(&entries, rules(BuildRules { group_limits: [2, 1, 1], ..PROTOTYPE_BUILD_RULES.rules })).unwrap();
    let build = r.resolve_build(&["an-phan-chan", "second-recovery-item"]).unwrap();
    let first = guard_recovery(build, &GuardRecoveryState::default(), contact(0, 1), 0, 100).unwrap();
    assert_eq!(first.restored_points, 10);
    let early = guard_recovery(build, &first.state, contact(300, 2), 0, 100).unwrap();
    assert_eq!(early.restored_points, 0);
    assert_eq!(guard_recovery(build, &early.state, contact(600, 3), 0, 100).unwrap().restored_points, 10);
}

#[test]
fn stale_item_or_build_rule_pins_never_auto_upgrade() {
    let r = registry();
    let build = r.resolve_build(&["phu-vu-bo"]).unwrap();
    let pin = build.pins().next().unwrap();
    assert!(matches!(r.resolve(ItemPin { revision: EquipmentRevision { version: 99, ..pin.revision }, ..pin }), Err(EquipmentError::VersionMismatch { .. })));
    assert!(matches!(r.resolve(ItemPin { revision: EquipmentRevision { config_hash: 1, ..pin.revision }, ..pin }), Err(EquipmentError::HashMismatch { .. })));
    let mut entries = PROTOTYPE_ITEMS;
    entries[2].definition.effect = EffectSpec::DashCostReduction { basis_points: 1_000 };
    entries[2] = repin(entries[2]);
    entries[2].revision.version = 2;
    let changed = EquipmentRegistry::new(&entries, PROTOTYPE_BUILD_RULES).unwrap();
    assert!(matches!(changed.revalidate(build), Err(EquipmentError::VersionMismatch { .. })));
    let changed = EquipmentRegistry::new(&PROTOTYPE_ITEMS, rules(BuildRules { budget: 7, ..PROTOTYPE_BUILD_RULES.rules })).unwrap();
    assert!(matches!(changed.revalidate(build), Err(EquipmentError::RulesRevisionMismatch { .. })));
}

#[test]
fn lock_is_usable_headlessly_but_combat_rejects_before_spawn() {
    let kits = KitRegistry::baseline().unwrap();
    let request = LoadoutRequest { equipment: &["phu-vu-bo", "an-phan-chan"], ..legacy_fixture_loadout() };
    let locked = kits.resolve_loadout(request).unwrap();
    assert_eq!(locked.equipment().dash_cost(25), 20);
    assert_eq!(locked.request().equipment, ["an-phan-chan", "phu-vu-bo"]);
    assert_eq!(kits.revalidate_loadout(locked).unwrap(), locked);
    assert!(matches!(Battle::with_loadout(Mode::Boss, 7, request), Err(RegistryError::Equipment(EquipmentError::RuntimeNotImplemented))));
}

#[test]
fn cost_and_guard_pressure_properties_hold_at_integer_boundaries() {
    let r = registry();
    let empty = r.resolve_build(&[]).unwrap();
    let dash = r.resolve_build(&["phu-vu-bo"]).unwrap();
    let guard = r.resolve_build(&["giap-tran-mach"]).unwrap();
    for value in (0..=10_000).chain([u32::MAX - 1, u32::MAX]) {
        assert_eq!(empty.dash_cost(value), value);
        assert_eq!(empty.guard_pressure(value, true, true), value);
        let cost = dash.dash_cost(value);
        assert!(cost <= value);
        assert!(cost >= 15.min(value));
        let pressure = guard.guard_pressure(value, true, true);
        assert!(pressure <= value);
        if value > 0 { assert!(pressure > 0); }
        assert_eq!(guard.guard_pressure(value, false, true), value);
        assert_eq!(guard.guard_pressure(value, true, false), value);
    }
}

#[test]
fn recovery_is_capped_single_source_action_and_cooldown_bound() {
    let build = registry().resolve_build(&["an-phan-chan"]).unwrap();
    let initial = GuardRecoveryState::default();
    let first = guard_recovery(build, &initial, contact(0, 1), 95, 100).unwrap();
    assert_eq!((first.restored_points, first.stamina_points), (5, 100));
    assert_eq!(initial, GuardRecoveryState::default()); // state-in was not mutated
    let repeated = guard_recovery(build, &first.state, contact(300, 1), 0, 100).unwrap();
    assert_eq!(repeated.restored_points, 0);
    let fresh = guard_recovery(build, &repeated.state, contact(300, 2), 0, 100).unwrap();
    assert_eq!(fresh.restored_points, 8);
    let suppressed = guard_recovery(build, &fresh.state, contact(301, 3), 0, 100).unwrap();
    assert_eq!(suppressed.restored_points, 0);
    let delayed_hit = guard_recovery(build, &suppressed.state, GuardContact { tick: 600, hit_index: 1, ..contact(301, 3) }, 0, 100).unwrap();
    assert_eq!(delayed_hit.restored_points, 0);
    let stale = guard_recovery(build, &delayed_hit.state, contact(600, 2), 0, 100).unwrap();
    assert_eq!(stale.restored_points, 0);
    assert_eq!(guard_recovery(build, &stale.state, contact(600, 4), 0, 100).unwrap().restored_points, 8);
    assert_eq!(guard_recovery(build, &fresh.state, contact(299, 4), 0, 100), Err(EffectError::TimeReversed));
    assert_eq!(guard_recovery(build, &fresh.state, GuardContact { target: FighterId(9), ..contact(301, 4) }, 0, 100), Err(EffectError::TargetMismatch));
    assert_eq!(guard_recovery(registry().resolve_build(&[]).unwrap(), &fresh.state, contact(301, 4), 0, 100), Err(EffectError::BuildMismatch));
    assert_eq!(guard_recovery(build, &initial, contact(0, 1), 101, 100), Err(EffectError::InvalidResource));
}

#[test]
fn equipment_and_nonperfect_back_unblockable_contacts_cannot_proc() {
    let build = registry().resolve_build(&["an-phan-chan"]).unwrap();
    for c in [
        GuardContact { origin: ProcOrigin::Equipment, ..contact(0, 1) },
        GuardContact { perfect: false, ..contact(0, 1) },
        GuardContact { front: false, ..contact(0, 1) },
        GuardContact { blockable: false, ..contact(0, 1) },
    ] {
        assert_eq!(guard_recovery(build, &GuardRecoveryState::default(), c, 0, 100).unwrap().restored_points, 0);
    }
    let empty = registry().resolve_build(&[]).unwrap();
    assert_eq!(guard_recovery(empty, &GuardRecoveryState::default(), contact(0, 1), 0, 100).unwrap().restored_points, 0);
}

#[test]
fn repeated_contacts_cannot_create_unbounded_refund_rate_or_tick_overflow() {
    let build = registry().resolve_build(&["an-phan-chan"]).unwrap();
    let mut state = GuardRecoveryState::default();
    let mut refunded = 0;
    for tick in 0..=3_000 {
        let result = guard_recovery(build, &state, contact(tick, tick), 0, 100).unwrap();
        state = result.state;
        refunded += result.restored_points;
    }
    assert_eq!(refunded, 11 * 8); // no more than one proc per 300 ticks
    let last = guard_recovery(build, &state, contact(u32::MAX, u32::MAX), 0, u32::MAX).unwrap();
    assert_eq!(last.restored_points, 8);
    let other_source = GuardContact { source: FighterId(2), ..contact(u32::MAX, 1) };
    assert_eq!(guard_recovery(build, &last.state, other_source, 0, u32::MAX).unwrap().restored_points, 0);
}

#[test]
fn equipment_wire_lines_are_rejected_by_v3_and_v4_no_silent_fallback() {
    let mut battle = Battle::with_loadout(Mode::Boss, 7, legacy_fixture_loadout()).unwrap();
    battle.step(CommandFrame::default()).unwrap();
    let v4 = battle.replay().to_text();
    assert!(v4.starts_with(PINNED_HEADER));
    assert_eq!(Replay::parse(&v4).unwrap().to_text(), v4);
    let injected = v4.replace("end ", "item phu-vu-bo 1 782d9b33f5a4124b\nend ");
    assert!(Replay::parse(&injected).is_err());
    let v3 = Battle::new(Mode::Boss, 7).unwrap().replay().to_text();
    assert!(v3.starts_with("myva-replay 3\n"));
    assert!(Replay::parse(&v3.replace("end ", "equipment phu-vu-bo\nend ")).is_err());
}

#[test]
fn headless_cli_checks_valid_build_and_returns_nonzero_for_invalid_builds() {
    let cli = env!("CARGO_BIN_EXE_myva-replay");
    let valid = std::process::Command::new(cli).args(["build-check", "phu-vu-bo", "an-phan-chan"]).output().unwrap();
    assert!(valid.status.success());
    let stdout = String::from_utf8(valid.stdout).unwrap();
    assert!(stdout.contains("offline build: budget=5"));
    assert!(stdout.contains("NOT_IMPLEMENTED"));
    for ids in [
        vec!["unknown-item"],
        vec!["phu-vu-bo", "phu-vu-bo"],
        vec!["an-phan-chan", "giap-tran-mach", "phu-vu-bo"],
    ] {
        assert!(!std::process::Command::new(cli).arg("build-check").args(ids).status().unwrap().success());
    }
}
