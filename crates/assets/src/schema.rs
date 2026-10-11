//! Định dạng file của pipeline (format version 1). Trường lạ bị từ chối để một bản ghi mới hơn
//! không bị đọc nhầm thành bản cũ; đổi ý nghĩa trường phải tăng `format_version`.

use serde::{Deserialize, Serialize};

pub const MANIFEST_FORMAT: &str = "myva.asset-manifest";
pub const SHEET_FORMAT: &str = "myva.sprite-sheet";
pub const LAYOUT_FORMAT: &str = "myva.map-layout";
pub const FORMAT_VERSION: u32 = 1;

/// Danh mục nội dung có version: mỗi file một mục có ID ổn định, hash và nguồn gốc.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub format_version: u32,
    /// Phiên bản nội dung cả bộ; client và server đối chiếu khi vào phiên.
    pub content_version: String,
    pub texels_per_unit: TexelsPerUnit,
    pub limits: Limits,
    pub assets: Vec<AssetEntry>,
    pub bundles: Vec<Bundle>,
}

impl Manifest {
    pub fn asset(&self, id: &str) -> Option<&AssetEntry> {
        self.assets.iter().find(|asset| asset.id == id)
    }

    pub fn bundle_of(&self, asset: &str) -> Option<&Bundle> {
        self.bundles
            .iter()
            .find(|bundle| bundle.assets.iter().any(|id| id == asset))
    }
}

/// Mật độ texel trên một px thiết kế (đơn vị thế giới của graybox) theo mặt phẳng.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TexelsPerUnit {
    pub gameplay: f32,
    pub backdrop: f32,
}

impl TexelsPerUnit {
    pub const fn of(&self, plane: Plane) -> f32 {
        match plane {
            Plane::Gameplay => self.gameplay,
            Plane::Backdrop => self.backdrop,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_texture_side: u32,
    /// Tổng byte RGBA8 sau decode của mọi texture, chưa tính mipmap.
    pub total_texture_bytes: u64,
    pub total_file_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Texture,
    SpriteSheet,
    MapLayout,
}

impl AssetKind {
    pub const fn extension(self) -> &'static str {
        match self {
            AssetKind::Texture => ".png",
            AssetKind::SpriteSheet | AssetKind::MapLayout => ".json",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetEntry {
    pub id: String,
    pub kind: AssetKind,
    /// Tương đối với thư mục chứa manifest, phân cách `/`.
    pub path: String,
    pub version: u32,
    pub bytes: u64,
    /// SHA-256 hex chữ thường của nguyên file.
    pub sha256: String,
    pub dependencies: Vec<String>,
    pub provenance: Provenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture: Option<TextureInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

impl TextureInfo {
    /// Byte sau decode trên CPU, cũng là ước lượng base level trên GPU (chưa mipmap, chưa nén).
    pub const fn decoded_bytes(&self) -> u64 {
        self.width as u64 * self.height as u64 * 4
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureFormat {
    Rgba8Srgb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// Sinh bằng công cụ trong repo; `tool` là đường dẫn công cụ.
    Procedural,
    /// Vẽ tay bởi người đóng góp.
    Authored,
    /// Nhập từ ngoài; `source` là nơi lấy, license phải cho phép phân phối.
    ThirdParty,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub origin: Origin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub author: String,
    /// SPDX ID hoặc `LicenseRef-*`; xem [`crate::ALLOWED_LICENSES`].
    pub license: String,
    pub redistribution: String,
    pub created: String,
}

/// Đơn vị tải: trùng `BundleKind` của planner D05-B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleKind {
    Character,
    Boss,
    Map,
    Vfx,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub id: String,
    pub kind: BundleKind,
    pub assets: Vec<String>,
    /// Bundle khác được phép tham chiếu; mọi tham chiếu ra ngoài khác là lỗi.
    pub dependencies: Vec<String>,
    pub budget: Budget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub texture_bytes: u64,
    pub file_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Character,
    Boss,
    Vfx,
    Tileset,
    Backdrop,
}

impl Role {
    /// Sprite có ô trong suốt quanh hình; tileset/backdrop phủ kín ô.
    pub const fn has_transparent_border(self) -> bool {
        matches!(self, Role::Character | Role::Boss | Role::Vfx)
    }

    pub const fn plane(self) -> Plane {
        match self {
            Role::Backdrop => Plane::Backdrop,
            _ => Plane::Gameplay,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plane {
    Gameplay,
    Backdrop,
}

/// Metadata atlas: vị trí frame, pivot, clip và mốc sự kiện.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteSheet {
    pub format: String,
    pub format_version: u32,
    pub id: String,
    pub texture: String,
    pub role: Role,
    pub plane: Plane,
    pub texels_per_unit: f32,
    /// Khoảng trống tối thiểu giữa các frame và tới mép atlas (texel).
    pub padding: u32,
    /// Số texel biên được nhân ra ngoài frame (tile lát liền); nằm trong `padding`.
    pub extrude: u32,
    pub frames: Vec<Frame>,
    pub clips: Vec<Clip>,
    /// `Kit::id` của lõi mà sheet thể hiện (nhân vật, boss).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

impl SpriteSheet {
    pub fn frame(&self, name: &str) -> Option<&Frame> {
        self.frames.iter().find(|frame| frame.name == name)
    }

    pub fn clip(&self, name: &str) -> Option<&Clip> {
        self.clips.iter().find(|clip| clip.name == name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub name: String,
    /// `[x, y, w, h]` trong atlas, gốc trên-trái.
    pub rect: [u32; 4],
    /// Điểm neo trong frame (texel, gốc trên-trái, y hướng xuống): chân chạm nền với nhân vật.
    pub pivot: [u32; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub name: String,
    pub frames: Vec<String>,
    /// Thời lượng mỗi frame khi clip chạy theo thời gian thực (idle, run, VFX). Clip đồng bộ
    /// theo pha đòn của lõi (`attack`) dùng mốc `hit` thay vì thời lượng này.
    pub frame_ms: u32,
    #[serde(rename = "loop")]
    pub looping: bool,
    pub events: Vec<ClipEvent>,
}

impl Clip {
    pub fn event(&self, name: &str) -> Option<usize> {
        self.events
            .iter()
            .find(|event| event.name == name)
            .map(|event| event.frame)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipEvent {
    pub frame: usize,
    pub name: String,
}

/// Bố cục bản đồ: nền lát gạch và các lớp phông, chỉ tham chiếu frame của sheet phụ thuộc.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapLayout {
    pub format: String,
    pub format_version: u32,
    pub id: String,
    pub name: String,
    pub ground: Ground,
    pub layers: Vec<LayerSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ground {
    pub sheet: String,
    pub top: String,
    pub fill: String,
    /// Mặt nền theo px thiết kế; lõi đặt nền ở y = 0.
    pub surface_y: i32,
    pub depth: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerSpec {
    pub sheet: String,
    pub frame: String,
    /// Thứ tự vẽ; âm là sau mặt phẳng gameplay.
    pub z: i32,
    /// Đáy lớp theo px thiết kế.
    pub y: i32,
    #[serde(default)]
    pub repeat: bool,
    /// Kéo giãn tới `[rộng, cao]` px thiết kế (dải màu trời).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stretch: Option<[u32; 2]>,
}
