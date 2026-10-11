mod cli;
mod command;
mod config;
mod doctor;
mod impact;
mod mobile;
mod plan;
mod process;
mod web;

use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Parser;

use crate::{
    cli::{App, Cli, Doctor, Task},
    config::{DEFAULT_PORT, Workspace},
    impact::Selection,
    process::{Blocked, Runner},
};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let status = if error.downcast_ref::<Blocked>().is_some() {
                "BLOCKED"
            } else {
                "FAIL"
            };
            eprintln!("{status}: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let workspace = Workspace::discover()?;
    let runner = Runner::new(cli.dry_run, cli.jobs);
    runner.install_signals()?;
    eprintln!(
        "MyVa xtask · {}{}",
        workspace.root.display(),
        if cli.dry_run {
            " · dry-run (NOT_RUN)"
        } else {
            ""
        }
    );
    let steps = match cli.task {
        Task::Build { target } => plan::build(&workspace, &target)?,
        Task::Run { target } => plan::run(&workspace, &target),
        Task::App {
            target: App::Web(address),
        } => return web::app(&workspace, &runner, address.port),
        Task::Verify { target } => plan::verify(&workspace, &target),
        Task::Doctor { target: None } => return doctor::repository(&workspace, &runner),
        Task::Doctor {
            target: Some(Doctor::Mobile { platform }),
        } => return mobile::doctor(&workspace, &runner, platform),
        Task::Devices { platform } => return mobile::devices(&workspace, &runner, platform),
        Task::Fmt { check } => vec![plan::format(&workspace, check)],
        Task::Preflight { changed_only, base } => {
            let selection = if changed_only {
                let paths = impact::changed_paths(&workspace, &runner, &base)?;
                let metadata = impact::load_metadata(&workspace, &runner)?;
                eprintln!(
                    "Diff from merge-base({base}, HEAD) + staged/unstaged/untracked: {} paths",
                    paths.len()
                );
                impact::select(&workspace, &metadata, &paths)?
            } else {
                Selection::all("full preflight")
            };
            eprintln!(
                "Selected impact: {}\nPackages: {}",
                selection.reason,
                if selection.full {
                    "workspace".into()
                } else {
                    selection
                        .packages
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            );
            plan::preflight(&workspace, &selection, test_port()?)
        }
        Task::Ci => plan::preflight(
            &workspace,
            &Selection::all("CI/full preflight"),
            test_port()?,
        ),
    };
    plan::execute(&workspace, &runner, &steps)
}

fn test_port() -> Result<u16> {
    let port = std::env::var("MYVA_WEB_TEST_PORT").map_or(Ok(DEFAULT_PORT), |port| {
        port.parse::<u16>()
            .context("MYVA_WEB_TEST_PORT must be an integer port")
    })?;
    anyhow::ensure!(port != 0, "MYVA_WEB_TEST_PORT must be in 1..65535");
    Ok(port)
}
