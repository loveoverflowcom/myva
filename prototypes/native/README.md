# Native feasibility baseline (#10)

**Trạng thái: standalone baseline và CMP Android phương án B thử nghiệm; device gate chưa đạt.** Cảnh Bevy có camera 2D và sprite màu xanh di chuyển theo ngón tay đầu tiên. Nhả/hủy pointer hoặc đưa app vào nền sẽ xóa ý định di chuyển. Không có asset bên ngoài, audio hay game state production.

## Phiên bản và build Android

Pin: Rust **1.97.1**, Bevy **0.20.0**, `winit` **0.30.13** (lockfile), Android NDK **27.0.12077973**, Gradle **9.7.0**, AGP **9.3.1**, GameActivity **4.4.0**, JDK **17**, SDK platform **37.0**, build-tools **36.0.0**. ARM64, minSdk 26, targetSdk 37. GameActivity version lấy từ mobile example của đúng tag Bevy; không thay riêng Java AAR khi chưa đối chiếu `android-activity`.

```bash
rustup toolchain install 1.97.1 --profile minimal
rustup +1.97.1 target add aarch64-linux-android
sdkmanager 'platforms;android-37.0' 'build-tools;36.0.0' 'ndk;27.0.12077973' 'platform-tools'
# Set ANDROID_HOME and JAVA_HOME if SDK/JDK are not already configured.
prototypes/native/scripts/build-android.sh check
prototypes/native/scripts/build-android.sh apk
```

Script mặc định tìm SDK trong `$HOME/Android/Sdk`; override `ANDROID_HOME` hoặc `ANDROID_NDK_ROOT` nếu cài ở nơi khác. Không cần `cargo-ndk`: script đặt linker, C/C++ compiler và archiver của NDK, dùng Cargo lockfile và giới hạn hai job. `check` chưa link; `build` tạo `.so`; `apk` mới đóng gói debug APK. Bản debug không phải build phát hành/signing production. Feature chỉ gồm renderer sprite 2D/window/lifecycle cần cho baseline; audio, UI, gilrs và renderer 3D không được bật.

Linker dùng `max-page-size=16384` và `common-page-size=16384` cho NDK r27 theo [hướng dẫn Android](https://developer.android.com/guide/practices/page-sizes). Kiểm tra ELF/ZIP alignment không thay thế thử runtime trên thiết bị 4 KiB/16 KiB.

Sau khi cắm **thiết bị thật** đã cho phép USB debugging, chọn serial tương ứng:

```bash
adb devices -l
adb -s "$DEVICE_SERIAL" install -r prototypes/native/android/app/build/outputs/apk/debug/app-debug.apk
adb -s "$DEVICE_SERIAL" shell am start -n com.myva.nativeprobe/.MainActivity
adb -s "$DEVICE_SERIAL" logcat -v threadtime -s RustStdoutStderr bevy myva
```

Không gọi install/start thành công là device PASS: phải thực hiện và ghi bằng chứng theo [report](../../docs/reports/native-feasibility.md). Filter tag tùy bản logger; dùng `adb logcat --pid=<pid>` nếu các tag trên không xuất log. `MYVA_NATIVE` là tiền tố do prototype ghi.

## CMP Android: phương án B thử một lượt

Sau khi thử standalone, có thể build riêng shell dùng **CMP 1.12.0 / Kotlin và Compose compiler 2.4.20**, `activity-compose` 1.12.4. `shared/src/commonMain` chứa UI dùng Compose Multiplatform; target hiện chỉ là Android, chưa có iOS compilation.

```bash
prototypes/native/scripts/build-android.sh cmp
adb -s "$DEVICE_SERIAL" install -r prototypes/native/android/cmp-shell/build/outputs/apk/debug/cmp-shell-debug.apk
adb -s "$DEVICE_SERIAL" shell am start -n com.myva.cmpprobe/.ShellActivity
```

Shell mở `GameScreenActivity` chứa native Bevy surface do GameActivity sở hữu và `ComposeView` HUD/nút đóng. Lệnh mở và kết quả đóng truyền qua Intent với `myva.protocol_version=1`, kết quả `myva.result=closed`; shell kiểm tra version và result trước khi hiển thị thành công. Đây là boundary navigation thưa, không phải FFI nhận world state. Overlay chỉ hiển thị hướng dẫn, chưa có telemetry gameplay hai chiều.

Nút mở bị khóa sau một lượt **trong process**, vì tạo lại event loop winit đã được tái hiện là lỗi. Đây là giới hạn cần sửa của prototype, không phải cách đạt kiểm tra vào/rời 30 lần. Activity recreation, việc trở về shell sau native teardown, tap routing của overlay, safe-area và resume đều cần thiết bị thật. Không dùng `force-stop` giữa các vòng để báo cáo đạt gate; chỉ dùng để bắt đầu một phiên thử độc lập khi cần.

## Minimal reproducer: event-loop recreation

```bash
xvfb-run -a cargo +1.97.1 run --locked --manifest-path prototypes/native/probes/event-loop/Cargo.toml
```

Probe tạo/drop/tạo lại `winit::EventLoop` trên Linux. Kỳ vọng `second_event_loop=RecreationAttempt`; exit code 0 nghĩa là đã tái hiện đúng giới hạn, **không** có nghĩa lifecycle mobile thành công. Khi có display thật có thể bỏ `xvfb-run`. Nó bác bỏ cách mount/unmount Composable bằng cách tạo mới `DefaultPlugins`/event loop mỗi lần trong cùng process; không bác bỏ mọi custom adapter hoặc phương án B.

## Preflight và phần chưa triển khai

```bash
python3 prototypes/native/scripts/preflight.py
```

Preflight chỉ đọc toolchain/device; kết quả `NOT_RUN` vẫn yêu cầu thử thủ công. `fps`, memory và số chu kỳ trả `null` đến khi có phép đo thật. Evidence mới lưu vào `prototypes/native/evidence/` (gitignored), chỉ commit bản đã loại thông tin nhạy cảm khi cần review.

Module `app` giữ baseline **standalone**; `cmp-shell` là thí nghiệm B riêng, chưa được chạy trên device. Chưa có ABI handle-based hoặc adapter A. Chưa có iOS packaging/build: cần macOS/Xcode, Apple signing và iPhone để thử đúng mobile example, rồi mới quyết định staticlib/xcframework và ownership UIKit. Không thêm C ABI `create/destroy` khi `DefaultPlugins` còn sở hữu event loop toàn ứng dụng.

Gradle wrapper được tạo bằng Gradle 9.7.0, mã wrapper do Gradle phát hành theo [Apache License 2.0](https://github.com/gradle/gradle/blob/v9.7.0/LICENSE); không phải license áp cho mã MyVa. Cảnh và Java Activity là mã prototype mới của dự án. Các API được đối chiếu [Bevy mobile v0.20.0](https://github.com/bevyengine/bevy/tree/v0.20.0/examples/mobile) và [winit v0.30.13](https://github.com/rust-windowing/winit/tree/v0.30.13).
