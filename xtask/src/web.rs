use std::{
    io::{IsTerminal, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;

use crate::{
    command::CommandSpec,
    config::{HOST, Workspace},
    doctor,
    process::{OwnedChild, Runner, blocked},
};

pub fn url(port: u16) -> String {
    format!("http://{HOST}:{port}/")
}

pub fn ensure_port_free(port: u16) -> Result<()> {
    TcpListener::bind((HOST, port)).map(drop).map_err(|error| {
        blocked(format!(
            "{HOST}:{port} is unavailable: {error}; choose --port; no process was stopped"
        ))
    })
}

pub struct Server {
    child: OwnedChild,
    pub log: PathBuf,
    pub port: u16,
}

impl Server {
    pub fn start(workspace: &Workspace, runner: &Runner, port: u16) -> Result<Self> {
        ensure_port_free(port)?;
        let expected = workspace.require_web_build()?;
        let token = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        );
        let log = workspace.evidence().join(format!("web-{port}.log"));
        let spec = CommandSpec::new(
            workspace,
            "python3",
            &[
                "-u",
                "scripts/serve-web-shell.py",
                "--host",
                HOST,
                "--port",
                &port.to_string(),
                "--ready-token",
                &token,
            ],
        );
        let child = runner.logged(&spec, &log)?;
        let mut server = Self { child, log, port };
        let started = Instant::now();
        loop {
            runner.check_cancelled()?;
            if let Some(status) = server.child.try_wait()? {
                bail!(
                    "web server exited before readiness: {status}\n{}",
                    server.diagnostics()
                );
            }
            // The token proves this is our child, not an unrelated server that won a bind race.
            if response_matches(port, &expected, Some(&token)).is_ok() {
                break;
            }
            if started.elapsed() >= Duration::from_secs(20) {
                bail!(
                    "web readiness timed out at {}bundle-report.json\n{}",
                    url(port),
                    server.diagnostics()
                );
            }
            std::thread::sleep(Duration::from_millis(75));
        }
        Ok(server)
    }

    fn diagnostics(&self) -> String {
        let text = std::fs::read_to_string(&self.log).unwrap_or_default();
        let tail: String = text
            .chars()
            .rev()
            .take(4000)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        format!("Log: {}\n{tail}", self.log.display())
    }

    fn dashboard(&self, workspace: &Workspace) {
        let (color, reset) = if std::io::stdout().is_terminal() {
            ("\x1b[1;36m", "\x1b[0m")
        } else {
            ("", "")
        };
        println!(
            "\n{color}MyVa · web ready{reset}\n  Play: {}\n  Readiness/build: {}bundle-report.json\n  Workspace: {}\n  PID: {}\n  Logs: {}\n  Open: cargo xtask app web --port {}\n  Stop: Ctrl+C (owned server only)\n  Restart: cargo xtask run web --port {}\n",
            url(self.port),
            url(self.port),
            workspace.root.display(),
            self.child.id(),
            self.log.display(),
            self.port,
            self.port
        );
    }

    pub fn serve(mut self, workspace: &Workspace, runner: &Runner) -> Result<()> {
        self.dashboard(workspace);
        match self.child.wait(runner, None) {
            Ok(status) => bail!(
                "web server exited unexpectedly: {status}\n{}",
                self.diagnostics()
            ),
            Err(_error) if runner.cancelled.load(std::sync::atomic::Ordering::Relaxed) => {
                println!("Stopped owned web server (PID {}).", self.child.id());
                drop(self);
                // A requested foreground shutdown is successful, but not a verification PASS.
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}

fn response_matches(port: u16, expected: &str, token: Option<&str>) -> Result<()> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(500))?;
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;
    stream.set_write_timeout(Some(Duration::from_millis(500)))?;
    stream.write_all(
        format!(
            "GET /bundle-report.json HTTP/1.0\r\nHost: {HOST}:{port}\r\nConnection: close\r\n\r\n"
        )
        .as_bytes(),
    )?;
    let mut response = String::new();
    stream.take(4 * 1024 * 1024).read_to_string(&mut response)?;
    let (headers, body) = response
        .split_once("\r\n\r\n")
        .context("invalid HTTP readiness response")?;
    ensure!(
        headers
            .lines()
            .next()
            .is_some_and(|line| line.split_whitespace().nth(1) == Some("200")),
        "web readiness did not return HTTP 200"
    );
    if let Some(token) = token {
        ensure!(
            headers.lines().any(|line| line
                .split_once(':')
                .is_some_and(|(name, value)| name.eq_ignore_ascii_case("X-Myva-Xtask")
                    && value.trim() == token)),
            "readiness belongs to another process"
        );
    }
    ensure!(
        body.trim() == expected.trim(),
        "server bundle report differs from this workspace's build"
    );
    Ok(())
}

pub fn app(workspace: &Workspace, runner: &Runner, port: u16) -> Result<()> {
    if runner.dry_run {
        println!(
            "NOT_RUN: require existing bundle + matching server at {}bundle-report.json; open browser at {}",
            url(port),
            url(port)
        );
        return Ok(());
    }
    let report = workspace.require_web_build()?;
    response_matches(port, &report, None).map_err(|error| blocked(format!(
        "existing MyVa server is not ready at {}: {error}; run `cargo xtask run web --port {port}`", url(port))))?;
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else {
        return Err(blocked(
            "browser opening currently supports Linux/macOS; open the printed loopback URL manually",
        ));
    };
    runner.run(&CommandSpec::new(workspace, program, &[&url(port)]))
}

pub fn tests(workspace: &Workspace, runner: &Runner, port: u16, args: &[String]) -> Result<()> {
    doctor::browser(workspace, runner)
        .map_err(|error| blocked(format!("browser prerequisites: {error:#}")))?;
    let mut server = Server::start(workspace, runner, port)?;
    let report = workspace.path("target/web-evidence/test-results.json");
    if report.exists() {
        std::fs::remove_file(&report).context("remove stale Playwright report")?;
    }
    let command = CommandSpec::new(
        workspace,
        "node",
        &["node_modules/@playwright/test/cli.js", "test"],
    )
    .args(args.iter().cloned())
    .env("MYVA_XTASK_WEB_SERVER", "1")
    .env("MYVA_WEB_TEST_PORT", port.to_string());
    runner
        .run(&command)
        .with_context(|| format!("web E2E failed\n{}", server.diagnostics()))?;
    ensure!(
        server.child_status_ok()?,
        "owned web server exited during browser tests"
    );
    let report: Value = serde_json::from_str(
        &std::fs::read_to_string(report)
            .context("Playwright did not write a fresh JSON execution report")?,
    )?;
    validate_test_report(&report)?;
    drop(server);
    Ok(())
}

impl Server {
    fn child_status_ok(&mut self) -> Result<bool> {
        Ok(self.child.try_wait()?.is_none())
    }
}

fn validate_test_report(report: &Value) -> Result<()> {
    let stats = &report["stats"];
    ensure!(
        stats["expected"].as_u64().is_some_and(|count| count > 0),
        "no passing browser tests executed"
    );
    for key in ["unexpected", "skipped", "flaky"] {
        ensure!(
            stats[key].as_u64() == Some(0),
            "browser report has {key} tests or incomplete statistics"
        );
    }
    ensure!(
        report["errors"]
            .as_array()
            .is_some_and(|errors| errors.is_empty()),
        "browser report contains global errors"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_skipped_and_stale_browser_results_cannot_pass() {
        let report = serde_json::json!({"stats": {"expected": 2, "unexpected": 0, "skipped": 0, "flaky": 0}, "errors": []});
        assert!(validate_test_report(&report).is_ok());
        for key in ["expected", "skipped", "unexpected", "flaky"] {
            let mut bad = report.clone();
            bad["stats"][key] = serde_json::json!(if key == "expected" { 0 } else { 1 });
            assert!(validate_test_report(&bad).is_err());
        }
        assert!(validate_test_report(&serde_json::json!({})).is_err());
    }

    #[test]
    fn occupied_port_is_rejected_without_touching_listener() {
        let listener = TcpListener::bind((HOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(ensure_port_free(port).is_err());
        assert!(TcpStream::connect((HOST, port)).is_ok());
    }

    #[test]
    fn actual_server_readiness_identity_and_drop_shutdown() {
        let root = tempfile::tempdir().unwrap();
        let w = Workspace {
            root: root.path().to_owned(),
        };
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
            let path = w.web_dir().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "{}").unwrap();
        }
        std::fs::create_dir_all(w.path("scripts")).unwrap();
        std::fs::write(
            w.path("scripts/serve-web-shell.py"),
            include_str!("../../scripts/serve-web-shell.py"),
        )
        .unwrap();
        let runner = Runner::new(false, None);
        // The ephemeral port is released before Python binds. Other parallel socket
        // tests can acquire it in between; retry only that availability race, never a
        // readiness/identity/child failure. The production CLI reports an occupied port.
        let (port, server) = (0..8)
            .find_map(|_| {
                let listener = TcpListener::bind((HOST, 0)).unwrap();
                let port = listener.local_addr().unwrap().port();
                drop(listener);
                match Server::start(&w, &runner, port) {
                    Ok(server) => Some((port, server)),
                    Err(error) if error.downcast_ref::<crate::process::Blocked>().is_some() => None,
                    Err(error) => panic!("{error:#}"),
                }
            })
            .expect("could not acquire a test port after eight attempts");
        assert!(response_matches(port, "{}", None).is_ok());
        assert!(response_matches(port, "{}", Some("foreign-token")).is_err());
        assert!(response_matches(port, "different-report", None).is_err());
        drop(server);
        assert!(ensure_port_free(port).is_ok());
    }
}
