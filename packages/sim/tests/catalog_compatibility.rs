//! Golden captured from develop e4bdba46a5750ab623c6fac1be120e5835fc093a,
//! Rust 1.97.1 / aarch64-apple-darwin. Metadata PR must preserve legacy combat/replay bytes.
use myva_sim::CommandFrame;
use myva_sim::battle::{Battle, Mode};
use myva_sim::catalog::{
    BASELINE_CATALOG, Catalog, KitId, Provenance, ProvenanceKind, ResolveError,
};

fn replay_with_metadata(catalog: Catalog<'_>) -> (String, u64) {
    catalog.validate().unwrap();
    let mut battle = Battle::new(Mode::Boss, 1);
    battle.bout_mut().set_autopilot(true);
    while !battle.is_settled() && battle.world().tick() < 18_000 {
        // A presentation consumer reads metadata alongside combat, without mutating Kit.
        assert!(!catalog.kits[0].metadata.display_name.is_empty());
        battle.step(CommandFrame::IDLE);
    }
    let replay = battle.replay();
    assert_eq!(replay.ticks, 3_763);
    assert_eq!(replay.checkpoints.len(), 63);
    assert_eq!(
        replay.play().unwrap().world.state_hash(),
        battle.world().state_hash()
    );
    (replay.to_text(), battle.world().state_hash())
}

#[test]
fn metadata_changes_preserve_legacy_combat_hash_and_replay_bytes() {
    let baseline = replay_with_metadata(BASELINE_CATALOG);
    let mut factions = *BASELINE_CATALOG.factions.first_chunk::<5>().unwrap();
    factions[0].metadata.display_name = "Presentation name changed";
    let mut kits = [BASELINE_CATALOG.kits[0]];
    kits[0].metadata.display_name = "Fixture label changed";
    kits[0].metadata.provenance = &[Provenance {
        reference: "new presentation source",
        kind: ProvenanceKind::GameFiction,
    }];
    let changed = Catalog {
        factions: &factions,
        kits: &kits,
        ..BASELINE_CATALOG
    };
    assert_eq!(
        changed.hierarchy(KitId::LONG_LUU),
        Err(ResolveError::UnmappedFixture(KitId::LONG_LUU))
    );
    assert_eq!(replay_with_metadata(changed), baseline);
    assert_eq!(baseline.1, 0x227d609f34dcd9e4);
    // Covers every serialized input and checkpoint, kit IDs, labels hashed via Kit,
    // and protocol text. FNV is only a regression digest, not a replay/hash redesign.
    assert_eq!(baseline.0.len(), 179_328);
    let digest = baseline.0.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    assert_eq!(digest, 0x5c27a4f922c21eb2);
}
