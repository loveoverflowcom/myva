//! Luật không cần đọc file: ID, path, dependency, bundle, budget, provenance, frame/pivot/clip và
//! tham chiếu bản đồ. Client chạy cùng bộ luật này khi tải manifest.

use std::collections::{BTreeMap, BTreeSet};

use crate::schema::*;
use crate::{ALLOWED_LICENSES, Code, Issue, MAX_TEXTURE_SIDE, MAX_TOTAL_TEXTURE_BYTES};

/// Sheet/layout đã parse, theo ID asset.
#[derive(Clone, Debug, Default)]
pub struct Documents {
    pub sheets: BTreeMap<String, SpriteSheet>,
    pub layouts: BTreeMap<String, MapLayout>,
}

pub(crate) fn push(issues: &mut Vec<Issue>, code: Code, subject: &str, message: impl Into<String>) {
    issues.push(Issue {
        code,
        subject: subject.to_owned(),
        message: message.into(),
    });
}

/// ID: chữ thường ASCII, số, `_`, phân đoạn bằng `.`; không rỗng.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
        && id.as_bytes()[0].is_ascii_lowercase()
}

/// Path tương đối an toàn: ASCII thường, không `..`, không tuyệt đối, không `\`.
pub fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'.')
                })
        })
}

pub fn check_manifest(manifest: &Manifest, issues: &mut Vec<Issue>) {
    if manifest.format != MANIFEST_FORMAT || manifest.format_version != FORMAT_VERSION {
        push(
            issues,
            Code::Format,
            "manifest",
            format!(
                "cần {MANIFEST_FORMAT} v{FORMAT_VERSION}, gặp {} v{}",
                manifest.format, manifest.format_version
            ),
        );
    }
    if manifest.content_version.trim().is_empty() {
        push(issues, Code::Format, "manifest", "content_version rỗng");
    }
    let tpu = manifest.texels_per_unit;
    if !(tpu.gameplay > 0.0 && tpu.backdrop > 0.0) {
        push(
            issues,
            Code::Scale,
            "manifest",
            "texels_per_unit phải dương",
        );
    }
    let limits = manifest.limits;
    if limits.max_texture_side > MAX_TEXTURE_SIDE
        || limits.total_texture_bytes > MAX_TOTAL_TEXTURE_BYTES
    {
        push(
            issues,
            Code::Budget,
            "manifest",
            format!(
                "limits vượt trần chính sách: cạnh ≤ {MAX_TEXTURE_SIDE}, texture ≤ {MAX_TOTAL_TEXTURE_BYTES} B"
            ),
        );
    }

    let mut seen = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for asset in &manifest.assets {
        let id = asset.id.as_str();
        if !valid_id(id) {
            push(
                issues,
                Code::BadId,
                id,
                "ID chỉ gồm a-z, 0-9, _ và dấu chấm",
            );
        }
        if !seen.insert(id) {
            push(issues, Code::DuplicateId, id, "ID asset trùng");
        }
        if !valid_path(&asset.path) || !asset.path.ends_with(asset.kind.extension()) {
            push(
                issues,
                Code::BadPath,
                id,
                format!(
                    "path `{}` phải tương đối, ASCII thường, đuôi {}",
                    asset.path,
                    asset.kind.extension()
                ),
            );
        }
        if !paths.insert(asset.path.as_str()) {
            push(issues, Code::BadPath, id, "hai asset cùng path");
        }
        if asset.version == 0 {
            push(issues, Code::Format, id, "version bắt đầu từ 1");
        }
        if asset.sha256.len() != 64
            || !asset
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            push(
                issues,
                Code::Format,
                id,
                "sha256 phải là 64 ký tự hex thường",
            );
        }
        match (asset.kind, asset.texture) {
            (AssetKind::Texture, None) => {
                push(
                    issues,
                    Code::Format,
                    id,
                    "texture thiếu width/height/format",
                );
            }
            (AssetKind::Texture, Some(info)) => {
                if info.width == 0 || info.height == 0 {
                    push(issues, Code::TextureDims, id, "kích thước texture bằng 0");
                }
                if info.width.max(info.height) > limits.max_texture_side {
                    push(
                        issues,
                        Code::TextureTooLarge,
                        id,
                        format!(
                            "{}x{} vượt cạnh tối đa {}",
                            info.width, info.height, limits.max_texture_side
                        ),
                    );
                }
            }
            (_, Some(_)) => push(issues, Code::Format, id, "chỉ texture có khối `texture`"),
            _ => {}
        }
        check_provenance(id, &asset.provenance, issues);
    }

    check_dependencies(manifest, issues);
    check_bundles(manifest, issues);
}

fn check_provenance(id: &str, p: &Provenance, issues: &mut Vec<Issue>) {
    for (field, value) in [
        ("author", &p.author),
        ("redistribution", &p.redistribution),
        ("created", &p.created),
    ] {
        if value.trim().is_empty() {
            push(
                issues,
                Code::Provenance,
                id,
                format!("provenance thiếu {field}"),
            );
        }
    }
    let missing = match p.origin {
        Origin::Procedural => p.tool.as_deref().is_none_or(|t| t.trim().is_empty()),
        Origin::ThirdParty => p.source.as_deref().is_none_or(|s| s.trim().is_empty()),
        Origin::Authored => false,
    };
    if missing {
        push(
            issues,
            Code::Provenance,
            id,
            "asset sinh bằng công cụ cần `tool`, asset bên thứ ba cần `source`",
        );
    }
    if !ALLOWED_LICENSES.contains(&p.license.as_str()) {
        push(
            issues,
            Code::License,
            id,
            format!(
                "license `{}` chưa được duyệt (cho phép: {})",
                p.license,
                ALLOWED_LICENSES.join(", ")
            ),
        );
    }
}

fn check_dependencies(manifest: &Manifest, issues: &mut Vec<Issue>) {
    for asset in &manifest.assets {
        let deps: Vec<_> = asset
            .dependencies
            .iter()
            .map(|dep| (dep, manifest.asset(dep)))
            .collect();
        for (dep, found) in &deps {
            if found.is_none() {
                push(
                    issues,
                    Code::UnknownDependency,
                    &asset.id,
                    format!("dependency `{dep}` không có trong manifest"),
                );
            }
        }
        let kinds: Vec<_> = deps
            .iter()
            .filter_map(|(_, found)| found.map(|a| a.kind))
            .collect();
        let ok = match asset.kind {
            AssetKind::Texture => kinds.is_empty(),
            AssetKind::SpriteSheet => kinds == [AssetKind::Texture],
            AssetKind::MapLayout => {
                !kinds.is_empty() && kinds.iter().all(|k| *k == AssetKind::SpriteSheet)
            }
        };
        if !ok {
            push(
                issues,
                Code::DependencyKind,
                &asset.id,
                "texture không phụ thuộc gì; sheet phụ thuộc đúng một texture; layout chỉ phụ thuộc sheet",
            );
        }
    }
    // DFS ba màu tìm chu trình.
    let mut state: BTreeMap<&str, u8> = BTreeMap::new();
    fn visit<'a>(
        manifest: &'a Manifest,
        id: &'a str,
        state: &mut BTreeMap<&'a str, u8>,
        issues: &mut Vec<Issue>,
    ) {
        match state.get(id) {
            Some(1) => {
                push(
                    issues,
                    Code::DependencyCycle,
                    id,
                    "dependency tạo chu trình",
                );
                return;
            }
            Some(_) => return,
            None => {}
        }
        state.insert(id, 1);
        if let Some(asset) = manifest.asset(id) {
            for dep in &asset.dependencies {
                visit(manifest, dep, state, issues);
            }
        }
        state.insert(id, 2);
    }
    for asset in &manifest.assets {
        visit(manifest, &asset.id, &mut state, issues);
    }
}

fn check_bundles(manifest: &Manifest, issues: &mut Vec<Issue>) {
    let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let (mut texture_total, mut file_total) = (0u64, 0u64);
    for bundle in &manifest.bundles {
        if !valid_id(&bundle.id) || !ids.insert(bundle.id.as_str()) {
            push(
                issues,
                Code::BadId,
                &bundle.id,
                "ID bundle không hợp lệ hoặc trùng",
            );
        }
        for dep in &bundle.dependencies {
            if !manifest.bundles.iter().any(|b| &b.id == dep) || dep == &bundle.id {
                push(
                    issues,
                    Code::UnknownDependency,
                    &bundle.id,
                    format!("bundle phụ thuộc `{dep}` không hợp lệ"),
                );
            }
        }
        let (mut texture_bytes, mut file_bytes) = (0u64, 0u64);
        for id in &bundle.assets {
            let Some(asset) = manifest.asset(id) else {
                push(
                    issues,
                    Code::BundleUnknownAsset,
                    &bundle.id,
                    format!("bundle liệt kê `{id}` không có trong manifest"),
                );
                continue;
            };
            if let Some(previous) = owner.insert(id, &bundle.id) {
                push(
                    issues,
                    Code::BundleDuplicateAsset,
                    id,
                    format!("thuộc cả `{previous}` và `{}`", bundle.id),
                );
            }
            file_bytes += asset.bytes;
            texture_bytes += asset.texture.map_or(0, |t| t.decoded_bytes());
        }
        if texture_bytes > bundle.budget.texture_bytes || file_bytes > bundle.budget.file_bytes {
            push(
                issues,
                Code::Budget,
                &bundle.id,
                format!(
                    "texture {texture_bytes}/{} B, file {file_bytes}/{} B",
                    bundle.budget.texture_bytes, bundle.budget.file_bytes
                ),
            );
        }
        texture_total += texture_bytes;
        file_total += file_bytes;
    }
    let limits = manifest.limits;
    if texture_total > limits.total_texture_bytes || file_total > limits.total_file_bytes {
        push(
            issues,
            Code::Budget,
            "manifest",
            format!(
                "tổng texture {texture_total}/{} B, file {file_total}/{} B",
                limits.total_texture_bytes, limits.total_file_bytes
            ),
        );
    }
    for asset in &manifest.assets {
        let Some(home) = owner.get(asset.id.as_str()) else {
            push(
                issues,
                Code::BundleOrphanAsset,
                &asset.id,
                "asset không thuộc bundle nào",
            );
            continue;
        };
        let allowed = manifest
            .bundles
            .iter()
            .find(|b| b.id == *home)
            .map(|b| &b.dependencies);
        for dep in &asset.dependencies {
            match owner.get(dep.as_str()) {
                Some(other)
                    if other != home
                        && !allowed.is_some_and(|list| list.iter().any(|b| b == other)) =>
                {
                    push(
                        issues,
                        Code::BundleCrossReference,
                        &asset.id,
                        format!(
                            "tham chiếu `{dep}` ở bundle `{other}` mà `{home}` không khai báo phụ thuộc"
                        ),
                    );
                }
                _ => {}
            }
        }
    }
}

/// Luật của một sheet đối chiếu với mục manifest và texture nó trỏ tới.
pub fn check_sheet(
    manifest: &Manifest,
    entry: &AssetEntry,
    sheet: &SpriteSheet,
    issues: &mut Vec<Issue>,
) {
    let id = entry.id.as_str();
    if sheet.format != SHEET_FORMAT || sheet.format_version != FORMAT_VERSION {
        push(
            issues,
            Code::Format,
            id,
            format!("cần {SHEET_FORMAT} v{FORMAT_VERSION}"),
        );
    }
    if sheet.id != entry.id {
        push(
            issues,
            Code::SheetTexture,
            id,
            format!("sheet ghi id `{}`", sheet.id),
        );
    }
    if entry.dependencies != [sheet.texture.clone()] {
        push(
            issues,
            Code::SheetTexture,
            id,
            format!(
                "texture `{}` phải là dependency duy nhất của sheet",
                sheet.texture
            ),
        );
    }
    let texture = manifest.asset(&sheet.texture).and_then(|t| t.texture);
    if sheet.role.plane() != sheet.plane
        || sheet.texels_per_unit != manifest.texels_per_unit.of(sheet.plane)
    {
        push(
            issues,
            Code::Scale,
            id,
            format!(
                "role {:?} cần plane {:?} với {} texel/px",
                sheet.role,
                sheet.role.plane(),
                manifest.texels_per_unit.of(sheet.role.plane())
            ),
        );
    }
    if matches!(sheet.role, Role::Character | Role::Boss)
        && sheet.subject.as_deref().is_none_or(str::is_empty)
    {
        push(
            issues,
            Code::Format,
            id,
            "sheet nhân vật/boss cần `subject` là kit id của lõi",
        );
    }
    if sheet.extrude * 2 > sheet.padding || sheet.padding == 0 {
        push(
            issues,
            Code::Padding,
            id,
            format!(
                "padding {} phải > 0 và ≥ 2 × extrude {}",
                sheet.padding, sheet.extrude
            ),
        );
    }

    let mut names = BTreeSet::new();
    for frame in &sheet.frames {
        let subject = format!("{id}#{}", frame.name);
        if !names.insert(frame.name.as_str()) {
            push(issues, Code::DuplicateId, &subject, "tên frame trùng");
        }
        let [x, y, w, h] = frame.rect;
        if w == 0 || h == 0 {
            push(issues, Code::FrameBounds, &subject, "frame rỗng");
        }
        if let Some(t) = texture {
            let pad = sheet.padding;
            if x.saturating_add(w) > t.width || y.saturating_add(h) > t.height {
                push(
                    issues,
                    Code::FrameBounds,
                    &subject,
                    format!(
                        "rect {:?} ra ngoài texture {}x{}",
                        frame.rect, t.width, t.height
                    ),
                );
            } else if x < pad || y < pad || x + w + pad > t.width || y + h + pad > t.height {
                push(
                    issues,
                    Code::Padding,
                    &subject,
                    format!("cách mép atlas < {pad} texel"),
                );
            }
        }
        if frame.pivot[0] > w || frame.pivot[1] > h {
            push(
                issues,
                Code::Pivot,
                &subject,
                format!("pivot {:?} nằm ngoài frame {w}x{h}", frame.pivot),
            );
        }
    }
    for (i, a) in sheet.frames.iter().enumerate() {
        for b in &sheet.frames[i + 1..] {
            let gap_x = gap(a.rect[0], a.rect[2], b.rect[0], b.rect[2]);
            let gap_y = gap(a.rect[1], a.rect[3], b.rect[1], b.rect[3]);
            let subject = format!("{id}#{}", b.name);
            if gap_x < 0 && gap_y < 0 {
                push(
                    issues,
                    Code::FrameOverlap,
                    &subject,
                    format!("chồng lên `{}`", a.name),
                );
            } else if gap_x.max(gap_y) < i64::from(sheet.padding) {
                push(
                    issues,
                    Code::Padding,
                    &subject,
                    format!("cách `{}` < {} texel", a.name, sheet.padding),
                );
            }
        }
    }
    // Nhân vật/boss: mọi frame cùng kích thước và pivot để animation không rung.
    if matches!(sheet.role, Role::Character | Role::Boss)
        && let Some(first) = sheet.frames.first()
    {
        for frame in &sheet.frames[1..] {
            if frame.rect[2..] != first.rect[2..] || frame.pivot != first.pivot {
                push(
                    issues,
                    Code::PivotInconsistent,
                    &format!("{id}#{}", frame.name),
                    format!(
                        "kích thước/pivot khác `{}` ({:?} {:?})",
                        first.name,
                        &first.rect[2..],
                        first.pivot
                    ),
                );
            }
        }
    }
    check_clips(id, sheet, issues);
}

fn gap(a0: u32, aw: u32, b0: u32, bw: u32) -> i64 {
    let (a0, a1, b0, b1) = (
        i64::from(a0),
        i64::from(a0) + i64::from(aw),
        i64::from(b0),
        i64::from(b0) + i64::from(bw),
    );
    (b0 - a1).max(a0 - b1)
}

/// Clip bắt buộc theo role, mốc combat và tên mốc được hiểu.
pub fn required_clips(role: Role) -> &'static [(&'static str, Option<&'static str>)] {
    match role {
        Role::Character => &[
            ("idle", None),
            ("run", None),
            ("jump", Some("apex")),
            ("attack", Some("hit")),
        ],
        Role::Boss => &[("idle", None), ("run", None), ("attack", Some("hit"))],
        Role::Vfx | Role::Tileset | Role::Backdrop => &[],
    }
}

const EVENTS: [&str; 2] = ["hit", "apex"];
const MIN_FRAME_MS: u32 = 16;
const MAX_FRAME_MS: u32 = 1_000;

fn check_clips(id: &str, sheet: &SpriteSheet, issues: &mut Vec<Issue>) {
    let mut names = BTreeSet::new();
    for clip in &sheet.clips {
        let subject = format!("{id}@{}", clip.name);
        if !names.insert(clip.name.as_str()) {
            push(issues, Code::DuplicateId, &subject, "tên clip trùng");
        }
        if clip.frames.is_empty() {
            push(issues, Code::ClipFrame, &subject, "clip rỗng");
        }
        for name in &clip.frames {
            if sheet.frame(name).is_none() {
                push(
                    issues,
                    Code::ClipFrame,
                    &subject,
                    format!("frame `{name}` không có"),
                );
            }
        }
        if !(MIN_FRAME_MS..=MAX_FRAME_MS).contains(&clip.frame_ms) {
            push(
                issues,
                Code::ClipTiming,
                &subject,
                format!(
                    "frame_ms {} ngoài {MIN_FRAME_MS}..={MAX_FRAME_MS}",
                    clip.frame_ms
                ),
            );
        }
        let mut events = BTreeSet::new();
        for event in &clip.events {
            if !EVENTS.contains(&event.name.as_str()) || !events.insert(event.name.as_str()) {
                push(
                    issues,
                    Code::ClipEvent,
                    &subject,
                    format!("mốc `{}` lạ hoặc lặp", event.name),
                );
            }
            if event.frame >= clip.frames.len() {
                push(
                    issues,
                    Code::ClipEvent,
                    &subject,
                    format!("mốc `{}` ở frame {} ngoài clip", event.name, event.frame),
                );
            }
        }
    }
    for (name, marker) in required_clips(sheet.role) {
        let subject = format!("{id}@{name}");
        let Some(clip) = sheet.clip(name) else {
            push(
                issues,
                Code::ClipMissing,
                &subject,
                format!("{:?} cần clip `{name}`", sheet.role),
            );
            continue;
        };
        let Some(marker) = marker else { continue };
        match clip.event(marker) {
            // Phải có ít nhất một frame chuẩn bị trước mốc trúng để báo đòn đọc được.
            Some(0) if *marker == "hit" => push(
                issues,
                Code::ClipEvent,
                &subject,
                "mốc `hit` ở frame 0: không còn frame báo đòn",
            ),
            Some(_) => {}
            None => push(
                issues,
                Code::ClipEvent,
                &subject,
                format!("thiếu mốc `{marker}`"),
            ),
        }
    }
    if sheet.role == Role::Vfx && sheet.clips.is_empty() {
        push(
            issues,
            Code::ClipMissing,
            id,
            "sheet VFX cần ít nhất một clip",
        );
    }
}

pub fn check_layout(
    manifest: &Manifest,
    entry: &AssetEntry,
    layout: &MapLayout,
    docs: &Documents,
    issues: &mut Vec<Issue>,
) {
    let id = entry.id.as_str();
    if layout.format != LAYOUT_FORMAT || layout.format_version != FORMAT_VERSION {
        push(
            issues,
            Code::Format,
            id,
            format!("cần {LAYOUT_FORMAT} v{FORMAT_VERSION}"),
        );
    }
    if layout.id != entry.id {
        push(
            issues,
            Code::LayoutRef,
            id,
            format!("layout ghi id `{}`", layout.id),
        );
    }
    let mut refer = |sheet: &str, frame: &str, role: Role| {
        if !entry.dependencies.iter().any(|d| d == sheet) {
            push(
                issues,
                Code::LayoutRef,
                id,
                format!("sheet `{sheet}` không là dependency"),
            );
            return;
        }
        match docs.sheets.get(sheet) {
            Some(s) if s.role != role => push(
                issues,
                Code::LayoutRef,
                id,
                format!("sheet `{sheet}` cần role {role:?}"),
            ),
            Some(s) if s.frame(frame).is_none() => push(
                issues,
                Code::LayoutRef,
                id,
                format!("frame `{frame}` không có trong `{sheet}`"),
            ),
            Some(_) => {}
            None if manifest.asset(sheet).is_some() => {}
            None => push(
                issues,
                Code::LayoutRef,
                id,
                format!("thiếu sheet `{sheet}`"),
            ),
        }
    };
    let ground = &layout.ground;
    refer(&ground.sheet, &ground.top, Role::Tileset);
    refer(&ground.sheet, &ground.fill, Role::Tileset);
    for layer in &layout.layers {
        refer(&layer.sheet, &layer.frame, Role::Backdrop);
    }
    // Phông luôn sau mặt phẳng gameplay: không che vùng báo đòn hay nhân vật.
    for layer in &layout.layers {
        if layer.z >= 0 {
            push(
                issues,
                Code::LayoutRef,
                id,
                format!("lớp `{}` có z {} ≥ 0 sẽ che gameplay", layer.frame, layer.z),
            );
        }
    }
}

/// Toàn bộ luật không cần file, trên manifest và tài liệu đã parse.
pub fn check_documents(manifest: &Manifest, docs: &Documents) -> Vec<Issue> {
    let mut issues = Vec::new();
    check_manifest(manifest, &mut issues);
    for asset in &manifest.assets {
        match asset.kind {
            AssetKind::SpriteSheet => {
                if let Some(sheet) = docs.sheets.get(&asset.id) {
                    check_sheet(manifest, asset, sheet, &mut issues);
                }
            }
            AssetKind::MapLayout => {
                if let Some(layout) = docs.layouts.get(&asset.id) {
                    check_layout(manifest, asset, layout, docs, &mut issues);
                }
            }
            AssetKind::Texture => {}
        }
    }
    issues
}
