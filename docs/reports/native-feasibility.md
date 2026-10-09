# Bevy native / CMP feasibility — issue #10

**Ngày kiểm tra: 2026-10-09. Kết luận: Android BLOCKED, iOS BLOCKED; chưa đạt mobile gate.** Đây là bằng chứng standalone, build CMP Android phương án B thử nghiệm và khảo sát ownership cho [ADR 0003](../decisions/0003-bevy-engine-adoption.md), không phải tuyên bố tích hợp CMP đã chạy trên thiết bị thật. Không đóng #10 dựa vào report này.

## Kết quả và điều kiện tái lập

| Hạng mục | Android | iOS |
| --- | --- | --- |
| Máy thực thi | Linux x86_64, kernel 7.0.0-38-generic | Cùng host Linux; không có macOS/Xcode |
| Rust / Bevy | 1.97.1 / 0.20.0, lockfile riêng | Chưa build target iOS |
| ABI / GPU / OS thiết bị | ARM64 build target; GPU và OS thiết bị **chưa có** | **Chưa có** |
| Toolchain | NDK 27.0.12077973, SDK 37.0, build-tools 36.0.0, JDK 17, Gradle 9.7.0, AGP 9.3.1 | `xcodebuild` không có; Apple SDK/signing/device chưa có |
| Standalone build/link | **PASS build**: ARM64 `.so` và debug APK đã tạo; chưa install | BLOCKED |
| Install/render/touch trên thiết bị thật | BLOCKED: `adb devices -l` trả danh sách rỗng | BLOCKED: không có iPhone và toolchain |
| CMP A: native view/surface chứa game | Chưa triển khai adapter; ownership cần giải quyết | Chưa triển khai adapter; ownership UIKit/Metal cần giải quyết |
| CMP B: điều hướng màn native toàn màn hình | Có shell + GameActivity/HUD thử một lượt; compile/package APK PASS, chưa chạy device | Ứng viên; chưa có controller prototype |
| Kết luận platform | **BLOCKED** | **BLOCKED** |

CMP **1.12.0**, Kotlin/Compose compiler **2.4.20**, `activity-compose` **1.12.4** được pin và đã compile `commonMain` + Android shell. Điều này chưa chứng minh runtime compatibility hoặc iOS support. Android GameActivity **4.4.0** lấy theo mobile example Bevy v0.20.0; lockfile Rust giải `android-activity` **0.6.1**, `winit` **0.30.13**, `wgpu` **30.0.1**.

Các lệnh build nằm trong [README prototype](../../prototypes/native/README.md). Script preflight đọc SDK, Rust, adb và Xcode; không đánh dấu `PASS` khi chỉ phát hiện thiết bị. Log preflight tại thời điểm kiểm tra:

```text
rustc 1.97.1 (8bab26f4f 2026-07-14)
adb devices -l
List of devices attached
xcodebuild: missing
```

[Transcript build và kiểm tra artifact](native-build-evidence.txt) ghi SHA-256/kích thước của hai APK, ELF alignment của toàn bộ thư viện đóng gói và kết quả ownership probe. APK/binary không commit vào Git. Hash nhận diện artifact local đã kiểm tra, không cam kết debug APK tái tạo bit-identical.

## Standalone và probe đã làm

[Mã nguồn](../../prototypes/native/src/lib.rs) dựng `Camera2d`, sprite màu 64×64, xử lý touch theo ID đầu tiên và xóa ý định di chuyển khi cancel/end hoặc suspend. `GameActivity` chịu Android native app; `bevy_winit` chịu render/event loop. `Msaa::Off` đi theo lưu ý Android trong mobile example của Bevy; vẫn phải xác minh renderer/driver trên máy thật.

Build Android thật trên host: `scripts/build-android.sh check` thành công; `scripts/build-android.sh apk` đã link `.so` ELF64/AArch64 và tạo APK chứa `lib/arm64-v8a/libmyva_native_probe.so`. Cấu hình Cargo dev/unoptimized, `debug=0`, minSdk 26; `.so` khoảng 199 MiB, **không** là ngân sách production hoặc benchmark. Chưa có install/render result. NDK r27 cần hai linker flags `max-page-size=16384` và `common-page-size=16384` theo [Android](https://developer.android.com/guide/practices/page-sizes); bản cuối có `LOAD` alignment `0x4000`, APK qua `zipalign -c -P 16`. Chưa chạy trên thiết bị 16 KiB.

Module [`cmp-shell`](../../prototypes/native/android/cmp-shell) dùng UI commonMain của CMP, Intent version 1 mở native `GameScreenActivity`, và nhận result version 1 / `closed`. `ComposeView` phủ thanh hướng dẫn/nút đóng lên màn game; Android back dùng cùng đường trả kết quả. Đây là **B một lượt**: shell khóa nút mở lại trong cùng process, ghi rõ giới hạn hiện hữu thay vì gây loop recreation. Chưa có telemetry gameplay, IME hoặc audio. Kotlin `:shared:compileAndroidMain`, `:cmp-shell:compileDebugKotlin` và đóng gói `:cmp-shell:assembleDebug` đã thành công; runtime vào/rời và overlay cần kiểm chứng trên thiết bị.

Probe riêng chạy bằng:

```bash
xvfb-run -a cargo +1.97.1 run --locked --manifest-path prototypes/native/probes/event-loop/Cargo.toml
```

Kết quả thực tế, exit code 0:

```text
first_event_loop=created
second_event_loop=RecreationAttempt (expected ownership constraint)
```

Đây là kết quả **Linux/Xvfb**, không phải mobile lifecycle test. Probe dựng event loop, drop rồi dựng lại. Nó tái hiện guard trong `winit::EventLoopBuilder::build`: một event loop được tạo cho cả ứng dụng, guard không reset khi drop. Vì vậy cách `Composable mount → DefaultPlugins → App::run → dispose → tạo App mới` không phải hợp đồng create/destroy hợp lệ. Probe không chứng minh mọi đường embedding đều bất khả thi.

## API ownership và quyết định A/B

- **Bevy v0.20.0:** [`WinitPlugin::build`](https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_winit/src/lib.rs) tạo `EventLoop` ngay khi add plugin và cài `winit_runner`. `run_on_any_thread` chỉ hỗ trợ Linux/Windows, không phải cách chuyển loop mobile sang thread tùy ý. `RawWinitWindowEvent` là thông báo đã xử lý; gửi loại event này vào Bevy không bơm input ngược cho loop.
- **Android:** cùng plugin gọi `with_android_app(ANDROID_APP...)`; [`ANDROID_APP`](https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_android/src/lib.rs) là `OnceLock` do entry point `#[bevy_main]` khởi tạo. `SurfaceView`/JNI pointer lấy từ CMP không tự trở thành `AndroidApp`. Mobile sample dùng [`GameActivity`](https://github.com/bevyengine/bevy/blob/v0.20.0/examples/mobile/android_example/app/src/main/java/org/bevyengine/example/MainActivity.java), không phải mẫu embedding `AndroidView`. **A cần adapter thật** phối hợp Bevy render surface/window, lifecycle và input; chưa có adapter nào được kiểm chứng trong repository.
- **winit 0.30.13:** [`EventLoopBuilder::build`](https://github.com/rust-windowing/winit/blob/v0.30.13/src/event_loop.rs) trả `RecreationAttempt` khi tạo loop lần nữa. **B cũng cần giải quyết re-entry**; finish Activity rồi gọi lại Bevy trong cùng process chưa được chấp nhận chỉ vì màn đầu chạy được. Không dùng kill process như một bằng chứng cleanup đúng.
- **iOS:** [`EventLoop::run`](https://github.com/rust-windowing/winit/blob/v0.30.13/src/platform_impl/ios/event_loop.rs#L273-L314) kiểm tra `UIApplication.sharedApplication` chưa tồn tại, rồi tự gọi `UIApplicationMain`. Nếu CMP đã khởi động UIKit, runner mặc định bị assertion chặn với lỗi `EventLoop cannot be run after a call to UIApplicationMain on iOS`. Đây là **blocker cụ thể từ mã nguồn**, chưa được thực thi trên iPhone trong phiên này. [`UIKitView`](https://kotlinlang.org/docs/multiplatform/compose-uikit-integration.html) chỉ cung cấp interop view; không tự chuyển ownership loop/Metal layer. Cả A và B iOS cần custom runner hoặc đảo ownership để winit khởi động application và gắn CMP sau đó; việc chỉ gọi `App::run()` từ controller không giải quyết lỗi. Chưa link staticlib/xcframework nên ABI, main-thread và CAMetalLayer đều **chưa kiểm chứng**.

Quyết định tạm: giữ **A là phương án ưu tiên nghiên cứu**, B là dự phòng với Android đã có build thử một lượt; cả hai OS vẫn BLOCKED về device evidence. Không sản xuất FFI `create/attach/resize/input/pause/resume/detach/destroy` giả trước khi có adapter sở hữu tài nguyên thật. Khi có adapter, mỗi handle phải có generation/lifetime rõ, pointer cancel và bounded command/event queue; không truyền world JSON mỗi frame. Shell sở hữu navigation, IME và UI ngoài gameplay theo ADR.

## Ma trận evidence còn thiếu

Mỗi ô `NOT RUN` cần log/capture gắn với device, OS, GPU, ABI, build profile/flags và commit; không dùng kết quả desktop hoặc simulator để thay.

| Kiểm tra | Android thật | iPhone thật | Artifact cần lưu |
| --- | --- | --- | --- |
| Install và sprite/touch standalone trên device | NOT RUN | NOT RUN | Build log, device info, ảnh/video, startup log |
| CMP HUD/nút đóng, command/result hai chiều | NOT RUN | NOT RUN | Video + versioned boundary log |
| Vào/rời game 30 lần | NOT RUN | NOT RUN | Cycle log, crash log, resource/memory trước/sau |
| Background/resume 20 lần | NOT RUN | NOT RUN | Lifecycle log, surface/context/texture recovery |
| Resize, orientation, safe-area | NOT RUN | NOT RUN | Các trạng thái màn hình + kích thước surface |
| Multi-pointer/cancel, overlay tap routing | NOT RUN | NOT RUN | Touch IDs/cancel trace + video |
| Focus và IME/chat tiếng Việt | NOT RUN | NOT RUN | Video nhập tiếng Việt, focus trace |
| Audio interruption/mute, back/foreground, termination | NOT RUN | NOT RUN | Audio/lifecycle log, hành vi resume/exit |
| FPS/frame pacing và memory | **Không đo** | **Không đo** | Perfetto/Instruments capture, điều kiện đo |

Audio, IME và telemetry gameplay chưa tồn tại; cần bổ sung consumer thực trước khi chạy các hàng đó. Overlay CMP và navigation đã có mã thử nhưng chưa được chạy. Chỉ khi đầy đủ evidence trên từng OS mới cập nhật ADR feasibility và cho phép mở rộng production assets/UI. iOS vẫn cần host macOS + signing/device; Android cần cắm thiết bị thật, chạy standalone trước, rồi đánh giá adapter A hoặc B và kiểm tra lại toàn bộ ma trận, đặc biệt re-entry chưa được giải quyết.
