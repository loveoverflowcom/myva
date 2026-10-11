use std::num::{NonZeroU32, NonZeroUsize};

use crate::config::DEFAULT_PORT;
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "MyVa — Thần Mạch developer workflow")]
pub struct Cli {
    /// Print the actual plan without building, starting processes or opening a browser.
    #[arg(long, global = true)]
    pub dry_run: bool,
    /// Override build concurrency; otherwise inherit CARGO_BUILD_JOBS / Cargo config (2).
    #[arg(long, global = true)]
    pub jobs: Option<NonZeroUsize>,
    #[command(subcommand)]
    pub task: Task,
}

#[derive(Debug, Subcommand)]
pub enum Task {
    /// Build an existing repository target.
    Build {
        #[command(subcommand)]
        target: Build,
    },
    /// Prepare and run a local application; web stays in the foreground.
    Run {
        #[command(subcommand)]
        target: Run,
    },
    /// Open an already built web app on an already running local server.
    App {
        #[command(subcommand)]
        target: App,
    },
    /// Run a verification contract; failures and missing prerequisites are nonzero.
    Verify {
        #[command(subcommand)]
        target: Verify,
    },
    /// Inspect prerequisites without installing tools or launching a device.
    Doctor {
        #[command(subcommand)]
        target: Option<Doctor>,
    },
    /// Format, lint, test and verify affected contracts (full by default).
    Preflight {
        #[arg(long)]
        changed_only: bool,
        /// Compare committed changes from the merge base plus all working-tree changes.
        #[arg(long, default_value = "origin/develop", requires = "changed_only")]
        base: String,
    },
    /// Run the full preflight contract; CI provisions all tools beforehand.
    Ci,
    /// Apply Rust formatting, or check it without rewriting files.
    Fmt {
        #[arg(long)]
        check: bool,
    },
    /// List Android devices and iOS simulators; never install or launch anything.
    Devices {
        #[arg(long, value_enum)]
        platform: Option<Platform>,
    },
}

#[derive(Debug, Subcommand)]
pub enum Build {
    Web(WebBuild),
    Graybox(GrayboxBuild),
    Sim(Release),
    Gameplay(Release),
    Economy(Release),
    Assets(Release),
    /// Root workspace, then both web bundles; native prototype is opt-in.
    All(Release),
    /// Existing Android prototype only. iOS reports BLOCKED, never a placeholder build.
    NativeProbe {
        #[arg(long, value_enum)]
        platform: Platform,
        #[arg(long, value_enum, default_value = "apk")]
        mode: NativeMode,
    },
}

#[derive(Debug, Default, Clone, Args)]
pub struct Release {
    #[arg(long)]
    pub release: bool,
}

#[derive(Debug, Clone, Args)]
pub struct GrayboxBuild {
    #[command(flatten)]
    pub build: Release,
    /// Explicitly omit native gamepad (useful on Linux without libudev development files).
    #[arg(long)]
    pub no_default_features: bool,
}

#[derive(Debug, Default, Clone, Args)]
pub struct WebBuild {
    #[arg(long, conflicts_with = "profile")]
    pub release: bool,
    /// Default keeps the existing optimized `web` profile. `dev` is an explicit opt-in.
    #[arg(long, value_enum)]
    pub profile: Option<WebProfile>,
}

impl WebBuild {
    pub fn profile(&self) -> WebProfile {
        if self.release {
            WebProfile::Release
        } else {
            self.profile.unwrap_or(WebProfile::Web)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WebProfile {
    Dev,
    Web,
    Release,
}

impl WebProfile {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Web => "web",
            Self::Release => "release",
        }
    }
}

#[derive(Debug, Args)]
pub struct WebAddress {
    #[arg(long, env = "MYVA_WEB_PORT", default_value_t = DEFAULT_PORT, value_parser = clap::value_parser!(u16).range(1..))]
    pub port: u16,
}

#[derive(Debug, Subcommand)]
pub enum Run {
    Web {
        #[command(flatten)]
        address: WebAddress,
        #[command(flatten)]
        build: WebBuild,
    },
    Graybox {
        #[command(flatten)]
        build: GrayboxBuild,
        /// Arguments for myva-graybox, after `--`.
        #[arg(last = true)]
        args: Vec<String>,
    },
    Headless {
        /// Arguments for myva-headless, after `--` (including seed/ticks/replay/hashes).
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Forward arguments to myva-replay (demo/verify/events).
    Replay {
        #[arg(last = true, required = true)]
        args: Vec<String>,
    },
    /// Forward arguments to myva-econ-sim; its CSV stays on stdout.
    Economy {
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Forward arguments to myva-assets (validate <manifest>).
    Assets {
        #[arg(last = true, required = true)]
        args: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum App {
    Web(WebAddress),
}

#[derive(Debug, Subcommand)]
pub enum Verify {
    /// Focused orchestration format/lint/unit + lifecycle tests.
    Xtask,
    Sim,
    Gameplay,
    Economy,
    Assets,
    Web {
        #[arg(long, env = "MYVA_WEB_TEST_PORT", default_value_t = DEFAULT_PORT, value_parser = clap::value_parser!(u16).range(1..))]
        port: u16,
        #[command(flatten)]
        build: WebBuild,
        /// Optional Playwright selection after `--`; zero/skipped tests do not pass.
        #[arg(last = true)]
        args: Vec<String>,
    },
    Determinism {
        #[arg(long, default_value = "3600")]
        ticks: NonZeroU32,
        #[arg(long, value_delimiter = ',', default_value = "1,7,12")]
        seeds: Vec<u64>,
    },
    /// Root Rust checks, documentation, asset and report contracts; no browser required.
    Repository,
}

#[derive(Debug, Subcommand)]
pub enum Doctor {
    Mobile {
        #[arg(long, value_enum)]
        platform: Platform,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Platform {
    Android,
    Ios,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum NativeMode {
    Check,
    Build,
    Apk,
    Cmp,
}

impl NativeMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Build => "build",
            Self::Apk => "apk",
            Self::Cmp => "cmp",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_contract_parses() {
        for command in [
            "build web",
            "build web --release",
            "build web --profile dev",
            "build graybox",
            "build sim",
            "build all",
            "run web",
            "run graybox -- --duel",
            "run headless -- --seed 7",
            "app web",
            "verify sim",
            "verify gameplay",
            "verify economy",
            "verify assets",
            "verify web -- tests/web/protocol.spec.mjs",
            "verify determinism --seeds 1,7,12",
            "verify repository",
            "doctor",
            "preflight",
            "preflight --changed-only",
            "ci",
            "doctor mobile --platform android",
            "doctor mobile --platform ios",
            "devices",
            "build native-probe --platform android",
            "build native-probe --platform android --mode cmp",
            "build all --dry-run --jobs 1",
        ] {
            Cli::try_parse_from(std::iter::once("xtask").chain(command.split_whitespace()))
                .unwrap_or_else(|error| panic!("{command}: {error}"));
        }
    }

    #[test]
    fn invalid_or_ambiguous_options_are_rejected() {
        for command in [
            "build web --release --profile dev",
            "run web --port 0",
            "run web --port 65536",
            "build native-probe",
            "build native-probe --platform android --mode install",
            "doctor mobile",
            "verify determinism --ticks 0",
            "build sim --jobs 0",
            "deploy",
            "run ios",
        ] {
            assert!(
                Cli::try_parse_from(std::iter::once("xtask").chain(command.split_whitespace()))
                    .is_err(),
                "{command}"
            );
        }
    }
}
