//! D05-B bounded render workloads. These sprites never acquire gameplay components or enter
//! the session/replay. The existing rival is recorded separately from the published count
//! basis (one player plus render-only actors/monsters). Raw `Time<Real>` intervals describe
//! Bevy Update scheduling, never presented FPS, GPU timing, or physical GPU memory.

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::image::TextureAtlas;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::sprite::Anchor;
use bevy::window::{PrimaryWindow, WindowRef};

use crate::art::{Art, ArtBody, ArtEntity, ArtTables, SheetView};

pub const CONFIG_ID: &str = "myva-d05b-bounded-scene-v1";
const WARMUP_SECONDS: f64 = 5.0;
// 600 seconds at 200 Hz. A longer/faster capture explicitly reports truncation.
const MAX_INTERVALS: usize = 120_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualityTier {
    High,
    Low,
}

impl QualityTier {
    pub const fn name(self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Low => "Low",
        }
    }

    pub const fn particles(self) -> usize {
        match self {
            Self::High => 100,
            Self::Low => 50,
        }
    }

    pub const fn parallax_layers(self) -> usize {
        match self {
            Self::High => 4,
            Self::Low => 2,
        }
    }

    pub const fn render_scale(self) -> f32 {
        match self {
            Self::High => 1.0,
            Self::Low => 0.75,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BenchmarkConfig {
    pub population: u16,
    pub tier: QualityTier,
    pub seed: u32,
}

impl BenchmarkConfig {
    /// Only the checked-in 50/200/500 presets are supported. No arbitrary workload is silently
    /// substituted for the requested one.
    pub fn new(population: u16, tier: QualityTier, seed: u32) -> Option<Self> {
        matches!(population, 50 | 200 | 500).then_some(Self {
            population,
            tier,
            seed,
        })
    }

    pub const fn preset(self) -> &'static str {
        match self.population {
            50 => "stress_50",
            200 => "stress_200",
            _ => "stress_500",
        }
    }

    pub const fn actors(self) -> usize {
        match self.population {
            50 => 19,
            200 => 79,
            _ => 199,
        }
    }

    pub const fn monsters(self) -> usize {
        match self.population {
            50 => 30,
            200 => 120,
            _ => 300,
        }
    }
}

/// None disables the render-only workload, leaving the normal match unchanged.
#[derive(Message)]
pub struct ConfigureBenchmark(pub Option<BenchmarkConfig>);

#[derive(Resource, Default)]
pub struct Benchmark {
    pub config: Option<BenchmarkConfig>,
    ready_since: Option<f64>,
    previous_sample_at: Option<f64>,
    intervals_ms: Vec<f64>,
    sample_window_ms: f64,
    truncated: bool,
    generation: u64,
    applied: Option<BenchmarkConfig>,
    actors: usize,
    monsters: usize,
    particles: usize,
    gameplay_fighters: usize,
    // Set by the scaling system only after the real offscreen target is attached.
    pub applied_render_scale: f32,
    pub render_width: u32,
    pub render_height: u32,
}

impl Benchmark {
    pub fn is_ready(&self) -> bool {
        self.config.is_some_and(|config| {
            self.applied == Some(config)
                && self.actors == config.actors()
                && self.monsters == config.monsters()
                && self.particles == config.tier.particles()
                && self.gameplay_fighters == 2
        })
    }

    /// Known extra color target only; internal Bevy/MSAA/depth/swapchain buffers are excluded.
    pub fn offscreen_color_bytes(&self) -> u64 {
        if self.applied_render_scale > 0.0 && self.applied_render_scale < 1.0 {
            u64::from(self.render_width) * u64::from(self.render_height) * 4
        } else {
            0
        }
    }

    pub fn intervals_truncated(&self) -> bool {
        self.truncated
    }

    pub fn sampled_ms(&self) -> f64 {
        self.sample_window_ms
    }

    pub fn parallax_layers(&self) -> Option<usize> {
        self.config.map(|config| config.tier.parallax_layers())
    }

    fn reset_samples(&mut self) {
        self.ready_since = None;
        self.previous_sample_at = None;
        self.intervals_ms.clear();
        self.sample_window_ms = 0.0;
        self.truncated = false;
    }

    /// Separate diagnostic snapshot, avoiding any change to gameplay telemetry schema v2.
    /// Scene counts are actual ECS sprite counts after deferred spawn/despawn is applied.
    pub fn summary_json(&self) -> String {
        self.json(false)
    }

    pub fn to_json(&self) -> String {
        self.json(true)
    }

    fn json(&self, include_samples: bool) -> String {
        let Some(config) = self.config else {
            return "{\"active\":false}".to_owned();
        };
        let intervals = if include_samples {
            self.intervals_ms
                .iter()
                .map(|value| format!("{value:.6}"))
                .collect::<Vec<_>>()
                .join(",")
        } else {
            String::new()
        };
        format!(
            "{{\"active\":true,\"config_id\":\"{CONFIG_ID}\",\"preset\":\"{}\",\"tier\":\"{}\",\"seed\":{},\"gameplay_players\":1,\"render_only_actors\":{},\"render_only_monsters\":{},\"cosmetic_particles\":{},\"parallax_layers\":{},\"render_scale\":{},\"applied_render_scale\":{},\"render_width_px\":{},\"render_height_px\":{},\"offscreen_rgba8_color_bytes\":{},\"hud\":true,\"boss_telegraphs\":\"unchanged\",\"combat_feedback\":\"unchanged\",\"actual_actors\":{},\"actual_monsters\":{},\"actual_particles\":{},\"additional_gameplay_rivals\":{},\"actual_body_sprites\":{},\"generation\":{},\"ready\":{},\"frame_source\":\"bevy_update\",\"warmup_seconds\":{WARMUP_SECONDS},\"interval_samples_included\":{include_samples},\"frame_sample_count\":{},\"window_ms\":{:.6},\"intervals_truncated\":{},\"frame_intervals_ms\":[{intervals}]}}",
            config.preset(),
            config.tier.name(),
            config.seed,
            config.actors(),
            config.monsters(),
            config.tier.particles(),
            config.tier.parallax_layers(),
            config.tier.render_scale(),
            self.applied_render_scale,
            self.render_width,
            self.render_height,
            self.offscreen_color_bytes(),
            self.actors,
            self.monsters,
            self.particles,
            self.gameplay_fighters.saturating_sub(1),
            self.actors + self.monsters + self.gameplay_fighters,
            self.generation,
            self.is_ready(),
            self.intervals_ms.len(),
            self.sample_window_ms,
            self.truncated,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Actor,
    Monster,
    Particle,
}

/// Only renderer components accompany this marker, never a gameplay/simulation mirror.
#[derive(Component)]
pub struct BenchmarkEntity {
    kind: Kind,
    index: usize,
    origin: Vec2,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CaptureSet;

pub fn plugin(app: &mut App) {
    app.init_resource::<Benchmark>()
        .init_resource::<RenderScale>()
        .add_message::<ConfigureBenchmark>()
        .add_systems(
            Update,
            (configure, apply_render_scale, maintain_scene, animate).chain(),
        )
        .add_systems(Last, sample.in_set(CaptureSet));
}

/// The low-tier world is genuinely rasterized into a smaller RGBA8 target. A separate
/// full-resolution camera presents that image and draws the existing HUD/touch UI. Changing
/// tier never changes combat camera framing, simulation, telegraph geometry or UI layout.
#[derive(Component)]
struct Presenter;

#[derive(Resource, Default)]
struct RenderScale {
    image: Option<Handle<Image>>,
    size: UVec2,
    logical_size: Vec2,
}

#[allow(clippy::too_many_arguments)]
fn apply_render_scale(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    art: Res<Art>,
    mut camera: Single<
        (&mut Camera, &mut RenderTarget, &mut Projection),
        With<crate::scene::ArenaCamera>,
    >,
    mut images: ResMut<Assets<Image>>,
    presenters: Query<Entity, With<Presenter>>,
    mut scale: ResMut<RenderScale>,
    mut benchmark: ResMut<Benchmark>,
) {
    let factor = if art.visible() {
        benchmark
            .config
            .map_or(1.0, |config| config.tier.render_scale())
    } else {
        1.0
    };
    let physical = window.physical_size();
    let size = UVec2::new(
        ((physical.x as f32 * factor).round() as u32).max(1),
        ((physical.y as f32 * factor).round() as u32).max(1),
    );
    let logical_size_changed = scale.logical_size != window.size();
    if benchmark.applied_render_scale != factor
        || benchmark.render_width != size.x
        || benchmark.render_height != size.y
        || logical_size_changed
    {
        // A contiguous acquisition describes one raster workload. Resizing or changing DPR
        // starts a new warmup/sample epoch instead of relabeling earlier frame intervals.
        benchmark.reset_samples();
    }
    scale.logical_size = window.size();
    benchmark.applied_render_scale = factor;
    benchmark.render_width = size.x;
    benchmark.render_height = size.y;
    if factor == 1.0 {
        if scale.image.take().is_some() {
            *camera.1 = RenderTarget::Window(WindowRef::Primary);
            camera.0.order = 0;
            // Bevy 0.20 camera_system does not invalidate computed target info merely because
            // RenderTarget changed. A projection change recomputes the real window viewport.
            camera.2.set_changed();
            for entity in &presenters {
                commands.entity(entity).despawn();
            }
            // Strong handles in camera/render world can outlive this frame. This operation
            // deliberately makes no claim about physical GPU memory reclamation.
        }
        return;
    }
    if scale.image.is_some() && scale.size == size && !logical_size_changed {
        return;
    }
    for entity in &presenters {
        commands.entity(entity).despawn();
    }
    let handle = match scale.image.clone() {
        Some(handle) => {
            if let Some(mut image) = images.get_mut(&handle) {
                image.resize(Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                });
            }
            handle
        }
        None => images.add(Image::new_target_texture(
            size.x,
            size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        )),
    };
    scale.size = size;
    scale.logical_size = window.size();
    scale.image = Some(handle.clone());
    *camera.1 = RenderTarget::Image(handle.clone().into());
    camera.0.order = -1;
    camera.2.set_changed();
    commands.spawn((
        Presenter,
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        IsDefaultUiCamera,
        RenderLayers::layer(31),
        bevy::core_pipeline::tonemapping::Tonemapping::None,
    ));
    let mut sprite = Sprite::from_image(handle);
    sprite.custom_size = Some(window.size());
    commands.spawn((Presenter, sprite, RenderLayers::layer(31)));
}

fn configure(mut requests: MessageReader<ConfigureBenchmark>, mut benchmark: ResMut<Benchmark>) {
    // Last command wins during rapid changes; earlier requests never spawn a stale scene.
    if let Some(request) = requests.read().last() {
        benchmark.config = request.0;
        benchmark.applied = None;
        benchmark.reset_samples();
    }
}

fn random(seed: u32, index: usize) -> u32 {
    let mut value = seed ^ (index as u32).wrapping_mul(0x9e37_79b9);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}

fn position(seed: u32, index: usize, population: usize, particle: bool) -> Vec2 {
    let columns = (population as f32).sqrt().ceil().max(1.0) as usize;
    let rows = population.div_ceil(columns).max(1);
    let jitter = random(seed, index) as f32 / u32::MAX as f32;
    Vec2::new(
        45.0 + (index % columns) as f32 * 1450.0 / columns as f32 + jitter * 12.0,
        if particle { 20.0 } else { 75.0 } + (index / columns) as f32 * 260.0 / rows as f32,
    )
}

fn sheet_for(tables: &ArtTables, kind: Kind) -> Option<&SheetView> {
    tables.sheets.get(match kind {
        Kind::Actor => "long_luu.sheet",
        Kind::Monster => "ke_giu_dap.sheet",
        Kind::Particle => "vfx.combat",
    })
}

fn clip_for(sheet: &SheetView, kind: Kind) -> Option<&crate::art::ClipView> {
    sheet.clips.get(if kind == Kind::Particle {
        "dust"
    } else {
        "idle"
    })
}

fn spawn_kind(
    commands: &mut Commands,
    tables: &ArtTables,
    config: BenchmarkConfig,
    kind: Kind,
    count: usize,
) {
    let Some(sheet) = sheet_for(tables, kind) else {
        return;
    };
    let Some(clip) = clip_for(sheet, kind).filter(|clip| !clip.frames.is_empty()) else {
        return;
    };
    for index in 0..count {
        let frame = clip.frames[random(config.seed, index) as usize % clip.frames.len()];
        let mut sprite = Sprite::from_atlas_image(
            sheet.image.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: frame.index,
            },
        );
        sprite.custom_size = Some(frame.size);
        sprite.color = Color::WHITE.with_alpha(if kind == Kind::Particle { 0.3 } else { 0.55 });
        let origin = position(config.seed, index, count, kind == Kind::Particle);
        let scale = match kind {
            Kind::Actor => 0.28,
            Kind::Monster => 0.16,
            Kind::Particle => 0.5,
        };
        commands.spawn((
            ArtEntity,
            BenchmarkEntity {
                kind,
                index,
                origin,
            },
            sprite,
            Anchor(frame.anchor),
            // All benchmark cosmetics stay behind gameplay bodies/telegraphs, so tier/stress
            // cannot hide the player, danger regions, health bars or combat feedback.
            Transform::from_xyz(origin.x, origin.y, 0.4).with_scale(Vec3::splat(scale)),
        ));
    }
}

fn maintain_scene(
    mut commands: Commands,
    art: Res<Art>,
    tables: Res<ArtTables>,
    entities: Query<Entity, With<BenchmarkEntity>>,
    mut benchmark: ResMut<Benchmark>,
) {
    let desired = benchmark.config.filter(|_| art.visible());
    let expected = desired.map_or(0, |config| {
        config.actors() + config.monsters() + config.tier.particles()
    });
    if benchmark.applied == desired && entities.iter().count() == expected {
        return;
    }
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    benchmark.applied = None;
    benchmark.reset_samples();
    if let Some(config) = desired {
        if [Kind::Actor, Kind::Monster, Kind::Particle]
            .iter()
            .any(|kind| {
                sheet_for(&tables, *kind)
                    .and_then(|sheet| clip_for(sheet, *kind))
                    .is_none_or(|clip| clip.frames.is_empty())
            })
        {
            return;
        }
        spawn_kind(&mut commands, &tables, config, Kind::Actor, config.actors());
        spawn_kind(
            &mut commands,
            &tables,
            config,
            Kind::Monster,
            config.monsters(),
        );
        spawn_kind(
            &mut commands,
            &tables,
            config,
            Kind::Particle,
            config.tier.particles(),
        );
        benchmark.applied = Some(config);
        benchmark.generation = benchmark.generation.saturating_add(1);
    }
}

fn animate(
    time: Res<Time<Real>>,
    tables: Res<ArtTables>,
    benchmark: Res<Benchmark>,
    mut entities: Query<(&BenchmarkEntity, &mut Sprite, &mut Anchor, &mut Transform)>,
) {
    let Some(config) = benchmark.applied else {
        return;
    };
    let elapsed = time.elapsed_secs();
    for (entity, mut sprite, mut anchor, mut transform) in &mut entities {
        let Some(clip) = sheet_for(&tables, entity.kind).and_then(|s| clip_for(s, entity.kind))
        else {
            continue;
        };
        let offset = random(config.seed, entity.index) as f32 / u32::MAX as f32;
        let frame = clip.frames
            [((elapsed / clip.frame_secs.max(0.001) + offset * 10.0) as usize) % clip.frames.len()];
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = frame.index;
        }
        sprite.custom_size = Some(frame.size);
        anchor.0 = frame.anchor;
        transform.translation.x = entity.origin.x + (elapsed * 0.35 + offset * 6.0).sin() * 8.0;
        if entity.kind == Kind::Particle {
            transform.translation.y = entity.origin.y + (elapsed + offset * 6.0).sin() * 10.0;
        }
    }
}

fn sample(
    time: Res<Time<Real>>,
    gameplay_bodies: Query<&Sprite, With<ArtBody>>,
    entities: Query<&BenchmarkEntity, With<Sprite>>,
    mut benchmark: ResMut<Benchmark>,
) {
    benchmark.actors = 0;
    benchmark.monsters = 0;
    benchmark.particles = 0;
    for entity in &entities {
        match entity.kind {
            Kind::Actor => benchmark.actors += 1,
            Kind::Monster => benchmark.monsters += 1,
            Kind::Particle => benchmark.particles += 1,
        }
    }
    benchmark.gameplay_fighters = gameplay_bodies
        .iter()
        .filter(|sprite| sprite.texture_atlas.is_some() && sprite.image != Handle::default())
        .count();
    if !benchmark.is_ready() {
        benchmark.previous_sample_at = None;
        return;
    }
    let now = time.elapsed_secs_f64();
    let since = *benchmark.ready_since.get_or_insert(now);
    if now - since < WARMUP_SECONDS {
        return;
    }
    if let Some(previous) = benchmark.previous_sample_at {
        let interval = (now - previous) * 1000.0;
        if interval.is_finite() && interval > 0.0 {
            if benchmark.intervals_ms.len() < MAX_INTERVALS {
                benchmark.intervals_ms.push(interval);
                benchmark.sample_window_ms += interval;
            } else {
                benchmark.truncated = true;
            }
        }
    }
    benchmark.previous_sample_at = Some(now);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_match_checked_in_count_basis() {
        for (population, actors, monsters) in [(50, 19, 30), (200, 79, 120), (500, 199, 300)] {
            let config = BenchmarkConfig::new(population, QualityTier::High, 7).unwrap();
            assert_eq!((config.actors(), config.monsters()), (actors, monsters));
            assert_eq!(1 + actors + monsters, population as usize);
        }
        assert!(BenchmarkConfig::new(100, QualityTier::High, 7).is_none());
    }

    #[test]
    fn low_reduces_cosmetics_only() {
        let high = BenchmarkConfig::new(500, QualityTier::High, 7).unwrap();
        let low = BenchmarkConfig::new(500, QualityTier::Low, 7).unwrap();
        assert_eq!(
            (high.actors(), high.monsters()),
            (low.actors(), low.monsters())
        );
        assert_eq!((low.tier.particles(), low.tier.parallax_layers()), (50, 2));
        assert_eq!(low.tier.render_scale(), 0.75);
    }

    #[test]
    fn seeded_positions_are_reproducible_and_visible() {
        for population in [19, 79, 199, 30, 120, 300] {
            for index in 0..population {
                let first = position(7, index, population, false);
                assert_eq!(first, position(7, index, population, false));
                assert!((0.0..1600.0).contains(&first.x));
                assert!((0.0..400.0).contains(&first.y));
            }
        }
        assert_ne!(position(7, 0, 19, false), position(8, 0, 19, false));
    }

    #[test]
    fn raw_update_intervals_never_claim_presented_frames() {
        let benchmark = Benchmark {
            config: BenchmarkConfig::new(50, QualityTier::Low, 7),
            intervals_ms: vec![16.0, 18.0],
            sample_window_ms: 34.0,
            ..default()
        };
        let json = benchmark.to_json();
        assert!(json.contains("\"frame_source\":\"bevy_update\""));
        assert!(json.contains("\"frame_intervals_ms\":[16.000000,18.000000]"));
        assert!(!json.contains("presented_fps"));
        assert!(!json.contains("gpu_time"));
        assert!(!json.contains("physical_gpu"));
    }

    fn tables() -> ArtTables {
        let mut tables = ArtTables::default();
        let frame = crate::art::FrameView {
            index: 0,
            size: Vec2::splat(12.0),
            anchor: Vec2::ZERO,
        };
        for (id, clip_name) in [
            ("long_luu.sheet", "idle"),
            ("ke_giu_dap.sheet", "idle"),
            ("vfx.combat", "dust"),
        ] {
            tables.sheets.insert(
                id.to_owned(),
                SheetView {
                    image: Handle::default(),
                    layout: Handle::default(),
                    frames: std::collections::BTreeMap::new(),
                    clips: std::collections::BTreeMap::from([(
                        clip_name.to_owned(),
                        crate::art::ClipView {
                            frames: vec![frame],
                            frame_secs: 0.1,
                            looping: true,
                            hit: None,
                            apex: None,
                        },
                    )]),
                },
            );
        }
        tables
    }

    fn install_test_scene(app: &mut App) {
        app.init_resource::<Assets<Image>>()
            .init_resource::<Time<Real>>()
            .insert_resource(tables())
            .insert_resource(Art {
                status: crate::art::ArtStatus::Ready,
                enabled: true,
                content_version: None,
            });
        plugin(app);
        app.world_mut().spawn((
            PrimaryWindow,
            Window {
                resolution: (1280, 720).into(),
                ..default()
            },
        ));
        app.world_mut().spawn((crate::scene::ArenaCamera, Camera2d));
    }

    #[test]
    fn all_presets_create_exact_renderer_counts_without_gameplay_components() {
        for population in [50, 200, 500] {
            for tier in [QualityTier::High, QualityTier::Low] {
                let mut app = App::new();
                install_test_scene(&mut app);
                let config = BenchmarkConfig::new(population, tier, 7).unwrap();
                app.world_mut()
                    .write_message(ConfigureBenchmark(Some(config)));
                app.update();
                let world = app.world_mut();
                let mut counts = [0; 3];
                for (entity, sprite, transform) in world
                    .query::<(&BenchmarkEntity, &Sprite, &Transform)>()
                    .iter(world)
                {
                    let index = match entity.kind {
                        Kind::Actor => 0,
                        Kind::Monster => 1,
                        Kind::Particle => 2,
                    };
                    counts[index] += 1;
                    assert!(sprite.texture_atlas.is_some());
                    assert!(
                        transform.translation.z < 1.0,
                        "cosmetics must remain behind telegraphs"
                    );
                }
                assert_eq!(
                    counts,
                    [config.actors(), config.monsters(), tier.particles()]
                );
                assert_eq!(
                    world
                        .query_filtered::<Entity, With<myva_gameplay::components::SimId>>()
                        .iter(world)
                        .count(),
                    0
                );
                assert_eq!(
                    world
                        .query_filtered::<Entity, With<ArtEntity>>()
                        .iter(world)
                        .count(),
                    counts.iter().sum::<usize>()
                );
            }
        }
    }

    #[test]
    fn low_uses_real_smaller_target_and_restores_window_without_orphan_presenters() {
        let mut app = App::new();
        install_test_scene(&mut app);
        let low = BenchmarkConfig::new(50, QualityTier::Low, 7).unwrap();
        app.world_mut().write_message(ConfigureBenchmark(Some(low)));
        app.update();
        let world = app.world_mut();
        let target = world
            .query_filtered::<&RenderTarget, With<crate::scene::ArenaCamera>>()
            .single(world)
            .unwrap();
        let RenderTarget::Image(target) = target else {
            panic!("Low must rasterize to an actual image target")
        };
        let image = world
            .resource::<Assets<Image>>()
            .get(&target.handle)
            .unwrap();
        assert_eq!((image.width(), image.height()), (960, 540));
        assert_eq!(
            world
                .query_filtered::<Entity, With<IsDefaultUiCamera>>()
                .iter(world)
                .count(),
            1
        );
        assert_eq!(
            world
                .query_filtered::<Entity, With<Presenter>>()
                .iter(world)
                .count(),
            2
        );
        assert!(matches!(
            world
                .query_filtered::<&RenderTarget, With<IsDefaultUiCamera>>()
                .single(world)
                .unwrap(),
            RenderTarget::Window(WindowRef::Primary)
        ));
        world.resource_mut::<Art>().enabled = false;
        app.update();
        let world = app.world_mut();
        assert!(world.resource::<RenderScale>().image.is_none());
        assert_eq!(
            world
                .query_filtered::<Entity, With<Presenter>>()
                .iter(world)
                .count(),
            0
        );
        assert_eq!(world.resource::<Benchmark>().offscreen_color_bytes(), 0);
        world.resource_mut::<Art>().enabled = true;
        app.update();
        let world = app.world_mut();
        assert!(world.resource::<RenderScale>().image.is_some());
        assert_eq!(
            world
                .query_filtered::<Entity, With<Presenter>>()
                .iter(world)
                .count(),
            2
        );
        world.write_message(ConfigureBenchmark(None));
        app.update();
        let world = app.world_mut();
        assert!(matches!(
            world
                .query_filtered::<&RenderTarget, With<crate::scene::ArenaCamera>>()
                .single(world)
                .unwrap(),
            RenderTarget::Window(WindowRef::Primary)
        ));
        assert_eq!(
            world
                .query_filtered::<Entity, With<Presenter>>()
                .iter(world)
                .count(),
            0
        );
        assert_eq!(
            world
                .query_filtered::<Entity, With<BenchmarkEntity>>()
                .iter(world)
                .count(),
            0
        );
    }

    #[test]
    fn tier_transitions_recompute_the_real_camera_target_size_without_resize_events() {
        let mut app = App::new();
        install_test_scene(&mut app);
        // Run Bevy's actual CPU camera computation without a renderer/window/GPU plugin.
        app.init_resource::<bevy::render::texture::ManualTextureViews>()
            .add_message::<bevy::window::WindowCreated>()
            .add_message::<bevy::window::WindowResized>()
            .add_message::<bevy::window::WindowScaleFactorChanged>()
            .add_message::<bevy::asset::AssetEvent<Image>>()
            .add_systems(PostUpdate, bevy::render::camera::camera_system);
        let target_size = |app: &mut App| {
            let world = app.world_mut();
            world
                .query_filtered::<&Camera, With<crate::scene::ArenaCamera>>()
                .single(world)
                .unwrap()
                .physical_target_size()
                .unwrap()
        };
        app.update();
        assert_eq!(target_size(&mut app), UVec2::new(1280, 720));
        app.world_mut()
            .write_message(ConfigureBenchmark(BenchmarkConfig::new(
                50,
                QualityTier::Low,
                7,
            )));
        app.update();
        assert_eq!(target_size(&mut app), UVec2::new(960, 540));
        app.world_mut()
            .write_message(ConfigureBenchmark(BenchmarkConfig::new(
                50,
                QualityTier::High,
                7,
            )));
        app.update();
        assert_eq!(target_size(&mut app), UVec2::new(1280, 720));
        app.world_mut()
            .write_message(ConfigureBenchmark(BenchmarkConfig::new(
                50,
                QualityTier::Low,
                7,
            )));
        app.update();
        app.world_mut().resource_mut::<Art>().enabled = false;
        app.update();
        assert_eq!(target_size(&mut app), UVec2::new(1280, 720));
    }

    #[test]
    fn changing_raster_dimensions_discards_prior_samples_and_restarts_warmup() {
        let mut app = App::new();
        install_test_scene(&mut app);
        app.world_mut()
            .write_message(ConfigureBenchmark(BenchmarkConfig::new(
                50,
                QualityTier::Low,
                7,
            )));
        app.update();
        {
            let mut benchmark = app.world_mut().resource_mut::<Benchmark>();
            benchmark.intervals_ms = vec![16.0, 18.0];
            benchmark.sample_window_ms = 34.0;
            benchmark.ready_since = Some(0.0);
            benchmark.previous_sample_at = Some(1.0);
        }
        let world = app.world_mut();
        world
            .query_filtered::<&mut Window, With<PrimaryWindow>>()
            .single_mut(world)
            .unwrap()
            .resolution
            .set_physical_resolution(1600, 900);
        app.update();
        let benchmark = app.world().resource::<Benchmark>();
        assert_eq!(
            (benchmark.render_width, benchmark.render_height),
            (1200, 675)
        );
        assert!(benchmark.intervals_ms.is_empty());
        assert_eq!(benchmark.sample_window_ms, 0.0);
        assert!(benchmark.ready_since.is_none());
        assert!(benchmark.previous_sample_at.is_none());
    }

    #[test]
    fn rapid_reconfigure_and_art_exit_do_not_stack_or_retain_scene_owners() {
        let mut app = App::new();
        install_test_scene(&mut app);
        for population in [500, 200, 50] {
            app.world_mut()
                .write_message(ConfigureBenchmark(BenchmarkConfig::new(
                    population,
                    QualityTier::High,
                    7,
                )));
        }
        app.update();
        assert_eq!(
            app.world()
                .resource::<Benchmark>()
                .config
                .unwrap()
                .population,
            50
        );
        assert_eq!(app.world().resource::<Benchmark>().actors, 19);
        app.world_mut().resource_mut::<Art>().enabled = false;
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<Entity, With<BenchmarkEntity>>()
                .iter(world)
                .count(),
            0
        );
        assert!(!world.resource::<Benchmark>().is_ready());
        world.resource_mut::<Art>().enabled = true;
        app.update();
        assert_eq!(app.world().resource::<Benchmark>().actors, 19);
    }

    #[test]
    fn render_stress_and_tier_changes_leave_every_gameplay_tick_hash_identical() {
        let mut baseline = myva_gameplay::headless::mach_app(7, myva_gameplay::Authority::Client);
        let mut stressed = myva_gameplay::headless::mach_app(7, myva_gameplay::Authority::Client);
        install_test_scene(&mut stressed);
        for frame in 0..360 {
            if frame % 60 == 0 {
                let population = [50, 200, 500][frame / 60 % 3];
                let tier = if frame % 120 == 0 {
                    QualityTier::Low
                } else {
                    QualityTier::High
                };
                stressed
                    .world_mut()
                    .write_message(ConfigureBenchmark(BenchmarkConfig::new(
                        population, tier, 7,
                    )));
            }
            baseline.update();
            stressed.update();
            assert_eq!(
                baseline
                    .world()
                    .resource::<myva_gameplay::headless::TickHistory>(),
                stressed
                    .world()
                    .resource::<myva_gameplay::headless::TickHistory>()
            );
        }
        assert!(
            !baseline
                .world()
                .resource::<myva_gameplay::headless::TickHistory>()
                .0
                .is_empty()
        );
        assert_eq!(
            baseline
                .world()
                .resource::<myva_gameplay::SimSession>()
                .get()
                .replay()
                .to_text(),
            stressed
                .world()
                .resource::<myva_gameplay::SimSession>()
                .get()
                .replay()
                .to_text()
        );
    }
    #[test]
    fn readiness_waits_for_actual_body_atlas_bindings_not_default_sprite_components() {
        let mut app = App::new();
        install_test_scene(&mut app);
        app.world_mut().write_message(ConfigureBenchmark(Some(
            BenchmarkConfig::new(50, QualityTier::High, 7).unwrap(),
        )));
        let first = app.world_mut().spawn((ArtBody, Sprite::default())).id();
        let second = app.world_mut().spawn((ArtBody, Sprite::default())).id();
        app.update();
        assert_eq!(app.world().resource::<Benchmark>().gameplay_fighters, 0);
        assert!(!app.world().resource::<Benchmark>().is_ready());
        // Synthetic component bindings test the census, not an uploaded/rendered texture.
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::default());
        for entity in [first, second] {
            app.world_mut()
                .entity_mut(entity)
                .insert(Sprite::from_atlas_image(
                    image.clone(),
                    TextureAtlas {
                        layout: Handle::default(),
                        index: 0,
                    },
                ));
        }
        app.update();
        assert_eq!(app.world().resource::<Benchmark>().gameplay_fighters, 2);
        assert!(app.world().resource::<Benchmark>().is_ready());
    }
}
