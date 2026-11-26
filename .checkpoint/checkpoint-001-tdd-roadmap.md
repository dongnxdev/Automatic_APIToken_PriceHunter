# Trạm Bàn Giao (Project Checkpoint) — Checkpoint 001

- **Thời điểm cập nhật (UTC):** `2026-11-26T03:15:00.000Z`
- **Tọa độ hiện tại (Current State):** Hoàn tất Phase 0 (Phân tích `Readme.md`, `template_view`, xây dựng trọn bộ TDD 5 phần và Roadmap 7 Phase chuẩn Production). Chưa khởi tạo mã nguồn Rust/WASM.
- **Tiến độ dự án:** `Phase 0 / 7` hoàn thành (`100%` tài liệu thiết kế & lộ trình đã khóa cứng).

## Biến Động Workspace (File Changes)
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\01_SYSTEM_OVERVIEW_AND_ARCHITECTURE.md]` — Thêm mới: Tổng quan 5W2H/4M1E, Tech Stack Lock (`Rust 1.83`, `Axum 0.8`, `SQLite WAL`, `Leptos 0.7 WASM`), Vertical Slice Architecture tỷ lệ 6:4 theo `Readme.md`, Single Canonical Standard.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\02_DATA_MODEL_AND_DATABASE_SCHEMA.md]` — Thêm mới: Schema SQLite 11 bảng (UUIDv7, UTC ms, Nano-USD anti-float), JSON Capability contract (`cache_read/write`, `min_charge_per_request`, `max_output_tokens_limit`, `supported_paths`, `video_contract`, `video_rate_cards`), công thức Blended Price 4 Workload Profiles.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\03_BACKEND_ENGINES_AND_API_CONTRACTS.md]` — Thêm mới: Crawler Engine (5 Adapters, Scheduler, DLQ), JSON Parser Sandbox, Canonical Mapper Engine, Model Deep-Dive & Live Tester (chống bait-and-switch), Smart Gateway Reverse Proxy (`/v1/*`, Auto-Failover `<100ms`, Price Circuit Breaker), toàn bộ REST API v1 & WebSocket `/ws/ops`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\04_FRONTEND_LEPTOS_WASM_AND_6_SCREENS.md]` — Thêm mới: Kiến trúc Leptos WASM + Tailwind tokens từ `template_view`, Global App Shell (Sidebar + 48px Top Bar), chi tiết 6 màn hình (`/radar`, `/models/:id`, `/providers`, `/aliases`, `/gateway`, `/ops`), cơ chế `on_cleanup` & RingBuffer 1000 dòng.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\05_TESTING_DEVOPS_SECURITY_AND_LIFECYCLE.md]` — Thêm mới: Bảo mật OWASP/AES-256-GCM/Argon2id/SHA-256, Observability JSON Logs/Masking/`/metrics`/`/health/*`, DLQ Replay, Data Retention, Graceful Shutdown, Testing & Docker multi-stage.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\ROADMAP.md]` — Thêm mới: Lộ trình thực thi tuần tự Phase 0 -> Phase 7 với Acceptance Gates và lệnh Verify rõ ràng.

## Các Hạng Mục Đã Chốt Hạ
1. Backend: Rust (`1.83.0`), `axum`, `tokio`, `sqlx` (SQLite WAL), `reqwest` (rustls + socks), `rust_decimal` + `i64` nano-USD (không dùng `float` tính tiền), `arc-swap` read snapshot.
2. Frontend: Leptos (`0.7.x` CSR WASM) + Trunk + Tailwind CSS local theo đúng Design Tokens của `template_view`.
3. Kiến trúc thư mục: Vertical Slice (`Features/<Slice>/{Common, Model, Services, Controller, Results}` & `Shared/{Common, Data, Utils, Services, Results}`) theo `Readme.md`, tỷ lệ 6:4 Backend/Frontend, mọi file `< 1000` dòng.
4. Cấu hình khởi động: `appsettings.json` (cổng `1098`, `Server`, `Provider`, `Management`, `feature_flags`) + env override + fail-fast.

## Lệnh Bàn Giao & Bước Tiếp Theo (Resume Action)
- **Bước tiếp theo:** Triển khai **Phase 1** theo `docs/ROADMAP.md` (khởi tạo `rust-toolchain.toml`, `Cargo.toml` workspace, `crates/shared_contract`, `backend/src/Shared/`, `migrations/0001_init.sql`, `appsettings.example.json` và Unit Tests cho `currency_math` + config loader).
- **Tài liệu bắt buộc đọc trước khi code Phase 1:**
  - `docs/tdd/01_SYSTEM_OVERVIEW_AND_ARCHITECTURE.md`
  - `docs/tdd/02_DATA_MODEL_AND_DATABASE_SCHEMA.md`
  - `docs/ROADMAP.md`
- **Lệnh kiểm tra môi trường trước khi bắt đầu Phase 1:**
  ```powershell
  rustc --version; cargo --version; rustup target list --installed
  ```
