//! `myva-assets validate <manifest.json>`: kiểm toàn bộ thư mục asset, in budget theo bundle và
//! từng lỗi; thoát mã 1 khi có lỗi. Không sửa file nào.

use std::path::PathBuf;
use std::process::ExitCode;

use myva_assets::files::validate_dir;

const USAGE: &str = "usage: myva-assets validate <manifest.json>";

fn mib(bytes: u64) -> String {
    format!("{:.2} MiB", bytes as f64 / f64::from(1 << 20))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [command, path] = args.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    if command != "validate" {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let report = validate_dir(&PathBuf::from(path));
    if let Some(manifest) = &report.manifest {
        println!(
            "{} v{} · {} · {} asset, {} bundle",
            manifest.format,
            manifest.format_version,
            manifest.content_version,
            manifest.assets.len(),
            manifest.bundles.len()
        );
        println!("| Bundle | Texture RGBA8 / budget | File / budget |");
        println!("| --- | --- | --- |");
        let (mut texture_total, mut file_total) = (0, 0);
        for bundle in &manifest.bundles {
            let assets = bundle.assets.iter().filter_map(|id| manifest.asset(id));
            let (texture, file) = assets.fold((0, 0), |(t, f), a| {
                (t + a.texture.map_or(0, |i| i.decoded_bytes()), f + a.bytes)
            });
            texture_total += texture;
            file_total += file;
            println!(
                "| {} | {} / {} | {} B / {} B |",
                bundle.id,
                mib(texture),
                mib(bundle.budget.texture_bytes),
                file,
                bundle.budget.file_bytes
            );
        }
        println!(
            "| tổng | {} / {} | {} B / {} B |",
            mib(texture_total),
            mib(manifest.limits.total_texture_bytes),
            file_total,
            manifest.limits.total_file_bytes
        );
    }
    for issue in &report.issues {
        println!("{issue}");
    }
    if report.is_ok() {
        println!("PASS: 0 lỗi");
        ExitCode::SUCCESS
    } else {
        println!("FAIL: {} lỗi", report.issues.len());
        ExitCode::FAILURE
    }
}
