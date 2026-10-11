//! Foreground ownership only: no PID files, daemon discovery, or killing by port/name.
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    num::NonZeroUsize,
    process::{Child, Command, ExitStatus, Output, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};

use crate::command::CommandSpec;

#[derive(Debug)]
pub struct Blocked(pub String);

impl std::fmt::Display for Blocked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Blocked {}

pub fn blocked(message: impl Into<String>) -> anyhow::Error {
    Blocked(message.into()).into()
}

pub struct Runner {
    pub dry_run: bool,
    pub jobs: Option<NonZeroUsize>,
    pub cancelled: Arc<AtomicBool>,
}

impl Runner {
    pub fn new(dry_run: bool, jobs: Option<NonZeroUsize>) -> Self {
        Self {
            dry_run,
            jobs,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn install_signals(&self) -> Result<()> {
        #[cfg(unix)]
        for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
            signal_hook::flag::register(signal, self.cancelled.clone())
                .context("register shutdown signal")?;
        }
        Ok(())
    }

    pub fn check_cancelled(&self) -> Result<()> {
        ensure!(
            !self.cancelled.load(Ordering::Relaxed),
            "interrupted; stopping owned processes"
        );
        Ok(())
    }

    fn command(&self, spec: &CommandSpec) -> Command {
        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .envs(&spec.env)
            .stdin(Stdio::null());
        if let Some(jobs) = self.jobs {
            command.env("CARGO_BUILD_JOBS", jobs.to_string());
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        command
    }

    fn spawn(&self, command: &mut Command, spec: &CommandSpec) -> Result<OwnedChild> {
        self.check_cancelled()?;
        command.spawn().map(OwnedChild).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                blocked(format!(
                    "missing executable {}; provision prerequisites first",
                    spec.program
                ))
            } else {
                anyhow::Error::new(error).context(format!("start {}", spec.display()))
            }
        })
    }

    pub fn run(&self, spec: &CommandSpec) -> Result<()> {
        ensure!(
            !self.dry_run,
            "internal error: attempted execution during dry-run"
        );
        eprintln!("  $ {}", spec.display());
        let mut command = self.command(spec);
        if let Some(path) = &spec.stdout {
            std::fs::create_dir_all(path.parent().context("stdout path needs a parent")?)?;
            command
                .stdout(File::create(path).with_context(|| format!("create {}", path.display()))?);
        }
        let mut child = self.spawn(&mut command, spec)?;
        let status = child.wait(self, None)?;
        ensure!(status.success(), "{} exited with {status}", spec.display());
        Ok(())
    }

    /// Read-only inspection, also used to compute a changed-only dry-run. File-backed capture
    /// avoids pipe deadlocks on cancellation or grandchildren retaining stdout.
    pub fn output(&self, spec: &CommandSpec, timeout: Duration) -> Result<Output> {
        let mut stdout = tempfile::tempfile().context("create stdout capture")?;
        let mut stderr = tempfile::tempfile().context("create stderr capture")?;
        let mut command = self.command(spec);
        command
            .stdout(stdout.try_clone()?)
            .stderr(stderr.try_clone()?);
        let mut child = self.spawn(&mut command, spec)?;
        let status = child.wait(self, Some(timeout))?;
        // Clean up descendants before reading their shared output files.
        drop(child);
        Ok(Output {
            status,
            stdout: read_capture(&mut stdout)?,
            stderr: read_capture(&mut stderr)?,
        })
    }

    pub fn text(&self, spec: &CommandSpec) -> Result<String> {
        let output = self.output(spec, Duration::from_secs(30))?;
        ensure!(
            output.status.success(),
            "{} exited with {}: {}{}",
            spec.display(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).context("command output must be UTF-8")
    }

    pub fn logged(&self, spec: &CommandSpec, log: &std::path::Path) -> Result<OwnedChild> {
        ensure!(
            !self.dry_run,
            "internal error: attempted server startup during dry-run"
        );
        std::fs::create_dir_all(log.parent().context("log needs a parent directory")?)?;
        let file = File::create(log).with_context(|| format!("create {}", log.display()))?;
        let mut command = self.command(spec);
        command.stdout(file.try_clone()?).stderr(file);
        self.spawn(&mut command, spec)
    }
}

fn read_capture(file: &mut File) -> Result<Vec<u8>> {
    const LIMIT: u64 = 4 * 1024 * 1024;
    ensure!(
        file.metadata()?.len() <= LIMIT,
        "inspection output exceeds 4 MiB"
    );
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= LIMIT,
        "inspection output exceeds 4 MiB"
    );
    Ok(bytes)
}

pub struct OwnedChild(Child);

impl OwnedChild {
    pub fn id(&self) -> u32 {
        self.0.id()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        self.0.try_wait().context("poll owned process")
    }

    pub fn wait(&mut self, runner: &Runner, timeout: Option<Duration>) -> Result<ExitStatus> {
        let start = Instant::now();
        loop {
            runner.check_cancelled()?;
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            if timeout.is_some_and(|limit| start.elapsed() >= limit) {
                bail!("owned process timed out after {:?}", timeout.unwrap());
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

#[cfg(unix)]
fn group_signal(pid: u32, signal: &str) -> bool {
    Command::new("/bin/kill")
        .args([signal, "--", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            let pid = self.0.id();
            if group_signal(pid, "-0") {
                group_signal(pid, "-TERM");
                let start = Instant::now();
                while start.elapsed() < Duration::from_millis(750) && group_signal(pid, "-0") {
                    let _ = self.0.try_wait();
                    std::thread::sleep(Duration::from_millis(20));
                }
                group_signal(pid, "-KILL");
            }
        }
        // Child::kill alone is the fallback on non-Unix; Unix kills only the group we created.
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::config::Workspace;

    fn spec(root: &std::path::Path, script: &str) -> CommandSpec {
        CommandSpec::new(
            &Workspace {
                root: root.to_owned(),
            },
            "sh",
            &["-c", script],
        )
    }

    #[test]
    fn propagates_failure_and_captures_large_output_without_deadlock() {
        let root = tempfile::tempdir().unwrap();
        let runner = Runner::new(false, None);
        assert!(runner.run(&spec(root.path(), "exit 23")).is_err());
        let output = runner
            .output(
                &spec(root.path(), "head -c 200000 /dev/zero"),
                Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(output.stdout.len(), 200000);
    }

    #[test]
    fn timeout_and_cancellation_cleanup_owned_descendants_only() {
        let root = tempfile::tempdir().unwrap();
        let runner = Runner::new(false, None);
        let foreign = runner
            .logged(
                &spec(root.path(), "sleep 30"),
                &root.path().join("foreign.log"),
            )
            .unwrap();
        let command = spec(root.path(), "sleep 30 & echo $! > descendant; wait");
        assert!(runner.output(&command, Duration::from_millis(500)).is_err());
        let pid = std::fs::read_to_string(root.path().join("descendant")).unwrap();
        let state = Command::new("ps")
            .args(["-o", "stat=", "-p", pid.trim()])
            .output()
            .unwrap();
        let state = String::from_utf8_lossy(&state.stdout);
        assert!(
            state.trim().is_empty() || state.trim().starts_with('Z'),
            "descendant survived: {state}"
        );
        assert!(
            group_signal(foreign.id(), "-0"),
            "unrelated process was stopped"
        );
        let mut child = runner
            .logged(
                // A direct child is reaped here. The descendant fixture above permits
                // terminated orphans awaiting init reaping; kill -0 sees those zombies too.
                &spec(root.path(), "exec sleep 30"),
                &root.path().join("cancel.log"),
            )
            .unwrap();
        runner.cancelled.store(true, Ordering::Relaxed);
        assert!(child.wait(&runner, None).is_err());
        let pid = child.id();
        drop(child);
        assert!(!group_signal(pid, "-0"));
    }

    #[test]
    fn success_does_not_leak_background_children() {
        let root = tempfile::tempdir().unwrap();
        let runner = Runner::new(false, None);
        runner
            .run(&spec(root.path(), "sleep 30 & echo $! > descendant"))
            .unwrap();
        let pid = std::fs::read_to_string(root.path().join("descendant")).unwrap();
        let state = Command::new("ps")
            .args(["-o", "stat=", "-p", pid.trim()])
            .output()
            .unwrap();
        let state = String::from_utf8_lossy(&state.stdout);
        assert!(state.trim().is_empty() || state.trim().starts_with('Z'));
    }

    #[test]
    fn dry_run_cannot_execute_and_job_override_reaches_children() {
        let root = tempfile::tempdir().unwrap();
        let runner = Runner::new(true, None);
        assert!(runner.run(&spec(root.path(), "touch unexpected")).is_err());
        assert!(!root.path().join("unexpected").exists());
        let runner = Runner::new(false, NonZeroUsize::new(1));
        assert_eq!(
            runner
                .text(&spec(root.path(), "printf '%s' \"$CARGO_BUILD_JOBS\""))
                .unwrap(),
            "1"
        );
    }
}
