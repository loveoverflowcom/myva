//! Kiểm một thư mục asset trên đĩa: file có mặt, kích thước, hash, PNG giải mã được và đúng
//! kích thước, viền frame trong suốt (không bị cắt), extrude của tile, file thừa.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::schema::*;
use crate::validate::{Documents, check_documents, push};
use crate::{Code, Issue, parse_layout, parse_manifest, parse_sheet, verify_bytes};

/// File được phép có trong thư mục mà không cần mục manifest.
const UNLISTED_ALLOWED: [&str; 2] = ["manifest.json", "README.md"];

pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Pixels {
    fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }
}

#[derive(Default)]
pub struct Report {
    pub manifest: Option<Manifest>,
    pub documents: Documents,
    pub issues: Vec<Issue>,
}

impl Report {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn has(&self, code: Code) -> bool {
        self.issues.iter().any(|issue| issue.code == code)
    }
}

/// Giải mã PNG RGBA8 không interlace; CRC từng chunk được kiểm.
pub fn decode_png(id: &str, bytes: &[u8]) -> Result<Pixels, Issue> {
    let fail = |code: Code, message: String| Issue {
        code,
        subject: id.to_owned(),
        message,
    };
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|e| fail(Code::PngInvalid, e.to_string()))?;
    let info = reader.info();
    if info.color_type != png::ColorType::Rgba
        || info.bit_depth != png::BitDepth::Eight
        || info.interlaced
    {
        return Err(fail(
            Code::TextureFormat,
            format!(
                "cần RGBA 8 bit không interlace, gặp {:?} {:?}",
                info.color_type, info.bit_depth
            ),
        ));
    }
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| fail(Code::PngInvalid, "kích thước ảnh quá lớn".to_owned()))?;
    let mut rgba = vec![0; size];
    let frame = reader
        .next_frame(&mut rgba)
        .map_err(|e| fail(Code::PngInvalid, e.to_string()))?;
    // Đọc tới IEND để phát hiện file bị cắt sau dữ liệu ảnh.
    reader
        .finish()
        .map_err(|e| fail(Code::PngInvalid, e.to_string()))?;
    rgba.truncate(frame.buffer_size());
    Ok(Pixels {
        width: frame.width,
        height: frame.height,
        rgba,
    })
}

pub fn validate_dir(manifest_path: &Path) -> Report {
    let mut report = Report::default();
    let root = manifest_path.parent().unwrap_or(Path::new("."));
    let bytes = match std::fs::read(manifest_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            push(
                &mut report.issues,
                Code::MissingFile,
                &manifest_path.display().to_string(),
                error.to_string(),
            );
            return report;
        }
    };
    let manifest = match parse_manifest(&bytes) {
        Ok(manifest) => manifest,
        Err(issue) => {
            report.issues.push(issue);
            return report;
        }
    };

    let mut pixels = BTreeMap::new();
    for asset in &manifest.assets {
        if !crate::validate::valid_path(&asset.path) {
            continue; // check_manifest báo; không đọc path ngoài thư mục.
        }
        let data = match std::fs::read(root.join(&asset.path)) {
            Ok(data) => data,
            Err(error) => {
                push(
                    &mut report.issues,
                    Code::MissingFile,
                    &asset.id,
                    format!("{}: {error}", asset.path),
                );
                continue;
            }
        };
        if let Err(issue) = verify_bytes(asset, &data) {
            report.issues.push(issue);
        }
        let parsed = match asset.kind {
            AssetKind::Texture => decode_png(&asset.id, &data).map(|image| {
                if let Some(info) = asset.texture
                    && (info.width, info.height) != (image.width, image.height)
                {
                    push(
                        &mut report.issues,
                        Code::TextureDims,
                        &asset.id,
                        format!(
                            "manifest ghi {}x{}, PNG là {}x{}",
                            info.width, info.height, image.width, image.height
                        ),
                    );
                }
                pixels.insert(asset.id.clone(), image);
            }),
            AssetKind::SpriteSheet => parse_sheet(&asset.id, &data).map(|sheet| {
                report.documents.sheets.insert(asset.id.clone(), sheet);
            }),
            AssetKind::MapLayout => parse_layout(&asset.id, &data).map(|layout| {
                report.documents.layouts.insert(asset.id.clone(), layout);
            }),
        };
        if let Err(issue) = parsed {
            report.issues.push(issue);
        }
    }

    report
        .issues
        .extend(check_documents(&manifest, &report.documents));
    for (id, sheet) in &report.documents.sheets {
        if let Some(image) = pixels.get(&sheet.texture) {
            check_pixels(id, sheet, image, &mut report.issues);
        }
    }
    check_unlisted(root, &manifest, &mut report.issues);
    report.manifest = Some(manifest);
    report
}

/// Sprite có viền trong suốt 1 texel trong ô: hình chạm mép ô gần như chắc chắn bị cắt.
/// Tile/backdrop có extrude: vòng texel ngoài frame phải lặp lại texel biên.
fn check_pixels(id: &str, sheet: &SpriteSheet, image: &Pixels, issues: &mut Vec<Issue>) {
    for frame in &sheet.frames {
        let [x, y, w, h] = frame.rect;
        let e = sheet.extrude;
        if w == 0 || h == 0 || x < e || y < e || x + w + e > image.width || y + h + e > image.height
        {
            continue; // check_sheet đã báo frame_bounds/padding.
        }
        let subject = format!("{id}#{}", frame.name);
        if sheet.role.has_transparent_border() {
            let edges = [
                ("trên", (x..x + w).map(|i| (i, y)).collect::<Vec<_>>()),
                ("dưới", (x..x + w).map(|i| (i, y + h - 1)).collect()),
                ("trái", (y..y + h).map(|j| (x, j)).collect()),
                ("phải", (y..y + h).map(|j| (x + w - 1, j)).collect()),
            ];
            let touched: Vec<_> = edges
                .iter()
                .filter(|(_, points)| points.iter().any(|&(px, py)| image.at(px, py)[3] != 0))
                .map(|(side, _)| *side)
                .collect();
            if !touched.is_empty() {
                push(
                    issues,
                    Code::FrameClipped,
                    &subject,
                    format!("hình chạm mép ô ({}): có thể bị cắt", touched.join(", ")),
                );
            }
        }
        if e > 0 {
            let mut bad = 0u32;
            for j in y - e..y + h + e {
                for i in x - e..x + w + e {
                    let inside = (x..x + w).contains(&i) && (y..y + h).contains(&j);
                    if !inside {
                        let src = image.at(i.clamp(x, x + w - 1), j.clamp(y, y + h - 1));
                        bad += u32::from(image.at(i, j) != src);
                    }
                }
            }
            if bad > 0 {
                push(
                    issues,
                    Code::Extrude,
                    &subject,
                    format!("{bad} texel extrude không lặp lại biên frame"),
                );
            }
        }
    }
}

fn check_unlisted(root: &Path, manifest: &Manifest, issues: &mut Vec<Issue>) {
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let relative = relative.to_string_lossy().replace('\\', "/");
            if UNLISTED_ALLOWED.contains(&relative.as_str())
                || manifest.assets.iter().any(|a| a.path == relative)
            {
                continue;
            }
            push(
                issues,
                Code::UnlistedFile,
                &relative,
                "file không có trong manifest",
            );
        }
    }
}
