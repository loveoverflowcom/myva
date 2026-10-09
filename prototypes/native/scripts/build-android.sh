#!/usr/bin/env bash
set -euo pipefail
probe_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
mode="${1:-apk}"
case "$mode" in check|build|apk|cmp) ;; *) echo 'Usage: build-android.sh [check|build|apk|cmp]' >&2; exit 2 ;; esac

export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
ndk_dir="${ANDROID_NDK_ROOT:-$ANDROID_HOME/ndk/27.0.12077973}"
case "$(uname -s)" in
  Linux) ndk_host=linux-x86_64 ;;
  Darwin) ndk_host=darwin-x86_64 ;;
  *) echo 'Run this script on Linux/macOS with Android NDK 27.0.12077973.' >&2; exit 2 ;;
esac
ndk_bin="$ndk_dir/toolchains/llvm/prebuilt/$ndk_host/bin"
if [[ ! -x "$ndk_bin/aarch64-linux-android26-clang" ]]; then
  echo "Missing pinned NDK compiler: $ndk_bin/aarch64-linux-android26-clang" >&2
  exit 2
fi
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$ndk_bin/aarch64-linux-android26-clang"
export CC_aarch64_linux_android="$ndk_bin/aarch64-linux-android26-clang"
export CXX_aarch64_linux_android="$ndk_bin/aarch64-linux-android26-clang++"
export AR_aarch64_linux_android="$ndk_bin/llvm-ar"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$probe_dir/target}"
cargo_action=rustc
# NDK r27 does not enable 16 KiB ELF alignment by default. Set it only on
# the final native library, preserving the dependency compilation cache.
extra_args=(-- -C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-z,common-page-size=16384)
if [[ "$mode" == check ]]; then cargo_action=check; extra_args=(); fi
cargo +1.97.1 "$cargo_action" --locked --manifest-path "$probe_dir/Cargo.toml" \
  --lib --target aarch64-linux-android "${extra_args[@]}"
if [[ "$mode" == check ]]; then exit 0; fi
mkdir -p "$probe_dir/android/app/src/main/jniLibs/arm64-v8a"
cp "$CARGO_TARGET_DIR/aarch64-linux-android/debug/libmyva_native_probe.so" \
  "$probe_dir/android/app/src/main/jniLibs/arm64-v8a/"
if [[ "$mode" == apk ]]; then
  "$probe_dir/android/gradlew" -p "$probe_dir/android" :app:assembleDebug
elif [[ "$mode" == cmp ]]; then
  "$probe_dir/android/gradlew" -p "$probe_dir/android" :cmp-shell:assembleDebug
fi
