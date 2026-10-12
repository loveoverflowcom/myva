//! Bản native của graybox Bevy: `cargo run -p myva-graybox`.
//!
//! Hai chế độ: đánh boss Kẻ Giữ Đập (mặc định) và đấu tập với bot B0 (`--duel`). Phím theo
//! combat.md §12; thêm Enter đấu lại, M đổi chế độ, B bật/tắt bot đấu tập, H hitbox. Riêng bản
//! native: F9 lưu replay vào thư mục làm việc, Esc thoát.
//!
//! `--demo` để bot lái người chơi (B1 khi đánh boss, B0 khi đấu tập);
//! `--screenshot <file.png>` chạy demo tới lúc boss đang báo đòn, lưu ảnh màn hình cùng replay
//! rồi thoát; `--no-art` giữ cảnh hình khối, không tải fixture art (phím G bật/tắt khi chạy).
//! `--benchmark 50|200|500 --benchmark-tier High|Low --benchmark-seed 7` thêm sprite render-only;
//! `--benchmark-output <file.json> --benchmark-seconds 60` ghi acquisition thô sau 5 s warmup
//! rồi thoát; Bevy Update không là presented FPS. Capture cần boss + art và không dùng screenshot.
//! Asset đọc từ `assets/` của repo; đặt `MYVA_ASSET_DIR` để dùng thư mục khác.

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;
use myva_gameplay::SimSession;
use myva_graybox::benchmark::{Benchmark, BenchmarkConfig, ConfigureBenchmark, QualityTier};
use myva_graybox::runtime::StreamRuntime;
use myva_graybox::{ArenaView, CombatLog, GrayboxPlugin, Match, Mode};
use myva_sim::Phase;

/// Đợi qua nhịp mở màn và vài đòn để ảnh có HUD, log và vùng báo.
const SCREENSHOT_AFTER_TICK: u32 = 900;
/// Số khung chờ ảnh ghi xong trước khi thoát.
const EXIT_AFTER_FRAMES: u32 = 30;

#[derive(Resource)]
struct Capture {
    path: String,
    taken_at: Option<u32>,
    frames: u32,
}

fn main() -> AppExit {
    let mut args = std::env::args().skip(1);
    let mut mode = Mode::Boss;
    let mut demo = false;
    let mut art = true;
    let mut capture = None;
    let mut population = None;
    let mut tier = QualityTier::High;
    let mut seed = 7;
    let mut benchmark_output = None;
    let mut benchmark_seconds = 60.0;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--duel" => mode = Mode::Duel,
            "--demo" => demo = true,
            "--no-art" => art = false,
            "--benchmark" => {
                population = args.next().and_then(|value| value.parse::<u16>().ok());
                if !population.is_some_and(|value| matches!(value, 50 | 200 | 500)) {
                    eprintln!("--benchmark requires 50, 200 or 500");
                    return AppExit::error();
                }
            }
            "--benchmark-tier" => {
                tier = match args.next().as_deref() {
                    Some("High") => QualityTier::High,
                    Some("Low") => QualityTier::Low,
                    _ => {
                        eprintln!("--benchmark-tier requires High or Low");
                        return AppExit::error();
                    }
                };
            }
            "--benchmark-seed" => match args.next().and_then(|value| value.parse::<u32>().ok()) {
                Some(value) => seed = value,
                None => {
                    eprintln!("--benchmark-seed requires a u32");
                    return AppExit::error();
                }
            },
            "--benchmark-seconds" => {
                benchmark_seconds = args
                    .next()
                    .and_then(|value| value.parse::<f64>().ok())
                    .unwrap_or(0.0);
                if !benchmark_seconds.is_finite() || benchmark_seconds <= 0.0 {
                    eprintln!("--benchmark-seconds requires positive finite seconds");
                    return AppExit::error();
                }
            }
            "--benchmark-output" => {
                benchmark_output = args.next();
                if benchmark_output.is_none() {
                    eprintln!("--benchmark-output requires a JSON path");
                    return AppExit::error();
                }
            }
            "--screenshot" => match args.next() {
                Some(path) => {
                    capture = Some(Capture {
                        path,
                        taken_at: None,
                        frames: 0,
                    });
                    demo = true;
                }
                None => eprintln!("--screenshot cần đường dẫn file .png"),
            },
            other => eprintln!("bỏ qua tham số lạ: {other}"),
        }
    }

    if benchmark_output.is_some() && capture.is_some() {
        eprintln!(
            "--benchmark-output cannot be combined with --screenshot (separate exit conditions)"
        );
        return AppExit::error();
    }
    if population.is_some() && (mode != Mode::Boss || !art) {
        eprintln!("--benchmark requires the boss scene with art enabled (no --duel/--no-art)");
        return AppExit::error();
    }
    if benchmark_output.is_some() && population.is_none() {
        eprintln!("--benchmark-output also requires --benchmark");
        return AppExit::error();
    }
    let mut app = App::new();
    let asset_dir = std::env::var("MYVA_ASSET_DIR")
        .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets").to_owned());
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "MyVa — Thần Mạch · graybox Bevy".into(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: asset_dir,
                // Fixture không dùng file .meta; manifest giữ hash và metadata.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(GrayboxPlugin {
        mode,
        autopilot: demo,
        art,
    });
    if let Err(error) = myva_graybox::graybox_startup_result(&app) {
        eprintln!("Graybox loadout rejected: {error}");
        return AppExit::error();
    }
    app.add_systems(Update, native_keys);
    if let Some(population) = population {
        let config = BenchmarkConfig::new(population, tier, seed).expect("checked preset");
        app.world_mut()
            .write_message(ConfigureBenchmark(Some(config)));
    }
    if let Some(path) = benchmark_output {
        app.insert_resource(BenchmarkCapture {
            path,
            seconds: benchmark_seconds,
            started_at: None,
            finished: false,
        })
        .add_systems(
            Last,
            write_benchmark
                .after(myva_graybox::benchmark::CaptureSet)
                .after(myva_graybox::runtime::AuditSet),
        );
    }
    if let Some(capture) = capture {
        app.insert_resource(capture)
            .add_systems(Update, take_screenshot);
    }
    app.run()
}

fn save_replay(game: &Match, sim: &SimSession, log: &mut CombatLog) -> Option<String> {
    let replay = sim.get().replay();
    let path = format!(
        "graybox-r{}-t{}.myva-replay",
        game.bout.round(),
        replay.ticks
    );
    match std::fs::write(&path, replay.to_text()) {
        Ok(()) => {
            log.note(format!("Da luu replay: {path}"));
            Some(path)
        }
        Err(error) => {
            log.note(format!("Khong luu duoc replay: {error}"));
            None
        }
    }
}

fn native_keys(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Match>,
    sim: Res<SimSession>,
    mut log: ResMut<CombatLog>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::F9) {
        save_replay(&game, &sim, &mut log);
    }
}

fn take_screenshot(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    arena: Res<ArenaView>,
    game: Res<Match>,
    sim: Res<SimSession>,
    mut log: ResMut<CombatLog>,
    mut exit: MessageWriter<AppExit>,
) {
    match capture.taken_at {
        None => {
            let winding_up = arena
                .fighter(game.bout.rival())
                .and_then(|boss| boss.action)
                .is_some_and(|action| action.phase == Phase::Startup);
            if arena.tick >= SCREENSHOT_AFTER_TICK && winding_up {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(capture.path.clone()));
                if let Some(path) = save_replay(&game, &sim, &mut log) {
                    println!("ảnh: {}; replay: {path}", capture.path);
                }
                capture.taken_at = Some(arena.tick);
            }
        }
        Some(_) => {
            capture.frames += 1;
            if capture.frames >= EXIT_AFTER_FRAMES {
                exit.write(AppExit::Success);
            }
        }
    }
}

/// A raw acquisition artifact, not a qualified presented-FPS/mobile performance report.
#[derive(Resource)]
struct BenchmarkCapture {
    path: String,
    seconds: f64,
    started_at: Option<f64>,
    finished: bool,
}

fn native_rss_peak() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        let kb = line
            .strip_prefix("VmHWM:")?
            .split_whitespace()
            .next()?
            .parse::<u64>()
            .ok()?;
        kb.checked_mul(1024)
    })
}

#[allow(clippy::too_many_arguments)]
fn write_benchmark(
    mut capture: ResMut<BenchmarkCapture>,
    time: Res<Time<Real>>,
    benchmark: Res<Benchmark>,
    runtime: Res<StreamRuntime>,
    game: Res<Match>,
    arena: Res<ArenaView>,
    window: Single<&Window, With<PrimaryWindow>>,
    adapter: Option<Res<RenderAdapterInfo>>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.finished {
        return;
    }
    let now = time.elapsed_secs_f64();
    let started = *capture.started_at.get_or_insert(now);
    let complete =
        benchmark.sampled_ms() >= capture.seconds * 1000.0 && !benchmark.intervals_truncated();
    let timed_out =
        (!benchmark.is_ready() && now - started >= 45.0) || now - started >= capture.seconds + 60.0;
    if !complete && !timed_out && !benchmark.intervals_truncated() {
        return;
    }
    capture.finished = true;
    let status = if complete { "COMPLETE" } else { "INCOMPLETE" };
    let stream = runtime.summary();
    let known_gpu_bytes = stream.estimated_texture_bytes + benchmark.offscreen_color_bytes();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    let renderer = adapter.as_ref().map(|adapter| {
        let device_type = format!("{:?}", adapter.device_type);
        let name = adapter.name.to_lowercase();
        let software = device_type == "Cpu" || ["swiftshader", "llvmpipe", "lavapipe", "softpipe", "software", "warp"].iter().any(|part| name.contains(part));
        let class = if software { "software" } else if matches!(device_type.as_str(), "IntegratedGpu" | "DiscreteGpu") { "hardware_gpu" } else { "unknown" };
        serde_json::json!({ "adapter": adapter.name, "backend": format!("{:?}", adapter.backend), "device_type": device_type, "class": class, "adapter_info": format!("{:?}", &***adapter) })
    });
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("model name")
                    .and_then(|rest| rest.split_once(':'))
                    .map(|(_, model)| model.trim().to_owned())
            })
        });
    let ram = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("MemTotal:")
                    .map(|value| value.trim().to_owned())
            })
        });
    let version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned());
    let verification = game.verification.as_ref().map(Result::is_ok);
    let data = serde_json::json!({
        "schema_version": 1, "evidence_kind": "raw_runtime_acquisition", "status": status,
        "metadata": {
            "captured_at_unix_ms": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(),
            "checkout_revision": git(&["rev-parse", "HEAD"]),
            "checkout_dirty": git(&["status", "--porcelain"]).map(|text| !text.is_empty()),
            "build_metadata_limit": "Checkout revision and runtime PATH rustc are observations; neither proves the running binary compiler/revision. No embedded build attestation.",
            "debug_assertions": cfg!(debug_assertions), "bevy_pin": "0.20.0", "rustc_on_runtime_path": version,
            "argv": std::env::args().collect::<Vec<_>>(),
            "os": std::env::consts::OS, "architecture": std::env::consts::ARCH,
            "cpu_proc_model": cpu, "ram_proc_memtotal": ram,
            "hardware_limit": "Linux proc hardware values may describe the host; unavailable elsewhere and do not establish container limits or a mobile device.",
            "width_px": window.physical_width(), "height_px": window.physical_height(), "dpr": window.scale_factor(),
            "renderer": renderer,
            "acquisition": "Time<Real> contiguous Bevy Update intervals after 5s warmup; no presented FPS, GPU timestamps or input-to-photon instrumentation",
        },
        "native_game_rss_peak_bytes": native_rss_peak(),
        "rss_method": "Linux proc self VmHWM at sample completion, game process only; unavailable elsewhere; not game+CMP budget",
        "known_texture_and_offscreen_rgba8_bytes": known_gpu_bytes,
        "gpu_estimate_scope": "Retained fixture base RGBA8 plus low offscreen color; excludes mipmaps, buffers, other renderer targets and driver allocations",
        "benchmark": serde_json::from_str::<serde_json::Value>(&benchmark.to_json()).expect("benchmark produces finite JSON"),
        "runtime": serde_json::from_str::<serde_json::Value>(&stream.to_json()).expect("runtime produces ASCII JSON"),
        "gameplay": { "tick": arena.tick, "hash": format!("{:016x}", arena.hash), "autopilot": game.bout.autopilot(), "mode": game.bout.mode().id(), "replay_verified": verification },
        "presented_fps": { "status": "NOT_RUN", "reason": "Update schedule is not presentation instrumentation" },
        "mobile": { "status": "BLOCKED", "reason": "Native desktop capture does not establish physical Android/iOS gates" },
    });
    let text = serde_json::to_string_pretty(&data).expect("finite acquisition JSON") + "\n";
    match std::fs::write(&capture.path, text) {
        Ok(()) => {
            println!("raw benchmark acquisition: {}", capture.path);
            if complete {
                exit.write(AppExit::Success);
            } else {
                eprintln!(
                    "benchmark capture incomplete (startup/progress timeout or sample buffer truncated); see {}",
                    capture.path
                );
                exit.write(AppExit::error());
            }
        }
        Err(error) => {
            eprintln!(
                "cannot write benchmark acquisition {}: {error}",
                capture.path
            );
            exit.write(AppExit::error());
        }
    }
}
