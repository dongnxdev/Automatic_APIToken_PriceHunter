# Trạm Bàn Giao (Project Checkpoint) — Checkpoint 002

- **Thời điểm cập nhật (UTC):** `2026-11-26T03:28:00.000Z`
- **Tọa độ hiện tại (Current State):** Hoàn tất **Phase 0 (Hiệu chỉnh theo dữ liệu API thực tế của nền tảng Ckey.vn)**. Đã fetch phản hồi thật từ Ckey (`650` models, `586.815` bytes) lưu vào `docs/api_test.json` và cập nhật đồng bộ toàn bộ 5 file TDD (`docs/tdd/01..05`) + `docs/ROADMAP.md`.
- **Tiến độ dự án:** `Phase 0 / 7` hoàn thành (`100%` đặc tả bám sát dữ liệu thực tế đã khóa cứng).

## Biến Động Workspace (File Changes)
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\api_test.json]` — Ghi dữ liệu phản hồi thực tế (HTTP 200, 650 models, 586.815 bytes) từ endpoint `ckey.vn/api/llm/models`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\01_SYSTEM_OVERVIEW_AND_ARCHITECTURE.md]` — Cập nhật định hướng single-platform **Ckey.vn** đa kênh bán (`seller_namespace`), đổi đơn vị gốc nội bộ sang `Micro-VND (i64)`, hỗ trợ `auth_mode = QUERY_KEY` và bóc tách an toàn `?key=` khỏi URL.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\02_DATA_MODEL_AND_DATABASE_SCHEMA.md]` — Cập nhật schema khớp đủ 19 trường thực tế của Ckey (`public_name`, `display_name`, `is_provider_model`, `input/output/cache_read/cache_write_price_per_million_vnd`, `price_per_request_vnd`, `min_charge_per_request_vnd`, `price_per_second_vnd`, `request_rate_limit_per_minute`, `max_output_tokens_limit`, `context_tokens_limit`, `supported_paths`, `video_durations_allowed`, `video_max_seconds`, `video_rate_cards`, `video_contract`) + công thức Effective Blended Price chống bẫy `min_charge` và giá khoán `0 đ`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\03_BACKEND_ENGINES_AND_API_CONTRACTS.md]` — Cập nhật `CkeyLlmModelsAdapter`, Circuit Breaker 2 cấp (Provider + từng `seller_namespace`), Canonical Mapper theo dữ liệu Ckey, và Smart Gateway Failover giữa các kênh bán trên Ckey.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\04_FRONTEND_LEPTOS_WASM_AND_6_SCREENS.md]` — Cập nhật 6 màn hình hiển thị trung thực dữ liệu kênh bán Ckey (`seller_namespace`, `min_charge`, `rpm_limit`, VNĐ mặc định, `QUERY_KEY` auth mode).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\05_TESTING_DEVOPS_SECURITY_AND_LIFECYCLE.md]` — Cập nhật bảo mật masking `?key=***REDACTED***` và bộ test kiểm định dùng `docs/api_test.json` làm fixture chuẩn.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\ROADMAP.md]` — Cập nhật Roadmap 7 Phase bám sát Ckey Adapter & fixture `docs/api_test.json`.

## Các Phát Hiện Quan Trọng Từ `docs/api_test.json` Đã Khóa Vào Thiết Kế
1. **Đơn vị gốc VNĐ có số lẻ thập phân (`773.5`, `61.75`, `15.6`):** Lưu `Micro-VND` (`i64`, `1 VNĐ = 1_000_000 micro_vnd`) làm chuẩn nội bộ để triệt tiêu sai số quy đổi USD trung gian.
2. **Đa kênh bán trong 1 sàn duy nhất:** `ckey.vn` chứa 650 model trải trên 90+ `seller_namespace` (`sypham98`, `rouyea98`, `ipeenkairdrop`, `thanhnhan9023`...) và 12 model `direct` (`is_provider_model = false`). Rank & Failover vận hành ở cấp `(provider_source_id, public_name)`.
3. **Bẫy phí ẩn & giá khoán `0 đ`:** `thanhnhan9023/claude-opus-4-6` có `in=0, out=0` nhưng thu `price_per_request_vnd = 65 đ`; `rouyea98/claude-opus-4-6` rẻ hơn về token (`474.5 đ`) nhưng gài `min_charge = 28.6 đ` nên ở 50K tokens/req lại đắt hơn `sypham98` (`526.5 đ`, `min_charge = 5.2 đ`). Công thức Effective Blended Price xử lý triệt để cả 3 loại (`PER_TOKEN`, `FLAT_PER_REQUEST`, `PER_SECOND_VIDEO`).
4. **Nhiễu `video_contract` & giới hạn RPM:** API Ckey gắn `video_contract` mặc định lên cả model text -> lọc bằng cờ `is_media_video_capable`. Một số kênh có `request_rate_limit_per_minute` (`10`, `100`) -> Smart Gateway tính vào tiêu chí định tuyến và failover `<100ms`.

## Lệnh Bàn Giao & Bước Tiếp Theo (Resume Action)
- **Bước tiếp theo:** Triển khai **Phase 1** trong `docs/ROADMAP.md` (`rust-toolchain.toml`, `Cargo.toml` workspace, `crates/shared_contract` với `MicroVnd` & `EffectiveBlendedFormula`, `backend/src/Shared/`, `migrations/0001_init.sql`, `migrations/0002_seed_default_rules.sql`, `appsettings.example.json`, và Unit Tests).
- **Lệnh kiểm tra môi trường trước khi code Phase 1:**
  ```powershell
  rustc --version; cargo --version; rustup target list --installed
  ```
