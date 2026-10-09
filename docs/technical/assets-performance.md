# Asset streaming và hiệu năng — MyVa · Thần Mạch

**Trạng thái:** đề xuất v0.1, ngày 09/10/2026. Chưa có asset runtime hoặc benchmark. Các budget là điểm khởi đầu để thử nghiệm, không phải kết quả đã đạt.

## 1. Tiêu chuẩn hình ảnh

2D HD nghĩa là silhouette rõ, nét đúng ở kích thước chơi, chuyển động có nhịp và VFX đọc được. Không mặc định texture lớn nhất là đẹp nhất. Thiết kế side-view phải ưu tiên đọc telegraph, hitbox, độ cao nền và đối thủ trên màn hình điện thoại.

| Loại nội dung | Hướng sản xuất |
| --- | --- |
| Nhân vật chơi/NPC quan trọng | Concept và rig nhất quán; atlas phân theo bộ nhân vật/lân cận cần gặp |
| Quái nhỏ, vật phẩm | Sprite/flipbook gọn, không cần rig nếu lợi ích thấp |
| Boss | Animation riêng, telegraph tách layer; tải trước khi vào đấu trường |
| Bản đồ | Tile/chunk theo biên; background nhiều lớp trong budget overdraw |
| VFX | Sprite, particle và shader theo tier; fallback đơn giản đọc được |
| Âm thanh | Nhạc stream theo map; sound combat giới hạn polyphony và độ dài |
| Portrait/UI | Bundle shell riêng; không giữ texture game chỉ để hiển thị avatar |

Animation render không quyết định luật hit. Startup/active/recovery, hurtbox và hitbox được định nghĩa bằng dữ liệu gameplay version hóa; asset phải khớp mốc sự kiện, không tự kéo dài cửa sổ tấn công khi thiết bị giảm FPS.

## 2. Skeletal animation là lựa chọn cần kiểm chứng

Không ghi skeletal animation thành tính năng đã có của MyVa hoặc mặc định Bevy cung cấp runtime skeletal 2D đó. Cần một spike đánh giá công cụ authoring, format export, runtime Rust/WASM/native và quyền sử dụng/phân phối.

So sánh ít nhất hai đường:

1. Rig runtime cho nhân vật chính: tiết kiệm một số frame texture, linh hoạt trang bị; đo CPU skinning, draw call, mesh và attachment.
2. Bake thành sprite atlas/flipbook: ít rủi ro runtime, cần budget texture và chất lượng xuất ảnh.

Acceptance: cùng clip đứng/đi/nhảy/combo 10 giây chạy trên web + Android + iOS, thiết bị thật, swap trang bị, pause/resume và memory không tăng sau reload. Lựa chọn phải ghi license của editor, runtime, nội dung và build pipeline; không cam kết vendor trước thử nghiệm.

## 3. Atlas, texture và shader

Bevy `0.20.0` cung cấp sprite/texture atlas và material 2D qua các module riêng [S1–S2]. Spike web chỉ bật những feature cần cho sprite và WebGL2; WebGPU hoặc backend native là các cấu hình phải kiểm chứng riêng. Shader/custom material phải được thử ở từng backend; không suy ra tương thích từ một ảnh render desktop. D05 [#13](https://github.com/loveoverflowcom/myva/issues/13) đo pipeline 2D sau khi platform tương ứng vượt gate tích hợp.

Ưu tiên atlas đóng gói offline để tính kích thước và ownership. Trong Bevy, strong asset handles giữ asset sống [S3]; danh sách preload, resource ECS, sprite/material và render-world có thể giữ tham chiếu sau khi rời map. Ghi rõ owner mỗi bundle, bỏ strong handles khi hết dùng, rồi đo CPU/GPU sau nhiều chu kỳ tải/rời. Despawn entity hoặc file tải nhỏ không đủ chứng minh đã trả hết bộ nhớ.

- Atlas chia theo bản đồ/roster/clip cần cùng lúc, thay vì một atlas chứa cả thế giới.
- Padding/extrude biên sprite tránh bleeding; mipmap và filter phải thử ở zoom/DPR mục tiêu.
- Giữ thông tin trim/pivot để animation không rung; định nghĩa anchor chân và vũ khí.
- Packing offline có báo cáo diện tích lãng phí; giới hạn kích thước texture theo capability thiết bị.
- GPU upload chia theo frame budget; khởi tạo material trước combat để tránh giật lần dùng đầu.
- Chỉ dùng texture compression sau khi test format/backend hỗ trợ; kích thước file nén không đại diện RAM/VRAM.

## 4. Pipeline nội dung đề xuất

`Nguồn có provenance → export chuẩn → validate → pack → hash → manifest → phát hành`.

Mỗi asset có ID ổn định, loại, kích thước, hash, dependency, variant chất lượng, version format, license/provenance và attribution nếu có. Validator phát hiện trùng ID, thiếu dependency, clip thiếu mốc combat, kích thước quá budget và reference qua bundle ngoài dự kiến.

Manifest được version hóa; client lấy root từ origin tin cậy qua TLS, kiểm chữ ký nếu dùng kho phân phối tách origin, sau đó kiểm hash từng chunk. Hash xác nhận toàn vẹn; chữ ký/root tin cậy xác nhận nguồn. Không chấp nhận manifest bất kỳ do một người chơi cung cấp.

Server chỉ định content/rules version tương thích cho phiên chơi; client chưa có phiên bản cần tải trước khi vào. Có cơ chế thu hồi bản lỗi. Không hot-swap rules combat giữa hai bản khác nhau trong một trận.

Chỉ tải nội dung dữ liệu đã biết schema; không tải và thực thi Rust/WASM/script tùy ý từ bundle map. Đổi executable phải đi qua release client phù hợp nền tảng.

## 5. Đơn vị tải và ưu tiên

| Mức | Nội dung | Thời điểm |
| --- | --- | --- |
| P0 | Shell, font, login, thông báo lỗi | Mở ứng dụng |
| P1 | Game runtime, nhân vật đã chọn, vùng khởi đầu, âm thanh cốt lõi | Người dùng chọn bắt đầu |
| P2 | Chunk lân cận, quái dự kiến, skill đang trang bị | Theo hướng di chuyển và ranh giới map |
| P3 | Boss/instance kế tiếp, vùng đến sau fast travel | Tại cửa vào; cần hoàn tất phần thiết yếu |
| P4 | Cosmetic hiếm, âm thanh phụ, vùng xa | Chỉ khi có budget và không ảnh hưởng P1–P3 |

P1 không chứa toàn bộ linh vực hoặc toàn bộ roster. Khi gặp người chơi có cosmetic chưa cache, dùng silhouette/fallback trung tính; giữ nguyên hình học gameplay và khả năng đọc vũ khí/telegraph.

Prefetch dựa vào chuyển cảnh, hướng di chuyển và quest đang chọn; có TTL, hủy khi đổi đường. Giới hạn ban đầu 2 lượt tải đồng thời trên mobile, 4 trên web; điều chỉnh qua đo mạng và frame time, không tăng tải vô hạn theo số request.

Không decode ảnh/animation lớn đồng bộ trên main thread trong lúc combat. Với web, worker hoặc host-assisted decode là phương án cần prototype; nếu runtime không hỗ trợ, chia nhỏ asset và decode trong màn chuyển cảnh. Native dùng hàng đợi công việc, upload GPU trên thread/context được phép.

## 6. Cache và lỗi tải

- Persistent cache đề xuất: browser host bridge với storage thích hợp; mobile file cache theo content hash. Chưa chốt API hay dung lượng bảo đảm của trình duyệt.
- Ghi file/chunk vào vùng tạm, kiểm hash rồi mới promote nguyên tử; file dở không được đánh dấu là hoàn tất.
- Retry giới hạn, backoff có jitter, timeout; hỗ trợ resume nếu origin/chunk format cho phép.
- Evict LRU nội dung không được pin; pin nhân vật, map hiện tại và dependency cần cho chuyến đi đang diễn ra.
- Cache key gồm version format/hash; dọn phiên bản cũ sau khi phiên chơi không còn dùng.
- Dung lượng cache ít hoặc bị hệ điều hành xóa phải tải lại được; không coi cache là lưu trữ sở hữu vật phẩm.
- Offline có thể xem nội dung đã cache nếu thiết kế cho phép; không tiếp tục phát thưởng, harvest, trade hoặc đồng bộ “thành quả offline” vào server.
- Thiếu asset thiết yếu trước cửa map: giữ người chơi tại điểm an toàn, hiển thị tiến độ/retry. Không để rơi vào bản đồ rỗng mà server vẫn tấn công.

Thước đo tải gồm tỷ lệ cache hit, bytes P1, thời gian tải cold/warm p50/p95, lỗi hash, retry và số lần bị chặn ở biên map. Không chỉ báo tốc độ CDN.

## 7. Tính bộ nhớ đúng cách

RGBA8 chưa nén có kích thước `rộng × cao × 4 byte`. Một texture 2048×2048 xấp xỉ **16 MiB**; chuỗi mipmap đầy đủ thêm khoảng 1/3, tổng khoảng **21,33 MiB**.

Nếu CPU giữ ảnh decode 16 MiB và GPU giữ bản base 16 MiB, ít nhất đã 32 MiB trước mipmap, buffer upload và overhead. File PNG/WebP 1 MiB không đồng nghĩa texture dùng 1 MiB bộ nhớ. Skeletal animation cũng cần texture, mesh, bone và pose buffer; không mặc định nhẹ hơn mọi flipbook.

Theo dõi riêng: WASM linear memory/heap, native RSS, ảnh decode/staging, texture/buffer GPU, render targets, audio decode và persistent cache trên đĩa. Công cụ đo GPU trên browser có giới hạn; dùng cả estimation theo allocation và công cụ thiết bị khi có.

## 8. Budget ban đầu để thử nghiệm

| Chỉ số | Mục tiêu thử nghiệm | Điều kiện |
| --- | --- | --- |
| Render | 60 FPS, frame p95 ≤ 16,67 ms; p99 ≤ 25 ms | Cảnh chuẩn, 10 phút; ghi rõ device và nhiệt |
| Tier thấp | 30 FPS, p95 ≤ 33,33 ms | Giảm VFX, render scale, background; luật vẫn giữ tick |
| Heap game WASM | ≤ 192 MiB | Không tính toàn bộ memory tiến trình browser |
| Native process RSS | ≤ 350 MiB | Bao gồm game + CMP trong cảnh chuẩn |
| Texture/buffer GPU ước tính | ≤ 128 MiB | Không tính cache trên đĩa; ghi độ chính xác phép đo |
| Cache đĩa | Mặc định 300 MiB, tùy thiết bị/storage | Người dùng có thể xóa và tải lại |
| P1 tải nén | ≤ 25 MiB game runtime + vùng đầu | Ngân sách, chưa phải kích thước build thực tế |
| Warm start | ≤ 3 giây từ bấm chơi tới điều khiển | Asset đã cache, sau login, server healthy |
| Cold start | p95 ≤ 20 giây | Mạng giả lập 15 Mbps, RTT 100 ms; bao gồm tải P1 |

Các mức này cần điều chỉnh bằng prototype. Nếu fail, giảm scope/cảnh trước khi âm thầm tăng budget. Đo thời gian matchmaking/login riêng để không che việc chờ network.

Mỗi capture phải ghi: build/revision, độ phân giải, DPR/render scale, tier, map, actor count, particle count, nhiệt, network profile và thời lượng chạy. Không lấy FPS trung bình thay cho p95/p99 và spike khi chuyển chunk.

## 9. Ma trận thiết bị và cảnh benchmark

Chưa chốt danh sách máy tối thiểu. Trước vertical slice chọn ít nhất:

- Web desktop Chrome/Firefox; Safari trên Mac và iPhone; kiểm tra canvas, audio, memory và context loss.
- Android 4 GB RAM đại diện tier thấp cùng một máy tầm trung; kiểm tra GPU thực tế, Android release và nhiệt.
- iPhone cũ nhất dự kiến hỗ trợ cùng một máy mới; kiểm tra native renderer, CMP overlay và resume.

Không kết luận “đạt Android/iOS” từ emulator hoặc desktop alone. OS/browser/toolchain được pin trong báo cáo theo thời điểm kiểm tra, không lấy ví dụ hôm nay làm compatibility cam kết mãi mãi.

Cảnh chuẩn đề xuất: 1 người local + 19 actor khác, 30 quái, 100 particle thấy được, 4 lớp nền và HUD. Cảnh stress tăng gấp đôi actor/VFX; cảnh chuyển map tải texture mới; cảnh soak 30 phút và lặp vào/rời 30 lần để phát hiện leak.

Chất lượng tự điều chỉnh bằng hysteresis để tránh đổi tier liên tục. Giảm ánh sáng phụ, screen effect, particle, lớp nền và render scale; giữ telegraph boss, hit flash, vùng nguy hiểm và các tín hiệu phục vụ chiến đấu.

## 10. Kiểm chứng trước phát hành nội dung

1. Validation asset/schema/provenance, không thiếu dependency.
2. Visual QA ở kích thước thực: sprite, camera, màn điện thoại, boss telegraph, người bị che bởi VFX.
3. Cold/warm cache, mạng yếu, tải bị hủy, file hỏng, hết dung lượng và context/surface bị mất.
4. Performance capture theo ma trận thiết bị, cả tier 60 và fallback 30 FPS.
5. Xác minh offline/cache không tạo inventory hoặc quyền gameplay mới.
6. Manifest rollback/thu hồi bản lỗi không làm client vào phiên rules không tương thích.

## Nguồn kiểm chứng

Truy cập ngày **09/10/2026**. Budget và pipeline là đề xuất của MyVa; nguồn dưới đây chỉ xác nhận API và hạn chế liên quan.

- **[S1]** [Bevy 0.20 sprite module](https://docs.rs/bevy/0.20.0/bevy/sprite/index.html).
- **[S2]** [Bevy 0.20 Material2d](https://docs.rs/bevy/0.20.0/bevy/sprite_render/trait.Material2d.html).
- **[S3]** [Bevy 0.20 asset handle implementation](https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_asset/src/handle.rs).
