use std::path::{Path, PathBuf};

use crate::process::blocked;
use anyhow::{Context, Result};

pub const WASM_TARGET: &str = "wasm32-unknown-unknown";
pub const BINDGEN_VERSION: &str = "wasm-bindgen 0.2.129";
pub const ANDROID_TARGET: &str = "aarch64-linux-android";
pub const IOS_TARGETS: &[&str] = &["aarch64-apple-ios", "aarch64-apple-ios-sim"];
pub const NDK_VERSION: &str = "27.0.12077973";
pub const HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
}

impl Workspace {
    pub fn discover() -> Result<Self> {
        let cwd = std::env::current_dir().context("read current directory")?;
        let compiled = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = find_root(&cwd)
            .or_else(|| find_root(compiled))
            .context("cannot locate MyVa workspace (Cargo.toml, packages/sim and xtask)")?;
        Ok(Self { root })
    }

    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    pub fn evidence(&self) -> PathBuf {
        self.path("target/xtask-evidence")
    }

    pub fn web_dir(&self) -> PathBuf {
        self.path("target/web-shell")
    }

    pub fn require_web_build(&self) -> Result<String> {
        for file in [
            "index.html",
            "bootstrap.js",
            "shell/myva_web_shell.js",
            "shell/myva_web_shell_bg.wasm",
            "game/index.html",
            "game/runtime.js",
            "game/pkg/myva_web_game.js",
            "game/pkg/myva_web_game_bg.wasm",
            "bundle-report.json",
        ] {
            if !self.web_dir().join(file).is_file() {
                return Err(blocked(format!(
                    "missing web artifact {file}; run `cargo xtask build web`"
                )));
            }
        }
        std::fs::read_to_string(self.web_dir().join("bundle-report.json"))
            .context("read existing web bundle report")
    }
}

fn find_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file()
                && path.join("xtask/Cargo.toml").is_file()
                && path.join("packages/sim/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_from_nested_directory_and_checks_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        for file in ["Cargo.toml", "xtask/Cargo.toml", "packages/sim/Cargo.toml"] {
            let path = temp.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        }
        assert_eq!(
            find_root(&temp.path().join("packages/sim/src")),
            Some(temp.path().to_owned())
        );
        assert!(
            Workspace {
                root: temp.path().to_owned()
            }
            .require_web_build()
            .is_err()
        );
    }
}
