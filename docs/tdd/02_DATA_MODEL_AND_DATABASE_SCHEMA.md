# Tài Liệu Thiết Kế Kỹ Thuật (TDD) — Phần 2: Mô Hình Dữ Liệu, Schema DB & Công Thức Tính Giá (Exact Decimal Ckey)

* LƯU Ý QUAN TRỌNG: Toàn bộ bảng dữ liệu dưới đây lưu trữ **chính xác 100% từng chữ số thập phân của Ckey** (không qua làm tròn), sử dụng kiểu **`TEXT` trong SQLite** để ánh xạ hoàn hảo sang **`rust_decimal::Decimal` trong Rust** (28 chữ số chính xác, không dùng `float`, không làm tròn số nguyên). Khóa chính **UUIDv7**, lưu thời gian UTC Milliseconds (`i64`), SQLite WAL mode pool `sqlx 0.8.x`.

---

## 1. Phân Tích Độ Phân Giải Thập Phân Thực Tế Từ `docs/api_test.json` (650 Models)

Kiểm tra trực tiếp trên 650 models trong `docs/api_test.json` cho thấy độ phân giải thập phân thực tế:

| Trường giá gốc từ Ckey | Đơn vị đo thực tế của Ckey | Số chữ số thập phân tối đa tìm thấy | Ví dụ thực tế từ `docs/api_test.json` |
| :--- | :--- | :---: | :--- |
| `input_price_per_million_vnd` | **VNĐ / 1M tokens** | 3 chữ số | `bamzdwi/deepseek-v4.1-flash`: `185.25 đ`<br>`bamzdwi/gpt-6-luna`: `123.5 đ`<br>`rouyea98/claude-opus-4-6`: `474.5 đ` |
| `output_price_per_million_vnd` | **VNĐ / 1M tokens** | 3 chữ số | `bamzdwi/glm-5.3-flash`: `61.75 đ`<br>`bamzdwi/DeepSeek-V4.1-Flash-1017-UNCENSORED`: `271.7 đ` |
| `cache_read_price_per_million_vnd` | **VNĐ / 1M tokens** | **6 chữ số** | `anhyeu00m/claude-opus-5`: `323.4816 đ`<br>`phamhuyhoang28072005/deepseek-v4.1-flash-1017`: `299.8905 đ`<br>`chuitv1907/deepseek-v4.1-flash-1017-uncensored`: `3515.8591 đ`<br>`phamhuyhoang28072005/gemini-4-carbon`: `4084.223 đ` |
| `cache_write_price_per_million_vnd` | **VNĐ / 1M tokens** | **6 chữ số** | `ipeenkairdrop/claude-opus-4.6`: `773.5 đ`<br>`anhyeu00m/claude-opus-5`: `1779.1488 đ`<br>`phamhuyhoang28072005/deepseek-v4.1-flash-1017`: `1992.4593 đ`<br>`chuitv1907/deepseek-v4.1-flash-1017-uncensored`: `14314.7732 đ`<br>`phamhuyhoang28072005/gemini-4-carbon`: `10210.5575 đ` |
| `price_per_request_vnd` | **VNĐ / request** | 2 chữ số | `h2ketnoi/claude-opus-4-6-thinking`: `15.6 đ`<br>`tdsang1999/claude-opus-4.8`: `88.37 đ`<br>`thanhnhan9023/claude-opus-4-6`: `65 đ`<br>`trikiet2045/claude-opus-4.6-thinking`: `1.3 đ` |
| `min_charge_per_request_vnd` | **VNĐ / request** | 2 chữ số | `sypham98/claude-opus-4-6`: `5.2 đ`<br>`trikiet2045/claude-opus-4.6-thinking`: `14.3 đ`<br>`rouyea98/claude-opus-4-6`: `28.6 đ`<br>`ngovutuananh/claude-opus-4.8`: `69 đ` |
| `price_per_second_vnd` | **VNĐ / giây** | 0 chữ số | `MiniMax-H3`: `1200 đ/giây` |

* KẾT LUẬN KIẾN TRÚC: **Nếu dùng kiểu làm tròn số nguyên thì toàn bộ các số lẻ trên (như `323.4816`, `1779.1488`, `474.5`, `15.6`, `88.37`) sẽ bị làm tròn bậy bạ, phá hủy hoàn toàn độ chính xác của bảng ma trận so sánh!** Do đó, toàn bộ cột giá trong SQLite được lưu dưới dạng `TEXT` chuẩn Decimal, được parse thẳng vào `rust_decimal::Decimal` trong Rust.

---

## 2. Chi Tiết Database Schema (`migrations/0001_init.sql`)

### 2.1. `provider_sources` — Kho Nguồn Crawl & API Key (Màn 3)
```sql
CREATE TABLE IF NOT EXISTS provider_sources (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    adapter_type TEXT NOT NULL CHECK (adapter_type IN ('CKEY_LLM', 'OPENAI_COMPAT', 'CUSTOM_JSON')),
    auth_mode TEXT NOT NULL DEFAULT 'QUERY_KEY' CHECK (auth_mode IN ('QUERY_KEY', 'BEARER_HEADER', 'X_API_KEY_HEADER')),
    auth_param_name TEXT NOT NULL DEFAULT 'key',
    models_endpoint_url TEXT NOT NULL,
    inference_base_url TEXT NOT NULL,
    api_key_ciphertext TEXT NOT NULL,
    api_key_masked TEXT NOT NULL,
    proxy_url_ciphertext TEXT,
    fx_currency TEXT NOT NULL DEFAULT 'VND' CHECK (fx_currency IN ('VND', 'USD', 'CNY')),
    fx_rate_to_vnd_decimal TEXT NOT NULL DEFAULT '1.0',
    price_multiplier_decimal TEXT NOT NULL DEFAULT '1.0',
    custom_mapping_json TEXT,
    scan_interval_seconds INTEGER NOT NULL DEFAULT 180 CHECK (scan_interval_seconds >= 15),
    is_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    status TEXT NOT NULL DEFAULT 'ONLINE' CHECK (status IN ('ONLINE', 'UNAUTHORIZED_401', 'RATE_LIMITED_429', 'TIMEOUT', 'CIRCUIT_OPEN', 'DISABLED')),
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    last_ping_ms INTEGER,
    last_model_count INTEGER NOT NULL DEFAULT 0,
    last_error_message TEXT,
    last_scanned_at_ms INTEGER,
    circuit_opened_at_ms INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
```

### 2.2. `canonical_models` — Danh Mục Model Chuẩn & Giá Đối Chiếu (Màn 1, 2, 4)
```sql
CREATE TABLE IF NOT EXISTS canonical_models (
    id TEXT PRIMARY KEY NOT NULL,
    canonical_id TEXT NOT NULL UNIQUE,              -- VD: claude-opus-4.6, claude-sonnet-4.6
    family TEXT NOT NULL,                           -- Anthropic, OpenAI, Google, DeepSeek, Qwen...
    modality TEXT NOT NULL DEFAULT 'TEXT' CHECK (modality IN ('TEXT', 'IMAGE', 'AUDIO', 'VIDEO', 'EMBEDDING')),
    official_anchor_input_vnd_decimal TEXT,         -- Giá đối chiếu chính xác nguyên bản (VD: "3000.0")
    official_anchor_output_vnd_decimal TEXT,        -- Giá đối chiếu chính xác nguyên bản (VD: "15000.0")
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
```

### 2.3. `model_aliases` & `regex_alias_rules` (Màn 4)
```sql
CREATE TABLE IF NOT EXISTS model_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    provider_source_id TEXT NOT NULL REFERENCES provider_sources(id) ON DELETE CASCADE,
    raw_public_name TEXT NOT NULL,                  -- VD: h2ketnoi/claude-opus-4-6-thinking
    raw_display_name TEXT NOT NULL,                 -- VD: claude-opus-4-6-thinking
    normalized_lookup_key TEXT NOT NULL,
    canonical_model_id TEXT REFERENCES canonical_models(id) ON DELETE SET NULL,
    mapping_status TEXT NOT NULL CHECK (mapping_status IN ('AUTO_MAPPED', 'MANUAL_MAPPED', 'UNMAPPED', 'BLACKLISTED')),
    first_seen_at_ms INTEGER NOT NULL,
    last_seen_at_ms INTEGER NOT NULL,
    UNIQUE(provider_source_id, raw_public_name)
);

CREATE INDEX IF NOT EXISTS idx_model_aliases_status ON model_aliases(mapping_status);

CREATE TABLE IF NOT EXISTS regex_alias_rules (
    id TEXT PRIMARY KEY NOT NULL,
    rule_name TEXT NOT NULL,
    rule_type TEXT NOT NULL CHECK (rule_type IN ('STRIP_PREFIX', 'VERSION_NORMALIZE', 'EXACT_ALIAS', 'BLACKLIST_IGNORE')),
    pattern_regex TEXT NOT NULL,
    replacement_template TEXT NOT NULL,
    priority_order INTEGER NOT NULL DEFAULT 100,
    is_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
```

### 2.4. `provider_model_quotes` — Bảng Giá & Capability Chuẩn Hóa (Lưu Đúng 100% Giá Ckey)
* LƯU Ý: Mọi cột giá đều lưu dạng `TEXT` đại diện cho `rust_decimal::Decimal`, đảm bảo giữ nguyên từng chữ số thập phân (`323.4816`, `1779.1488`, `88.37`, `474.5`, `15.6`, `28.6`). Cột `effective_blended_vnd_decimal` lưu kết quả tính toán Blended chính xác (không làm tròn) để phục vụ query order index.

```sql
CREATE TABLE IF NOT EXISTS provider_model_quotes (
    id TEXT PRIMARY KEY NOT NULL,
    provider_source_id TEXT NOT NULL REFERENCES provider_sources(id) ON DELETE CASCADE,
    canonical_model_id TEXT REFERENCES canonical_models(id) ON DELETE SET NULL,
    seller_namespace TEXT NOT NULL,                 -- VD: 'rouyea98', 'sypham98', 'direct'
    public_name TEXT NOT NULL,                      -- VD: 'rouyea98/claude-opus-4-6'
    display_name TEXT NOT NULL,                     -- VD: 'claude-opus-4-6'
    is_provider_model INTEGER NOT NULL CHECK (is_provider_model IN (0, 1)),
    pricing_model_kind TEXT NOT NULL CHECK (pricing_model_kind IN ('PER_TOKEN', 'FLAT_PER_REQUEST', 'PER_SECOND_VIDEO')),
    input_vnd_decimal TEXT NOT NULL,                -- Chính xác 100% chuỗi Decimal của Ckey
    output_vnd_decimal TEXT NOT NULL,
    cache_enabled INTEGER NOT NULL DEFAULT 0 CHECK (cache_enabled IN (0, 1)),
    cache_read_vnd_decimal TEXT NOT NULL DEFAULT '0.0',   -- VD: "323.4816", "253.5", "0.0"
    cache_write_vnd_decimal TEXT NOT NULL DEFAULT '0.0',  -- VD: "1779.1488", "773.5", "0.0"
    price_per_request_vnd_decimal TEXT NOT NULL DEFAULT '0.0', -- VD: "15.6", "88.37", "0.0"
    min_charge_per_request_vnd_decimal TEXT NOT NULL DEFAULT '0.0', -- VD: "5.2", "28.6", "0.0"
    price_per_second_vnd_decimal TEXT NOT NULL DEFAULT '0.0',   -- VD: "1200.0", "0.0"
    effective_blended_vnd_decimal TEXT NOT NULL DEFAULT '0.0',  -- Giá Blended chính xác của profile mặc định
    request_rate_limit_per_minute INTEGER NOT NULL DEFAULT 0,
    max_output_tokens_limit INTEGER NOT NULL DEFAULT 0,
    context_tokens_limit INTEGER NOT NULL DEFAULT 0,
    supported_paths_json TEXT NOT NULL DEFAULT '[]',
    video_durations_allowed_json TEXT NOT NULL DEFAULT '[]',
    video_max_seconds INTEGER NOT NULL DEFAULT 0,
    video_rate_cards_json TEXT,
    video_contract_json TEXT,
    is_media_video_capable INTEGER NOT NULL DEFAULT 0 CHECK (is_media_video_capable IN (0, 1)),
    seller_circuit_status TEXT NOT NULL DEFAULT 'CLOSED' CHECK (seller_circuit_status IN ('CLOSED', 'OPEN', 'HALF_OPEN')),
    seller_consecutive_failures INTEGER NOT NULL DEFAULT 0,
    raw_payload_hash TEXT NOT NULL,
    last_live_test_status TEXT CHECK (last_live_test_status IN ('MATCHED', 'MISMATCHED', 'FAILED') OR last_live_test_status IS NULL),
    last_live_test_ttft_ms INTEGER,
    last_live_test_tps_decimal TEXT,
    updated_at_ms INTEGER NOT NULL,
    UNIQUE(provider_source_id, public_name)
);

CREATE INDEX IF NOT EXISTS idx_quotes_canonical ON provider_model_quotes(canonical_model_id);
CREATE INDEX IF NOT EXISTS idx_quotes_seller ON provider_model_quotes(seller_namespace);
CREATE INDEX IF NOT EXISTS idx_quotes_blended ON provider_model_quotes(canonical_model_id, effective_blended_vnd_decimal);
```

### 2.5. `price_history_points`, `routing_policies`, `internal_api_keys`, `gateway_usage_logs`, `alert_rules`, `alert_deliveries`, `dead_letter_events`
- Toàn bộ cột giá trong các bảng này chuyển sang `TEXT` (lưu giá trị Decimal chính xác của VNĐ: `input_vnd_decimal`, `output_vnd_decimal`, `cache_read_vnd_decimal`, `blended_vnd_decimal`, `price_ceiling_vnd_decimal`, `actual_cost_vnd_decimal`, `anchor_cost_vnd_decimal`, `saved_vnd_decimal`), loại bỏ hoàn toàn việc ép kiểu số nguyên làm tròn.

---

## 3. Công Thức Tính Giá Exact Effective Blended (Zero Rounding)

### 3.1. Tính Toán Chuẩn 100% Bằng `rust_decimal::Decimal`
Không dùng bất kỳ phép chia lấy phần nguyên hay hàm `round()` nào. Mọi phép toán thực hiện trực tiếp trên struct `Decimal`:

1. **Xác định giá Cache hiệu dụng (`P_cache_eff`):**
   - Nếu `cache_enabled == true` và `cache_read_vnd_decimal > Decimal::ZERO`: `P_cache_eff = cache_read_vnd_decimal`.
   - Nếu `cache_enabled == false`: `P_cache_eff = input_vnd_decimal` (vì 30% lượng token cache bị tính giá Input gốc).
2. **Giá Token Blended thuần (`P_token_blended` / 1M tokens):**
   - Profile `Coding / Agent`:
     `P_token_blended = (input_vnd_decimal * Decimal::new(60, 2)) + (P_cache_eff * Decimal::new(30, 2)) + (output_vnd_decimal * Decimal::new(10, 2))`
   - Profile `Chat thường`:
     `P_token_blended = (input_vnd_decimal * Decimal::new(75, 2)) + (output_vnd_decimal * Decimal::new(25, 2))`
   - Profile `Dịch thuật`:
     `P_token_blended = (input_vnd_decimal * Decimal::new(50, 2)) + (output_vnd_decimal * Decimal::new(50, 2))`
3. **Giá Exact Effective Blended theo kích thước Request (`assumed_tokens_per_req`):**
   - Ví dụ với `Coding / Agent` (mặc định `50.000` tokens/req):
     ```text
     tokens_ratio           = Decimal(assumed_tokens_per_req) / Decimal(1_000_000)   -- VD: 0.05
     requests_per_1m        = Decimal(1_000_000) / Decimal(assumed_tokens_per_req)   -- VD: 20
     raw_token_cost_per_req = P_token_blended * tokens_ratio
     total_cost_per_req     = raw_token_cost_per_req + price_per_request_vnd_decimal
     effective_cost_per_req = max(total_cost_per_req, min_charge_per_request_vnd_decimal)
     effective_blended_1m   = effective_cost_per_req * requests_per_1m
     ```

### 3.2. Bằng Chứng Tính Toán Thực Tế (Không Làm Tròn)
- **Kênh `sypham98/claude-opus-4-6`:**
  - `input = 526.5 đ`, `cache_read = 253.5 đ`, `output = 1170.0 đ`, `min_charge = 5.2 đ`, `req = 0.0 đ`.
  - `P_token_blended = 526.5 * 0.6 + 253.5 * 0.3 + 1170 * 0.1 = 315.9 + 76.05 + 117.0 = 508.95 đ / 1M`.
  - `raw_token_cost_per_req = 508.95 * 0.05 = 25.4475 đ`.
  - Vì `25.4475 đ > 5.2 đ` -> `effective_cost_per_req = 25.4475 đ`.
  - `effective_blended_1m = 25.4475 * 20 =` **`508.95 đ / 1M`** (Giữ chính xác `508.95`, tuyệt đối không làm tròn thành `509`).
- **Kênh `rouyea98/claude-opus-4-6`:**
  - `input = 474.5 đ`, `cache_read = 189.8 đ`, `output = 910.0 đ`, `min_charge = 28.6 đ`, `req = 0.0 đ`.
  - `P_token_blended = 474.5 * 0.6 + 189.8 * 0.3 + 910 * 0.1 = 284.7 + 56.94 + 91.0 = 432.64 đ / 1M`.
  - `raw_token_cost_per_req = 432.64 * 0.05 = 21.632 đ`.
  - Vì `21.632 đ < 28.6 đ` -> dính sàn `min_charge = 28.6 đ`!
  - `effective_blended_1m = 28.6 * 20 =` **`572.0 đ / 1M`**.
  - Kết quả so sánh chính xác tuyệt đối: **`508.95 đ < 572.0 đ`**, chứng minh `sypham98` rẻ hơn `rouyea98` chính xác **`63.05 đ / 1M`**! Nếu làm tròn số trung gian thì sự chênh lệch này bị sai lệch hoàn toàn.
- **Kênh `anhyeu00m/claude-opus-5` (Số lẻ 4 chữ số):**
  - `input = 416.0 đ`, `cache_read = 323.4816 đ`, `output = 650.0 đ`.
  - `P_token_blended = 416.0 * 0.6 + 323.4816 * 0.3 + 650.0 * 0.1 = 249.6 + 97.04448 + 65.0 =` **`411.64448 đ / 1M`**.
  - Hệ thống bảo toàn trọn vẹn số **`411.64448 đ / 1M`** trên toàn bộ pipeline!
