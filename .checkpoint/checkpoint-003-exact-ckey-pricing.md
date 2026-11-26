# Trạm Bàn Giao (Project Checkpoint) — Checkpoint 003

- **Thời điểm cập nhật (UTC):** `2026-11-26T03:36:00.000Z`
- **Tọa độ hiện tại (Current State):** Hoàn tất **Phase 0 (Khóa cứng nguyên tắc Exact Decimal — Zero Rounding theo đúng đơn vị hiện tại của Ckey.vn)**. Đã kiểm tra thực tế 650 models trong `docs/api_test.json`, phát hiện các trường số lẻ lên tới 6 chữ số thập phân (`323.4816 đ`, `1779.1488 đ`, `474.5 đ`, `88.37 đ`, `15.6 đ`), và cập nhật toàn bộ 5 tài liệu TDD (`docs/tdd/01..05`) + `docs/ROADMAP.md`.
- **Tiến độ dự án:** `Phase 0 / 7` hoàn thành (`100%` quy chuẩn dữ liệu chính xác tuyệt đối đã được niêm phong).

## Biến Động Workspace (File Changes)
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\01_SYSTEM_OVERVIEW_AND_ARCHITECTURE.md]` — Khóa cứng nguyên tắc Zero Rounding ở mọi tầng (Crawl, Ingestion, DB, Blended Math, Ranking, UI). Đơn vị gốc: VNĐ của Ckey (`rust_decimal::Decimal`).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\02_DATA_MODEL_AND_DATABASE_SCHEMA.md]` — Chuyển toàn bộ các cột giá trong SQLite sang `TEXT` lưu chuỗi Decimal chuẩn (`input_vnd_decimal`, `output_vnd_decimal`, `cache_read_vnd_decimal`, `cache_write_vnd_decimal`, `price_per_request_vnd_decimal`, `min_charge_per_request_vnd_decimal`, `price_per_second_vnd_decimal`, `effective_blended_vnd_decimal`). Bằng chứng toán học Decimal phân tích `rouyea98` (`572.0 đ`) vs `sypham98` (`508.95 đ`) vs `anhyeu00m` (`411.64448 đ`).
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\03_BACKEND_ENGINES_AND_API_CONTRACTS.md]` — Cập nhật backend engines vận hành 100% trên `rust_decimal::Decimal`, so sánh xếp hạng và kiểm tra Price Circuit Breaker chính xác tuyệt đối không dùng số nguyên.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\04_FRONTEND_LEPTOS_WASM_AND_6_SCREENS.md]` — Khóa cứng `ExactPriceFormatter` cho Leptos WASM, giữ nguyên từng chữ số thập phân có nghĩa trên cả 6 màn hình (`474.5 đ`, `88.37 đ`, `323.4816 đ`, `15.6 đ`), cấm tiệt `round(0)`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\tdd\05_TESTING_DEVOPS_SECURITY_AND_LIFECYCLE.md]` — Bổ sung các ca kiểm thử assert exact Decimal string match dựa trên số liệu thực tế của `anhyeu00m` (4 chữ số lẻ), `chuitv1907`, `rouyea98`, `sypham98`.
- `[D:\Data\Source_Code\Automatic_APIToken_PriceHunter\docs\ROADMAP.md]` — Điều chỉnh Acceptance Gates cho cả 7 Phase bắt buộc kiểm tra tính toàn vẹn của các chữ số thập phân.

## Các Hạng Mục Đã Chốt Hạ Tuyệt Đối
1. **Zero Rounding:** Tuyệt đối không làm tròn ở bất kỳ khâu nào. Lưu `TEXT` Decimal trong SQLite, xử lý bằng `rust_decimal::Decimal` (28 chữ số chính xác) trong Rust, hiển thị đúng từng con số lẻ trên UI.
2. **Đơn vị Ckey:** VNĐ / 1M tokens (cho input, output, cache read, cache write), VNĐ / request (cho `price_per_request` và `min_charge`), VNĐ / giây (cho video).
3. **Hiển thị USD:** Chia `fx_rate` bằng Decimal và giữ tối thiểu 6 chữ số thập phân (không làm tròn số cụt).

## Lệnh Bàn Giao & Bước Tiếp Theo (Resume Action)
- **Bước tiếp theo:** Bắt đầu **Phase 1** theo `docs/ROADMAP.md` (khởi tạo `rust-toolchain.toml`, `Cargo.toml` workspace, `crates/shared_contract` với `currency_math.rs` dùng `rust_decimal::Decimal`, `backend/src/Shared/`, `migrations/0001_init.sql` cột `TEXT` Decimal, `appsettings.example.json`, và Unit Tests kiểm tra số lẻ Ckey).
- **Lệnh kiểm tra môi trường trước khi code Phase 1:**
  ```powershell
  rustc --version; cargo --version; rustup target list --installed
  ```
