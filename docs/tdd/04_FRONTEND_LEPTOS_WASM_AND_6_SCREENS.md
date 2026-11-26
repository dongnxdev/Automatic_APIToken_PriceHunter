# Tài Liệu Thiết Kế Kỹ Thuật (TDD) — Phần 4: Frontend Leptos WASM, Design System & 6 Màn Hình Độc Lập (Hiển Thị Exact Decimal Ckey)

* LƯU Ý QUAN TRỌNG: Giao diện Web Frontend Leptos WASM (`0.7.x`) tuân thủ nghiêm ngặt nguyên tắc **KHÔNG LÀM TRÒN SỐ NGUYÊN (Zero Integer Rounding)** trên toàn bộ các bảng, thẻ số và đồ thị. Mọi giá tiền VNĐ được hiển thị trung thực với đầy đủ số thập phân có nghĩa từ dữ liệu Ckey (`474.5 đ`, `88.37 đ`, `323.4816 đ`, `508.95 đ`, `15.6 đ`).

---

## 1. Quy Chuẩn Format Hiển Thị Giá Tiền (Exact Price Formatter)

Trong `frontend/src/Shared/Utils/price_format.rs`:
1. **Chế độ VNĐ (Mặc định tự nhiên):**
   - Đọc trực tiếp chuỗi Decimal hoặc `rust_decimal::Decimal`.
   - Chuẩn hóa: Giữ nguyên các chữ số thập phân có nghĩa (chỉ cắt trailing zero thừa sau dấu phẩy nếu là `.000000`, nhưng giữ ít nhất 1 chữ số thập phân nếu vốn có số lẻ, ví dụ `474.5 đ`, `88.37 đ`, `323.4816 đ`).
   - Định dạng nhóm hàng nghìn bằng dấu chấm (`.`): `1.779,1488 đ` hoặc `1,779.1488 đ` (theo chuẩn thiết lập locale người dùng, mặc định theo phong cách hiển thị Việt Nam rõ ràng).
   - **TUYỆT ĐỐI CẤM** dùng `round(0)` để ép `474.5 đ` thành `475 đ`!
2. **Chế độ USD (Khi bật toggle trên Top Bar):**
   - Quy đổi: `usd_val = vnd_val / Decimal::from(fx_rate_vnd_per_usd)`.
   - Hiển thị với độ chính xác cao: giữ **tối thiểu 6 chữ số thập phân** (ví dụ `$0.018644` / 1M tokens), không làm tròn thành `$0.02`.

---

## 2. Chi Tiết 6 Màn Hình Độc Lập Với Số Liệu Ckey Chính Xác

### 2.1. Màn 1: Price Radar (`/radar`)
- **Ngai Vàng #1 (Spotlight Card):**
  - Hiển thị con số Display Large chính xác: `508.95 đ` (kèm chữ `Blended / 1M tokens`).
  - 4 ô breakdown hiển thị đúng số lẻ Ckey:
    - `Input / 1M`: `526.5 đ`
    - `Output / 1M`: `1.170.0 đ`
    - `Cache Read / 1M`: `253.5 đ`
    - `Max Context`: `1.000.000`
- **Bảng Ma Trận So Sánh:**
  - Cột `In / Out`: `526.5 / 1.170.0`, `474.5 / 910.0`
  - Cột `Min/Req`: `5.2 đ`, `28.6 đ`, `65.0 đ` (bóc trần bẫy `min_charge`)
  - Cột `Blended`: `508.95 đ`, `572.0 đ`, `718.0 đ`, `1.300.0 đ` (xếp đúng thứ tự, không bị dồn chung số)

### 2.2. Màn 2: Model Deep-Dive & Live Tester (`/models/:id`)
- **Biểu đồ Lịch sử Giá (SVG Pure Reactive):** Trục Y và tooltip hiển thị số thập phân chính xác của từng điểm giá (`323.4816 đ`, `508.95 đ`).
- **Bảng Capability Diff:** Hiển thị đúng 100% số lẻ của Ckey cho từng trường:
  - `cache_read / write`: `253.5 đ / 684.45 đ` vs `323.4816 đ / 1779.1488 đ`
  - `min_charge_per_request`: `5.2 đ` vs `28.6 đ`
  - `price_per_request`: `0.0 đ` vs `15.6 đ` vs `88.37 đ`
- **Live Tester Panel:** Bắn request thật, tính `Chi phí thực: ~25.4475 đ` (hiển thị đủ 4 số lẻ), không làm tròn thành `25 đ`.

### 2.3. Màn 3, Màn 4, Màn 5, Màn 6
- **Màn 3 (Provider Vault):** JSON Parser Sandbox hiển thị chuỗi Decimal gốc.
- **Màn 5 (Smart Gateway):** Input `Trần giá an toàn` chấp nhận số thập phân (VD: `3000.0 đ`). Thống kê `Tiền đã tiết kiệm` hiển thị chính xác từng đồng.
- **Màn 6 (Ops & Live Terminal):** Log `[PRICE_CHANGE]` ghi chi tiết từng con số lẻ: `giảm từ 740.5 đ -> 718.25 đ / 1M`.
