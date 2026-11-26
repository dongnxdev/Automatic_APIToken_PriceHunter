# Trạm Bàn Giao (Project Checkpoint) — Checkpoint 005

- **Thời điểm cập nhật (UTC):** `2026-11-26T04:30:00.000Z`
- **Tọa độ hiện tại (Current State):** **Hoàn tất 100% Phase 2** (Auth Management, Provider Vault, AES-256-GCM reveal secret audit, JSON Sandbox parse, CkeyLlmModelsAdapter parse 650 models nạp thẳng Decimal, Delta-Write SQLite, Provider Circuit Breaker, WireMock Integration Tests phát lại fixture `docs/api_test.json` pass 100%, Clippy `-D warnings` đạt ZERO WARNINGS).
- **Tiến độ dự án:** `Phase 2 / 7` hoàn thành (`[x] Phase 0`, `[x] Phase 1`, `[x] Phase 2`).

## Biến Động Workspace (File Changes)
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\Cargo.toml]` — Cấu hình `[lib]` name `modelradar_server` và dev-dependencies `wiremock`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\lib.rs]` — Export `Features`, `Shared`, `app_state`, và hàm `build_app(state: AppState) -> Router`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\app_state.rs]` — Quản lý `AppState` (db pool, http client, session store, is_shutting_down, price_index ArcSwap, crawler mutex lock, auto_scan_enabled).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\Features\AuthManagement\]` — Đầy đủ 5 thư mục con (`Common, Model, Services, Controller, Results`): Login/Logout (Argon2id + Constant-time SHA256 + HttpOnly cookie `modelradar_session`), `/health/live`, `/health/ready` (kiểm tra SQLite db), `/metrics` (Prometheus gauge).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\Features\ProviderVault\]` — Đầy đủ 5 thư mục con: CRUD Provider, bóc tách an toàn `?key=`, `AES-256-GCM` encryption, `POST /api/v1/providers/{id}/reveal-secret` ghi `system_audit_logs`, `POST /api/v1/providers/sandbox-parse` (JSON Parser Sandbox preview dry-run).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\Features\CrawlerEngine\]` — Đầy đủ 5 thư mục con: `CkeyParser` parse 19 trường Ckey nạp thẳng `Decimal` (không làm tròn số nguyên, xử lý `is_media_video_capable`), `CkeyCrawlerService` (gọi Ckey `?key=...`, Delta-Write SQLite khi hash thay đổi, Rebuild in-memory snapshot, Circuit Breaker 3 lần lỗi / 1 lần 401, DLQ `dead_letter_events`, `force_scan_all`).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\src\main.rs]` — Server entrypoint lắng nghe port 1098, đồng bộ provider ban đầu từ `appsettings.json`, Graceful Shutdown `tokio::signal::ctrl_c()`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\backend\tests\crawler_and_vault_integration.rs]` — WireMock Integration Tests phát lại fixture `docs/api_test.json`: parse trọn vẹn 650 models, kiểm chứng từng số lẻ thập phân (`526.5 đ`, `508.95 đ`, `323.4816 đ`, `1779.1488 đ`, `88.37 đ`, `65.0 đ`, `1200.0 đ`), kiểm chứng Circuit Breaker on 401.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\ROADMAP.md]` — Cập nhật checklist bàn giao Phase 2 thành `[x]`.

## Kết Quả Kiểm Định (Verification Gate)
1. `cargo test --test crawler_and_vault_integration`: **2 tests passed, 0 failed** (full flow 650 models crawl & vault exact decimal + circuit breaker on 401).
2. `cargo test --workspace`: **17 tests passed, 0 failed** (10 lib + 2 integration + 5 shared_contract).
3. `cargo clippy --workspace --all-targets -- -D warnings`: **ZERO WARNINGS**.

## Lệnh Bàn Giao & Bước Tiếp Theo (Resume Action)
- **Bước tiếp theo:** Bắt đầu **Phase 3** theo `docs/ROADMAP.md`:
  1. `Features/CanonicalMapper`: Chuẩn hóa danh mục 650 models Ckey (áp dụng 4 luật Regex đã seed, Tab 1 hàng chờ chưa map, Tab 2 luật regex, API `rerun-regex`).
  2. `PriceSnapshotIndex`: Gom nhóm theo `canonical_id`, cấu trúc cache RAM O(1).
  3. `Features/PriceRadar`: API `/api/v1/price-radar` lọc theo 4 Workload, xếp hạng Ngai Vàng #1 và Ma Trận so sánh các kênh bán Ckey (bóc trần bẫy `min_charge` và bẫy `0 đ khoán`).
  4. `Features/ModelDeepDive`: Biểu đồ lịch sử giá exact decimal + Capability Diff 19 trường Ckey + Live Tester bắn request thật đo TTFT, tok/s, chi phí thực và phát hiện model `MISMATCHED`.
- **Lệnh verify Phase 2 bất kỳ lúc nào:**
  ```powershell
  cargo test --test crawler_and_vault_integration; cargo clippy --workspace --all-targets -- -D warnings
  ```
