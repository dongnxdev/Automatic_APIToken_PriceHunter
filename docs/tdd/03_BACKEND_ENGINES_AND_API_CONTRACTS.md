# Tài Liệu Thiết Kế Kỹ Thuật (TDD) — Phần 3: Backend Engines, Reverse Proxy & API Contracts (Exact Decimal Ckey)

* LƯU Ý QUAN TRỌNG: Mọi tính toán, so sánh xếp hạng tại `PriceRadar`, kiểm tra trần giá tại `SmartGateway`, và xác minh tại `ModelDeepDive` đều vận hành trên kiểu **`rust_decimal::Decimal` chính xác 100% theo đơn vị hiện tại của Ckey.vn (VNĐ)**, tuân thủ nghiêm ngặt nguyên tắc **KHÔNG LÀM TRÒN (Zero Rounding)**.

---

## 1. Crawler Engine & Ckey Adapter (`Features/CrawlerEngine` & `Features/ProviderVault`)

### 1.1. `CkeyLlmModelsAdapter` (Xử Lý Exact Decimal)
- **Crawl Endpoint:** `https://ckey.vn/api/llm/models?key={decrypted_key}` qua `auth_mode = QUERY_KEY`.
- **Zero Rounding Ingestion:**
  - Parse trực tiếp các trường số từ JSON Ckey sang `rust_decimal::Decimal` (hỗ trợ đầy đủ số lẻ tới 6 chữ số thập phân: `323.4816`, `1779.1488`, `474.5`, `15.6`, `88.37`).
  - Lưu nguyên vẹn chuỗi Decimal chuẩn vào SQLite (`input_vnd_decimal`, `output_vnd_decimal`, `cache_read_vnd_decimal`, `cache_write_vnd_decimal`, `price_per_request_vnd_decimal`, `min_charge_per_request_vnd_decimal`, `price_per_second_vnd_decimal`).
  - Tính `effective_blended_vnd_decimal` bằng thuật toán Decimal thuần túy và ghi đồng thời vào bảng `provider_model_quotes` qua batch transaction (Delta-Write khi `raw_payload_hash` thay đổi).
  - Tách `seller_namespace` từ `public_name` (`"rouyea98"`, `"sypham98"`, `"direct"`...).

### 1.2. JSON Parser Sandbox (Màn 3 — `/api/v1/providers/sandbox-parse`)
- Khi Operator nhập endpoint và key trong Drawer Màn 3 và bấm `Test Fetch & Preview`:
  - Trả về `raw_sample_json` (chứa các số lẻ nguyên bản) và `normalized_preview` (giữ nguyên từng số thập phân: `in_vnd: 474.5`, `out_vnd: 910`, `cache_read_vnd: 189.8`, `min_charge_vnd: 28.6`, `effective_blended_vnd: 572.0`).

### 1.3. Circuit Breaker 2 Cấp (Provider Level + Seller Level)
- Cấp 1 (`Ckey.vn`): trip khi 1 lần lỗi `401` hoặc 3 lần lỗi kết nối liên tiếp.
- Cấp 2 (`Seller Channel`): trip riêng kênh bán đó (`rouyea98/claude-opus-4-6`) khi dính `429` (vượt RPM 10/100) hoặc `5xx` hoặc `MISMATCHED` model, không làm gián đoạn các kênh khác của Ckey.

---

## 2. Canonical Mapper Engine (`Features/CanonicalMapper`)

- Tiền xử lý: `trim()` -> NFC -> `lowercase()`.
- Chạy tập luật Regex Rules trên 650 models Ckey:
  - `STRIP_PREFIX`: Cắt `^[^/]+/`.
  - `VERSION_NORMALIZE`: `(\d+)-(\d+)` -> `$1.$2`, `^anthropic-` -> `""`.
  - `BLACKLIST_IGNORE`: `^(text-embedding|bge-|rerank)` (cho phép bật/tắt hiển thị ở tab Media/Embedding).
  - Phân biệt biến thể hậu tố `-thinking`, `-kiro`, `-china`, `-indosia`, `-uncensored` theo quyết định của Operator trong Tab 1.

---

## 3. Price Radar & Model Deep-Dive + Live Tester (`Features/PriceRadar` & `Features/ModelDeepDive`)

### 3.1. Price Radar Ranking (`/api/v1/price-radar`)
- **Xếp hạng chính xác tuyệt đối:**
  - So sánh trực tiếp giá trị `effective_blended_vnd_decimal` kiểu `rust_decimal::Decimal` giữa các kênh bán của cùng một `canonical_id`.
  - Kênh nào có giá Blended thấp hơn dù chỉ `0.0001 đ` cũng được xếp trên một cách chính xác (không bị dồn chung hạng do làm tròn số nguyên).
  - Trả về `Ngai Vàng #1` và `Ma Trận So Sánh` giữ nguyên vẹn các chữ số thập phân (`508.95 đ`, `572.0 đ`, `718.0 đ`, `411.64448 đ`).

### 3.2. Model Deep-Dive & Live Tester (`/api/v1/models/{canonical_id}/deep-dive` & `live-test`)
- **Capability Diff:** Soi từng số lẻ chính xác của Ckey:
  - `cache_read_vnd_decimal` / `cache_write_vnd_decimal` (VD: `253.5 đ / 684.45 đ` vs `323.4816 đ / 1779.1488 đ`).
  - `price_per_request_vnd_decimal` & `min_charge_per_request_vnd_decimal` (VD: `15.6 đ`, `28.6 đ`, `88.37 đ`).
  - `request_rate_limit_per_minute` (RPM limit).
- **Live Ping & Token Tester:** Bắn request thật tới `https://ckey.vn/v1/chat/completions` (hoặc `/v1/messages`), đo `TTFT`, `tok/s`, tính chi phí thực tế chính xác bằng Decimal:
  `cost_vnd = max(token_count * token_rate + price_per_request, min_charge)` (không làm tròn), và xác minh model integrity chống treo đầu dê bán thịt chó (`MISMATCHED`).

---

## 4. Smart Gateway Reverse Proxy (`Features/SmartGateway`)

- **Định tuyến & Failover `< 100ms`:**
  - Tra cứu kênh bán rẻ nhất có `canonical_id` tương ứng, còn hạn mức RPM và trạng thái `seller_circuit_status != OPEN`.
  - **Price Circuit Breaker Chính Xác:** So sánh `effective_blended_vnd_decimal <= price_ceiling_vnd_decimal` (Decimal vs Decimal, ví dụ trần `3000.0 đ / 1M`). Nếu giá kênh khả dụng vọt quá trần an toàn -> chặn request ngay lập tức (`402/429`), không làm tròn khiến request bị lọt lưới.
  - Failover tự động sang kênh kế tiếp nếu kênh đầu dính `429`/`5xx` trước byte đầu tiên.
  - Thống kê tiền tiết kiệm chính xác: `saved_vnd_decimal = max(Decimal::ZERO, anchor_cost - actual_cost)`.
