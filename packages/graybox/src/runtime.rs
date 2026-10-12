//! Bevy backend for the bounded streaming planner. This module owns presentation only.
//!
//! Bevy 0.20 has no public request-abort API. Cancellation invalidates ownership, but a pending
//! strong handle remains in `flights` until AssetServer reports a terminal state. The planner's
//! slot is never acknowledged early. No transport cancellation or physical GPU release is claimed.

use std::collections::BTreeMap;

use crate::mapping::{self, KIT_ASSETS, MappingError};
use bevy::asset::LoadState;
use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use myva_assets::Manifest;
use myva_sim::battle::Mode;
use myva_sim::registry::KitPin;

use crate::art::{self, Art, ArtCatalog, ArtEntity, ArtFixture, ArtStatus, ArtTables};
use crate::fight::Match;
use crate::streaming::{
    BundleKey, BundleKind, BundleState, Completion, Limits, LoadFailure, LoadTicket, SceneState,
    StreamingPlanner,
};

pub const LIMITS: Limits = Limits {
    max_live_bundles: 4,
    max_in_flight: 2,
    max_attempts: 3,
    request_timeout_ms: 10_000,
    retry_backoff_ms: 250,
};

/// Explicit presentation lifecycle commands. They never pause or mutate gameplay/replay.
#[derive(Message, Clone, Copy, Debug)]
pub enum ArtCommand {
    SetEnabled(bool),
    Retry,
}

struct Flight {
    handle: Handle<ArtFixture>,
    started_ms: u64,
    cancelled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeSummary {
    pub scene: &'static str,
    pub generation: u64,
    pub selected_kit: String,
    pub mapping_error: Option<String>,
    pub bundles: usize,
    pub planner_owners: usize,
    pub flights: usize,
    pub cancelled_draining: usize,
    pub catalog_pending: bool,
    pub catalog_attempts: u32,
    pub catalog_timed_out: bool,
    pub cancellations: u64,
    pub retries: u64,
    pub completed_loads: u64,
    pub failed_loads: u64,
    pub sprite_image_owners: usize,
    pub table_image_owners: usize,
    pub table_layout_owners: usize,
    pub fixture_assets: usize,
    pub image_assets: usize,
    pub atlas_assets: usize,
    /// RGBA8 base texture bytes, no mipmaps/buffers/driver allocations. This is an estimate.
    pub estimated_texture_bytes: u64,
    /// Request to loaded-with-dependencies; excludes proof of GPU upload/presentation.
    pub load_latency_ms: Vec<u64>,
}

impl RuntimeSummary {
    pub fn to_json(&self) -> String {
        let latencies = self
            .load_latency_ms
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let selection = serde_json::json!({"kit_id": self.selected_kit, "mapping_error": self.mapping_error, "art_review": "placeholder_fixture"});
        format!(
            "{{\"selection\":{selection},\"scene\":\"{}\",\"generation\":{},\"bundles\":{},\"planner_owners\":{},\"flights\":{},\"cancelled_draining\":{},\"catalog_pending\":{},\"catalog_attempts\":{},\"catalog_timed_out\":{},\"cancellations\":{},\"retries\":{},\"completed_loads\":{},\"failed_loads\":{},\"sprite_image_owners\":{},\"table_image_owners\":{},\"table_layout_owners\":{},\"fixture_assets\":{},\"image_assets\":{},\"atlas_assets\":{},\"estimated_texture_bytes\":{},\"gpu_estimate_method\":\"retained bundle RGBA8 base textures, no mipmaps; excludes buffers and driver memory\",\"load_latency_source\":\"AssetServer request to loaded-with-dependencies, not GPU upload\",\"load_latency_ms\":[{}],\"transport_abort_supported\":false}}",
            self.scene,
            self.generation,
            self.bundles,
            self.planner_owners,
            self.flights,
            self.cancelled_draining,
            self.catalog_pending,
            self.catalog_attempts,
            self.catalog_timed_out,
            self.cancellations,
            self.retries,
            self.completed_loads,
            self.failed_loads,
            self.sprite_image_owners,
            self.table_image_owners,
            self.table_layout_owners,
            self.fixture_assets,
            self.image_assets,
            self.atlas_assets,
            self.estimated_texture_bytes,
            latencies,
        )
    }
}

#[derive(Resource)]
pub struct StreamRuntime {
    planner: StreamingPlanner<Handle<ArtFixture>>,
    catalog: Option<Manifest>,
    catalog_handle: Option<Handle<ArtCatalog>>,
    catalog_attempts: u32,
    catalog_retry_ms: u64,
    catalog_started_ms: u64,
    catalog_timed_out: bool,
    flights: BTreeMap<LoadTicket, Flight>,
    failures: BTreeMap<BundleKey, String>,
    keys: Vec<BundleKey>,
    installed: Vec<BundleKey>,
    active_selection: Option<(Mode, KitPin)>,
    planned_selection: Option<(Mode, KitPin)>,
    was_enabled: bool,
    summary: RuntimeSummary,
}

impl Default for StreamRuntime {
    fn default() -> Self {
        Self {
            planner: StreamingPlanner::new(LIMITS).expect("nonzero runtime limits"),
            catalog: None,
            catalog_handle: None,
            catalog_attempts: 0,
            catalog_retry_ms: 0,
            catalog_started_ms: 0,
            catalog_timed_out: false,
            flights: BTreeMap::new(),
            failures: BTreeMap::new(),
            keys: Vec::new(),
            installed: Vec::new(),
            active_selection: None,
            planned_selection: None,
            was_enabled: false,
            summary: RuntimeSummary::default(),
        }
    }
}

impl StreamRuntime {
    pub fn summary(&self) -> RuntimeSummary {
        self.summary.clone()
    }

    fn cancel(&mut self, tickets: Vec<LoadTicket>) {
        for ticket in tickets {
            if let Some(flight) = self.flights.get_mut(&ticket) {
                flight.cancelled = true;
            }
            self.summary.cancellations += 1;
        }
    }

    fn enter(&mut self, mode: Mode, pin: KitPin, now: u64) -> Result<(), MappingError> {
        self.planned_selection = Some((mode, pin));
        self.summary.selected_kit = pin.id.as_str().to_owned();
        self.summary.mapping_error = None;
        let catalog = self
            .catalog
            .as_ref()
            .ok_or_else(|| MappingError::Planner("catalog unavailable".into()))?;
        let planned = scene_keys(catalog, mode, pin.id.as_str());
        let (required, optional) = match planned {
            Ok(plan) => plan,
            Err(error) => {
                let cancellations = self
                    .planner
                    .exit_scene()
                    .map_err(|e| MappingError::Planner(format!("{e:?}")))?;
                self.cancel(cancellations);
                self.active_selection = None;
                self.keys.clear();
                self.installed.clear();
                self.summary.mapping_error = Some(error.to_string());
                return Err(error);
            }
        };
        let change = self
            .planner
            .enter_scene(&required, &optional, now)
            .map_err(|e| MappingError::Planner(format!("{e:?}")))?;
        self.cancel(change.cancellations);
        self.summary.generation = change.generation;
        self.keys = required.into_iter().chain(optional).collect();
        self.active_selection = Some((mode, pin));
        self.installed.clear();
        Ok(())
    }
}

fn kind(kind: myva_assets::BundleKind) -> BundleKind {
    match kind {
        myva_assets::BundleKind::Character => BundleKind::Character,
        myva_assets::BundleKind::Map => BundleKind::Map,
        myva_assets::BundleKind::Boss => BundleKind::Boss,
        myva_assets::BundleKind::Vfx => BundleKind::Vfx,
    }
}

fn key(manifest: &Manifest, bundle: &myva_assets::Bundle) -> BundleKey {
    // Debug representations are deterministic for this schema (ordered Vecs, no HashMaps).
    // Any catalog metadata/hash/version change invalidates retained content identities.
    let identity = myva_assets::sha256_hex(format!("{manifest:?}").as_bytes());
    BundleKey {
        kind: kind(bundle.kind),
        id: format!("{}@{}", bundle.id, identity),
    }
}

fn bundle_id(key: &BundleKey) -> &str {
    key.id.split_once('@').map_or(key.id.as_str(), |(id, _)| id)
}

fn path(key: &BundleKey) -> String {
    format!("runtime-{}.myva", bundle_id(key))
}

fn scene_keys(
    manifest: &Manifest,
    mode: Mode,
    kit: &str,
) -> Result<(Vec<BundleKey>, Vec<BundleKey>), MappingError> {
    let (_, player) = mapping::resolve(manifest, KIT_ASSETS, kit)?;
    let map = mapping::bundle(manifest, "map.dap_co", myva_assets::BundleKind::Map)?;
    let vfx = mapping::bundle(manifest, "vfx.combat", myva_assets::BundleKind::Vfx)?;
    let mut required = vec![key(manifest, player), key(manifest, map)];
    if mode == Mode::Boss {
        let (_, boss) = mapping::resolve(manifest, KIT_ASSETS, "ke-giu-dap")?;
        required.push(key(manifest, boss));
    }
    Ok((required, vec![key(manifest, vfx)]))
}

fn projection(catalog: &Manifest, key: &BundleKey) -> Option<Manifest> {
    let bundle = catalog.bundles.iter().find(|b| b.id == bundle_id(key))?;
    let mut expected = catalog.clone();
    expected.assets.retain(|a| bundle.assets.contains(&a.id));
    for asset in &mut expected.assets {
        asset.path = format!("fixture/{}", asset.path);
    }
    expected.bundles = vec![bundle.clone()];
    Some(expected)
}

/// Recheck the immutable projection at completion as well as in the loader.
fn valid_projection(catalog: &Manifest, fixture: &ArtFixture, key: &BundleKey) -> bool {
    projection(catalog, key).as_ref() == Some(&fixture.manifest)
}

fn error_code(message: &str) -> String {
    message
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map_or("io", |(code, _)| code)
        .to_owned()
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuditSet;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<StreamRuntime>()
        .add_message::<ArtCommand>()
        .add_systems(Last, audit.in_set(AuditSet));
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn drive(
    mut commands: Commands,
    mut runtime: ResMut<StreamRuntime>,
    mut art_state: ResMut<Art>,
    mut tables: ResMut<ArtTables>,
    server: Res<AssetServer>,
    catalogs: Res<Assets<ArtCatalog>>,
    fixtures: Res<Assets<ArtFixture>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    entities: Query<Entity, With<ArtEntity>>,
    game: Res<Match>,
    time: Res<Time<Real>>,
    mut messages: MessageReader<ArtCommand>,
) {
    let now = time.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let mut retry = false;
    for command in messages.read() {
        match command {
            ArtCommand::SetEnabled(on) => art_state.enabled = *on,
            ArtCommand::Retry => retry = true,
        }
    }
    let mode = game.bout.mode();
    let pin = game.bout.loadout().pin();
    if art_state.enabled
        && !runtime.was_enabled
        && runtime.catalog.is_none()
        && runtime.catalog_handle.is_none()
    {
        runtime.catalog_attempts = 0;
        runtime.catalog_retry_ms = now;
    }
    runtime.was_enabled = art_state.enabled;
    if !art_state.enabled && runtime.active_selection.is_some() {
        let cancellations = runtime.planner.exit_scene().expect("generation capacity");
        runtime.cancel(cancellations);
        runtime.active_selection = None;
        runtime.planned_selection = None;
        runtime.keys.clear();
        runtime.installed.clear();
        clear(&mut commands, &entities, &mut tables);
    }
    if !art_state.enabled {
        runtime.planned_selection = None;
        art_state.status = ArtStatus::Off;
    }

    // Catalog requests also drain to a terminal state when disabled/timed out. Metadata has no
    // texture owners; retaining its parsed value is safe and prevents redundant catalog I/O.
    if let Some(handle) = runtime.catalog_handle.clone() {
        if now.saturating_sub(runtime.catalog_started_ms) >= LIMITS.request_timeout_ms {
            runtime.catalog_timed_out = true;
        }
        let state = server.load_state(&handle);
        let failed = match state {
            LoadState::Loaded => {
                if runtime.catalog_timed_out {
                    Some("timed_out".to_owned())
                } else if let Some(catalog) = catalogs.get(&handle) {
                    runtime.catalog = Some(catalog.0.clone());
                    runtime.catalog_handle = None;
                    None
                } else {
                    None
                }
            }
            LoadState::Failed(error) => Some(error_code(&error.to_string())),
            LoadState::Loading | LoadState::NotLoaded => None,
        };
        if let Some(code) = failed {
            runtime.catalog_handle = None;
            runtime.catalog_retry_ms =
                now.saturating_add(LIMITS.retry_backoff_ms * u64::from(runtime.catalog_attempts));
            if runtime.catalog_attempts >= LIMITS.max_attempts && art_state.enabled {
                art_state.status = ArtStatus::Failed {
                    code,
                    message: "catalog retries exhausted".into(),
                };
            }
        }
    }
    if retry && runtime.catalog.is_none() && runtime.catalog_handle.is_none() {
        runtime.catalog_attempts = 0;
        runtime.catalog_retry_ms = now;
    }
    if art_state.enabled
        && runtime.catalog.is_none()
        && runtime.catalog_handle.is_none()
        && runtime.catalog_attempts < LIMITS.max_attempts
        && now >= runtime.catalog_retry_ms
    {
        runtime.catalog_attempts += 1;
        runtime.catalog_started_ms = now;
        runtime.catalog_timed_out = false;
        runtime.catalog_handle = Some(server.load(art::FIXTURE_MANIFEST));
        art_state.status = ArtStatus::Loading;
    }
    if art_state.enabled
        && runtime.catalog.is_some()
        && (runtime.planned_selection != Some((mode, pin)) || retry)
    {
        clear(&mut commands, &entities, &mut tables);
        match runtime.enter(mode, pin, now) {
            Ok(()) => art_state.status = ArtStatus::Loading,
            Err(error) => {
                art_state.status = ArtStatus::Failed {
                    code: "asset_mapping".into(),
                    message: error.to_string(),
                }
            }
        }
    }

    // Both live and cancelled Bevy requests must settle. Never acknowledge an in-progress
    // cancellation, and never install a stale completion in the new scene.
    let finished: Vec<_> = runtime
        .flights
        .iter()
        .filter_map(|(ticket, flight)| {
            let result = match server.load_state(&flight.handle) {
                LoadState::Loaded if server.is_loaded_with_dependencies(&flight.handle) => {
                    let fixture = fixtures.get(&flight.handle)?;
                    let mapped = KIT_ASSETS
                        .iter()
                        .find(|m| m.bundle == bundle_id(&ticket.bundle));
                    let mapping_valid = match mapped {
                        Some(mapping) => mapping::verify_fixture(fixture, mapping).is_ok(),
                        None => {
                            !matches!(ticket.bundle.kind, BundleKind::Character | BundleKind::Boss)
                                && fixture.documents.sheets.values().all(|s| {
                                    !matches!(
                                        s.role,
                                        myva_assets::Role::Character | myva_assets::Role::Boss
                                    )
                                })
                        }
                    };
                    if runtime
                        .catalog
                        .as_ref()
                        .is_some_and(|c| valid_projection(c, fixture, &ticket.bundle))
                    {
                        if mapping_valid {
                            Ok(flight.handle.clone())
                        } else {
                            Err(LoadFailure::Backend("asset_mapping".into()))
                        }
                    } else {
                        Err(LoadFailure::Backend("projection_mismatch".into()))
                    }
                }
                LoadState::Failed(error) => {
                    Err(LoadFailure::Backend(error_code(&error.to_string())))
                }
                _ => return None,
            };
            Some((ticket.clone(), result))
        })
        .collect();
    for (ticket, result) in finished {
        let Some(flight) = runtime.flights.remove(&ticket) else {
            continue;
        };
        let succeeded = result.is_ok();
        let failure_code = match &result {
            Err(LoadFailure::Backend(code)) => Some(code.clone()),
            _ => None,
        };
        let completion = runtime.planner.complete(ticket.clone(), result, now);
        if completion == Completion::Applied {
            if let Some(code) = failure_code {
                runtime.failures.insert(ticket.bundle.clone(), code);
                runtime.summary.failed_loads += 1;
            } else if succeeded {
                runtime.summary.completed_loads += 1;
                if runtime.summary.load_latency_ms.len() == 256 {
                    runtime.summary.load_latency_ms.remove(0);
                }
                runtime
                    .summary
                    .load_latency_ms
                    .push(now.saturating_sub(flight.started_ms));
            }
        } else if completion == Completion::TimedOut {
            runtime.summary.failed_loads += 1;
        }
    }

    let actions = runtime.planner.poll(now);
    runtime.summary.failed_loads += actions.cancellations.len() as u64;
    runtime.cancel(actions.cancellations);
    for ticket in actions.loads {
        let asset_path = path(&ticket.bundle);
        if ticket.attempt > 1 {
            runtime.summary.retries += 1;
        }
        // Request retries of Failed/NotLoaded assets are implemented by Bevy itself. Calling
        // reload() here would create detached force-load tasks outside the planner bound.
        let expected = runtime
            .catalog
            .as_ref()
            .and_then(|catalog| projection(catalog, &ticket.bundle));
        let handle = server
            .load_builder()
            .with_settings(move |settings: &mut art::ArtSettings| {
                settings.expected = expected.clone()
            })
            .load::<ArtFixture>(asset_path);
        runtime.flights.insert(
            ticket,
            Flight {
                handle,
                started_ms: now,
                cancelled: false,
            },
        );
    }

    if runtime.active_selection.is_some() {
        match runtime.planner.scene_state() {
            SceneState::Ready => {
                let ready: Vec<_> = runtime
                    .keys
                    .iter()
                    .filter(|key| runtime.planner.bundle_state(key) == Some(&BundleState::Ready))
                    .cloned()
                    .collect();
                if ready != runtime.installed {
                    // Optional content can settle after the required presentation is already
                    // visible. Merge only the newly ready load units: replacing healthy tables
                    // and entities here would briefly hide bodies and reset ongoing effects.
                    let mut map_added = false;
                    for key in ready.iter().filter(|key| !runtime.installed.contains(key)) {
                        if let Some(handle) = runtime.planner.owned_handle(key)
                            && let Some(fixture) = fixtures.get(handle)
                        {
                            art::install_bundle(&mut tables, fixture, &mut layouts);
                            map_added |= key.kind == BundleKind::Map;
                        }
                    }
                    if map_added {
                        art::spawn_map(&mut commands, &tables);
                    }
                    runtime.installed = ready;
                }
                art_state.content_version =
                    runtime.catalog.as_ref().map(|c| c.content_version.clone());
                art_state.status = ArtStatus::Ready;
            }
            SceneState::Blocked => {
                clear(&mut commands, &entities, &mut tables);
                runtime.installed.clear();
                let code = runtime
                    .keys
                    .iter()
                    .find_map(|key| match runtime.planner.bundle_state(key) {
                        Some(BundleState::RequiredBlocked(LoadFailure::TimedOut)) => {
                            Some("timed_out".to_owned())
                        }
                        Some(BundleState::RequiredBlocked(_)) => Some(
                            runtime
                                .failures
                                .get(key)
                                .cloned()
                                .unwrap_or_else(|| "io".into()),
                        ),
                        _ => None,
                    })
                    .unwrap_or_else(|| "io".into());
                art_state.status = ArtStatus::Failed { code, message: "required art bundle retries exhausted; geometric combat presentation remains".into() };
            }
            SceneState::Preloading => art_state.status = ArtStatus::Loading,
            SceneState::Idle => {}
        }
    }
}

fn clear(
    commands: &mut Commands,
    entities: &Query<Entity, With<ArtEntity>>,
    tables: &mut ArtTables,
) {
    for entity in entities.iter() {
        commands.entity(entity).try_despawn();
    }
    *tables = ArtTables::default();
}

#[allow(clippy::too_many_arguments)]
fn audit(
    mut runtime: ResMut<StreamRuntime>,
    entities: Query<&Sprite, With<ArtEntity>>,
    tables: Res<ArtTables>,
    fixtures: Res<Assets<ArtFixture>>,
    images: Res<Assets<Image>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    let estimated = runtime
        .keys
        .iter()
        .filter_map(|key| runtime.planner.owned_handle(key))
        .filter_map(|handle| fixtures.get(handle))
        .flat_map(|fixture| &fixture.manifest.assets)
        .filter_map(|asset| asset.texture)
        .map(|info| info.decoded_bytes())
        .sum();
    runtime.summary.scene = match runtime.planner.scene_state() {
        SceneState::Idle => "idle",
        SceneState::Preloading => "preloading",
        SceneState::Ready => "ready",
        SceneState::Blocked => "blocked",
    };
    runtime.summary.bundles = runtime.planner.live_bundle_count();
    runtime.summary.planner_owners = runtime.planner.owned_handle_count();
    runtime.summary.flights = runtime.planner.in_flight_count();
    runtime.summary.catalog_pending = runtime.catalog_handle.is_some();
    runtime.summary.catalog_attempts = runtime.catalog_attempts;
    runtime.summary.catalog_timed_out =
        runtime.catalog_timed_out && runtime.catalog_handle.is_some();
    runtime.summary.cancelled_draining = runtime
        .flights
        .values()
        .filter(|flight| flight.cancelled)
        .count();
    runtime.summary.sprite_image_owners = entities
        .iter()
        .filter(|sprite| sprite.image != Handle::default())
        .count();
    runtime.summary.table_image_owners = tables.sheets.len();
    runtime.summary.table_layout_owners = tables.sheets.len();
    runtime.summary.fixture_assets = fixtures.len();
    runtime.summary.image_assets = images.len();
    runtime.summary.atlas_assets = layouts.len();
    runtime.summary.estimated_texture_bytes = estimated;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Manifest {
        myva_assets::parse_manifest(include_bytes!("../../../assets/fixture/manifest.json"))
            .unwrap()
    }

    #[test]
    fn boss_and_duel_plan_only_needed_bundles() {
        let catalog = catalog();
        let (required, optional) = scene_keys(&catalog, Mode::Boss, "long-luu").unwrap();
        assert_eq!(required.len(), 3);
        assert_eq!(optional.len(), 1);
        let (duel, _) = scene_keys(&catalog, Mode::Duel, "long-luu").unwrap();
        assert_eq!(duel.len(), 2);
        assert!(duel.iter().all(|key| key.kind != BundleKind::Boss));
    }

    #[test]
    fn selected_plan_skips_every_unrelated_character_and_reports_missing_mapping() {
        let mut catalog = catalog();
        let mut unrelated = catalog.bundles[0].clone();
        unrelated.id = "character.unused".into();
        unrelated.assets.clear();
        catalog.bundles.push(unrelated);
        let (required, _) = scene_keys(&catalog, Mode::Duel, "long-luu").unwrap();
        assert_eq!(
            required
                .iter()
                .filter(|k| k.kind == BundleKind::Character)
                .count(),
            1
        );
        assert!(required.iter().all(|k| bundle_id(k) != "character.unused"));
        assert!(matches!(
            scene_keys(&catalog, Mode::Duel, "unmapped"),
            Err(MappingError::UnknownKit(_))
        ));
        catalog.bundles.retain(|b| b.id != "character.long_luu");
        assert!(matches!(
            scene_keys(&catalog, Mode::Duel, "long-luu"),
            Err(MappingError::MissingBundle(_))
        ));
    }

    #[test]
    fn identities_change_with_content_hashes() {
        let mut catalog = catalog();
        let before = key(&catalog, &catalog.bundles[0]);
        catalog.assets[0].sha256 = "0".repeat(64);
        assert_ne!(before, key(&catalog, &catalog.bundles[0]));
    }

    #[test]
    fn projection_is_exact_and_cannot_replace_catalog_hashes() {
        let catalog = catalog();
        let manifest = myva_assets::parse_manifest(include_bytes!(
            "../../../assets/runtime-character.long_luu.myva"
        ))
        .unwrap();
        let mut fixture = ArtFixture {
            manifest,
            documents: myva_assets::Documents::default(),
            textures: BTreeMap::new(),
        };
        let key = key(&catalog, &catalog.bundles[0]);
        assert!(valid_projection(&catalog, &fixture, &key));
        fixture.manifest.assets[0].sha256 = "0".repeat(64);
        assert!(!valid_projection(&catalog, &fixture, &key));
    }
    fn headless(path: &str) -> App {
        use bevy::asset::AssetMetaCheck;
        let mut app = App::new();
        let (bout, _) = myva_sim::battle::Bout::start(Mode::Boss, 1).unwrap();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin {
                file_path: path.to_owned(),
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .init_asset::<Image>()
            .init_asset::<TextureAtlasLayout>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<myva_gameplay::SimEvent>()
            .insert_resource(crate::ArenaView::of(
                &myva_sim::battle::Battle::new(Mode::Boss, 1).unwrap(),
            ))
            .insert_resource(Match::new(bout))
            .add_plugins(art::ArtPlugin { enabled: true });
        app
    }

    fn until(app: &mut App, predicate: impl Fn(&App) -> bool) {
        let started = std::time::Instant::now();
        while !predicate(app) {
            assert!(
                started.elapsed() < std::time::Duration::from_secs(15),
                "AssetServer headless test did not settle: {}",
                app.world().resource::<StreamRuntime>().summary().to_json()
            );
            app.update();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    fn ready(app: &App) -> bool {
        let runtime = app.world().resource::<StreamRuntime>();
        app.world().resource::<Art>().status == ArtStatus::Ready
            && runtime.planner.in_flight_count() == 0
    }

    #[test]
    fn real_asset_server_loads_unloads_30_cycles_without_retained_scene_owners() {
        // This exercises real file I/O/PNG decode/AssetServer asset stores without a GPU.
        // It cannot establish render-world or driver release, RSS stability, or presented FPS.
        let mut app = headless(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        app.world_mut()
            .resource_mut::<crate::ArenaView>()
            .projectiles
            .push(myva_sim::snapshot::ProjectileView {
                id: myva_sim::projectile::ProjectileId(0),
                owner: myva_sim::FighterId(0),
                action: myva_sim::ActionKind::Skill(0),
                rect: myva_sim::kit::Rect {
                    x0: 100_000,
                    x1: 120_000,
                    y0: 20_000,
                    y1: 40_000,
                },
                dir: 1,
            });
        for cycle in 0..30 {
            app.world_mut().resource_mut::<Art>().enabled = true;
            until(&mut app, ready);
            app.world_mut().write_message(myva_gameplay::SimEvent(
                myva_sim::protocol::EventRecord {
                    id: myva_sim::protocol::EventId {
                        tick: 0,
                        index: cycle,
                    },
                    event: myva_sim::Event::Hit {
                        attacker: myva_sim::FighterId(1),
                        target: myva_sim::FighterId(0),
                        action: myva_sim::ActionKind::Light(0),
                        damage: 1,
                    },
                },
            ));
            app.update();
            let mut bodies = app
                .world_mut()
                .query_filtered::<&Sprite, With<art::ArtBody>>();
            assert_eq!(bodies.iter(app.world()).count(), 2);
            assert!(
                bodies
                    .iter(app.world())
                    .all(|sprite| sprite.image != Handle::default())
            );
            let vfx_image = app.world().resource::<ArtTables>().sheets["vfx.combat"]
                .image
                .id();
            let mut owners = app.world_mut().query_filtered::<&Sprite, With<ArtEntity>>();
            assert!(
                owners
                    .iter(app.world())
                    .filter(|sprite| sprite.image.id() == vfx_image)
                    .count()
                    >= 2,
                "actual projectile and hit-effect sprite owners must exist before exit"
            );
            assert_eq!(app.world().resource::<ArtTables>().sheets.len(), 5);
            assert_eq!(
                app.world()
                    .resource::<StreamRuntime>()
                    .summary()
                    .planner_owners,
                4
            );
            app.world_mut().resource_mut::<Art>().enabled = false;
            until(&mut app, |app| {
                let runtime = app.world().resource::<StreamRuntime>();
                runtime.planner.scene_state() == SceneState::Idle
                    && app.world().resource::<Assets<ArtFixture>>().is_empty()
                    && app.world().resource::<Assets<Image>>().is_empty()
                    && app
                        .world()
                        .resource::<Assets<TextureAtlasLayout>>()
                        .is_empty()
            });
            assert!(app.world().resource::<ArtTables>().sheets.is_empty());
            assert_eq!(
                app.world()
                    .resource::<StreamRuntime>()
                    .summary()
                    .planner_owners,
                0
            );
            let mut query = app.world_mut().query_filtered::<Entity, With<ArtEntity>>();
            assert_eq!(query.iter(app.world()).count(), 0);
        }
        let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/xtask-evidence/headless-asset-cycles.json");
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        let summary = app.world().resource::<StreamRuntime>().summary();
        assert_eq!(summary.completed_loads, 120);
        let artifact = serde_json::json!({
            "schema_version": 1,
            "evidence_kind": "headless_asset_lifecycle",
            "cycles": 30,
            "captured_at_unix_ms": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            "method": "Real native AssetServer filesystem reads, SHA-256 validation, PNG decode and loaded-with-dependencies polling via Time<Real>; 5ms test polling granularity; no RenderPlugin, window, upload or presentation",
            "scope": "CPU asset stores and scene ownership only; not browser/network/GPU/driver-memory or presented-FPS evidence",
            "presentation_owners_exercised": ["map", "two fighters", "projectile", "hit VFX"],
            "runtime": serde_json::from_str::<serde_json::Value>(&summary.to_json()).unwrap(),
            "rendering": "NOT_RUN: headless CPU asset integration test",
        });
        std::fs::write(output, serde_json::to_vec_pretty(&artifact).unwrap()).unwrap();
    }

    #[test]
    fn real_asset_server_scene_switch_drops_boss_and_retains_immutable_common_bundles() {
        let mut app = headless(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        until(&mut app, ready);
        let character = app.world().resource::<ArtTables>().sheets["long_luu.sheet"]
            .image
            .id();
        let (bout, _) = myva_sim::battle::Bout::start(Mode::Duel, 2).unwrap();
        *app.world_mut().resource_mut::<Match>() = Match::new(bout);
        app.update();
        until(&mut app, |app| {
            ready(app) && app.world().resource::<Assets<ArtFixture>>().len() == 3
        });
        let tables = app.world().resource::<ArtTables>();
        assert!(!tables.sheets.contains_key("ke_giu_dap.sheet"));
        assert_eq!(tables.sheets["long_luu.sheet"].image.id(), character);
        assert_eq!(
            app.world()
                .resource::<StreamRuntime>()
                .summary()
                .planner_owners,
            3
        );
    }

    #[test]
    fn installing_late_optional_bundle_preserves_existing_scene_owners() {
        let mut app = headless(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        until(&mut app, ready);
        // Model the exact installation boundary after a late optional completion. Required
        // owners/tables already exist; the validated VFX load unit is newly Ready in the planner.
        app.world_mut()
            .resource_mut::<StreamRuntime>()
            .installed
            .retain(|key| key.kind != BundleKind::Vfx);
        app.world_mut()
            .resource_mut::<ArtTables>()
            .sheets
            .remove("vfx.combat");
        let sheet = app.world().resource::<ArtTables>().sheets["long_luu.sheet"].clone();
        let layout = sheet.layout.id();
        let body = app
            .world_mut()
            .spawn((
                ArtEntity,
                art::ArtBody,
                Sprite::from_atlas_image(
                    sheet.image.clone(),
                    bevy::image::TextureAtlas {
                        layout: sheet.layout,
                        index: 0,
                    },
                ),
                Visibility::Inherited,
            ))
            .id();
        let mut query = app.world_mut().query_filtered::<Entity, With<ArtEntity>>();
        let owners: Vec<_> = query.iter(app.world()).collect();
        app.update();
        let tables = app.world().resource::<ArtTables>();
        assert!(tables.sheets.contains_key("vfx.combat"));
        assert_eq!(tables.sheets["long_luu.sheet"].layout.id(), layout);
        assert_eq!(app.world().get::<Sprite>(body).unwrap().image, sheet.image);
        assert_eq!(
            app.world().get::<Visibility>(body),
            Some(&Visibility::Inherited)
        );
        let mut query = app.world_mut().query_filtered::<Entity, With<ArtEntity>>();
        assert_eq!(query.iter(app.world()).collect::<Vec<_>>(), owners);
    }
    struct TempAssets(std::path::PathBuf);
    impl Drop for TempAssets {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &to.join(entry.file_name()));
            } else {
                std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
            }
        }
    }
    fn fault_assets() -> TempAssets {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let next = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("myva-runtime-{}-{next}", std::process::id()));
        assert!(!path.exists());
        copy_tree(
            std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets")),
            &path,
        );
        TempAssets(path)
    }

    #[test]
    fn real_asset_server_never_requests_unselected_character_payloads() {
        let assets = fault_assets();
        let mut manifest = catalog();
        let mut unused = manifest.assets[0].clone();
        unused.id = "unused.atlas".into();
        unused.path = "unused/atlas.png".into();
        let mut bundle = manifest.bundles[0].clone();
        bundle.id = "character.unused".into();
        bundle.assets = vec![unused.id.clone()];
        manifest.assets.push(unused);
        manifest.bundles.push(bundle);
        let mut issues = Vec::new();
        myva_assets::validate::check_manifest(&manifest, &mut issues);
        assert!(issues.is_empty(), "{issues:?}");
        std::fs::write(
            assets.0.join("fixture/manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        // No unused texture or runtime-character.unused.myva exists. A whole-Character scan
        // would try loading it and block required art. Selected load units settle normally.
        let mut app = headless(assets.0.to_str().unwrap());
        until(&mut app, ready);
        let runtime = app.world().resource::<StreamRuntime>();
        assert_eq!(runtime.summary().completed_loads, 4);
        assert_eq!(runtime.summary().failed_loads, 0);
        assert_eq!(runtime.summary().selected_kit, "long-luu");
        assert!(
            runtime
                .keys
                .iter()
                .all(|k| bundle_id(k) != "character.unused")
        );
        assert!(
            !app.world()
                .resource::<ArtTables>()
                .sheets
                .contains_key("unused.sheet")
        );
    }

    #[test]
    fn mapping_rejection_cancels_pending_selected_bundle_and_never_installs_stale_handles() {
        let mut app = headless(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        until(&mut app, |app| {
            app.world()
                .resource::<StreamRuntime>()
                .planner
                .in_flight_count()
                > 0
        });
        {
            let mut runtime = app.world_mut().resource_mut::<StreamRuntime>();
            let pin = myva_sim::registry::KitPin {
                id: myva_sim::catalog::KitId::new("test-no-mapping").unwrap(),
                revision: myva_sim::registry::IMPLEMENTED_KITS[0].revision,
            };
            assert!(matches!(
                runtime.enter(Mode::Boss, pin, 0),
                Err(MappingError::UnknownKit(_))
            ));
        }
        app.world_mut().resource_mut::<Art>().enabled = false;
        app.update();
        until(&mut app, |app| {
            app.world()
                .resource::<StreamRuntime>()
                .planner
                .in_flight_count()
                == 0
                && app.world().resource::<Assets<ArtFixture>>().is_empty()
        });
        assert!(app.world().resource::<ArtTables>().sheets.is_empty());
        assert_eq!(
            app.world()
                .resource::<StreamRuntime>()
                .summary()
                .planner_owners,
            0
        );
        assert!(
            app.world()
                .resource::<StreamRuntime>()
                .summary()
                .mapping_error
                .is_some()
        );
    }

    #[test]
    fn missing_selected_mapping_is_failed_once_without_silent_identity_fallback() {
        let assets = fault_assets();
        let mut manifest = catalog();
        manifest.bundles.retain(|b| b.id != "character.long_luu");
        manifest.assets.retain(|a| !a.id.starts_with("long_luu."));
        std::fs::write(
            assets.0.join("fixture/manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let mut app = headless(assets.0.to_str().unwrap());
        until(&mut app, |app| {
            matches!(
                app.world().resource::<Art>().status,
                ArtStatus::Failed { .. }
            )
        });
        let before = app.world().resource::<StreamRuntime>().summary().generation;
        for _ in 0..10 {
            app.update();
        }
        let runtime = app.world().resource::<StreamRuntime>();
        assert_eq!(runtime.summary().generation, before);
        assert_eq!(runtime.summary().completed_loads, 0);
        assert_eq!(runtime.summary().flights, 0);
        assert!(
            runtime
                .summary()
                .mapping_error
                .as_ref()
                .unwrap()
                .contains("MissingBundle")
        );
        assert!(app.world().resource::<ArtTables>().sheets.is_empty());
    }

    #[test]
    fn real_asset_server_required_missing_is_bounded_and_explicit_retry_recovers() {
        let assets = fault_assets();
        let missing = assets.0.join("fixture/long_luu/atlas.png");
        std::fs::remove_file(&missing).unwrap();
        let mut app = headless(assets.0.to_str().unwrap());
        until(&mut app, |app| {
            matches!(
                app.world().resource::<Art>().status,
                ArtStatus::Failed { .. }
            )
        });
        let runtime = app.world().resource::<StreamRuntime>();
        let key = runtime
            .keys
            .iter()
            .find(|k| k.kind == BundleKind::Character)
            .unwrap();
        assert!(matches!(
            runtime.planner.bundle_state(key),
            Some(BundleState::RequiredBlocked(_))
        ));
        assert_eq!(runtime.summary().retries, 2);
        assert!(app.world().resource::<ArtTables>().sheets.is_empty());
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../assets/fixture/long_luu/atlas.png"
            ),
            missing,
        )
        .unwrap();
        app.world_mut().write_message(ArtCommand::Retry);
        app.update();
        until(&mut app, ready);
        assert_eq!(app.world().resource::<ArtTables>().sheets.len(), 5);
    }

    #[test]
    fn real_asset_server_optional_missing_keeps_required_art_ready_without_vfx_owners() {
        let assets = fault_assets();
        std::fs::remove_file(assets.0.join("fixture/vfx/atlas.png")).unwrap();
        let mut app = headless(assets.0.to_str().unwrap());
        until(&mut app, |app| {
            let runtime = app.world().resource::<StreamRuntime>();
            ready(app)
                && runtime
                    .keys
                    .iter()
                    .filter(|key| key.kind == BundleKind::Vfx)
                    .all(|key| {
                        matches!(
                            runtime.planner.bundle_state(key),
                            Some(BundleState::OptionalFallback(_))
                        )
                    })
        });
        let runtime = app.world().resource::<StreamRuntime>();
        let key = runtime
            .keys
            .iter()
            .find(|k| k.kind == BundleKind::Vfx)
            .unwrap();
        assert!(matches!(
            runtime.planner.bundle_state(key),
            Some(BundleState::OptionalFallback(_))
        ));
        assert_eq!(runtime.summary().retries, 2);
        assert_eq!(app.world().resource::<ArtTables>().sheets.len(), 4);
        assert!(
            !app.world()
                .resource::<ArtTables>()
                .sheets
                .contains_key("vfx.combat")
        );
    }

    #[test]
    fn real_asset_server_rapid_exit_drops_stale_completion_before_next_scene() {
        let mut app = headless(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        until(&mut app, |app| {
            app.world()
                .resource::<StreamRuntime>()
                .planner
                .in_flight_count()
                > 0
        });
        app.world_mut().resource_mut::<Art>().enabled = false;
        app.update();
        until(&mut app, |app| {
            let runtime = app.world().resource::<StreamRuntime>();
            runtime.planner.in_flight_count() == 0
                && app.world().resource::<Assets<ArtFixture>>().is_empty()
        });
        assert_eq!(
            app.world()
                .resource::<StreamRuntime>()
                .summary()
                .planner_owners,
            0
        );
        assert!(app.world().resource::<ArtTables>().sheets.is_empty());
        app.world_mut().resource_mut::<Art>().enabled = true;
        until(&mut app, ready);
        assert_eq!(app.world().resource::<ArtTables>().sheets.len(), 5);
    }
}
