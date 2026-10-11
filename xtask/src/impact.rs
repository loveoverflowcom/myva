//! Conservative reverse dependency closure, including dev/build dependencies from Cargo metadata.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{command::CommandSpec, config::Workspace, process::Runner};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    pub full: bool,
    pub packages: BTreeSet<String>,
    pub sim: bool,
    pub gameplay: bool,
    pub economy: bool,
    pub assets: bool,
    pub reports: bool,
    pub web: bool,
    pub determinism: bool,
    pub native: bool,
    pub reason: String,
}

impl Selection {
    pub fn all(reason: impl Into<String>) -> Self {
        Self {
            full: true,
            sim: true,
            gameplay: true,
            economy: true,
            assets: true,
            reports: true,
            web: true,
            determinism: true,
            reason: reason.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub packages: Vec<Package>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub manifest_path: PathBuf,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    pub path: Option<PathBuf>,
}

pub fn changed_paths(workspace: &Workspace, runner: &Runner, base: &str) -> Result<Vec<String>> {
    let merge = runner
        .text(&CommandSpec::new(
            workspace,
            "git",
            &["merge-base", base, "HEAD"],
        ))
        .with_context(|| {
            format!(
                "cannot compare against {base}; fetch it or pass an available --base (no gates ran)"
            )
        })?;
    let merge = merge.trim();
    let mut paths = BTreeSet::new();
    for args in [
        vec![
            "diff",
            "--name-only",
            "-z",
            "--no-renames",
            merge,
            "HEAD",
            "--",
        ],
        vec![
            "diff",
            "--name-only",
            "-z",
            "--no-renames",
            "--cached",
            "HEAD",
            "--",
        ],
        vec!["diff", "--name-only", "-z", "--no-renames", "--"],
        vec!["ls-files", "--others", "--exclude-standard", "-z", "--"],
    ] {
        let output = runner.text(&CommandSpec::new(workspace, "git", &args))?;
        paths.extend(
            output
                .split('\0')
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
        );
    }
    Ok(paths.into_iter().collect())
}

pub fn load_metadata(workspace: &Workspace, runner: &Runner) -> Result<Metadata> {
    let output = runner.text(&CommandSpec::new(
        workspace,
        "cargo",
        &[
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--locked",
            "--offline",
        ],
    ))?;
    serde_json::from_str(&output).context("parse workspace Cargo metadata")
}

pub fn select(workspace: &Workspace, metadata: &Metadata, paths: &[String]) -> Result<Selection> {
    let mut directories = BTreeMap::new();
    let mut reverse: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in &metadata.packages {
        let dir = package
            .manifest_path
            .parent()
            .context("package manifest needs a parent")?;
        let relative = dir
            .strip_prefix(&workspace.root)
            .with_context(|| format!("local package outside workspace: {}", dir.display()))?;
        directories.insert(
            relative.to_string_lossy().replace('\\', "/"),
            package.name.clone(),
        );
    }
    for package in &metadata.packages {
        for dependency in &package.dependencies {
            if let Some(path) = &dependency.path {
                let relative = path.strip_prefix(&workspace.root).with_context(|| {
                    format!(
                        "dependency outside workspace: {}; cannot determine impact",
                        path.display()
                    )
                })?;
                let dir = relative.to_string_lossy().replace('\\', "/");
                ensure!(
                    directories.contains_key(&dir),
                    "unmapped local dependency {dir}; cannot determine impact"
                );
                reverse
                    .entry(directories[&dir].clone())
                    .or_default()
                    .insert(package.name.clone());
            }
        }
    }
    if paths.is_empty() {
        return Ok(Selection::all("empty diff; use full verification"));
    }
    let mut selected = Selection::default();
    let mut unknown = Vec::new();
    for path in paths {
        if within(path, "prototypes/native") {
            selected.native = true;
        } else if let Some((_, name)) = directories.iter().find(|(dir, _)| within(path, dir)) {
            if name == "xtask" {
                unknown.push(path.clone());
            }
            selected.packages.insert(name.clone());
        } else if within(path, "apps/web/public") || within(path, "tests/web") {
            selected
                .packages
                .extend(["myva-web-shell".into(), "myva-web-game".into()]);
            selected.web = true;
        } else if within(path, "assets") || within(path, "benchmarks") {
            selected.packages.insert("myva-assets".into());
            selected.assets = true;
            selected.reports = true;
        } else if path == "tests/test_asset_performance_report.py" {
            selected.reports = true;
        } else {
            // Docs can express protocol/gameplay contracts. CI, toolchains, scripts, future
            // packages and unknown inputs also need full verification rather than a guess.
            unknown.push(path.clone());
        }
    }
    if !unknown.is_empty() {
        return Ok(Selection {
            native: selected.native,
            ..Selection::all(format!("shared/unknown inputs: {}", unknown.join(", ")))
        });
    }
    loop {
        let before = selected.packages.len();
        for name in selected.packages.clone() {
            if let Some(consumers) = reverse.get(&name) {
                selected.packages.extend(consumers.iter().cloned());
            }
        }
        if before == selected.packages.len() {
            break;
        }
    }
    let has = |name: &str| selected.packages.contains(name);
    selected.web |= has("myva-web-shell") || has("myva-web-game") || has("myva-graybox");
    selected.gameplay = has("myva-gameplay") || has("myva-graybox");
    selected.sim = has("myva-sim") || selected.gameplay;
    selected.economy = has("myva-economy");
    selected.assets |= has("myva-assets");
    selected.reports |= selected.assets;
    selected.determinism = selected.sim || selected.web;
    if selected.sim {
        selected.packages.insert("myva-sim".into());
    }
    if selected.gameplay {
        selected.packages.insert("myva-gameplay".into());
    }
    selected.reason = "Cargo reverse dependency closure and explicit non-Rust consumers".into();
    Ok(selected)
}

fn within(path: &str, directory: &str) -> bool {
    path == directory
        || path
            .strip_prefix(directory)
            .is_some_and(|s| s.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Workspace, Metadata) {
        let w = Workspace {
            root: "/repo".into(),
        };
        let packages = [
            ("myva-sim", "packages/sim", vec![]),
            ("myva-gameplay", "packages/gameplay", vec!["packages/sim"]),
            ("myva-assets", "packages/assets", vec![]),
            ("myva-economy", "packages/economy", vec![]),
            (
                "myva-graybox",
                "packages/graybox",
                vec!["packages/gameplay", "packages/sim", "packages/assets"],
            ),
            (
                "myva-web-game",
                "apps/web/game",
                vec!["packages/graybox", "packages/gameplay"],
            ),
            ("myva-web-shell", "apps/web/shell", vec![]),
            ("xtask", "xtask", vec![]),
        ]
        .into_iter()
        .map(|(name, dir, deps)| Package {
            name: name.into(),
            manifest_path: w.path(dir).join("Cargo.toml"),
            dependencies: deps
                .into_iter()
                .map(|dir| Dependency {
                    path: Some(w.path(dir)),
                })
                .collect(),
        })
        .collect();
        (w, Metadata { packages })
    }

    fn impact(paths: &[&str]) -> Selection {
        let (w, metadata) = fixture();
        select(
            &w,
            &metadata,
            &paths.iter().map(|p| (*p).into()).collect::<Vec<_>>(),
        )
        .unwrap()
    }

    #[test]
    fn simulation_and_gameplay_cover_consumers_and_determinism() {
        for path in [
            "packages/sim/src/tick.rs",
            "packages/gameplay/tests/deleted.rs",
        ] {
            let s = impact(&[path]);
            assert!(s.sim && s.gameplay && s.web && s.determinism);
            assert!(s.packages.contains("myva-graybox") && s.packages.contains("myva-web-game"));
            assert!(!s.economy && !s.full);
        }
    }

    #[test]
    fn economy_is_narrow_assets_and_web_include_browser_contracts() {
        let s = impact(&["packages/economy/src/ledger.rs"]);
        assert!(s.economy && !s.web && !s.determinism);
        assert_eq!(s.packages, BTreeSet::from(["myva-economy".into()]));
        let s = impact(&["assets/fixture/manifest.json"]);
        assert!(s.assets && s.reports && s.web && s.determinism);
        let s = impact(&["apps/web/public/bootstrap.js"]);
        assert!(s.web && s.determinism);
        assert!(impact(&["prototypes/native/android/shared/build.gradle.kts"]).native);
    }

    #[test]
    fn unknown_shared_tooling_and_empty_changes_fail_safe() {
        for paths in [
            vec![],
            vec!["Cargo.lock"],
            vec![".cargo/config.toml"],
            vec![".github/workflows/ci.yml"],
            vec!["rust-toolchain.toml"],
            vec!["docs/design/combat.md"],
            vec!["xtask/src/main.rs"],
            vec!["new-package/src/lib.rs"],
        ] {
            let s = impact(&paths);
            assert!(s.full && s.web && s.determinism && s.assets);
        }
        let (w, mut metadata) = fixture();
        metadata.packages[0].dependencies.push(Dependency {
            path: Some("/outside".into()),
        });
        assert!(select(&w, &metadata, &["packages/sim/src/lib.rs".into()]).is_err());
    }

    #[test]
    fn git_diff_covers_commits_staging_worktree_untracked_deletions_and_both_rename_ends() {
        let root = tempfile::tempdir().unwrap();
        let w = Workspace {
            root: root.path().to_owned(),
        };
        let r = Runner::new(false, None);
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "xtask@example.invalid"],
            vec!["config", "user.name", "xtask tests"],
        ] {
            r.text(&CommandSpec::new(&w, "git", &args)).unwrap();
        }
        for file in ["committed", "staged", "unstaged", "deleted", "old"] {
            std::fs::write(w.path(file), "base").unwrap();
        }
        r.text(&CommandSpec::new(&w, "git", &["add", "."])).unwrap();
        r.text(&CommandSpec::new(&w, "git", &["commit", "-qm", "base"]))
            .unwrap();
        let base = r
            .text(&CommandSpec::new(&w, "git", &["rev-parse", "HEAD"]))
            .unwrap();
        std::fs::write(w.path("committed"), "changed").unwrap();
        r.text(&CommandSpec::new(&w, "git", &["commit", "-qam", "change"]))
            .unwrap();
        std::fs::write(w.path("staged"), "changed").unwrap();
        r.text(&CommandSpec::new(&w, "git", &["add", "staged"]))
            .unwrap();
        std::fs::write(w.path("unstaged"), "changed").unwrap();
        std::fs::remove_file(w.path("deleted")).unwrap();
        r.text(&CommandSpec::new(&w, "git", &["mv", "old", "new"]))
            .unwrap();
        std::fs::write(w.path("untracked with space"), "new").unwrap();
        let paths = changed_paths(&w, &r, base.trim()).unwrap();
        assert_eq!(
            paths,
            [
                "committed",
                "deleted",
                "new",
                "old",
                "staged",
                "unstaged",
                "untracked with space"
            ]
        );
        assert!(changed_paths(&w, &r, "missing-base").is_err());
    }
}
