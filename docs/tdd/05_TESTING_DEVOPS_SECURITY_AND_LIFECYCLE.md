# Tài Liệu Thiết Kế Kỹ Thuật (TDD) — Phần 5: Bảo Mật, Kiểm Thử Tự Động (Exact Decimal), Observability & DevOps

* LƯU Ý QUAN TRỌNG: Mọi Unit Test và Integration Test liên quan đến giá tiền bắt buộc phải kiểm tra **tính toàn vẹn của các chữ số thập phân** (không cho phép sai lệch dù chỉ `0.000001 đ`), sử dụng trực tiếp các mẫu số lẻ thực tế từ `docs/api_test.json`.

---

## 1. Kiểm Thử Tự Động Với Số Liệu Exact Decimal Từ `docs/api_test.json`

| Kịch bản kiểm thử | Dữ liệu mẫu thực tế từ `docs/api_test.json` | Kết quả kỳ vọng bắt buộc (Exact Match) |
| :--- | :--- | :--- |
| **Tính Cache Read/Write lẻ 6 chữ số** | `anhyeu00m/claude-opus-5`: `in = 416.0`, `cache_read = 323.4816`, `out = 650.0` | `P_token_blended = 416.0*0.6 + 323.4816*0.3 + 650.0*0.1 =` **`411.64448 đ / 1M`** (Chính xác 5 số lẻ, cấm làm tròn `412`) |
| **Tính Cache Write lẻ 4 chữ số** | `chuitv1907/deepseek-v4.1-flash-1017-uncensored`: `cache_write = 14314.7732` | Parse và lưu trữ chuỗi `"14314.7732"` trong SQLite không mất bất kỳ chữ số nào. |
| **Bẫy Min Charge ở 50K tokens** | `rouyea98/claude-opus-4-6` (`in=474.5, cache_read=189.8, out=910.0, min=28.6`) vs `sypham98/claude-opus-4-6` (`in=526.5, cache_read=253.5, out=1170.0, min=5.2`) | `rouyea98` bị kéo sàn = **`572.0 đ / 1M`**; `sypham98` = **`508.95 đ / 1M`**. Xếp hạng: `sypham98` (#1) đứng TRÊN `rouyea98` (#2)! |
| **Bẫy Phí Request lẻ** | `tdsang1999/claude-opus-4.8`: `price_per_request_vnd = 88.37` | Chi phí request lưu đúng `"88.37"`, không làm tròn thành `"88"`. |
| **Bẫy Giá Khoán 0đ** | `thanhnhan9023/claude-opus-4-6`: `in=0, out=0, req=65.0` | Tính `effective_blended_1m = 65.0 * 20 =` **`1300.0 đ / 1M`** (Không bị rơi vào giá `0 đ` giả mạo). |

---

## 2. Bảo Mật Bí Mật, Observability & DevOps

- **Bảo vệ `?key=...`:** Masking 100% query string nhạy cảm thành `?key=***REDACTED***`.
- **Mã hóa AES-256-GCM:** Bảo vệ API Key và Proxy URL.
- **Structured JSON Logging:** Ghi log không làm tròn số tiền biến động (`PRICE_CHANGE`), xoay vòng hàng ngày + nén gzip.
- **Graceful Shutdown (`SIGTERM`/`SIGINT`):** Đóng an toàn pool SQLite và xả sạch event queue trước khi dừng tiến trình.
- **Container Production:** Dockerfile multi-stage đóng gói binary Rust + Leptos WASM, expose port `1098`.
