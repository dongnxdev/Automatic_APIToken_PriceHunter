# Roadmap Triển Khai Production — ModelRadar (Exact Decimal Ckey Pricing)

* LƯU Ý QUAN TRỌNG VỀ QUY TẮC KIỂM ĐỊNH:
  - Toàn bộ pipeline (Crawl, Ingestion, Database, Tính toán Blended, Xếp hạng, Gateway Proxy, Hiển thị UI) bắt buộc vận hành theo nguyên tắc **Exact Decimal (Zero Rounding)**, giữ đúng 100% từng chữ số thập phân của Ckey (`323.4816`, `1779.1488`, `88.37`, `474.5`, `15.6`, `28.6`).
  - Mọi Unit/Integration Test phải assert bằng exact Decimal string match, không dùng so sánh xấp xỉ `float` (`assert_eq!(blended, Decimal::from_str("508.95").unwrap())`).

---

## Chuỗi Phụ Thuộc Tuần Tự (Phase 0 -> Phase 7)

```text
Phase 0 (TDD & Spec Lock + Fixture api_test.json + Exact Decimal Rule)
   │
   ▼
Phase 1 (Workspace, Shared Contract Exact Decimal, Config & SQLite Migrations TEXT Decimal)
   │
   ▼
Phase 2 (Auth Management, Provider Vault, AES-GCM & CkeyLlmModelsAdapter Parse Exact Decimal)
   │
   ▼
Phase 3 (Canonical Mapper 650 Models, ArcSwap Exact Snapshot, Price Radar & Deep-Dive Tester)
   │
   ▼
Phase 4 (Smart Gateway Proxy /v1/*, Seller Failover <100ms, Price Breaker Exact Decimal)
   │
   ▼
Phase 5 (Ops Console, WebSocket /ws/ops, Alert Dispatcher & Seller Breaker Reset)
   │
   ▼
Phase 6 (Frontend Leptos WASM: App Shell + 6 Màn Hình Hiển Thị Exact Decimal Không Làm Tròn)
   │
   ▼
Phase 7 (Playwright E2E Kiểm Tra Hiển Thị Số Lẻ Thực Tế, Container Packaging & Handover)
```

---

## Phase 0: Chốt Đặc Tả Kỹ Thuật, Fixture Ckey & Nguyên Tắc Zero Rounding
- **Checklist nghiệm thu:**
  - [x] Khóa cứng nguyên tắc Zero Rounding (Exact Decimal), thu thập dữ liệu 650 models vào `docs/api_test.json`, cập nhật đồng bộ TDD `01..05` và Roadmap.

---

## Phase 1: Workspace, Shared Contract (`Decimal`), Config & SQLite Migrations
- **Đầu vào bắt buộc:** Phase 0 đạt `[x]`.
- **Phạm vi thực thi:**
  1. `rust-toolchain.toml` (`1.83.0`), `Cargo.toml` workspace.
  2. `crates/shared_contract`:
     - `currency_math.rs`: struct `ExactPrice`, `rust_decimal::Decimal`, công thức `ExactEffectiveBlended` (**cấm gọi `round()`**), formatters giữ nguyên chữ số có nghĩa.
     - `api_envelope.rs`, `enums.rs`, `model_capabilities.rs`, `ws_events.rs`.
  3. `backend/src/Shared/`:
     - `Common/config.rs`: đọc `appsettings.json` (tương thích `Readme.md`), bóc tách `?key=`, fail-fast.
     - `Common/crypto.rs`: AES-256-GCM.
     - `Data/sqlite_pool.rs`: SQLite WAL + `sqlx::migrate!`.
     - `Utils/`: UTC ms, NFC/trim/lowercase, `MaskingVisitor` (`?key=***REDACTED***`).
  4. `migrations/0001_init.sql` (toàn bộ cột giá là `TEXT` Decimal chuẩn) và `0002_seed_default_rules.sql`.
  5. Unit Tests kiểm tra tính chính xác các số lẻ Ckey (`411.64448 đ`, `508.95 đ`, `572.0 đ`, `88.37 đ`).
- **Verify Gate:** `cargo clippy --workspace --all-targets -- -D warnings` và `cargo test -p shared_contract`.
- **Checklist bàn giao Phase 1:**
  - [x] Workspace biên dịch sạch, Migrations TEXT Decimal thành công, Unit Tests exact Decimal pass 100%.

---

## Phase 2: Auth Management, Provider Vault & `CkeyLlmModelsAdapter` Exact Ingestion
- **Đầu vào bắt buộc:** Phase 1 đạt `[x]`.
- **Phạm vi thực thi:**
  1. `Features/AuthManagement`: Login/Logout Argon2id + HttpOnly JWT, `/health/*`, `/metrics`.
  2. `Features/ProviderVault`: CRUD Provider, `QUERY_KEY` auth, reveal secret audit, JSON Sandbox preview exact decimal.
  3. `Features/CrawlerEngine`: `CkeyLlmModelsAdapter` parse 650 models từ Ckey nạp thẳng vào `Decimal` không làm tròn, Delta-Write SQLite, Provider Circuit Breaker, DLQ.
  4. Integration Tests với `wiremock` phát lại `docs/api_test.json`.
- **Verify Gate:** `cargo clippy -p modelradar-server --all-targets -- -D warnings` và `cargo test --test crawler_and_vault_integration`.
- **Checklist bàn giao Phase 2:**
  - [x] Auth, Provider Vault, và Crawler parse trọn vẹn 650 models từ `docs/api_test.json` giữ đúng 100% số lẻ thập phân pass tests.

---

## Phase 3: Canonical Mapper, PriceSnapshotIndex, Price Radar & Deep-Dive + Live Tester
- **Đầu vào bắt buộc:** Phase 2 đạt `[x]`.
- **Phạm vi thực thi:**
  1. `Features/CanonicalMapper`: Chuẩn hóa 650 models Ckey.
  2. `PriceSnapshotIndex`: Gom nhóm `seller_namespace` theo `canonical_id`, sắp xếp bằng `Decimal` chính xác.
  3. `Features/PriceRadar`: API `/api/v1/price-radar` lọc theo 4 Workload, xếp hạng chuẩn xác bẫy `min_charge` và bẫy `0 đ khoán`, trả về Ngai Vàng #1 và Ma Trận đầy đủ số lẻ.
  4. `Features/ModelDeepDive`: Biểu đồ lịch sử giá exact decimal + Capability Diff 19 trường Ckey + Live Tester bắn request thật tính chi phí chính xác từng hào.
- **Verify Gate:** `cargo clippy -p modelradar-server --all-targets -- -D warnings` và `cargo test --test mapper_radar_deepdive_tests`.
- **Checklist bàn giao Phase 3:**
  - [ ] Xếp hạng Price Radar và Capability Diff giữ nguyên vẹn từng số lẻ Ckey pass 100% tests.

---

## Phase 4: Smart Gateway Reverse Proxy, Seller Failover & Price Circuit Breaker Exact
- **Đầu vào bắt buộc:** Phase 3 đạt `[x]`.
- **Phạm vi thực thi:**
  1. `Features/SmartGateway`:
     - Internal Keys (`mr-live-...`).
     - Proxy `/v1/*` định tuyến sang seller rẻ nhất trên Ckey, Seller Auto-Failover `<100ms`.
     - Price Circuit Breaker so sánh chính xác theo Decimal (chặn nếu vượt trần an toàn, không làm tròn).
     - Async batch usage logs & tính tiền tiết kiệm.
- **Verify Gate:** `cargo clippy -p modelradar-server --all-targets -- -D warnings` và `cargo test --test smart_gateway_tests`.
- **Checklist bàn giao Phase 4:**
  - [ ] Proxy `/v1/*`, Seller Failover và Price Breaker exact decimal pass 100% tests.

---

## Phase 5: Ops Console, WebSocket `/ws/ops`, Alert Dispatcher & Breaker Recovery
- **Đầu vào bắt buộc:** Phase 4 đạt `[x]`.
- **Phạm vi thực thi:**
  1. `Features/OpsConsole` & WS Hub: WebSocket `/ws/ops`, Alert Dispatcher, Circuit Breaker Reset 2 cấp.
  2. `DeadLetterReplayer`, Data Retention và Graceful Shutdown.
- **Verify Gate:** `cargo clippy -p modelradar-server --all-targets -- -D warnings` và `cargo test --test ops_ws_alerts_tests`.
- **Checklist bàn giao Phase 5:**
  - [ ] WebSocket `/ws/ops` và Circuit Breaker Reset 2 cấp pass 100% tests.

---

## Phase 6: Frontend Leptos WASM — App Shell & 6 Màn Hình Hiển Thị Exact Decimal
- **Đầu vào bắt buộc:** Phase 5 đạt `[x]`.
- **Phạm vi thực thi:**
  1. Setup Trunk, Tailwind tokens, `GlobalAppShell` (Sidebar + 48px Top Bar).
  2. Xây dựng 6 màn hình (`/radar`, `/models/:id`, `/providers`, `/aliases`, `/gateway`, `/ops`) hiển thị đúng 100% từng số thập phân có nghĩa (`474.5 đ`, `88.37 đ`, `323.4816 đ`, `508.95 đ`), tuyệt đối không làm tròn số nguyên.
  3. Serve static `frontend/dist` từ Axum port `1098`.
- **Verify Gate:** `cargo clippy -p modelradar-web --target wasm32-unknown-unknown -- -D warnings` và `trunk build --release`.
- **Checklist bàn giao Phase 6:**
  - [ ] Toàn bộ 6 màn hình Leptos WASM build sạch, hiển thị chính xác từng số thập phân Ckey, zero rounding.

---

## Phase 7: E2E Playwright, Soak Audit & Container Production
- **Đầu vào bắt buộc:** Phase 6 đạt `[x]`.
- **Phạm vi thực thi:**
  1. Playwright E2E verify UI hiển thị đúng số lẻ từ `docs/api_test.json`, verify flow định tuyến, live test, failover.
  2. Soak Audit (zero memory leak, zero rounding bug).
  3. Multi-stage Dockerfile + docker-compose.
- **Verify Gate:** `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `docker build -t modelradar:prod .`.
- **Checklist bàn giao Phase 7:**
  - [ ] 100% Tests pass, Zero Warnings, Production Container build & verify xanh 100%.
