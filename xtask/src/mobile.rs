use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    time::Duration,
};

use anyhow::{Result, ensure};

use crate::{
    cli::{NativeMode, Platform},
    config::{ANDROID_TARGET, IOS_TARGETS, NDK_VERSION, Workspace},
    doctor::{Checks, probe, target},
    process::{Runner, blocked},
};

fn sdk_root() -> Result<PathBuf> {
    std::env::var_os("ANDROID_HOME")
        .or_else(|| std::env::var_os("ANDROID_SDK_ROOT"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Android/Sdk")))
        .ok_or_else(|| blocked("set ANDROID_HOME or ANDROID_SDK_ROOT"))
}

/// A build checks only its prerequisites; doctor additionally checks device tooling.
pub fn android(workspace: &Workspace, runner: &Runner, mode: Option<NativeMode>) -> Result<()> {
    let mut checks = Checks::default();
    checks.check(
        "Android Rust target",
        target(workspace, runner, ANDROID_TARGET),
    );
    checks.check("Bash", probe(workspace, runner, "bash", &["--version"]));
    let host = match std::env::consts::OS {
        "linux" => "linux-x86_64",
        "macos" => "darwin-x86_64",
        other => {
            return Err(blocked(format!(
                "Android build script supports Linux/macOS, found {other}"
            )));
        }
    };
    let sdk = sdk_root()?;
    let ndk = std::env::var_os("ANDROID_NDK_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| sdk.join("ndk").join(NDK_VERSION));
    let compiler = ndk.join(format!(
        "toolchains/llvm/prebuilt/{host}/bin/aarch64-linux-android26-clang"
    ));
    checks.check(
        "NDK compiler (API 26)",
        probe(
            workspace,
            runner,
            &compiler.to_string_lossy(),
            &["--version"],
        ),
    );
    checks.path(
        "NDK archiver",
        &ndk.join(format!("toolchains/llvm/prebuilt/{host}/bin/llvm-ar")),
    );
    if mode.is_none() || matches!(mode, Some(NativeMode::Apk | NativeMode::Cmp)) {
        let java = std::env::var_os("JAVA_HOME").map(|home| PathBuf::from(home).join("bin/java"));
        let program = java
            .as_ref()
            .map_or_else(|| "java".into(), |p| p.to_string_lossy().into_owned());
        checks.check(
            "JDK 17",
            probe(workspace, runner, &program, &["-version"]).and_then(|version| {
                ensure!(
                    version.contains("version \"17.") || version.contains("openjdk 17."),
                    "prototype pins JDK 17; found {}",
                    version.lines().next().unwrap_or("unknown")
                );
                Ok(version)
            }),
        );
        checks.path(
            "Android SDK platform 37.0",
            &sdk.join("platforms/android-37.0/android.jar"),
        );
        for tool in ["aapt2", "zipalign", "apksigner"] {
            checks.path(tool, &sdk.join("build-tools/36.0.0").join(tool));
        }
        for file in [
            "gradlew",
            "gradle/wrapper/gradle-wrapper.jar",
            "gradle/wrapper/gradle-wrapper.properties",
        ] {
            checks.path(
                "Gradle wrapper",
                &workspace.path("prototypes/native/android").join(file),
            );
        }
    }
    if mode.is_none() {
        checks.check(
            "ADB",
            probe(workspace, runner, &adb_program()?, &["version"]),
        );
        let manager = sdk.join("cmdline-tools/latest/bin/sdkmanager");
        checks.path("Android SDK manager (diagnostic only)", &manager);
    }
    println!(
        "NOT_RUN: Android runtime/device matrix; compilation and device discovery do not verify CMP lifecycle"
    );
    checks.finish()
}

pub fn doctor(workspace: &Workspace, runner: &Runner, platform: Platform) -> Result<()> {
    if runner.dry_run {
        println!("NOT_RUN: {platform:?} prerequisite probes; no tools or devices started");
        return Ok(());
    }
    match platform {
        Platform::Android => android(workspace, runner, None),
        Platform::Ios => {
            let mut checks = Checks::default();
            checks.check(
                "macOS host",
                if cfg!(target_os = "macos") {
                    Ok("macOS".into())
                } else {
                    Err(blocked("iOS requires macOS/Xcode"))
                },
            );
            for name in IOS_TARGETS {
                checks.check("iOS Rust target", target(workspace, runner, name));
            }
            checks.check(
                "Xcode",
                probe(workspace, runner, "xcodebuild", &["-version"]),
            );
            checks.check(
                "active Xcode",
                probe(workspace, runner, "xcode-select", &["-p"]),
            );
            checks.check(
                "iPhone SDK",
                probe(
                    workspace,
                    runner,
                    "xcrun",
                    &["--sdk", "iphoneos", "--show-sdk-path"],
                ),
            );
            checks.check(
                "simulator tooling",
                probe(workspace, runner, "xcrun", &["simctl", "help"]),
            );
            checks.check(
                "iOS build/packaging",
                Err(blocked("not implemented in prototypes/native")),
            );
            println!(
                "NOT_RUN: Apple signing, physical iPhone, CMP embedding and lifecycle; check manually on the target host"
            );
            checks.finish()
        }
    }
}

fn adb_program() -> Result<String> {
    let path = sdk_root()?.join("platform-tools/adb");
    Ok(if path.is_file() {
        path.to_string_lossy().into_owned()
    } else {
        "adb".into()
    })
}

pub fn devices(workspace: &Workspace, runner: &Runner, platform: Option<Platform>) -> Result<()> {
    if runner.dry_run {
        println!("NOT_RUN: device discovery for {platform:?}; no install, boot or launch");
        return Ok(());
    }
    let mut checks = Checks::default();
    if platform.is_none() || platform == Some(Platform::Android) {
        checks.check(
            "Android device enumeration",
            (|| -> Result<String> {
                let port: u16 = std::env::var("ANDROID_ADB_SERVER_PORT")
                    .unwrap_or_else(|_| "5037".into())
                    .parse()?;
                let output = android_devices(port)?;
                runner.check_cancelled()?;
                println!("List of devices attached\n{output}");
                Ok("listed; zero devices is a valid discovery result, never a runtime PASS".into())
            })(),
        );
    }
    if platform.is_none() || platform == Some(Platform::Ios) {
        if cfg!(target_os = "macos") {
            checks.check(
                "iOS simulator enumeration",
                probe(
                    workspace,
                    runner,
                    "xcrun",
                    &["simctl", "list", "devices", "available"],
                )
                .map(|output| {
                    println!("{output}");
                    "listed".into()
                }),
            );
        } else if platform == Some(Platform::Ios) {
            checks.check(
                "iOS simulator enumeration",
                Err(blocked("requires macOS/Xcode")),
            );
        } else {
            println!("NOT_RUN: iOS simulator discovery requires macOS/Xcode");
        }
    }
    println!("NOT_RUN: all device runtime tests; no device was installed, booted or launched");
    checks.finish()
}

fn android_devices(port: u16) -> Result<String> {
    ensure!(port != 0, "ANDROID_ADB_SERVER_PORT must be in 1..65535");
    let mut socket = TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(500))
        .map_err(|error| blocked(format!("ADB server unavailable on loopback:{port}: {error}; start it explicitly before listing devices")))?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    socket.set_write_timeout(Some(Duration::from_secs(2)))?;
    // Smart socket's read-only host:devices-l service cannot auto-start, kill or replace
    // a shared daemon (unlike the adb CLI's version negotiation). Never select a transport.
    socket.write_all(b"000ehost:devices-l")?;
    read_adb_response(&mut socket)
}

fn read_adb_response(reader: &mut impl Read) -> Result<String> {
    let mut status = [0; 4];
    reader.read_exact(&mut status)?;
    ensure!(
        matches!(&status, b"OKAY" | b"FAIL"),
        "invalid ADB smart socket status"
    );
    let mut length = [0; 4];
    reader.read_exact(&mut length)?;
    // Four hex digits bound the response to 64 KiB. Truncated/invalid data fails closed.
    let length = usize::from_str_radix(std::str::from_utf8(&length)?, 16)?;
    let mut payload = vec![0; length];
    reader.read_exact(&mut payload)?;
    let payload = String::from_utf8(payload)?;
    ensure!(
        &status == b"OKAY",
        "ADB refused device discovery: {payload}"
    );
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adb_discovery_handles_empty_devices_and_rejects_failed_or_truncated_data() {
        assert_eq!(read_adb_response(&mut &b"OKAY0000"[..]).unwrap(), "");
        assert_eq!(
            read_adb_response(&mut &b"OKAY000babc\tdevice\n"[..]).unwrap(),
            "abc\tdevice\n"
        );
        for mut response in [
            &b"FAIL0004oops"[..],
            &b"OKAY0005abc"[..],
            &b"OKAYzzzz"[..],
            &b"junk"[..],
        ] {
            assert!(read_adb_response(&mut response).is_err());
        }
    }

    #[test]
    fn adb_socket_sends_only_read_only_host_request() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let thread = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = [0; 18];
            socket.read_exact(&mut request).unwrap();
            assert_eq!(&request, b"000ehost:devices-l");
            socket.write_all(b"OKAY0000").unwrap();
        });
        assert_eq!(android_devices(port).unwrap(), "");
        thread.join().unwrap();
    }
}
