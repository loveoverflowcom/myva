//! One planner for local preflight and CI. Scripts remain implementation primitives.
use anyhow::{Context, Result};

use crate::{
    cli::{Build, NativeMode, Platform, Run, Verify, WebProfile},
    command::CommandSpec,
    config::{DEFAULT_PORT, WASM_TARGET, Workspace},
    doctor,
    impact::Selection,
    mobile,
    process::{Blocked, Runner, blocked},
    web,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Command(CommandSpec),
    WebPrerequisites,
    BrowserPrerequisites,
    AndroidPrerequisites(NativeMode),
    PortAvailable(u16),
    WebTests { port: u16, args: Vec<String> },
    Serve(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub label: String,
    pub action: Action,
}

impl Step {
    fn new(label: impl Into<String>, action: Action) -> Self {
        Self {
            label: label.into(),
            action,
        }
    }
    fn command(label: impl Into<String>, command: CommandSpec) -> Self {
        Self::new(label, Action::Command(command))
    }

    fn describe(&self) -> String {
        match &self.action {
            Action::Command(command) => {
                format!("{} [cwd: {}]", command.display(), command.cwd.display())
            }
            Action::WebPrerequisites => {
                "probe wasm32 target, pinned wasm-bindgen, Python and Bash; no install".into()
            }
            Action::BrowserPrerequisites => {
                "probe local Playwright module and browser; no npm/browser install".into()
            }
            Action::AndroidPrerequisites(mode) => format!(
                "probe Android {} prerequisites; no install or device launch",
                mode.name()
            ),
            Action::PortAvailable(port) => {
                format!("require free loopback port {port}; never kill a listener")
            }
            Action::WebTests { port, args } => format!(
                "own python3 scripts/serve-web-shell.py --port {port}; require readiness + build identity; node node_modules/@playwright/test/cli.js test {args:?}; require fresh executed-test report; stop owned server"
            ),
            Action::Serve(port) => format!(
                "own python3 scripts/serve-web-shell.py --port {port}; readiness at {}bundle-report.json; log/PID dashboard; Ctrl+C shutdown",
                web::url(*port)
            ),
        }
    }
}

pub fn execute(workspace: &Workspace, runner: &Runner, steps: &[Step]) -> Result<()> {
    if !runner.dry_run {
        std::fs::create_dir_all(workspace.evidence())
            .context("create verification artifact directory")?;
    }
    for (index, step) in steps.iter().enumerate() {
        eprintln!("\n==> {}", step.label);
        if runner.dry_run {
            eprintln!("NOT_RUN: {}", step.describe());
            continue;
        }
        let result = match &step.action {
            Action::Command(command) => runner.run(command),
            Action::WebPrerequisites => doctor::web(workspace, runner),
            Action::BrowserPrerequisites => doctor::browser(workspace, runner)
                .map(|path| println!("Browser: {}", path.trim()))
                .map_err(|e| blocked(format!("browser prerequisites: {e:#}"))),
            Action::AndroidPrerequisites(mode) => mobile::android(workspace, runner, Some(*mode)),
            Action::PortAvailable(port) => web::ensure_port_free(*port),
            Action::WebTests { port, args } => web::tests(workspace, runner, *port, args),
            Action::Serve(port) => web::Server::start(workspace, runner, *port)
                .and_then(|server| server.serve(workspace, runner)),
        };
        if let Err(error) = result {
            let status = if error.downcast_ref::<Blocked>().is_some() {
                "BLOCKED"
            } else {
                "FAIL"
            };
            eprintln!("{status}: {}: {error:#}", step.label);
            for pending in &steps[index + 1..] {
                eprintln!("NOT_RUN: {} (earlier stage did not pass)", pending.label);
            }
            return Err(error.context(step.label.clone()));
        }
        if !matches!(step.action, Action::Serve(_)) {
            eprintln!("PASS: {}", step.label);
        }
    }
    Ok(())
}

pub fn build(workspace: &Workspace, target: &Build) -> Result<Vec<Step>> {
    let w = workspace;
    Ok(match target {
        Build::Web(options) => web_build(w, options.profile()),
        Build::Sim(options) => vec![build_package(w, "myva-sim", options.release)],
        Build::Gameplay(options) => vec![build_package(w, "myva-gameplay", options.release)],
        Build::Economy(options) => vec![build_package(w, "myva-economy", options.release)],
        Build::Assets(options) => vec![build_package(w, "myva-assets", options.release)],
        Build::Graybox(options) => vec![Step::command(
            "build desktop graybox",
            CommandSpec::new(w, "cargo", &["build", "--locked", "-p", "myva-graybox"])
                .args(release_args(options.build.release))
                .args(feature_args(options.no_default_features)),
        )],
        Build::All(options) => {
            let command = CommandSpec::new(w, "cargo", &["build", "--locked", "--workspace"])
                .args(release_args(options.release));
            let mut steps = vec![Step::command("build root workspace (native host)", command)];
            steps.extend(web_build(
                w,
                if options.release {
                    WebProfile::Release
                } else {
                    WebProfile::Web
                },
            ));
            steps
        }
        Build::NativeProbe {
            platform: Platform::Ios,
            ..
        } => {
            return Err(blocked(
                "iOS build/packaging has no implementation; see docs/reports/native-feasibility.md",
            ));
        }
        Build::NativeProbe {
            platform: Platform::Android,
            mode,
        } => android_build(w, *mode),
    })
}

fn release_args(release: bool) -> Vec<String> {
    if release {
        vec!["--release".into()]
    } else {
        vec![]
    }
}
fn feature_args(no_default: bool) -> Vec<String> {
    if no_default {
        vec!["--no-default-features".into()]
    } else {
        vec![]
    }
}

fn build_package(w: &Workspace, package: &str, release: bool) -> Step {
    Step::command(
        format!("build {package}"),
        CommandSpec::new(w, "cargo", &["build", "--locked", "-p", package])
            .args(release_args(release)),
    )
}

pub fn web_build(w: &Workspace, profile: WebProfile) -> Vec<Step> {
    vec![
        Step::new("web prerequisites", Action::WebPrerequisites),
        Step::command(
            "build Leptos + Bevy WASM bundles",
            CommandSpec::new(w, "bash", &["scripts/build-web-shell.sh"])
                .env("PROFILE", profile.name()),
        ),
    ]
}

fn android_build(w: &Workspace, mode: NativeMode) -> Vec<Step> {
    vec![
        Step::new(
            format!("Android {} prerequisites", mode.name()),
            Action::AndroidPrerequisites(mode),
        ),
        Step::command(
            format!("Android {} compile/package (runtime NOT_RUN)", mode.name()),
            CommandSpec::new(
                w,
                "bash",
                &["prototypes/native/scripts/build-android.sh", mode.name()],
            ),
        ),
    ]
}

pub fn run(w: &Workspace, target: &Run) -> Vec<Step> {
    match target {
        Run::Web { address, build } => {
            let mut steps = vec![Step::new("web port", Action::PortAvailable(address.port))];
            steps.extend(web_build(w, build.profile()));
            steps.push(Step::new("serve web", Action::Serve(address.port)));
            steps
        }
        Run::Graybox { build, args } => vec![Step::command(
            "run desktop graybox",
            CommandSpec::new(w, "cargo", &["run", "--locked", "-p", "myva-graybox"])
                .args(release_args(build.build.release))
                .args(feature_args(build.no_default_features))
                .args(std::iter::once("--".into()).chain(args.iter().cloned())),
        )],
        Run::Headless { args } => vec![Step::command(
            "run headless gameplay",
            CommandSpec::new(
                w,
                "cargo",
                &[
                    "run",
                    "--locked",
                    "-p",
                    "myva-gameplay",
                    "--bin",
                    "myva-headless",
                    "--",
                ],
            )
            .args(args.iter().cloned()),
        )],
        Run::Replay { args } => vec![Step::command(
            "run replay tool",
            cli_command(w, "myva-sim", "myva-replay", args),
        )],
        Run::Economy { args } => vec![Step::command(
            "run economy simulator",
            cli_command(w, "myva-economy", "myva-econ-sim", args),
        )],
        Run::Assets { args } => vec![Step::command(
            "run asset validator",
            cli_command(w, "myva-assets", "myva-assets", args),
        )],
    }
}

fn cli_command(w: &Workspace, package: &str, binary: &str, args: &[String]) -> CommandSpec {
    CommandSpec::new(
        w,
        "cargo",
        &[
            "run", "--locked", "--quiet", "-p", package, "--bin", binary, "--",
        ],
    )
    .args(args.iter().cloned())
}

fn package_tests(w: &Workspace, packages: &[&str]) -> Step {
    let args: Vec<String> = packages
        .iter()
        .flat_map(|p| ["-p".into(), (*p).into()])
        .collect();
    Step::command(
        format!("Rust tests: {}", packages.join(", ")),
        CommandSpec::new(w, "cargo", &["test", "--locked"]).args(args),
    )
}

pub fn verify(w: &Workspace, target: &Verify) -> Vec<Step> {
    match target {
        Verify::Xtask => vec![
            format(w, true),
            Step::command(
                "xtask clippy",
                CommandSpec::new(
                    w,
                    "cargo",
                    &[
                        "clippy",
                        "--locked",
                        "-p",
                        "xtask",
                        "--all-targets",
                        "--",
                        "-D",
                        "warnings",
                    ],
                ),
            ),
            package_tests(w, &["xtask"]),
        ],
        Verify::Sim => {
            let mut steps = vec![package_tests(w, &["myva-sim"])];
            steps.extend(sim(w));
            steps
        }
        Verify::Gameplay => {
            let mut steps = vec![package_tests(w, &["myva-sim", "myva-gameplay"])];
            steps.extend(gameplay(w));
            steps
        }
        Verify::Economy => {
            let mut steps = vec![package_tests(w, &["myva-economy"])];
            steps.extend(economy(w));
            steps
        }
        Verify::Assets => {
            let mut steps = vec![package_tests(w, &["myva-assets"])];
            steps.extend(assets(w));
            steps.push(reports(w));
            steps
        }
        Verify::Web { port, build, args } => browser(w, *port, build.profile(), args),
        Verify::Determinism { ticks, seeds } => determinism(w, ticks.get(), seeds),
        Verify::Repository => {
            let selection = Selection {
                web: false,
                determinism: false,
                ..Selection::all("repository verification")
            };
            preflight(w, &selection, DEFAULT_PORT)
        }
    }
}

pub fn format(w: &Workspace, check: bool) -> Step {
    let command = CommandSpec::new(w, "cargo", &["fmt", "--all"]).args(if check {
        vec!["--".into(), "--check".into()]
    } else {
        vec![]
    });
    Step::command("Rust format", command)
}

pub fn preflight(w: &Workspace, selection: &Selection, port: u16) -> Vec<Step> {
    let mut steps = vec![
        Step::command(
            "Rust format",
            CommandSpec::new(w, "cargo", &["fmt", "--all", "--", "--check"]),
        ),
        Step::command(
            "documentation links and anchors",
            CommandSpec::new(
                w,
                "python3",
                &[
                    "scripts/check-docs.py",
                    "README.md",
                    "CONTRIBUTING.md",
                    "docs",
                    "prototypes/native/README.md",
                    "xtask/README.md",
                ],
            ),
        ),
    ];
    if selection.full || !selection.packages.is_empty() {
        let scope: Vec<String> = if selection.full {
            vec!["--workspace".into()]
        } else {
            selection
                .packages
                .iter()
                .flat_map(|p| ["-p".into(), p.clone()])
                .collect()
        };
        steps.push(Step::command(
            "native Rust clippy",
            CommandSpec::new(w, "cargo", &["clippy", "--locked", "--all-targets"])
                .args(scope.iter().cloned())
                .args(["--".into(), "-D".into(), "warnings".into()]),
        ));
        steps.push(Step::command(
            "native Rust tests",
            CommandSpec::new(w, "cargo", &["test", "--locked"]).args(scope),
        ));
    }
    if selection.sim {
        steps.extend(sim(w));
    }
    if selection.gameplay {
        steps.extend(gameplay(w));
    }
    if selection.economy {
        steps.extend(economy(w));
    }
    if selection.assets {
        steps.extend(assets(w));
    }
    if selection.reports {
        steps.push(reports(w));
    }
    if selection.determinism {
        append_unique(&mut steps, determinism(w, 3600, &[1, 7, 12]));
    }
    if selection.web {
        append_unique(&mut steps, browser(w, port, WebProfile::Web, &[]));
    }
    if selection.native {
        for manifest in [
            "prototypes/native/Cargo.toml",
            "prototypes/native/probes/event-loop/Cargo.toml",
        ] {
            steps.push(Step::command(
                format!("native prototype format: {manifest}"),
                CommandSpec::new(
                    w,
                    "cargo",
                    &["fmt", "--manifest-path", manifest, "--", "--check"],
                ),
            ));
        }
        steps.push(Step::command(
            "Android script syntax",
            CommandSpec::new(
                w,
                "bash",
                &["-n", "prototypes/native/scripts/build-android.sh"],
            ),
        ));
        for mode in [NativeMode::Check, NativeMode::Apk, NativeMode::Cmp] {
            steps.extend(android_build(w, mode));
        }
    }
    steps
}

fn append_unique(steps: &mut Vec<Step>, additional: Vec<Step>) {
    for step in additional {
        if !steps.contains(&step) {
            steps.push(step);
        }
    }
}

fn sim(w: &Workspace) -> Vec<Step> {
    let path = w
        .evidence()
        .join("sim-smoke.myva-replay")
        .to_string_lossy()
        .into_owned();
    ["demo", "verify"]
        .into_iter()
        .map(|action| {
            Step::command(
                format!("simulation replay {action}"),
                CommandSpec::new(
                    w,
                    "cargo",
                    &[
                        "run",
                        "--locked",
                        "-p",
                        "myva-sim",
                        "--bin",
                        "myva-replay",
                        "--",
                        action,
                        &path,
                    ],
                ),
            )
        })
        .collect()
}

fn gameplay(w: &Workspace) -> Vec<Step> {
    let mut steps = Vec::new();
    for seed in [1, 7, 12] {
        for mach in [false, true] {
            let name = format!("{}-{seed}", if mach { "mach" } else { "gameplay" });
            let replay = w
                .evidence()
                .join(format!("{name}.myva-replay"))
                .to_string_lossy()
                .into_owned();
            let hashes = w
                .evidence()
                .join(format!("{name}-hashes.txt"))
                .to_string_lossy()
                .into_owned();
            let command = CommandSpec::new(
                w,
                "cargo",
                &[
                    "run",
                    "--locked",
                    "-p",
                    "myva-gameplay",
                    "--bin",
                    "myva-headless",
                    "--",
                    "--seed",
                    &seed.to_string(),
                    "--ticks",
                    "3600",
                    "--replay",
                    &replay,
                    "--hashes",
                    &hashes,
                ],
            )
            .args(if mach { vec!["--mach".into()] } else { vec![] });
            steps.push(Step::command(
                format!("headless {name}: per-tick ECS/core hashes"),
                command,
            ));
            steps.push(Step::command(
                format!("verify {name} replay"),
                CommandSpec::new(
                    w,
                    "cargo",
                    &[
                        "run",
                        "--locked",
                        "-p",
                        "myva-sim",
                        "--bin",
                        "myva-replay",
                        "--",
                        "verify",
                        &replay,
                    ],
                ),
            ));
        }
    }
    steps
}

fn economy(w: &Workspace) -> Vec<Step> {
    vec![Step::command(
        "economy invariants: seed 7, 90 days",
        CommandSpec::new(
            w,
            "cargo",
            &[
                "run",
                "--locked",
                "-p",
                "myva-economy",
                "--bin",
                "myva-econ-sim",
                "--",
                "--seed",
                "7",
                "--days",
                "90",
            ],
        )
        .stdout(w.evidence().join("economy-seed7-90days.csv")),
    )]
}

fn assets(w: &Workspace) -> Vec<Step> {
    vec![
        Step::command(
            "generated asset fixture check",
            CommandSpec::new(
                w,
                "python3",
                &["scripts/generate-asset-fixture.py", "--check"],
            ),
        ),
        Step::command(
            "asset manifest/provenance/hash/resource budgets",
            CommandSpec::new(
                w,
                "cargo",
                &[
                    "run",
                    "--locked",
                    "-p",
                    "myva-assets",
                    "--",
                    "validate",
                    "assets/fixture/manifest.json",
                ],
            ),
        ),
    ]
}

fn reports(w: &Workspace) -> Step {
    Step::command(
        "asset performance report contracts (synthetic tests, not measurements)",
        CommandSpec::new(
            w,
            "python3",
            &[
                "-m",
                "unittest",
                "discover",
                "-s",
                "tests",
                "-p",
                "test_*.py",
            ],
        ),
    )
}

fn wasm(w: &Workspace) -> Vec<Step> {
    vec![
        Step::new("web prerequisites", Action::WebPrerequisites),
        Step::command(
            "WASM Rust clippy + bridge compilation",
            CommandSpec::new(
                w,
                "cargo",
                &[
                    "clippy",
                    "--locked",
                    "-p",
                    "myva-web-shell",
                    "-p",
                    "myva-web-game",
                    "-p",
                    "myva-sim",
                    "-p",
                    "myva-gameplay",
                    "-p",
                    "myva-assets",
                    "--target",
                    WASM_TARGET,
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
        ),
        Step::command(
            "renderer-independent WASM simulation",
            CommandSpec::new(
                w,
                "cargo",
                &[
                    "build",
                    "--locked",
                    "-p",
                    "myva-sim",
                    "--target",
                    WASM_TARGET,
                ],
            ),
        ),
    ]
}

fn determinism(w: &Workspace, ticks: u32, seeds: &[u64]) -> Vec<Step> {
    let mut steps = vec![Step::new("web prerequisites", Action::WebPrerequisites)];
    let args = std::iter::once(ticks.to_string()).chain(seeds.iter().map(u64::to_string));
    steps.push(Step::command(
        "native/WASM determinism: tick hashes, seed and Mach fixture",
        CommandSpec::new(w, "bash", &["scripts/check-wasm-determinism.sh"]).args(args),
    ));
    steps
}

fn browser(w: &Workspace, port: u16, profile: WebProfile, args: &[String]) -> Vec<Step> {
    let mut steps = vec![
        Step::new("web test port", Action::PortAvailable(port)),
        Step::new("browser prerequisites", Action::BrowserPrerequisites),
    ];
    append_unique(&mut steps, wasm(w));
    append_unique(&mut steps, web_build(w, profile));
    steps.push(Step::new(
        "Playwright web/bridge/gameplay/asset E2E",
        Action::WebTests {
            port,
            args: args.to_owned(),
        },
    ));
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Release;

    fn workspace() -> Workspace {
        Workspace {
            root: "/repo".into(),
        }
    }

    #[test]
    fn web_profiles_reuse_the_same_primitive_and_all_is_sequential() {
        let w = workspace();
        for profile in [WebProfile::Web, WebProfile::Dev, WebProfile::Release] {
            let steps = web_build(&w, profile);
            let Action::Command(command) = &steps[1].action else {
                panic!()
            };
            assert_eq!(command.program, "bash");
            assert_eq!(command.args, ["scripts/build-web-shell.sh"]);
            assert_eq!(command.env["PROFILE"], profile.name());
        }
        let steps = build(&w, &Build::All(Release { release: true })).unwrap();
        let Action::Command(command) = &steps[0].action else {
            panic!()
        };
        assert_eq!(
            command.args,
            ["build", "--locked", "--workspace", "--release"]
        );
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn mobile_modes_are_real_and_ios_never_creates_a_build() {
        let w = workspace();
        for mode in [
            NativeMode::Check,
            NativeMode::Build,
            NativeMode::Apk,
            NativeMode::Cmp,
        ] {
            let steps = build(
                &w,
                &Build::NativeProbe {
                    platform: Platform::Android,
                    mode,
                },
            )
            .unwrap();
            let Action::Command(command) = &steps[1].action else {
                panic!()
            };
            assert_eq!(
                command.args,
                ["prototypes/native/scripts/build-android.sh", mode.name()]
            );
            assert!(
                !command
                    .args
                    .iter()
                    .any(|arg| arg.contains("install") || arg.contains("launch"))
            );
        }
        assert!(
            build(
                &w,
                &Build::NativeProbe {
                    platform: Platform::Ios,
                    mode: NativeMode::Apk
                }
            )
            .is_err()
        );
    }

    #[test]
    fn preflight_full_and_ci_share_a_deduplicated_plan() {
        let steps = preflight(
            &workspace(),
            &Selection::all("CI/full preflight"),
            DEFAULT_PORT,
        );
        assert_eq!(
            steps
                .iter()
                .filter(|s| matches!(s.action, Action::WebPrerequisites))
                .count(),
            1
        );
        assert_eq!(
            steps
                .iter()
                .filter(|s| matches!(s.action, Action::WebTests { .. }))
                .count(),
            1
        );
        assert!(steps.iter().any(|s| s.label.contains("determinism")));
        assert!(steps.iter().any(|s| s.label.contains("report contracts")));
        assert!(
            !steps
                .iter()
                .any(|s| matches!(s.action, Action::AndroidPrerequisites(_)))
        );
    }

    #[test]
    fn failure_stops_the_plan_and_dry_run_never_creates_artifacts() {
        let root = tempfile::tempdir().unwrap();
        let w = Workspace {
            root: root.path().to_owned(),
        };
        let steps = vec![
            Step::command(
                "failing gate",
                CommandSpec::new(&w, "sh", &["-c", "exit 9"]),
            ),
            Step::command(
                "must not run",
                CommandSpec::new(&w, "sh", &["-c", "touch unexpected"]),
            ),
        ];
        let dry = Runner::new(true, None);
        execute(&w, &dry, &steps).unwrap();
        assert!(!w.evidence().exists());
        assert!(!w.path("unexpected").exists());
        assert!(execute(&w, &Runner::new(false, None), &steps).is_err());
        assert!(!w.path("unexpected").exists());
    }
}
