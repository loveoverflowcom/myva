//! Asset lỗi: mỗi test chép fixture thật vào thư mục tạm, làm hỏng đúng một chỗ và kiểm
//! validator báo đúng mã. Khi sửa nội dung JSON/PNG, hash trong manifest được cập nhật theo để lỗi
//! cần kiểm không bị che bởi `hash_mismatch`.

use std::path::{Path, PathBuf};

use myva_assets::files::{Report, decode_png, validate_dir};
use myva_assets::{
    Code, Documents, check_documents, parse_layout, parse_manifest, parse_sheet, sha256_hex,
};
use serde_json::{Value, json};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixture")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("myva-assets-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        copy_dir(&fixture(), &dir);
        Self(dir)
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.0.join(rel)
    }

    fn manifest(&self, edit: impl FnOnce(&mut Value)) {
        let path = self.path("manifest.json");
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        edit(&mut value);
        std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }

    fn asset(&self, id: &str, edit: impl FnOnce(&mut Value)) {
        self.manifest(|m| {
            let asset = m["assets"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|a| a["id"] == id)
                .unwrap();
            edit(asset);
        });
    }

    /// Ghi lại file và cập nhật bytes/sha256 trong manifest.
    fn replace(&self, rel: &str, bytes: &[u8]) {
        std::fs::write(self.path(rel), bytes).unwrap();
        let (len, hash) = (bytes.len(), sha256_hex(bytes));
        self.manifest(|m| {
            for asset in m["assets"].as_array_mut().unwrap() {
                if asset["path"] == rel {
                    asset["bytes"] = json!(len);
                    asset["sha256"] = json!(hash);
                }
            }
        });
    }

    fn json(&self, rel: &str, edit: impl FnOnce(&mut Value)) {
        let mut value: Value =
            serde_json::from_slice(&std::fs::read(self.path(rel)).unwrap()).unwrap();
        edit(&mut value);
        self.replace(rel, &serde_json::to_vec_pretty(&value).unwrap());
    }

    fn pixels(&self, rel: &str, edit: impl FnOnce(u32, &mut [u8])) {
        let image = decode_png(rel, &std::fs::read(self.path(rel)).unwrap()).unwrap();
        let mut rgba = image.rgba;
        edit(image.width, &mut rgba);
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&rgba)
            .unwrap();
        self.replace(rel, &out);
    }

    fn validate(&self) -> Report {
        validate_dir(&self.path("manifest.json"))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[track_caller]
fn expect(report: &Report, code: Code) {
    assert!(
        report.has(code),
        "cần {}, gặp: {:#?}",
        code.as_str(),
        report.issues
    );
}

#[track_caller]
fn only(report: &Report, code: Code) {
    expect(report, code);
    let other: Vec<_> = report.issues.iter().filter(|i| i.code != code).collect();
    assert!(
        other.is_empty(),
        "chỉ cần {}, gặp thêm {other:#?}",
        code.as_str()
    );
}

fn sheet_frame(rel: &str, index: usize) -> [u32; 4] {
    let sheet: Value =
        serde_json::from_slice(&std::fs::read(fixture().join(rel)).unwrap()).unwrap();
    let rect = &sheet["frames"][index]["rect"];
    [0, 1, 2, 3].map(|i| rect[i].as_u64().unwrap() as u32)
}

#[test]
fn committed_fixture_passes() {
    let report = validate_dir(&fixture().join("manifest.json"));
    assert!(report.is_ok(), "{:#?}", report.issues);
    let manifest = report.manifest.unwrap();
    assert_eq!(manifest.bundles.len(), 4);
    assert_eq!(report.documents.sheets.len(), 5);
    assert_eq!(report.documents.layouts.len(), 1);
}

#[test]
fn client_rules_run_without_files() {
    // Client tải bytes qua AssetServer rồi chạy cùng luật trên tài liệu đã parse.
    let dir = fixture();
    let manifest = parse_manifest(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
    let mut docs = Documents::default();
    for asset in &manifest.assets {
        let bytes = std::fs::read(dir.join(&asset.path)).unwrap();
        myva_assets::verify_bytes(asset, &bytes).unwrap();
        match asset.kind {
            myva_assets::AssetKind::SpriteSheet => {
                docs.sheets
                    .insert(asset.id.clone(), parse_sheet(&asset.id, &bytes).unwrap());
            }
            myva_assets::AssetKind::MapLayout => {
                docs.layouts
                    .insert(asset.id.clone(), parse_layout(&asset.id, &bytes).unwrap());
            }
            myva_assets::AssetKind::Texture => {}
        }
    }
    assert_eq!(check_documents(&manifest, &docs), []);
}

#[test]
fn edited_file_fails_hash() {
    let s = Scratch::new("hash");
    let path = s.path("long_luu/sheet.json");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.push(b'\n');
    std::fs::write(path, bytes).unwrap();
    expect(&s.validate(), Code::SizeMismatch);

    let s = Scratch::new("hash2");
    s.asset("vfx.combat_atlas", |a| a["sha256"] = json!("0".repeat(64)));
    only(&s.validate(), Code::HashMismatch);
}

#[test]
fn corrupt_png_is_rejected() {
    let s = Scratch::new("corrupt");
    let mut bytes = std::fs::read(s.path("vfx/atlas.png")).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF; // phá dữ liệu IDAT: CRC chunk sai
    s.replace("vfx/atlas.png", &bytes);
    only(&s.validate(), Code::PngInvalid);

    let s = Scratch::new("truncated");
    let bytes = std::fs::read(s.path("vfx/atlas.png")).unwrap();
    s.replace("vfx/atlas.png", &bytes[..bytes.len() - 20]);
    only(&s.validate(), Code::PngInvalid);
}

#[test]
fn missing_and_unlisted_files() {
    let s = Scratch::new("missing");
    std::fs::remove_file(s.path("vfx/atlas.png")).unwrap();
    only(&s.validate(), Code::MissingFile);

    let s = Scratch::new("unlisted");
    std::fs::write(s.path("long_luu/old.png"), b"x").unwrap();
    only(&s.validate(), Code::UnlistedFile);
}

#[test]
fn ids_paths_and_dependencies() {
    let s = Scratch::new("dup");
    s.asset("vfx.combat", |a| a["id"] = json!("vfx.combat_atlas"));
    expect(&s.validate(), Code::DuplicateId);

    let s = Scratch::new("traversal");
    s.asset("vfx.combat", |a| a["path"] = json!("../manifest.json"));
    expect(&s.validate(), Code::BadPath);

    let s = Scratch::new("unknown-dep");
    s.asset("dap_co.layout", |a| {
        a["dependencies"] = json!(["dap_co.ground", "dap_co.nope"])
    });
    expect(&s.validate(), Code::UnknownDependency);

    let s = Scratch::new("cycle");
    s.asset("long_luu.atlas", |a| {
        a["dependencies"] = json!(["long_luu.sheet"])
    });
    let report = s.validate();
    expect(&report, Code::DependencyCycle);
    expect(&report, Code::DependencyKind);
}

#[test]
fn texture_size_and_limits() {
    let s = Scratch::new("dims");
    s.asset("vfx.combat_atlas", |a| a["texture"]["width"] = json!(512));
    expect(&s.validate(), Code::TextureDims);

    let s = Scratch::new("side");
    s.manifest(|m| m["limits"]["max_texture_side"] = json!(1024));
    expect(&s.validate(), Code::TextureTooLarge);

    let s = Scratch::new("policy");
    s.manifest(|m| m["limits"]["total_texture_bytes"] = json!(1u64 << 30));
    only(&s.validate(), Code::Budget);
}

#[test]
fn bundle_budget_and_ownership() {
    let s = Scratch::new("budget");
    s.manifest(|m| m["bundles"][0]["budget"]["texture_bytes"] = json!(1 << 20));
    only(&s.validate(), Code::Budget);

    let s = Scratch::new("orphan");
    s.manifest(|m| {
        let assets = m["bundles"][2]["assets"].as_array_mut().unwrap();
        assets.retain(|a| a != "vfx.combat");
    });
    only(&s.validate(), Code::BundleOrphanAsset);

    let s = Scratch::new("cross");
    s.manifest(|m| {
        let bundles = m["bundles"].as_array_mut().unwrap();
        let map = bundles
            .iter_mut()
            .find(|b| b["id"] == "map.dap_co")
            .unwrap();
        map["assets"]
            .as_array_mut()
            .unwrap()
            .retain(|a| a != "dap_co.ground");
        let vfx = bundles
            .iter_mut()
            .find(|b| b["id"] == "vfx.combat")
            .unwrap();
        vfx["assets"]
            .as_array_mut()
            .unwrap()
            .push(json!("dap_co.ground"));
    });
    // Sheet nền chuyển sang bundle VFX: layout và texture của nó tham chiếu chéo bundle.
    only(&s.validate(), Code::BundleCrossReference);

    let s = Scratch::new("declared");
    s.manifest(|m| {
        let bundles = m["bundles"].as_array_mut().unwrap();
        let map = bundles
            .iter_mut()
            .find(|b| b["id"] == "map.dap_co")
            .unwrap();
        map["assets"]
            .as_array_mut()
            .unwrap()
            .retain(|a| a != "dap_co.ground_atlas");
        map["dependencies"] = json!(["vfx.combat"]);
        let vfx = bundles
            .iter_mut()
            .find(|b| b["id"] == "vfx.combat")
            .unwrap();
        vfx["assets"]
            .as_array_mut()
            .unwrap()
            .push(json!("dap_co.ground_atlas"));
    });
    assert!(
        s.validate().is_ok(),
        "tham chiếu tới bundle đã khai báo là hợp lệ"
    );
}

#[test]
fn provenance_and_license() {
    let s = Scratch::new("license");
    s.asset("long_luu.atlas", |a| {
        a["provenance"]["license"] = json!("UNKNOWN")
    });
    only(&s.validate(), Code::License);

    let s = Scratch::new("author");
    s.asset("long_luu.atlas", |a| a["provenance"]["author"] = json!(" "));
    only(&s.validate(), Code::Provenance);

    let s = Scratch::new("source");
    s.asset("long_luu.atlas", |a| {
        a["provenance"]["origin"] = json!("third_party")
    });
    only(&s.validate(), Code::Provenance);
}

#[test]
fn schema_version_and_unknown_fields() {
    let s = Scratch::new("version");
    s.manifest(|m| m["format_version"] = json!(2));
    only(&s.validate(), Code::Format);

    let s = Scratch::new("field");
    s.json("vfx/sheet.json", |v| v["mipmaps"] = json!(true));
    only(&s.validate(), Code::Json);
}

#[test]
fn frame_rects_and_padding() {
    let s = Scratch::new("bounds");
    s.json("vfx/sheet.json", |v| {
        v["frames"][0]["rect"][0] = json!(5000)
    });
    expect(&s.validate(), Code::FrameBounds);

    let s = Scratch::new("overlap");
    s.json("vfx/sheet.json", |v| {
        v["frames"][1]["rect"] = v["frames"][0]["rect"].clone()
    });
    expect(&s.validate(), Code::FrameOverlap);

    let s = Scratch::new("padding");
    s.json("vfx/sheet.json", |v| v["padding"] = json!(40));
    expect(&s.validate(), Code::Padding);
}

#[test]
fn pivots_are_inside_and_consistent() {
    let s = Scratch::new("pivot");
    s.json("vfx/sheet.json", |v| {
        v["frames"][0]["pivot"] = json!([500, 0])
    });
    only(&s.validate(), Code::Pivot);

    let s = Scratch::new("jitter");
    s.json("long_luu/sheet.json", |v| {
        v["frames"][3]["pivot"][1] = json!(240)
    });
    only(&s.validate(), Code::PivotInconsistent);
}

#[test]
fn clips_and_combat_markers() {
    let s = Scratch::new("no-jump");
    s.json("long_luu/sheet.json", |v| {
        v["clips"]
            .as_array_mut()
            .unwrap()
            .retain(|c| c["name"] != "jump");
    });
    only(&s.validate(), Code::ClipMissing);

    let s = Scratch::new("no-hit");
    s.json("ke_giu_dap/sheet.json", |v| {
        let attack = v["clips"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["name"] == "attack")
            .unwrap();
        attack["events"] = json!([]);
    });
    only(&s.validate(), Code::ClipEvent);

    let s = Scratch::new("hit-first");
    s.json("long_luu/sheet.json", |v| {
        let attack = v["clips"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["name"] == "attack")
            .unwrap();
        attack["events"][0]["frame"] = json!(0);
    });
    only(&s.validate(), Code::ClipEvent);

    let s = Scratch::new("ghost-frame");
    s.json("vfx/sheet.json", |v| {
        v["clips"][0]["frames"][0] = json!("hit.9")
    });
    only(&s.validate(), Code::ClipFrame);

    let s = Scratch::new("timing");
    s.json("vfx/sheet.json", |v| v["clips"][0]["frame_ms"] = json!(0));
    only(&s.validate(), Code::ClipTiming);
}

#[test]
fn scale_and_layout_references() {
    let s = Scratch::new("scale");
    s.json("long_luu/sheet.json", |v| v["texels_per_unit"] = json!(1.0));
    only(&s.validate(), Code::Scale);

    let s = Scratch::new("layout-frame");
    s.json("dap_co/layout.json", |v| {
        v["layers"][1]["frame"] = json!("mountains")
    });
    only(&s.validate(), Code::LayoutRef);

    let s = Scratch::new("layout-z");
    s.json("dap_co/layout.json", |v| v["layers"][3]["z"] = json!(5));
    only(&s.validate(), Code::LayoutRef);
}

#[test]
fn art_touching_the_cell_edge_is_clipped() {
    let s = Scratch::new("clipped");
    let [x, y, w, _] = sheet_frame("long_luu/sheet.json", 0);
    s.pixels("long_luu/atlas.png", |width, rgba| {
        let i = ((y * width + x + w / 2) * 4) as usize;
        rgba[i..i + 4].copy_from_slice(&[255, 0, 0, 255]);
    });
    only(&s.validate(), Code::FrameClipped);
}

#[test]
fn broken_tile_extrude_is_detected() {
    let s = Scratch::new("extrude");
    let [x, y, ..] = sheet_frame("dap_co/ground/sheet.json", 0);
    s.pixels("dap_co/ground/atlas.png", |width, rgba| {
        let i = (((y - 1) * width + x + 10) * 4) as usize;
        rgba[i..i + 4].copy_from_slice(&[255, 0, 255, 255]);
    });
    only(&s.validate(), Code::Extrude);
}
