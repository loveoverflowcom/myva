#!/usr/bin/env python3
"""Report availability, never interpret a connected device as a test pass."""
import datetime
import json
import platform
import shutil
import subprocess


def run(*command):
    if not shutil.which(command[0]):
        return {"status": "missing", "output": ""}
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=30)
        return {"status": result.returncode, "output": (result.stdout + result.stderr).strip()}
    except subprocess.TimeoutExpired:
        return {"status": "timeout", "output": ""}


adb = run("adb", "devices", "-l")
devices = [
    line for line in adb["output"].splitlines()
    if len(line.split()) >= 2 and line.split()[1] == "device"
]
# Serial names alone cannot prove physical hardware; users must verify the model.
physical_candidates = [line for line in devices if not line.startswith("emulator-")]
xcode = run("xcodebuild", "-version")
report = {
    "recorded_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "host": platform.platform(),
    "rust": run("rustc", "+1.97.1", "--version"),
    "rust_targets": run("rustup", "+1.97.1", "target", "list", "--installed"),
    "android": {
        "gate": "NOT_RUN" if physical_candidates else "BLOCKED",
        "reason": "Verify physical hardware, then run the device matrix" if physical_candidates else "No authorized physical Android device detected",
        "adb": adb,
        "candidate_physical_device_count": len(physical_candidates),
        "sdk": run("sdkmanager", "--list_installed"),
    },
    "ios": {
        "gate": "NOT_RUN" if xcode["status"] == 0 else "BLOCKED",
        "reason": "Physical iPhone and signing must be checked manually" if xcode["status"] == 0 else "Xcode toolchain unavailable",
        "xcode": xcode,
    },
    "measurements": {"fps": None, "memory": None, "entry_exit_cycles": None, "background_resume_cycles": None},
}
print(json.dumps(report, ensure_ascii=False, indent=2))
