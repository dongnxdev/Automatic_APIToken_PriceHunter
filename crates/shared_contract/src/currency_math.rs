//! Module Toán Học Tiền Tệ Chính Xác Tuyệt Đối (Exact Decimal Math - Zero Rounding)
//!
//! Khóa cứng nguyên tắc: Không làm tròn số nguyên ở mọi tầng tính toán và lưu trữ.
//! Đơn vị chuẩn đối với Ckey.vn là VNĐ (VNĐ / 1M tokens, VNĐ / request, VNĐ / giây).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Cấu hình kịch bản sử dụng (Workload Profile)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkloadProfile {
    /// Coding / AI Agent: 60% Input, 30% Cache Read, 10% Output, mặc định 50.000 tokens/request
    CodingAgent,
    /// Chat thông thường: 75% Input, 25% Output, mặc định 8.000 tokens/request
    ChatNormal,
    /// Dịch thuật ngữ cảnh: 50% Input, 50% Output, mặc định 12.000 tokens/request
    Translation,
    /// Video / Media đa phương tiện
    VideoMedia,
}

impl WorkloadProfile {
    /// Lấy kích thước request giả định mặc định (số tokens)
    #[inline]
    pub fn default_assumed_tokens(&self) -> u32 {
        match self {
            Self::CodingAgent => 50_000,
            Self::ChatNormal => 8_000,
            Self::Translation => 12_000,
            Self::VideoMedia => 50_000,
        }
    }

    /// Trọng số (w_in, w_cache, w_out)
    #[inline]
    pub fn weights(&self) -> (Decimal, Decimal, Decimal) {
        match self {
            Self::CodingAgent => (
                Decimal::new(60, 2), // 0.60
                Decimal::new(30, 2), // 0.30
                Decimal::new(10, 2), // 0.10
            ),
            Self::ChatNormal => (
                Decimal::new(75, 2), // 0.75
                Decimal::ZERO,        // 0.00
                Decimal::new(25, 2), // 0.25
            ),
            Self::Translation => (
                Decimal::new(50, 2), // 0.50
                Decimal::ZERO,        // 0.00
                Decimal::new(50, 2), // 0.50
            ),
            Self::VideoMedia => (
                Decimal::new(50, 2),
                Decimal::ZERO,
                Decimal::new(50, 2),
            ),
        }
    }
}

/// Struct đại diện cho bộ giá chính xác của một quote/model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactPrice {
    /// Giá Input / 1M tokens (VNĐ)
    pub input_vnd: Decimal,
    /// Giá Output / 1M tokens (VNĐ)
    pub output_vnd: Decimal,
    /// Model có hỗ trợ cache không
    pub cache_enabled: bool,
    /// Giá Cache Read / 1M tokens (VNĐ)
    pub cache_read_vnd: Decimal,
    /// Giá Cache Write / 1M tokens (VNĐ)
    pub cache_write_vnd: Decimal,
    /// Phụ phí cố định cho mỗi request (VNĐ)
    pub price_per_request_vnd: Decimal,
    /// Phí sàn tối thiểu cho mỗi request (VNĐ)
    pub min_charge_per_request_vnd: Decimal,
    /// Đơn giá theo giây video (VNĐ)
    pub price_per_second_vnd: Decimal,
}

impl ExactPrice {
    /// Tính giá Effective Blended trên 1 triệu tokens chính xác tuyệt đối (Zero Rounding).
    ///
    /// * w_in * Input + w_cache * Effective_Cache + w_out * Output
    /// * Nếu kênh không có cache (cache_enabled = false), phần cache bị tính giá Input gốc.
    /// * Áp dụng đúng công thức chi phí 1 request và chặn sàn min_charge_per_request_vnd.
    pub fn calculate_effective_blended(
        &self,
        profile: WorkloadProfile,
        assumed_tokens_per_req: u32,
    ) -> Decimal {
        let (w_in, w_cache, w_out) = profile.weights();

        // 1. Xác định giá cache hiệu dụng
        let effective_cache_price = if self.cache_enabled && self.cache_read_vnd > Decimal::ZERO {
            self.cache_read_vnd
        } else {
            self.input_vnd
        };

        // 2. Giá token blended trên 1M tokens thuần túy
        let token_blended_1m =
            (self.input_vnd * w_in) + (effective_cache_price * w_cache) + (self.output_vnd * w_out);

        // Nếu là mô hình tính cố định hoàn toàn theo request (in=0, out=0, price_per_req > 0)
        let is_flat_per_req = self.input_vnd.is_zero()
            && self.output_vnd.is_zero()
            && (self.price_per_request_vnd > Decimal::ZERO
                || self.min_charge_per_request_vnd > Decimal::ZERO);

        let assumed_tokens = Decimal::from(assumed_tokens_per_req.max(1));
        let one_million = Decimal::new(1_000_000, 0);

        if is_flat_per_req {
            let cost_per_req = self
                .price_per_request_vnd
                .max(self.min_charge_per_request_vnd);
            let reqs_per_1m = one_million / assumed_tokens;
            return cost_per_req * reqs_per_1m;
        }

        // 3. Quy đổi theo request giả định để xử lý min_charge_per_request và phụ phí request
        let tokens_ratio = assumed_tokens / one_million;
        let requests_per_1m = one_million / assumed_tokens;

        let raw_token_cost_per_req = token_blended_1m * tokens_ratio;
        let total_cost_per_req = raw_token_cost_per_req + self.price_per_request_vnd;
        let final_cost_per_req = total_cost_per_req.max(self.min_charge_per_request_vnd);

        final_cost_per_req * requests_per_1m
    }
}

/// Định dạng giá VNĐ chính xác, giữ nguyên toàn bộ số thập phân có nghĩa, không làm tròn số nguyên.
///
/// Ví dụ:
/// - `508.95` -> `"508.95 đ"`
/// - `88.37` -> `"88.37 đ"`
/// - `323.4816` -> `"323.4816 đ"`
/// - `1200` -> `"1200 đ"`
pub fn format_vnd_exact(val: Decimal) -> String {
    let normalized = val.normalize();
    let s = normalized.to_string();
    format!("{s} đ")
}

/// Định dạng giá USD chính xác từ giá VNĐ thông qua tỷ giá FX.
///
/// Chia tỷ giá bằng Decimal và giữ tối thiểu 6 chữ số thập phân, cấm cắt cụt.
pub fn format_usd_exact(vnd_val: Decimal, fx_rate_vnd_per_usd: Decimal) -> String {
    if fx_rate_vnd_per_usd.is_zero() {
        return "$0.00".to_string();
    }
    let usd = vnd_val / fx_rate_vnd_per_usd;
    let s = usd.normalize().to_string();
    format!("${s}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_sypham98_claude_opus_4_6_exact() {
        // Dữ liệu thật từ api_test.json:
        // sypham98/claude-opus-4-6 | in=526.5 | out=1170 | cache=true (r=253.5, w=684.45) | req=0 | min=5.2
        let price = ExactPrice {
            input_vnd: Decimal::from_str("526.5").unwrap(),
            output_vnd: Decimal::from_str("1170.0").unwrap(),
            cache_enabled: true,
            cache_read_vnd: Decimal::from_str("253.5").unwrap(),
            cache_write_vnd: Decimal::from_str("684.45").unwrap(),
            price_per_request_vnd: Decimal::ZERO,
            min_charge_per_request_vnd: Decimal::from_str("5.2").unwrap(),
            price_per_second_vnd: Decimal::ZERO,
        };

        let blended = price.calculate_effective_blended(WorkloadProfile::CodingAgent, 50_000);
        // P_token = 526.5 * 0.6 + 253.5 * 0.3 + 1170 * 0.1 = 315.9 + 76.05 + 117 = 508.95 đ
        // req_cost = 508.95 * 0.05 = 25.4475 > 5.2 -> blended = 508.95
        assert_eq!(blended, Decimal::from_str("508.95").unwrap());
        assert_eq!(format_vnd_exact(blended), "508.95 đ");
    }

    #[test]
    fn test_rouyea98_min_charge_floor_exact() {
        // Dữ liệu thật từ api_test.json:
        // rouyea98/claude-opus-4-6 | in=474.5 | out=910 | cache=true (r=189.8, w=616.85) | req=0 | min=28.6
        let price = ExactPrice {
            input_vnd: Decimal::from_str("474.5").unwrap(),
            output_vnd: Decimal::from_str("910.0").unwrap(),
            cache_enabled: true,
            cache_read_vnd: Decimal::from_str("189.8").unwrap(),
            cache_write_vnd: Decimal::from_str("616.85").unwrap(),
            price_per_request_vnd: Decimal::ZERO,
            min_charge_per_request_vnd: Decimal::from_str("28.6").unwrap(),
            price_per_second_vnd: Decimal::ZERO,
        };

        let blended = price.calculate_effective_blended(WorkloadProfile::CodingAgent, 50_000);
        // raw_token_cost = 432.64 * 0.05 = 21.632 < 28.6 -> dính sàn min_charge 28.6
        // blended = 28.6 * 20 = 572.0 đ
        assert_eq!(blended, Decimal::from_str("572.0").unwrap());
        assert_eq!(format_vnd_exact(blended), "572 đ");
    }

    #[test]
    fn test_anhyeu00m_high_precision_cache_exact() {
        // Dữ liệu thật từ api_test.json:
        // anhyeu00m/claude-opus-5 | in=416 | out=650 | cache=true (r=323.4816, w=1779.1488) | req=0 | min=19.5
        let price = ExactPrice {
            input_vnd: Decimal::from_str("416.0").unwrap(),
            output_vnd: Decimal::from_str("650.0").unwrap(),
            cache_enabled: true,
            cache_read_vnd: Decimal::from_str("323.4816").unwrap(),
            cache_write_vnd: Decimal::from_str("1779.1488").unwrap(),
            price_per_request_vnd: Decimal::ZERO,
            min_charge_per_request_vnd: Decimal::from_str("19.5").unwrap(),
            price_per_second_vnd: Decimal::ZERO,
        };

        let (w_in, w_cache, w_out) = WorkloadProfile::CodingAgent.weights();
        let token_blended =
            price.input_vnd * w_in + price.cache_read_vnd * w_cache + price.output_vnd * w_out;
        // 416 * 0.6 = 249.6
        // 323.4816 * 0.3 = 97.04448
        // 650 * 0.1 = 65.0
        // tổng = 411.64448
        assert_eq!(token_blended, Decimal::from_str("411.64448").unwrap());
        assert_eq!(format_vnd_exact(token_blended), "411.64448 đ");
    }

    #[test]
    fn test_thanhnhan9023_flat_per_request_exact() {
        // Dữ liệu thật: thanhnhan9023/claude-opus-4-6 | in=0 | out=0 | req=65 | min=65
        let price = ExactPrice {
            input_vnd: Decimal::ZERO,
            output_vnd: Decimal::ZERO,
            cache_enabled: false,
            cache_read_vnd: Decimal::ZERO,
            cache_write_vnd: Decimal::ZERO,
            price_per_request_vnd: Decimal::from_str("65.0").unwrap(),
            min_charge_per_request_vnd: Decimal::from_str("65.0").unwrap(),
            price_per_second_vnd: Decimal::ZERO,
        };

        let blended = price.calculate_effective_blended(WorkloadProfile::CodingAgent, 50_000);
        // 20 req * 65.0 = 1300.0 đ / 1M
        assert_eq!(blended, Decimal::from_str("1300.0").unwrap());
    }

    #[test]
    fn test_tdsang1999_request_fee_exact() {
        // tdsang1999/claude-opus-4.8 | in=897 | out=1677 | req=88.37 | min=26
        let price = ExactPrice {
            input_vnd: Decimal::from_str("897.0").unwrap(),
            output_vnd: Decimal::from_str("1677.0").unwrap(),
            cache_enabled: false,
            cache_read_vnd: Decimal::ZERO,
            cache_write_vnd: Decimal::ZERO,
            price_per_request_vnd: Decimal::from_str("88.37").unwrap(),
            min_charge_per_request_vnd: Decimal::from_str("26.0").unwrap(),
            price_per_second_vnd: Decimal::ZERO,
        };

        let blended = price.calculate_effective_blended(WorkloadProfile::ChatNormal, 8_000);
        // in=897*0.75 + out=1677*0.25 = 672.75 + 419.25 = 1092.0 đ
        // tokens_ratio = 8000 / 1_000_000 = 0.008
        // raw_token_cost = 1092 * 0.008 = 8.736 đ
        // total_req = 8.736 + 88.37 = 97.106 đ > 26 -> blended = 97.106 * 125 = 12138.25 đ
        assert_eq!(blended, Decimal::from_str("12138.25").unwrap());
        assert_eq!(format_vnd_exact(blended), "12138.25 đ");
    }
}
