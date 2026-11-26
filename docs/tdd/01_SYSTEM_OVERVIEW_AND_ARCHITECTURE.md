# Tài Liệu Thiết Kế Kỹ Thuật (TDD) — Phần 1: Tổng Quan Hệ Thống & Kiến Trúc Cốt Lõi

* LƯU Ý QUAN TRỌNG: Tài liệu này là bản chốt kiến trúc Production cho dự án **ModelRadar (Price API Hunter & Smart Gateway)**. Đơn vị giá và số liệu tài chính được chuẩn hóa **CHÍNH XÁC TUYỆT ĐỐI THEO HIỆN TẠI CỦA CKEY.VN** (đối chiếu trực tiếp với 650 models thực tế trong `docs/api_test.json`). **QUY TẮC CỨNG: TUYỆT ĐỐI KHÔNG ĐƯỢC PHÉP LÀM TRÒN** (Zero Rounding) ở bất kỳ tầng nào (Crawl, Ingestion, Database, Tính toán Blended, Xếp hạng hay Hiển thị UI), vì bất kỳ hành vi làm tròn số nguyên nào cũng làm sai lệch hoàn toàn bản chất phân tích và gây thiệt hại tài chính khi định tuyến.

---

## 1. Phân Tích Toàn Diện Theo Hệ Quy Chiếu 5W2H & 4M1E

### 1.1. Hệ quy chiếu 5W2H
- **What (Làm cái gì?):** Xây dựng hệ thống **ModelRadar** phục vụ săn giá, kiểm chứng chất lượng và định tuyến API LLM theo thời gian thực trên nền tảng **Ckey.vn** (chuẩn bị sẵn kiến trúc Adapter để mở rộng sang các sàn khác về sau):
  1. **Price API Hunter (Radar Săn Giá):** Tự động thu thập toàn bộ 650+ model/kênh bán (`seller_namespace` + `public_name`) từ endpoint `https://ckey.vn/api/llm/models?key=<API_KEY>`, chuẩn hóa 19 trường dữ liệu thực tế:
     - Giá token chính xác nguyên bản: `input_price_per_million_vnd`, `output_price_per_million_vnd`, `cache_read_price_per_million_vnd`, `cache_write_price_per_million_vnd` (chứa số lẻ lên tới **6 chữ số thập phân**, ví dụ `323.4816 đ`, `1779.1488 đ`, `299.8905 đ`, `1992.4593 đ`, `474.5 đ`, `773.5 đ`).
     - Phụ phí & Đơn giá: `price_per_request_vnd` (VD: `15.6 đ`, `88.37 đ`), `min_charge_per_request_vnd` (VD: `5.2 đ`, `28.6 đ`, `69 đ`), `price_per_second_vnd` (VD: `1200 đ/s`).
     - Thông số kỹ thuật: `request_rate_limit_per_minute` (RPM: `0`, `10`, `100`), `context_tokens_limit` (`0`, `400000`, `1000000`), `max_output_tokens_limit`, `supported_paths`, `video_contract`, `video_rate_cards`, `video_durations_allowed`, `video_max_seconds`.
     - Gom nhóm theo `canonical_id` và xếp hạng kênh bán rẻ nhất theo công thức **Exact Effective Blended Price** (tính bằng `rust_decimal::Decimal` chính xác tuyệt đối, **không làm tròn**).
  2. **Smart Gateway (Reverse Proxy Thông Minh):** Đóng vai trò trạm trung chuyển API tương thích đa giao thức (`/v1/chat/completions`, `/v1/messages`, `/v1/responses`, `/v1/models`) cho Cursor, Cline, Claude Code; tự động chọn kênh bán rẻ nhất trên Ckey (ví dụ `rouyea98/claude-opus-4-6` thay vì `claude-opus-4.6` direct đắt gấp nhiều lần), tự động failover `<100ms` sang kênh kế tiếp khi gặp `429`/`5xx`/vượt RPM, áp trần giá an toàn (`Price Circuit Breaker`), đo tiền tiết kiệm thực tế và gửi cảnh báo Telegram/Discord/Webhook.
- **Why (Tại sao phải cấm tiệt làm tròn và giữ đúng đơn vị Ckey?):**
  - Thực tế dữ liệu trong `docs/api_test.json` chứng minh:
    - Kênh `anhyeu00m/claude-opus-5` có `cache_read = 323.4816 đ` và `cache_write = 1779.1488 đ`. Nếu làm tròn thành `323 đ` và `1779 đ` là làm sai lệch bảng tính chi phí khi agent chạy hàng chục triệu token cache.
    - Kênh `tdsang1999/claude-opus-4.8` có `price_per_request_vnd = 88.37 đ`. Nếu làm tròn thành `88 đ` là mất tiền chênh lệch của người dùng.
    - Kênh `h2ketnoi/claude-opus-4-6-thinking` có `price_per_request_vnd = 15.6 đ` và `min_charge_per_request_vnd = 15.6 đ`.
    - Kênh `rouyea98/claude-opus-4-6` (`in=474.5 đ`, `cache_read=189.8 đ`, `out=910 đ`, `min_charge=28.6 đ`) so với `sypham98/claude-opus-4-6` (`in=526.5 đ`, `cache_read=253.5 đ`, `out=1170 đ`, `min_charge=5.2 đ`): Tại prompt 50K token, `rouyea98` bị kéo sàn `28.6 đ * 20 = 572.0 đ / 1M`, trong khi `sypham98` mất chính xác `508.95 đ / 1M`. Nếu làm tròn thành `509 đ` hay làm tròn các bước trung gian thì thứ tự so sánh bị méo mó.
- **Where (Thực thi ở đâu?):** Chạy dưới dạng Single Native Binary Rust (hoặc Docker container stateless gắn volume SQLite WAL & logs) phục vụ cả Backend API, Reverse Proxy `/v1/*`, WebSocket `/ws/ops` và Leptos WASM Frontend trên cổng `1098` (`appsettings.json`).
- **Who (Ai tương tác?):** Operator quản trị hệ thống; các IDE/CLI (`Cursor`, `Cline`, `Claude Code`) kết nối Gateway; Background Workers (Crawler, Mapper, Alert Dispatcher, DLQ Replayer, Retention Cleaner).
- **When (Vòng đời kích hoạt?):** Khởi động Fail-fast validate `appsettings.json` -> Chạy `sqlx` Migrations -> Nạp `PriceSnapshotIndex` vào `ArcSwap` -> Mở cổng `1098` -> Quét định kỳ hoặc cưỡng bức qua `FORCE SCAN ALL`.
- **How (Hoạt động ra sao?):** Crawler đọc JSON 650+ models từ Ckey, nạp nguyên vẹn chuỗi số thập phân vào `rust_decimal::Decimal`, lưu DB dưới dạng chuỗi Decimal chuẩn (`TEXT`), tính toán xếp hạng bằng Decimal chính xác 100%, hoán đổi con trỏ `PriceSnapshotIndex` trên RAM và phát delta qua WebSocket.
- **How Much (Độ phức tạp & Tài nguyên?):** Payload phản hồi của Ckey khoảng `587 KB` cho `650` models; xử lý parse & build index trong `< 15ms`, RAM vận hành `< 65 MB` RSS, tra cứu xếp hạng & định tuyến Proxy ở độ phức tạp `O(1)`.

### 1.2. Hệ quy chiếu 4M1E
- **Man:** Operator nhìn thấy con số giá thật nguyên bản từng hào từng xu (`474.5 đ`, `88.37 đ`, `323.4816 đ`), không bị bẫy bởi số làm tròn vô căn cứ.
- **Machine:** Tận dụng kiểu fixed-point 128-bit của `rust_decimal::Decimal` (28 chữ số chính xác) để xử lý số học không có sai số máy tính.
- **Method:** **QUY TẮC BẢO TOÀN GIÁ GỐC:** Lưu trữ nguyên bản chuỗi số Ckey; tính toán trên `Decimal`; hiển thị đúng số thập phân có nghĩa; cấm tuyệt đối hàm làm tròn `round()`, `round_dp(0)`, `floor()`, `ceil()` trên dữ liệu giá Ckey.
- **Material:** Dữ liệu JSON thực tế từ Ckey (`docs/api_test.json`) và cấu hình `appsettings.json`.
- **Environment:** Thống nhất cho Local Windows/Linux, Staging và Docker Production.

---

## 2. Stack Công Nghệ Chốt Hạ (Technology Stack Lock)

* LƯU Ý: Khóa cứng phiên bản trong `Cargo.toml` và `rust-toolchain.toml`.

| Thành phần | Công nghệ & Crate / Tool chốt | Phiên bản khóa | Vai trò trong hệ thống |
| :--- | :--- | :--- | :--- |
| **Ngôn ngữ & Toolchain** | Rust (`stable`) + `wasm32-unknown-unknown` | `1.83.0` | Biên dịch Backend native & Frontend WASM |
| **Web / API / WS Server** | `axum` + `tower` + `tower-http` | `0.8.x` | REST `/api/v1/*`, Proxy `/v1/*`, WS `/ws/ops`, Static WASM |
| **Async Runtime** | `tokio` (`full`) + `tokio-util` | `1.41.x` | Scheduler, bounded channels, cancellation token |
| **HTTP Client & Proxy** | `reqwest` (`rustls-tls`, `socks`, `stream`, `gzip`) | `0.12.x` | Crawler Ckey (`?key=...`), Live Tester, Proxy streaming |
| **Database & Migration** | `sqlx` (`sqlite`, `runtime-tokio`, `migrate`) | `0.8.x` | SQLite WAL pool, parameterized queries, versioned SQL |
| **Toán học Tiền tệ** | `rust_decimal` (`serde-with-str`) | `1.36.x` | **Tính toán Decimal chính xác tuyệt đối, cấm làm tròn** |
| **Lock-free Snapshot** | `arc-swap` + `dashmap` | `1.7.x` / `6.1.x` | Read-heavy index cho `/radar` & `/v1/*` |
| **ID & Thời gian** | `uuid` (`v7`, `serde`) + `chrono` (`serde`) | `1.11.x` / `0.4.x` | UUIDv7 time-sortable, UTC ISO-8601 & int64 ms |
| **Validation & Regex** | `validator` + `regex` + `serde_json` | `0.19.x` / `1.11.x` | Validate 2 tầng DTO/Domain, Canonical Mapper |
| **Bảo mật & Mã hóa** | `aes-gcm` + `argon2` + `sha2` + `jsonwebtoken` + `zeroize` | Pin lockfile | Mã hóa `API_Key`/`Proxy`, Auth Management, Hash Internal Key |
| **Observability** | `tracing` + `tracing-subscriber` + `tracing-appender` + `metrics-exporter-prometheus` | Pin lockfile | JSON logs, rotation, `/metrics`, masking `?key=` & `sk-` |
| **Frontend WASM** | **Leptos** (`csr`) + `leptos_router` + `gloo-net` + `web-sys` | `0.7.x` | SPA 6 màn hình chạy trên WebAssembly |
| **Build Assets UI** | `trunk` + Tailwind CSS Standalone CLI | `0.21.x` / `3.4.15` | Đóng gói WASM + CSS local (zero CDN lúc chạy) |

---

## 3. Kiến Trúc Thư Mục Vertical Slice Architecture (Tỷ Lệ 6:4)

Cấu trúc `/Project_Root <Tỉ lệ hoàn hảo 6:4>` theo `Readme.md`, chia nhỏ nhiều file, mọi file luôn `< 1000` dòng:

```text
Automatic_APIToken_PriceHunter/
├── appsettings.json                        # Cấu hình runtime ngoài binary (Port 1098, Provider Ckey, Management, Flags)
├── appsettings.example.json                # Mẫu cấu hình chuẩn không chứa secret thật
├── Cargo.toml                              # Cargo Workspace: crates/shared_contract, backend, frontend
├── rust-toolchain.toml                     # Khóa Rust 1.83.0 + wasm32-unknown-unknown
├── Dockerfile                              # Multi-stage build (Trunk WASM + Rust Server)
├── docker-compose.yml                      # Triển khai kèm volume SQLite & logs
├── migrations/                             # SQLx migrations (0001_init.sql, 0002_seed_default_rules.sql)
├── docs/                                   # TDD (01..05), ROADMAP.md, api_test.json (fixture 650 models)
├── crates/
│   └── shared_contract/                    # DTOs, Enums, Decimal Exact Pricing Math, API Envelope
│       └── src/
│           ├── lib.rs
│           ├── api_envelope.rs             # ApiResponse<T>, ApiError, ApiMeta
│           ├── currency_math.rs            # Exact Decimal Math (cấm làm tròn), EffectiveBlendedFormula, String formatters
│           ├── model_capabilities.rs       # SupportedPath, VideoContract, VideoRateCard, QuoteCapabilities
│           ├── enums.rs                    # AdapterType, AuthMode, PricingModelKind, ProviderStatus
│           └── ws_events.rs                # WsServerEvent, WsClientCommand
├── backend/                                # [60%] Rust Server (Axum + Tokio + SQLite WAL)
│   └── src/
│       ├── main.rs                         # Fail-fast config -> DB migrate -> Build router -> Graceful shutdown
│       ├── app_state.rs                    # AppState (DbPool, ArcSwap<PriceIndex>, ArcSwap<FeatureFlags>, Channels)
│       ├── Shared/
│       │   ├── mod.rs
│       │   ├── Common/                     # Config loader (đọc key Readme.md), Error, Constants, Flags, AES-GCM
│       │   ├── Data/                       # SQLite WAL pool, Migration runner, PriceSnapshotIndex, Retention cleaner
│       │   ├── Utils/                      # Time UTC, NFC/Trim, Masking (?key=***, sk-****, mr-live-****), Regex
│       │   ├── Services/                   # Auth/JWT, ReqwestClientPool (Proxy + Keep-Alive), WS Broadcast hub
│       │   └── Results/                    # Envelope Axum IntoResponse, Metrics recorder, Audit writer
│       └── Features/                       # 8 Vertical Feature Slices độc lập (Common, Model, Services, Controller, Results)
│           ├── AuthManagement/
│           ├── CrawlerEngine/              # CkeyLlmModelsAdapter (Production) + Scheduler + DLQ
│           ├── PriceRadar/                 # Màn 1 (/radar): Exact Effective Blended Ranking theo seller Ckey
│           ├── ModelDeepDive/              # Màn 2 (/models/:id): Exact Price History, Capability Diff, Live Tester
│           ├── ProviderVault/              # Màn 3 (/providers): CRUD Nguồn Ckey, Encrypted Key Unlock, JSON Sandbox
│           ├── CanonicalMapper/            # Màn 4 (/aliases): Unmapped Queue, Regex Rules, Blacklist
│           ├── SmartGateway/               # Màn 5 (/gateway + /v1/*): Reverse Proxy, Failover <100ms, Price Breaker
│           └── OpsConsole/                 # Màn 6 (/ops + /ws/ops): Alert Rules, Circuit Breaker Reset, Live Log
├── frontend/                               # [40%] Leptos WASM Dashboard (Trunk + Tailwind CSS)
│   └── src/
│       ├── main.rs
│       ├── Shared/                         # GlobalAppShell (Sidebar + 48px TopBar), Signals, WS Client, Exact Price Formatters
│       └── Features/                       # 6 Vertical UI Feature Slices (PriceRadar, ModelDeepDive, ProviderVault, CanonicalMapper, SmartGateway, OpsConsole)
└── tests/                                  # Unit Tests (Exact Decimal), Wiremock Tests (dùng api_test.json), E2E Playwright
```

---

## 4. Quy Chuẩn Kỹ Thuật Cứng (Single Canonical Standard)

### 4.1. Thời gian, Văn bản & Case Handling
- 100% Backend/DB dùng **UTC** (`i64` Unix Milliseconds 13 số + ISO-8601 `YYYY-MM-DDTHH:mm:ss.sssZ`). Quy đổi `Asia/Ho_Chi_Minh` chỉ thực hiện tại Frontend.
- Văn bản 100% UTF-8 không BOM, LF (`\n`), chuẩn hóa **NFC** + `trim()`, chuỗi rỗng ép thành `null`. Các định danh (`canonical_id`, `seller_namespace`, `username`, `slug`) ép chữ thường trước khi lưu và so sánh.

### 4.2. Chuẩn Hóa Tiền Tệ & Đơn Vị Giá Gốc (Exact Decimal VNĐ Của Ckey — Zero Rounding)
1. **Đơn vị gốc:** Giữ đúng 100% đơn vị thực tế mà Ckey đang trả về:
   - Token prices: **VNĐ / 1.000.000 tokens** (kiểu `rust_decimal::Decimal`, lưu DB dạng `TEXT`).
   - Request fees: **VNĐ / request** (`rust_decimal::Decimal`).
   - Min charge: **VNĐ / request** (`rust_decimal::Decimal`).
   - Second prices: **VNĐ / giây** (`rust_decimal::Decimal`).
2. **Quy định cấm làm tròn:**
   - **CẤM** làm tròn về số nguyên (ví dụ `474.5 đ` phải hiển thị đúng `474.5 đ`, `88.37 đ` phải hiển thị đúng `88.37 đ`, `323.4816 đ` phải hiển thị đúng `323.4816 đ`, `15.6 đ` phải hiển thị đúng `15.6 đ`).
   - **CẤM** cắt xén các chữ số thập phân có nghĩa.
   - Khi chuyển sang hiển thị `USD` trên Top Bar: thực hiện chia cho `fx_rate_vnd_per_usd` (VD: `25450`) bằng `rust_decimal::Decimal` và **giữ tối thiểu 6 chữ số thập phân** (ví dụ `$0.020786` thay vì làm tròn thành `$0.02`).

### 4.3. Cấu Hình `appsettings.json` & Boundary Mapping theo `Readme.md`
Hỗ trợ đọc trực tiếp `appsettings.json` theo `Readme.md` (`Server.Port = 1098`, `Provider`, `Management`) và bóc tách an toàn `?key=` khỏi URL endpoint:
```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 1098,
    "database_url": "sqlite://data/modelradar.db?mode=rwc",
    "public_base_url": "http://localhost:1098"
  },
  "management": {
    "username": "admin",
    "password_hash": "$argon2id$v=19$m=19456,t=2,p=1$...",
    "session_ttl_seconds": 86400
  },
  "fx_defaults": {
    "usd_to_vnd_rate": "25450",
    "default_display_currency": "VND"
  },
  "crawler": {
    "auto_scan_enabled": true,
    "default_interval_seconds": 180,
    "request_timeout_ms": 15000,
    "max_concurrent_providers": 4,
    "queue_capacity": 256
  },
  "feature_flags": {
    "enable_auto_scan": true,
    "enable_smart_gateway": true,
    "enable_live_tester": true,
    "enable_alert_dispatcher": true,
    "enable_price_circuit_breaker": true
  },
  "provider": [
    {
      "name": "Ckey",
      "adapter_type": "CKEY_LLM",
      "auth_mode": "QUERY_KEY",
      "query_param_name": "key",
      "api_key": "${CKEY_API_KEY}",
      "endpoint": "https://ckey.vn/api/llm/models",
      "inference_base_url": "https://ckey.vn/v1",
      "fx_currency": "VND"
    }
  ]
}
```
