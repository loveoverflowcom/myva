use std::{path::Path, time::Duration};

use anyhow::{Result, ensure};

use crate::{
    command::CommandSpec,
    config::{BINDGEN_VERSION, WASM_TARGET, Workspace},
    process::{Runner, blocked},
};

#[derive(Default)]
pub struct Checks {
    missing: Vec<String>,
}

impl Checks {
    pub fn check(&mut self, label: &str, result: Result<String>) {
        match result {
            Ok(detail) => println!(
                "PASS: {label}: {}",
                detail.trim().lines().next().unwrap_or("available")
            ),
            Err(error) => {
                println!("BLOCKED: {label}: {error:#}");
                self.missing.push(label.into());
            }
        }
    }

    pub fn path(&mut self, label: &str, path: &Path) {
        self.check(
            label,
            if path.exists() {
                Ok(path.display().to_string())
            } else {
                Err(blocked(format!("missing {}", path.display())))
            },
        );
    }

    pub fn finish(self) -> Result<()> {
        if self.missing.is_empty() {
            Ok(())
        } else {
            Err(blocked(format!(
                "missing prerequisites: {} (nothing installed)",
                self.missing.join(", ")
            )))
        }
    }
}

pub fn probe(
    workspace: &Workspace,
    runner: &Runner,
    program: &str,
    args: &[&str],
) -> Result<String> {
    let spec = CommandSpec::new(workspace, program, args);
    let output = runner.output(&spec, Duration::from_secs(30))?;
    let detail = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    ensure!(
        output.status.success(),
        "{}: {}: {}",
        spec.display(),
        output.status,
        detail.trim()
    );
    Ok(detail)
}

pub fn target(workspace: &Workspace, runner: &Runner, name: &str) -> Result<String> {
    let targets = probe(
        workspace,
        runner,
        "rustup",
        &["+1.97.1", "target", "list", "--installed"],
    )?;
    ensure!(
        targets.lines().any(|line| line.trim() == name),
        "Rust target {name} is not installed"
    );
    Ok(name.into())
}

pub fn web(workspace: &Workspace, runner: &Runner) -> Result<()> {
    let mut checks = Checks::default();
    checks.check("WASM Rust target", target(workspace, runner, WASM_TARGET));
    checks.check(
        "wasm-bindgen pin",
        probe(workspace, runner, "wasm-bindgen", &["--version"]).and_then(|version| {
            ensure!(
                version.trim() == BINDGEN_VERSION,
                "expected {BINDGEN_VERSION}, found {}",
                version.trim()
            );
            Ok(version)
        }),
    );
    checks.check(
        "Python",
        probe(workspace, runner, "python3", &["--version"]),
    );
    checks.check("Bash", probe(workspace, runner, "bash", &["--version"]));
    checks.finish()
}

pub fn repository(workspace: &Workspace, runner: &Runner) -> Result<()> {
    if runner.dry_run {
        println!(
            "NOT_RUN: doctor probes Rust/components, WASM, Python, Node/npm, browser and native Linux prerequisites"
        );
        return Ok(());
    }
    let mut checks = Checks::default();
    checks.check("Rust", probe(workspace, runner, "rustc", &["--version"]));
    checks.check("Cargo", probe(workspace, runner, "cargo", &["--version"]));
    checks.check("Git", probe(workspace, runner, "git", &["--version"]));
    checks.check(
        "rustfmt + clippy",
        probe(
            workspace,
            runner,
            "rustup",
            &["component", "list", "--installed"],
        )
        .and_then(|components| {
            ensure!(
                components.contains("rustfmt") && components.contains("clippy"),
                "rustfmt and clippy are required"
            );
            Ok("installed".into())
        }),
    );
    checks.check(
        "Web build prerequisites",
        web(workspace, runner).map(|()| "available".into()),
    );
    checks.check("Node", probe(workspace, runner, "node", &["--version"]));
    checks.check("npm", probe(workspace, runner, "npm", &["--version"]));
    checks.check("Playwright + browser", browser(workspace, runner));
    if cfg!(target_os = "linux") {
        checks.check(
            "graybox libudev",
            probe(
                workspace,
                runner,
                "pkg-config",
                &["--modversion", "libudev"],
            ),
        );
    }
    println!(
        "NOT_RUN: Android/iOS prerequisites; use `cargo xtask doctor mobile --platform android|ios`"
    );
    println!(
        "NOT_RUN: browser interaction and native device runtime; doctor is environment evidence only"
    );
    checks.finish()
}

pub fn browser(workspace: &Workspace, runner: &Runner) -> Result<String> {
    probe(
        workspace,
        runner,
        "node",
        &[
            "--input-type=module",
            "-e",
            "import { chromium } from '@playwright/test'; import { existsSync } from 'node:fs'; const path = process.env.CHROME_PATH || chromium.executablePath(); if (!existsSync(path)) throw new Error('Browser missing; provision Chromium or set CHROME_PATH'); console.log(path);",
        ],
    )
}
