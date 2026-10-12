//! Fixture art D05-A trên `AssetServer` (docs/technical/asset-pipeline.md). Chỉ thể hiện: đọc
//! [`ArenaView`] và [`SimEvent`], không ghi gì vào phiên, replay hay luật combat.
//!
//! Catalog đọc `fixture/manifest.json` không tải texture. Runtime planner tải manifest mỗi bundle
//! (`runtime-*.myva`) qua `AssetServer`, kiểm kích thước và
//! SHA-256 trước khi parse/giải mã, rồi chạy cùng luật `myva_assets::check_documents` với CLI.
//! Bundle thiết yếu lỗi sau retry làm presentation `Failed`; VFX tùy chọn giữ geometric fallback.
//! Texture chỉ giữ ở render world (`RenderAssetUsages::RENDER_WORLD`) sau khi upload.
//!
//! Hình học gameplay không đổi: vùng báo, hitbox debug, viền đỡ và tường vẫn là hình khối; sprite
//! thay phần thân hình chữ nhật và nền. Animation đòn đồng bộ theo pha của lõi: frame có mốc
//! `hit` hiện đúng lúc đòn vào `Active`, không theo FPS. Phím G bật/tắt art.

use std::collections::BTreeMap;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, RenderAssetUsages};
use bevy::image::{
    CompressedImageFormats, ImageSampler, ImageType, TextureAtlas, TextureAtlasLayout,
};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use myva_assets::{AssetKind, Documents, Manifest, MapLayout, Role, SpriteSheet};
use myva_gameplay::SimEvent;
use myva_sim::fighter::ARENA_WIDTH;
use myva_sim::snapshot::FighterView;
use myva_sim::{Event, FighterId, Phase, State};

use crate::fight::{ArenaView, Match};
use crate::scene::{Cover, px};

/// Đường dẫn manifest trong nguồn asset mặc định (`assets/` của repo khi chạy native, `assets/`
/// cạnh game document trên web).
pub const FIXTURE_MANIFEST: &str = "fixture/manifest.json";

const BODY_Z: f32 = 2.0;
const VFX_Z: f32 = 3.5;
const PROJECTILE_Z: f32 = 3.0;
/// Giảm alpha khi bất tử, giống thân hình khối.
const INVULNERABLE_ALPHA: f32 = 0.45;

// Every scene-scoped entity that clones image/atlas handles carries this marker. Runtime exit
// despawns these owners before dropping lookup tables and planner bundle handles.
#[derive(Component)]
pub struct ArtEntity;

/// Body sprite census, distinct from gameplay mirror entities and render-only benchmarks.
#[derive(Component)]
pub struct ArtBody;

// ---------------------------------------------------------------- Asset và loader

/// Toàn bộ fixture đã kiểm hash và luật; texture là sub-asset gắn nhãn theo ID.
#[derive(Asset, TypePath, Debug)]
pub struct ArtFixture {
    pub manifest: Manifest,
    pub documents: Documents,
    pub textures: BTreeMap<String, Handle<Image>>,
}

#[derive(Debug)]
pub struct ArtError {
    /// Mã ngắn ổn định cho telemetry: `io`, `json`, `hash_mismatch`, `image`, `invalid`, ...
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for ArtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ArtError {}

impl From<myva_assets::Issue> for ArtError {
    fn from(issue: myva_assets::Issue) -> Self {
        Self {
            code: issue.code.as_str(),
            message: format!("{}: {}", issue.subject, issue.message),
        }
    }
}

fn fail(code: &str, message: impl std::fmt::Display) -> ArtError {
    ArtError {
        code: code.to_owned(),
        message: message.to_string(),
    }
}

/// Lightweight validated catalog. Loading it never fetches or decodes texture content.
#[derive(Asset, TypePath, Debug)]
pub struct ArtCatalog(pub Manifest);

#[derive(Default, TypePath)]
pub struct CatalogLoader;

impl AssetLoader for CatalogLoader {
    type Asset = ArtCatalog;
    type Settings = ();
    type Error = ArtError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _context: &mut LoadContext<'_>,
    ) -> Result<ArtCatalog, ArtError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| fail("io", e))?;
        let manifest = myva_assets::parse_manifest(&bytes)?;
        let mut issues = Vec::new();
        myva_assets::validate::check_manifest(&manifest, &mut issues);
        if let Some(issue) = issues.into_iter().next() {
            return Err(issue.into());
        }
        Ok(ArtCatalog(manifest))
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

/// Immutable expected catalog projection, compared before any payload I/O or decode.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ArtSettings {
    pub expected: Option<Manifest>,
}

#[derive(Default, TypePath)]
pub struct ArtLoader;

impl AssetLoader for ArtLoader {
    type Asset = ArtFixture;
    type Settings = ArtSettings;
    type Error = ArtError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &ArtSettings,
        context: &mut LoadContext<'_>,
    ) -> Result<ArtFixture, ArtError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| fail("io", e))?;
        let manifest = myva_assets::parse_manifest(&bytes)?;
        let mut issues = Vec::new();
        myva_assets::validate::check_manifest(&manifest, &mut issues);
        if let Some(issue) = issues.into_iter().next() {
            return Err(issue.into());
        }
        if settings.expected.as_ref() != Some(&manifest) {
            return Err(fail(
                "projection_mismatch",
                "bundle differs from validated catalog before payload I/O",
            ));
        }
        let base = context
            .path()
            .path()
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let mut documents = Documents::default();
        let mut textures = BTreeMap::new();
        for asset in &manifest.assets {
            if !myva_assets::validate::valid_path(&asset.path) {
                return Err(fail("bad_path", &asset.path));
            }
            let path = myva_assets::join(&base, &asset.path);
            let data = context
                .read_asset_bytes(path.clone())
                .await
                .map_err(|e| fail("io", format!("{path}: {e}")))?;
            myva_assets::verify_bytes(asset, &data)?;
            match asset.kind {
                AssetKind::Texture => {
                    // Check PNG dimensions before allocating the decoded image. Hash/size alone
                    // cannot bound a decompression bomb when the manifest itself is malformed.
                    let Some(info) = asset.texture else {
                        return Err(fail("texture_dims", "texture metadata missing"));
                    };
                    if data.len() < 24
                        || !data.starts_with(b"\x89PNG\r\n\x1a\n")
                        || u32::from_be_bytes(data[16..20].try_into().expect("checked length"))
                            != info.width
                        || u32::from_be_bytes(data[20..24].try_into().expect("checked length"))
                            != info.height
                    {
                        return Err(fail(
                            "texture_dims",
                            format!("{}: PNG header differs from manifest", asset.id),
                        ));
                    }
                    let image = Image::from_buffer(
                        &data,
                        ImageType::Extension("png"),
                        CompressedImageFormats::NONE,
                        true,
                        ImageSampler::linear(),
                        RenderAssetUsages::RENDER_WORLD,
                    )
                    .map_err(|e| fail("image", format!("{}: {e}", asset.id)))?;
                    let Some(info) = asset.texture else {
                        return Err(fail("texture_dims", "texture metadata missing"));
                    };
                    if image.size() != UVec2::new(info.width, info.height) {
                        return Err(fail(
                            "texture_dims",
                            format!("{}: decoded PNG differs from manifest", asset.id),
                        ));
                    }
                    let handle = context.add_labeled_asset(asset.id.clone(), image);
                    textures.insert(asset.id.clone(), handle);
                }
                AssetKind::SpriteSheet => {
                    let sheet = myva_assets::parse_sheet(&asset.id, &data)?;
                    documents.sheets.insert(asset.id.clone(), sheet);
                }
                AssetKind::MapLayout => {
                    let layout = myva_assets::parse_layout(&asset.id, &data)?;
                    documents.layouts.insert(asset.id.clone(), layout);
                }
            }
        }
        let issues = myva_assets::check_documents(&manifest, &documents);
        if let Some(first) = issues.first() {
            return Err(ArtError {
                code: first.code.as_str(),
                message: issues
                    .iter()
                    .map(|issue| format!("{}: {}", issue.subject, issue.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            });
        }
        Ok(ArtFixture {
            manifest,
            documents,
            textures,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["myva"]
    }
}

// ---------------------------------------------------------------- Bảng tra runtime

/// Một frame: chỉ số atlas, kích thước theo px thiết kế và anchor Bevy từ pivot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameView {
    pub index: usize,
    pub size: Vec2,
    pub anchor: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipView {
    pub frames: Vec<FrameView>,
    pub frame_secs: f32,
    pub looping: bool,
    pub hit: Option<usize>,
    pub apex: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct SheetView {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub frames: BTreeMap<String, FrameView>,
    pub clips: BTreeMap<String, ClipView>,
}

impl SheetView {
    fn sprite(&self, frame: FrameView, flip: bool) -> (Sprite, Anchor) {
        let mut sprite = Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index: frame.index,
            },
        );
        sprite.custom_size = Some(frame.size);
        sprite.flip_x = flip;
        (sprite, Anchor(anchor(frame.anchor, flip)))
    }
}

/// Pivot (texel, gốc trên-trái, y xuống) → anchor Bevy (−0,5..0,5, y lên).
pub fn pivot_anchor(rect: [u32; 4], pivot: [u32; 2]) -> Vec2 {
    let (w, h) = (rect[2] as f32, rect[3] as f32);
    Vec2::new(pivot[0] as f32 / w - 0.5, 0.5 - pivot[1] as f32 / h)
}

/// Lật ngang đổi UV nhưng giữ vị trí quad, nên anchor cũng phải lật để pivot đứng yên.
fn anchor(anchor: Vec2, flip: bool) -> Vec2 {
    if flip {
        Vec2::new(-anchor.x, anchor.y)
    } else {
        anchor
    }
}

fn sheet_view(
    sheet: &SpriteSheet,
    image: Handle<Image>,
    texture_size: UVec2,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> SheetView {
    let mut atlas = TextureAtlasLayout::new_empty(texture_size);
    let mut frames = BTreeMap::new();
    for frame in &sheet.frames {
        let [x, y, w, h] = frame.rect;
        let index = atlas.add_texture(URect::new(x, y, x + w, y + h));
        frames.insert(
            frame.name.clone(),
            FrameView {
                index,
                size: Vec2::new(w as f32, h as f32) / sheet.texels_per_unit,
                anchor: pivot_anchor(frame.rect, frame.pivot),
            },
        );
    }
    let clips = sheet
        .clips
        .iter()
        .map(|clip| {
            (
                clip.name.clone(),
                ClipView {
                    frames: clip
                        .frames
                        .iter()
                        .filter_map(|f| frames.get(f).copied())
                        .collect(),
                    frame_secs: clip.frame_ms as f32 / 1_000.0,
                    looping: clip.looping,
                    hit: clip.event("hit"),
                    apex: clip.event("apex"),
                },
            )
        })
        .collect();
    SheetView {
        image,
        layout: layouts.add(atlas),
        frames,
        clips,
    }
}

/// Sheet theo ID asset và theo `subject` (kit id của lõi).
#[derive(Resource, Default)]
pub struct ArtTables {
    pub sheets: BTreeMap<String, SheetView>,
    pub subjects: BTreeMap<String, String>,
    pub layout: Option<MapLayout>,
}

impl ArtTables {
    fn for_kit(&self, kit: &str) -> Option<&SheetView> {
        self.subjects.get(kit).and_then(|id| self.sheets.get(id))
    }

    fn vfx(&self) -> Option<&SheetView> {
        self.sheets.get("vfx.combat")
    }
}

// ---------------------------------------------------------------- Trạng thái

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtStatus {
    /// Host tắt art (`--no-art`).
    Off,
    Loading,
    Ready,
    Failed {
        code: String,
        message: String,
    },
}

#[derive(Resource)]
pub struct Art {
    pub status: ArtStatus,
    /// Người chơi bật/tắt bằng phím G; chỉ có tác dụng khi `Ready`.
    pub enabled: bool,
    pub content_version: Option<String>,
}

impl Art {
    pub fn visible(&self) -> bool {
        self.enabled && self.status == ArtStatus::Ready
    }
}

/// Tóm tắt cho telemetry host (ASCII).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArtSummary {
    pub status: &'static str,
    pub visible: bool,
    pub sheets: usize,
    pub failure: Option<String>,
}

pub fn summary(art: &Art, tables: &ArtTables) -> ArtSummary {
    ArtSummary {
        status: match art.status {
            ArtStatus::Off => "off",
            ArtStatus::Loading => "loading",
            ArtStatus::Ready => "ready",
            ArtStatus::Failed { .. } => "failed",
        },
        visible: art.visible(),
        sheets: tables.sheets.len(),
        failure: match &art.status {
            ArtStatus::Failed { code, .. } => Some(code.clone()),
            _ => None,
        },
    }
}

/// Thêm vào app có `DefaultPlugins` trước [`crate::GrayboxPlugin`]. `enabled = false` không tải
/// gì và giữ cảnh hình khối.
pub struct ArtPlugin {
    pub enabled: bool,
}

impl Plugin for ArtPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ArtCatalog>()
            .register_asset_loader(CatalogLoader)
            .init_asset::<ArtFixture>()
            .register_asset_loader(ArtLoader)
            .init_resource::<ArtTables>()
            .add_message::<VfxRequest>()
            .insert_resource(Art {
                status: if self.enabled {
                    ArtStatus::Loading
                } else {
                    ArtStatus::Off
                },
                enabled: self.enabled,
                content_version: None,
            })
            .add_plugins(crate::runtime::plugin)
            .add_systems(
                Update,
                (
                    toggle_art,
                    crate::runtime::drive,
                    (
                        sync_visibility,
                        animate_fighters,
                        projectiles,
                        spawn_vfx,
                        animate_vfx,
                    )
                        .run_if(|art: Res<Art>| art.status == ArtStatus::Ready),
                )
                    .chain(),
            );
    }
}

/// Merge one validated load unit into scene-scoped tables. No hidden global handle cache.
pub(crate) fn install_bundle(
    built: &mut ArtTables,
    fixture: &ArtFixture,
    layouts: &mut Assets<TextureAtlasLayout>,
) {
    for (id, sheet) in &fixture.documents.sheets {
        let (Some(image), Some(info)) = (
            fixture.textures.get(&sheet.texture),
            fixture
                .manifest
                .asset(&sheet.texture)
                .and_then(|a| a.texture),
        ) else {
            continue;
        };
        let view = sheet_view(
            sheet,
            image.clone(),
            UVec2::new(info.width, info.height),
            layouts,
        );
        if let Some(subject) = &sheet.subject
            && matches!(sheet.role, Role::Character | Role::Boss)
        {
            built.subjects.insert(subject.clone(), id.clone());
        }
        built.sheets.insert(id.clone(), view);
    }
    if let Some(layout) = fixture.documents.layouts.values().next() {
        built.layout = Some(layout.clone());
    }
}

fn toggle_art(keys: Res<ButtonInput<KeyCode>>, mut art: ResMut<Art>) {
    if keys.just_pressed(KeyCode::KeyG) {
        art.enabled = !art.enabled;
    }
}

/// Phần hình khối mà art đang thay; `scene::shapes` bỏ qua các phần này.
pub fn cover(art: &Art, tables: &ArtTables, arena: &ArenaView, player: FighterId) -> Cover {
    if !art.visible() {
        return Cover::default();
    }
    Cover {
        floor: tables.layout.is_some(),
        bodies: arena
            .fighters
            .iter()
            .filter(|f| tables.for_kit(f.kit.id).is_some())
            .map(|f| f.id)
            .collect(),
        projectiles_of: tables
            .vfx()
            .filter(|vfx| vfx.clips.contains_key("bolt"))
            .map(|_| player),
    }
}

/// Nâng nhãn tên khi art hiện: đầu sprite cao hơn hộp va chạm vài px.
pub fn tag_lift(art: &Art) -> f32 {
    if art.visible() { 14.0 } else { 0.0 }
}

// ---------------------------------------------------------------- Bản đồ

#[derive(Component)]
struct ArtLayer {
    parallax: Option<usize>,
}

pub(crate) fn spawn_map(commands: &mut Commands, tables: &ArtTables) {
    let Some(layout) = &tables.layout else {
        return;
    };
    let width = px(ARENA_WIDTH);
    if let Some(ground) = tables.sheets.get(&layout.ground.sheet) {
        let (Some(top), Some(fill)) = (
            ground.frames.get(&layout.ground.top),
            ground.frames.get(&layout.ground.fill),
        ) else {
            return;
        };
        let rows = (layout.ground.depth as f32 / fill.size.y).ceil() as i32;
        let columns = (width / top.size.x).ceil() as i32;
        for row in 0..rows {
            let frame = if row == 0 { *top } else { *fill };
            for column in 0..columns {
                let (sprite, anchor) = ground.sprite(frame, false);
                commands.spawn((
                    ArtEntity,
                    ArtLayer { parallax: None },
                    sprite,
                    anchor,
                    Transform::from_xyz(
                        column as f32 * frame.size.x,
                        layout.ground.surface_y as f32 - row as f32 * frame.size.y,
                        0.05,
                    ),
                    Visibility::Hidden,
                ));
            }
        }
    }
    for (depth, layer) in layout.layers.iter().enumerate() {
        let Some(sheet) = tables.sheets.get(&layer.sheet) else {
            continue;
        };
        let Some(frame) = sheet.frames.get(&layer.frame).copied() else {
            continue;
        };
        // Phông phủ hơn bề rộng arena để màn hình rộng không lộ mép.
        let (start, end) = (-width * 0.5, width * 1.5);
        let z = layer.z as f32 / 100.0;
        if let Some([w, h]) = layer.stretch {
            let size = Vec2::new(w as f32, h as f32);
            let (mut sprite, anchor) = sheet.sprite(FrameView { size, ..frame }, false);
            sprite.custom_size = Some(size);
            commands.spawn((
                ArtEntity,
                ArtLayer {
                    parallax: Some(depth),
                },
                sprite,
                anchor,
                Transform::from_xyz(width / 2.0 - size.x / 2.0, layer.y as f32, z),
                Visibility::Hidden,
            ));
            continue;
        }
        let mut x = if layer.repeat { start } else { 0.0 };
        loop {
            let (sprite, anchor) = sheet.sprite(frame, false);
            commands.spawn((
                ArtEntity,
                ArtLayer {
                    parallax: Some(depth),
                },
                sprite,
                anchor,
                Transform::from_xyz(x, layer.y as f32, z),
                Visibility::Hidden,
            ));
            x += frame.size.x;
            if !layer.repeat || x >= end {
                break;
            }
        }
    }
}

fn sync_visibility(
    art: Res<Art>,
    benchmark: Option<Res<crate::benchmark::Benchmark>>,
    mut layers: Query<(&ArtLayer, &mut Visibility)>,
) {
    let limit = benchmark.as_ref().and_then(|b| b.parallax_layers());
    // New map entities start hidden, so update even when Art itself has not changed.
    for (layer, mut visibility) in &mut layers {
        let visible = art.visible()
            && !layer
                .parallax
                .zip(limit)
                .is_some_and(|(depth, limit)| depth >= limit);
        visibility.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

// ---------------------------------------------------------------- Nhân vật

#[derive(Component)]
struct FighterArt {
    id: FighterId,
    clip: &'static str,
    frame: usize,
    timer: f32,
    last: Option<Vec2>,
    grounded: bool,
    phase: Option<Phase>,
}

/// Clip và frame cho một fighter, từ trạng thái lõi và chuyển động giữa hai khung hình.
pub fn pick_frame(
    sheet: &SheetView,
    fighter: &FighterView,
    velocity: Vec2,
    loop_frame: usize,
) -> Option<(&'static str, usize)> {
    let has = |name: &str| sheet.clips.contains_key(name);
    if let Some((action, spec)) = fighter.action.zip(fighter.action_spec())
        && let Some(clip) = sheet.clips.get("attack")
    {
        let hit = clip.hit.unwrap_or(0).min(clip.frames.len() - 1);
        let after = clip.frames.len() - 1 - hit;
        let frame = match action.phase {
            Phase::Startup => {
                let t = action.elapsed as f32 / spec.startup.max(1) as f32;
                ((t * hit as f32) as usize).min(hit.saturating_sub(1))
            }
            Phase::Active => hit,
            Phase::Recovery if after == 0 => hit,
            Phase::Recovery => {
                let into = action.elapsed.saturating_sub(spec.startup + spec.active);
                let t = into as f32 / spec.recovery.max(1) as f32;
                hit + 1 + ((t * after as f32) as usize).min(after - 1)
            }
        };
        return Some(("attack", frame));
    }
    if !fighter.grounded
        && let Some(clip) = sheet.clips.get("jump")
    {
        let apex = clip.apex.unwrap_or(clip.frames.len() / 2);
        let frame = if velocity.y > 0.6 {
            apex.saturating_sub(1)
        } else if velocity.y < -0.6 {
            (apex + 1).min(clip.frames.len() - 1)
        } else {
            apex
        };
        return Some(("jump", frame));
    }
    let moving = velocity.x.abs() > 0.2 || matches!(fighter.state, State::Dash { .. });
    if moving && has("run") {
        return Some(("run", loop_frame));
    }
    has("idle").then_some(("idle", loop_frame))
}

/// Màu phủ theo trạng thái, nhẹ hơn màu thân graybox để vẫn thấy hình vẽ.
fn tint(fighter: &FighterView) -> Color {
    let base = match fighter.state {
        State::Guard { .. } => Color::srgb(0.75, 0.9, 1.0),
        State::Hitstun { .. } => Color::srgb(1.0, 0.6, 0.85),
        State::GuardBreak { .. } => Color::srgb(0.75, 0.6, 1.0),
        State::Downed => Color::srgb(0.45, 0.45, 0.5),
        _ => Color::WHITE,
    };
    if fighter.invulnerable && fighter.state != State::Downed {
        base.with_alpha(INVULNERABLE_ALPHA)
    } else {
        base
    }
}

#[allow(clippy::too_many_arguments)]
fn animate_fighters(
    mut commands: Commands,
    art: Res<Art>,
    tables: Res<ArtTables>,
    arena: Res<ArenaView>,
    time: Res<Time>,
    mut sprites: Query<(
        Entity,
        &mut FighterArt,
        &mut Sprite,
        &mut Anchor,
        &mut Transform,
        &mut Visibility,
    )>,
    mut vfx: MessageWriter<VfxRequest>,
) {
    let mut seen = Vec::new();
    for (entity, mut state, mut sprite, mut anchor_c, mut transform, mut visibility) in &mut sprites
    {
        let fighter = arena.fighter(state.id);
        let sheet = fighter.and_then(|f| tables.for_kit(f.kit.id));
        let (Some(fighter), Some(sheet), true) = (fighter, sheet, art.visible()) else {
            *visibility = Visibility::Hidden;
            if arena.fighter(state.id).is_none() {
                commands.entity(entity).despawn();
            }
            continue;
        };
        seen.push(state.id);
        let at = Vec2::new(px(fighter.x), px(fighter.y));
        let velocity = state.last.map_or(Vec2::ZERO, |last| at - last);
        state.last = Some(at);
        // Đáp đất: bụi dưới chân.
        if fighter.grounded && !state.grounded {
            vfx.write(VfxRequest::new("dust", at + Vec2::new(0.0, 8.0), false));
        }
        state.grounded = fighter.grounded;
        // Đòn cận chiến vào Active: vệt chém ở hộp đòn thật.
        let phase = fighter.action.map(|a| a.phase);
        if phase == Some(Phase::Active)
            && state.phase != Some(Phase::Active)
            && let Some(attack) = fighter.attack_box
        {
            let center = Vec2::new(
                px(attack.x0 + attack.x1) / 2.0,
                px(attack.y0 + attack.y1) / 2.0,
            );
            vfx.write(VfxRequest::new("slash", center, fighter.facing < 0));
        }
        state.phase = phase;

        let loop_clip = |name: &str| sheet.clips.get(name).map_or(1, |c| c.frames.len().max(1));
        let (clip, frame) = match pick_frame(sheet, fighter, velocity, state.frame) {
            Some(pick) => pick,
            None => continue,
        };
        if clip != state.clip {
            state.clip = clip;
            state.frame = 0;
            state.timer = 0.0;
        }
        let Some(view) = sheet.clips.get(clip) else {
            continue;
        };
        let index = if matches!(clip, "idle" | "run") {
            state.timer += time.delta_secs();
            while state.timer >= view.frame_secs {
                state.timer -= view.frame_secs;
                state.frame = (state.frame + 1) % loop_clip(clip);
            }
            state.frame
        } else {
            frame
        };
        let Some(frame) = view.frames.get(index).copied() else {
            continue;
        };
        let flip = fighter.facing < 0;
        let (fresh, new_anchor) = sheet.sprite(frame, flip);
        sprite.image = fresh.image;
        sprite.texture_atlas = fresh.texture_atlas;
        sprite.custom_size = fresh.custom_size;
        sprite.flip_x = flip;
        sprite.color = tint(fighter);
        *anchor_c = new_anchor;
        transform.translation = at.extend(BODY_Z + f32::from(fighter.id.0) * 0.01);
        *visibility = Visibility::Inherited;
    }
    if !art.visible() {
        return;
    }
    for fighter in &arena.fighters {
        if seen.contains(&fighter.id) || tables.for_kit(fighter.kit.id).is_none() {
            continue;
        }
        if sprites.iter().any(|(_, s, ..)| s.id == fighter.id) {
            continue;
        }
        commands.spawn((
            ArtEntity,
            ArtBody,
            FighterArt {
                id: fighter.id,
                clip: "",
                frame: 0,
                timer: 0.0,
                last: None,
                grounded: fighter.grounded,
                phase: None,
            },
            Sprite::default(),
            Anchor::default(),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
}

// ---------------------------------------------------------------- Đạn và VFX

#[derive(Component)]
struct ProjectileArt(usize);

#[allow(clippy::too_many_arguments)]
fn projectiles(
    mut commands: Commands,
    art: Res<Art>,
    tables: Res<ArtTables>,
    arena: Res<ArenaView>,
    game: Res<Match>,
    time: Res<Time>,
    mut pool: Query<(
        &ProjectileArt,
        &mut Sprite,
        &mut Anchor,
        &mut Transform,
        &mut Visibility,
    )>,
    mut clock: Local<f32>,
) {
    let Some(vfx) = tables.vfx() else {
        return;
    };
    let Some(bolt) = vfx.clips.get("bolt") else {
        return;
    };
    *clock += time.delta_secs();
    let frame = bolt.frames[(*clock / bolt.frame_secs) as usize % bolt.frames.len()];
    let mine: Vec<_> = if art.visible() {
        arena
            .projectiles
            .iter()
            .filter(|p| p.owner == game.bout.player())
            .collect()
    } else {
        Vec::new()
    };
    let mut used = 0;
    for (slot, mut sprite, mut anchor, mut transform, mut visibility) in &mut pool {
        used = used.max(slot.0 + 1);
        match mine.get(slot.0) {
            Some(projectile) => {
                let (fresh, new_anchor) = vfx.sprite(frame, projectile.dir < 0);
                *sprite = fresh;
                *anchor = new_anchor;
                let r = projectile.rect;
                transform.translation =
                    Vec3::new(px(r.x0 + r.x1) / 2.0, px(r.y0 + r.y1) / 2.0, PROJECTILE_Z);
                *visibility = Visibility::Inherited;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
    for index in used..mine.len() {
        commands.spawn((
            ArtEntity,
            ProjectileArt(index),
            Sprite::default(),
            Anchor::default(),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
}

#[derive(Message, Clone, Debug)]
pub struct VfxRequest {
    clip: &'static str,
    at: Vec2,
    flip: bool,
}

impl VfxRequest {
    fn new(clip: &'static str, at: Vec2, flip: bool) -> Self {
        Self { clip, at, flip }
    }
}

#[derive(Component)]
struct VfxArt {
    clip: &'static str,
    flip: bool,
    age: f32,
}

fn spawn_vfx(
    mut commands: Commands,
    art: Res<Art>,
    tables: Res<ArtTables>,
    arena: Res<ArenaView>,
    mut events: MessageReader<SimEvent>,
    mut requests: MessageReader<VfxRequest>,
) {
    let mut wanted: Vec<VfxRequest> = requests.read().cloned().collect();
    for SimEvent(record) in events.read() {
        let (target, clip) = match record.event {
            Event::Hit { target, .. } => (target, "hit"),
            Event::Countered { attacker, .. } => (attacker, "hit"),
            Event::Blocked { target, .. } => (target, "guard"),
            _ => continue,
        };
        if let Some(fighter) = arena.fighter(target) {
            let h = fighter.hurtbox;
            let center = Vec2::new(px(h.x0 + h.x1) / 2.0, px(h.y0 + h.y1) / 2.0 + 10.0);
            wanted.push(VfxRequest::new(clip, center, fighter.facing < 0));
        }
    }
    let Some(sheet) = tables.vfx().filter(|_| art.visible()) else {
        return;
    };
    for request in wanted {
        let Some(frame) = sheet.clips.get(request.clip).and_then(|c| c.frames.first()) else {
            continue;
        };
        let (sprite, anchor) = sheet.sprite(*frame, request.flip);
        commands.spawn((
            ArtEntity,
            VfxArt {
                clip: request.clip,
                flip: request.flip,
                age: 0.0,
            },
            sprite,
            anchor,
            Transform::from_translation(request.at.extend(VFX_Z)),
        ));
    }
}

fn animate_vfx(
    mut commands: Commands,
    art: Res<Art>,
    tables: Res<ArtTables>,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut VfxArt, &mut Sprite, &mut Anchor)>,
) {
    let sheet = tables.vfx();
    for (entity, mut effect, mut sprite, mut anchor) in &mut effects {
        effect.age += time.delta_secs();
        let clip = sheet.and_then(|s| s.clips.get(effect.clip));
        let (Some(sheet), Some(clip), true) = (sheet, clip, art.visible()) else {
            commands.entity(entity).despawn();
            continue;
        };
        let index = (effect.age / clip.frame_secs) as usize;
        let Some(frame) = clip.frames.get(index) else {
            commands.entity(entity).despawn();
            continue;
        };
        let (fresh, new_anchor) = sheet.sprite(*frame, effect.flip);
        sprite.texture_atlas = fresh.texture_atlas;
        sprite.custom_size = fresh.custom_size;
        *anchor = new_anchor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myva_sim::battle::{Battle, Mode};
    use myva_sim::protocol::{Action, Attack, CommandFrame};

    fn long_luu() -> SheetView {
        let sheet = myva_assets::parse_sheet(
            "long_luu.sheet",
            include_bytes!("../../../assets/fixture/long_luu/sheet.json"),
        )
        .unwrap();
        let mut layouts = Assets::<TextureAtlasLayout>::default();
        sheet_view(
            &sheet,
            Handle::default(),
            UVec2::new(4096, 4096),
            &mut layouts,
        )
    }

    #[test]
    fn attack_frames_follow_core_phases_not_fps() {
        let sheet = long_luu();
        let hit = sheet.clips["attack"].hit.unwrap();
        let mut battle = Battle::new(Mode::Duel, 1).unwrap();
        battle.toggle_sparring_bot();
        battle.step(CommandFrame {
            action: Some(Action::Attack(Attack::Light)),
            ..CommandFrame::IDLE
        });
        let player = battle.bout().player();
        let mut seen = Vec::new();
        loop {
            let arena = ArenaView::of(&battle);
            let fighter = arena.fighter(player).unwrap();
            let Some(action) = fighter.action else { break };
            let (clip, frame) = pick_frame(&sheet, fighter, Vec2::ZERO, 0).unwrap();
            assert_eq!(clip, "attack");
            match action.phase {
                Phase::Startup => assert!(frame < hit, "frame báo đòn trước mốc hit"),
                Phase::Active => assert_eq!(frame, hit, "mốc hit hiện đúng lúc Active"),
                Phase::Recovery => assert!(frame > hit),
            }
            seen.push(frame);
            battle.step(CommandFrame::IDLE);
        }
        assert!(
            seen.windows(2).all(|w| w[0] <= w[1]),
            "frame không lùi: {seen:?}"
        );
        assert_eq!(seen.first(), Some(&0));
        assert_eq!(seen.last(), Some(&(sheet.clips["attack"].frames.len() - 1)));
    }

    #[test]
    fn airborne_and_moving_pick_jump_and_run() {
        let sheet = long_luu();
        let battle = Battle::new(Mode::Duel, 1).unwrap();
        let arena = ArenaView::of(&battle);
        let mut fighter = arena.fighter(battle.bout().player()).unwrap().clone();
        assert_eq!(
            pick_frame(&sheet, &fighter, Vec2::ZERO, 2),
            Some(("idle", 2))
        );
        assert_eq!(
            pick_frame(&sheet, &fighter, Vec2::new(4.0, 0.0), 1),
            Some(("run", 1))
        );
        fighter.grounded = false;
        let apex = sheet.clips["jump"].apex.unwrap();
        assert_eq!(
            pick_frame(&sheet, &fighter, Vec2::new(0.0, 5.0), 0),
            Some(("jump", apex - 1))
        );
        assert_eq!(
            pick_frame(&sheet, &fighter, Vec2::ZERO, 0),
            Some(("jump", apex))
        );
        assert_eq!(
            pick_frame(&sheet, &fighter, Vec2::new(0.0, -5.0), 0),
            Some(("jump", apex + 1))
        );
    }

    #[test]
    fn pivot_maps_to_anchor_and_flips() {
        // Chân ở giữa đáy ô 304x256 với lề 12 texel.
        let a = pivot_anchor([0, 0, 304, 256], [144, 244]);
        assert!((a.x - (144.0 / 304.0 - 0.5)).abs() < 1e-6);
        assert!((a.y - (0.5 - 244.0 / 256.0)).abs() < 1e-6);
        assert_eq!(anchor(a, true), Vec2::new(-a.x, a.y));
        assert_eq!(pivot_anchor([0, 0, 10, 10], [5, 5]), Vec2::ZERO);
    }
}
