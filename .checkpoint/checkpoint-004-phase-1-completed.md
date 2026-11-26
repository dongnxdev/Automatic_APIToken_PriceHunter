# Trạm Bàn Giao (Project Checkpoint) — Checkpoint 004

- **Thời điểm cập nhật (UTC):** `2026-11-26T04:15:00.000Z`
- **Tọa độ hiện tại (Current State):** **Hoàn tất 100% Phase 1** (Workspace Cargo 3 crates, `crates/shared_contract`, `backend/src/Shared/`, Migrations SQLite WAL 13 bảng TEXT Decimal, `appsettings.json` tương thích Readme.md + Ckey, Unit Tests 100% pass, Zero Warnings với Clippy `-D warnings`, binary `modelradar-server` khởi động và chạy migrations thực tế thành công).
- **Tiến độ dự án:** `Phase 1 / 7` hoàn thành (`[x] Phase 0`, `[x] Phase 1`).

## Biến Động Workspace (File Changes)
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\rust-toolchain.toml]` — Khóa toolchain Rust stable (`wasm32-unknown-unknown`, `clippy`, `rustfmt`).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\Cargo.toml]` — Cargo Workspace chuẩn kết nối `crates/shared_contract`, `backend`, `frontend`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\crates\shared_contract\src\currency_math.rs]` — Xây dựng Exact Decimal Math, `ExactPrice`, `WorkloadProfile`, `calculate_effective_blended` (cấm gọi `round()`), formatters VNĐ/USD exact. Đã test pass 5 ca số lẻ thực tế từ Ckey (`anhyeu00m`, `sypham98`, `rouyea98`, `thanhnhan9023`, `tdsang1999`).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\crates\shared_contract\src\api_envelope.rs]` — Unified API Envelope `{ success, data, error, meta }`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\crates\shared_contract\src\enums.rs]` — Single Canonical Enums (`AdapterType`, `AuthMode`, `PricingModelKind`, `ProviderStatus`, `RoutingMode`, `DisplayCurrency`...).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\crates\shared_contract\src\model_capabilities.rs]` — 13 SupportedPaths Ckey, VideoContract, VideoRateCard, NormalizedCkeyModel.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\crates\shared_contract\src\ws_events.rs]` — WebSocket realtime event frames.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\migrations\0001_init.sql]` — 13 bảng SQLite WAL với toàn bộ cột giá dạng `TEXT` Decimal.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\migrations\0002_seed_default_rules.sql]` — Seed 7 Canonical Models cơ bản và 4 luật Regex chuẩn hóa.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\Shared\Common\config.rs]` — Config loader đọc `appsettings.json` (tương thích Readme.md: `Server.Port = 1098`, `Provider`, `Management`), bóc tách `?key=`, fail-fast.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\Shared\Common\crypto.rs]` — AES-256-GCM encryption/decryption, derive master key.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\Shared\Data\sqlite_pool.rs]` — SQLite WAL connection pool + `sqlx::migrate!`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\Shared\Utils\masking.rs]` — Masking secret (`47e8••••f3a3`) và URL redaction (`?key=***REDACTED***`).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\Shared\Utils\time_text.rs]` — UTC ISO/ms, NFC/trim, lowercase lookup key.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\main.rs]` — Bootstrap server, nạp config, kiểm tra AES cipher, chạy migrations, kiểm tra seed data.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\appsettings.json]` & `appsettings.example.json` — Cấu hình chuẩn Port 1098, provider Ckey.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\.gitignore]` — Chặn target, data DB SQLite, và các file chứa secret.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\ROADMAP.md]` — Cập nhật checklist bàn giao Phase 1 thành `[x]`.

## Kết Quả Kiểm Định (Verification Gate)
1. `cargo test --workspace`: **11 tests passed, 0 failed** (5 shared_contract + 6 backend).
2. `cargo clippy --workspace --all-targets -- -D warnings`: **ZERO WARNINGS**.
3. `cargo run -p modelradar-server`: Khởi động thực tế thành công, nạp config port 1098, mã hóa AES-256-GCM khớp 100%, khởi tạo pool SQLite WAL, áp dụng migrations và seed thành công 7 models chuẩn vào file `data/modelradar.db`.

## Lệnh Bàn Giao & Bước Tiếp Theo (Resume Action)
- **Bước tiếp theo:** Bắt đầu **Phase 2** theo `docs/ROADMAP.md`:
  1. `Features/AuthManagement`: Đăng nhập/Đăng xuất Management (Argon2id + HttpOnly JWT), `/health/live`, `/health/ready`, `/metrics`.
  2. `Features/ProviderVault`: CRUD Provider, `QUERY_KEY` auth, reveal secret audit, JSON Sandbox preview exact decimal.
  3. `Features/CrawlerEngine`: `CkeyLlmModelsAdapter` nạp trực tiếp 650 models từ Ckey vào Decimal, Scheduler, Provider Circuit Breaker, DLQ.
  4. Integration tests phát lại fixture `docs/api_test.json`.
- **Lệnh verify Phase 1 bất kỳ lúc nào:**
  ```powershell
  cargo test --workspace; cargo clippy --workspace --all-targets -- -D warnings
  ```
