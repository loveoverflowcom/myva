//! Pipeline asset 2D của MyVa — Thần Mạch (D05-A, `docs/technical/asset-pipeline.md`).
//!
//! Crate này định nghĩa định dạng manifest, sprite sheet và bố cục bản đồ, cùng validator dùng
//! chung cho CLI/CI và client Bevy. Không phụ thuộc Bevy, renderer hay luật combat: asset chỉ thể
//! hiện trạng thái lõi, không quyết định hitbox hay thời gian đòn.
//!
//! - [`validate::check_documents`]: luật trên dữ liệu đã parse, chạy được trong client.
//! - [`verify_bytes`]: kích thước và SHA-256 của một file so với manifest.
//! - `files::validate_dir` (feature `fs`): đọc thư mục, giải mã PNG, kiểm viền frame/extrude
//!   và file không khai báo.

pub mod schema;
pub mod validate;

#[cfg(feature = "fs")]
pub mod files;

use sha2::{Digest, Sha256};

pub use schema::*;
pub use validate::{Documents, check_documents};

/// License đã được chủ dự án chấp nhận cho asset trong repo. `LicenseRef-MyVa-Unreleased` là nội
/// dung do dự án tạo, chưa có license phát hành (CONTRIBUTING.md); không dùng cho asset ngoài.
pub const ALLOWED_LICENSES: [&str; 2] = ["LicenseRef-MyVa-Unreleased", "CC0-1.0"];
/// Trần chính sách: cạnh texture an toàn cho mobile/WebGL2 và budget GPU ước tính 128 MiB
/// (`docs/technical/assets-performance.md` §8). Manifest chỉ được đặt limit thấp hơn.
pub const MAX_TEXTURE_SIDE: u32 = 4_096;
pub const MAX_TOTAL_TEXTURE_BYTES: u64 = 128 << 20;

/// Mã lỗi ổn định cho test và báo cáo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    Json,
    Format,
    BadId,
    DuplicateId,
    BadPath,
    MissingFile,
    UnlistedFile,
    SizeMismatch,
    HashMismatch,
    UnknownDependency,
    DependencyCycle,
    DependencyKind,
    PngInvalid,
    TextureFormat,
    TextureDims,
    TextureTooLarge,
    SheetTexture,
    FrameBounds,
    FrameOverlap,
    FrameClipped,
    Padding,
    Extrude,
    Pivot,
    PivotInconsistent,
    Scale,
    ClipFrame,
    ClipMissing,
    ClipEvent,
    ClipTiming,
    LayoutRef,
    BundleUnknownAsset,
    BundleDuplicateAsset,
    BundleOrphanAsset,
    BundleCrossReference,
    Budget,
    Provenance,
    License,
}

impl Code {
    pub fn as_str(self) -> String {
        let name = format!("{self:?}");
        let mut out = String::with_capacity(name.len() + 4);
        for (i, ch) in name.chars().enumerate() {
            if ch.is_ascii_uppercase() {
                if i > 0 {
                    out.push('_');
                }
                out.push(ch.to_ascii_lowercase());
            } else {
                out.push(ch);
            }
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub code: Code,
    /// ID asset, `sheet#frame`, `sheet@clip` hoặc path.
    pub subject: String,
    pub message: String,
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.code.as_str(),
            self.subject,
            self.message
        )
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Kích thước và hash của file đã đọc; dùng trước khi parse hay giải mã.
pub fn verify_bytes(entry: &AssetEntry, bytes: &[u8]) -> Result<(), Issue> {
    if bytes.len() as u64 != entry.bytes {
        return Err(Issue {
            code: Code::SizeMismatch,
            subject: entry.id.clone(),
            message: format!("manifest ghi {} B, file có {} B", entry.bytes, bytes.len()),
        });
    }
    let actual = sha256_hex(bytes);
    if actual != entry.sha256 {
        return Err(Issue {
            code: Code::HashMismatch,
            subject: entry.id.clone(),
            message: format!("sha256 {actual} khác manifest {}", entry.sha256),
        });
    }
    Ok(())
}

pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, Issue> {
    parse(bytes, "manifest")
}

pub fn parse_sheet(id: &str, bytes: &[u8]) -> Result<SpriteSheet, Issue> {
    parse(bytes, id)
}

pub fn parse_layout(id: &str, bytes: &[u8]) -> Result<MapLayout, Issue> {
    parse(bytes, id)
}

fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8], subject: &str) -> Result<T, Issue> {
    serde_json::from_slice(bytes).map_err(|error| Issue {
        code: Code::Json,
        subject: subject.to_owned(),
        message: error.to_string(),
    })
}

/// Thư mục chứa `path` (phần trước `/` cuối), để nối path tương đối của manifest.
pub fn join(base_dir: &str, relative: &str) -> String {
    if base_dir.is_empty() {
        relative.to_owned()
    } else {
        format!("{}/{relative}", base_dir.trim_end_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_snake_case() {
        assert_eq!(Code::HashMismatch.as_str(), "hash_mismatch");
        assert_eq!(Code::Json.as_str(), "json");
    }

    #[test]
    fn ids_and_paths_are_strict() {
        assert!(validate::valid_id("long_luu.atlas"));
        assert!(!validate::valid_id("Long.atlas"));
        assert!(!validate::valid_id("a..b"));
        assert!(!validate::valid_id("1a"));
        assert!(validate::valid_path("dap_co/ground/atlas.png"));
        for bad in [
            "../x.png",
            "/x.png",
            "a//b.png",
            "a\\b.png",
            "A.png",
            "a/./b.png",
        ] {
            assert!(!validate::valid_path(bad), "{bad}");
        }
    }
}
