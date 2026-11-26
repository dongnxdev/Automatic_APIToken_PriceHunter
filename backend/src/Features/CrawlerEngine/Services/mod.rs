//! CrawlerEngine Business Services (Parser & Crawler & Scheduler)

use crate::app_state::{AppState, PriceSnapshotIndex};
use crate::Features::CrawlerEngine::Common::{
    extract_seller_namespace, is_media_video_model, parse_decimal_value,
};
use crate::Features::CrawlerEngine::Model::CkeyApiResponseRaw;
use crate::Shared::Common::decrypt_aes_gcm;
use crate::Shared::Utils::now_utc_ms;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use shared_contract::{
    ExactPrice, NormalizedCkeyModel, SupportedPath, VideoContract, WorkloadProfile,
};
use sqlx::Row;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct CkeyParser;

impl CkeyParser {
    /// Parse toàn bộ JSON payload của Ckey (như docs/api_test.json) sang danh sách NormalizedCkeyModel
    pub fn parse_models(
        json_val: &serde_json::Value,
    ) -> Result<(Vec<NormalizedCkeyModel>, Vec<String>), String> {
        let raw_resp: CkeyApiResponseRaw = serde_json::from_value(json_val.clone())
            .map_err(|e| format!("Lỗi cấu trúc phản hồi Ckey API: {e}"))?;

        if !raw_resp.success || raw_resp.status != 200 {
            return Err(format!(
                "Phản hồi từ Ckey không thành công: status={}, message={}",
                raw_resp.status, raw_resp.message
            ));
        }

        let mut warnings = Vec::new();
        let mut normalized_list = Vec::with_capacity(raw_resp.data.models.len());

        for m in raw_resp.data.models {
            let in_vnd = parse_decimal_value(&m.input_price_per_million_vnd);
            let out_vnd = parse_decimal_value(&m.output_price_per_million_vnd);
            let read_vnd = parse_decimal_value(&m.cache_read_price_per_million_vnd);
            let write_vnd = parse_decimal_value(&m.cache_write_price_per_million_vnd);
            let price_req = parse_decimal_value(&m.price_per_request_vnd);
            let min_charge = parse_decimal_value(&m.min_charge_per_request_vnd);
            let price_sec = parse_decimal_value(&m.price_per_second_vnd);

            let seller = extract_seller_namespace(&m.public_name, m.is_provider_model);

            // Chuyển đổi supported_paths
            let mut paths = Vec::new();
            for p in &m.supported_paths {
                if let Ok(sp) = serde_json::from_value::<SupportedPath>(serde_json::Value::String(p.clone())) {
                    paths.push(sp);
                } else {
                    paths.push(SupportedPath::Other(p.clone()));
                }
            }

            // Parse video contract
            let video_contract = m.video_contract.and_then(|vc| {
                serde_json::from_value::<VideoContract>(vc).ok()
            });

            let durations = m.video_durations_allowed.unwrap_or_default();
            let max_sec = m.video_max_seconds.unwrap_or(0);
            let is_video = is_media_video_model(&paths, price_sec, max_sec);

            // Kiểm tra cảnh báo bất thường
            if in_vnd.is_zero() && out_vnd.is_zero() && price_req.is_zero() && price_sec.is_zero() {
                warnings.push(format!("Model {} có toàn bộ đơn vị giá = 0 đ", m.public_name));
            }

            normalized_list.push(NormalizedCkeyModel {
                public_name: m.public_name,
                display_name: m.display_name,
                seller_namespace: seller,
                is_provider_model: m.is_provider_model,
                input_price_per_million_vnd: in_vnd,
                output_price_per_million_vnd: out_vnd,
                cache_enabled: m.cache_enabled,
                cache_read_price_per_million_vnd: read_vnd,
                cache_write_price_per_million_vnd: write_vnd,
                price_per_request_vnd: price_req,
                min_charge_per_request_vnd: min_charge,
                price_per_second_vnd: price_sec,
                request_rate_limit_per_minute: m.request_rate_limit_per_minute.unwrap_or(0),
                max_output_tokens_limit: m.max_output_tokens_limit.unwrap_or(0),
                context_tokens_limit: m.context_tokens_limit.unwrap_or(0),
                supported_paths: paths,
                video_durations_allowed: durations,
                video_max_seconds: max_sec,
                video_rate_cards: None,
                video_contract,
                is_media_video_capable: is_video,
            });
        }

        Ok((normalized_list, warnings))
    }
}

pub struct CkeyCrawlerService;

impl CkeyCrawlerService {
    /// Quét và đồng bộ toàn bộ model từ một Provider Ckey
    pub async fn crawl_and_sync_provider(
        state: &AppState,
        provider_id: &str,
    ) -> Result<usize, String> {
        let row = sqlx::query(
            r#"
            SELECT name, models_endpoint_url, api_key_ciphertext, proxy_url_ciphertext,
                   auth_mode, auth_param_name, status, consecutive_failures
            FROM provider_sources
            WHERE id = ? AND is_enabled = 1
            "#,
        )
        .bind(provider_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Provider không tồn tại hoặc đã bị tắt".to_string())?;

        let name: String = row.try_get("name").map_err(|e| e.to_string())?;
        let endpoint_url: String = row.try_get("models_endpoint_url").map_err(|e| e.to_string())?;
        let key_cipher: String = row.try_get("api_key_ciphertext").map_err(|e| e.to_string())?;
        let auth_param: String = row.try_get("auth_param_name").map_err(|e| e.to_string())?;
        let auth_mode: String = row.try_get("auth_mode").map_err(|e| e.to_string())?;
        let consec_fails: i64 = row.try_get("consecutive_failures").map_err(|e| e.to_string())?;

        let api_key = decrypt_aes_gcm(&key_cipher, &state.config.master_key)
            .map_err(|e| format!("Giải mã API Key thất bại: {e}"))?;

        let start_time = std::time::Instant::now();
        let mut req_builder = state.http_client.get(&endpoint_url);

        if auth_mode == "QUERY_KEY" {
            req_builder = req_builder.query(&[(&auth_param, &api_key)]);
        } else {
            req_builder = req_builder.bearer_auth(&api_key);
        }

        let resp_res = req_builder.send().await;
        let ping_ms = start_time.elapsed().as_millis() as i64;
        let now = now_utc_ms();

        let resp = match resp_res {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!("Lỗi mạng khi kết nối {name}: {e}");
                warn!(error = %err_msg, "Crawl thất bại");
                Self::handle_crawl_failure(state, provider_id, consec_fails + 1, &err_msg, ping_ms).await;
                return Err(err_msg);
            }
        };

        let status_code = resp.status().as_u16();
        if status_code == 401 {
            let err_msg = format!("Lỗi xác thực 401 Unauthorized từ {name}. Cần kiểm tra lại API Key.");
            error!(error = %err_msg, "Provider 401");
            Self::handle_401_failure(state, provider_id, &err_msg, ping_ms).await;
            return Err(err_msg);
        }

        if !resp.status().is_success() {
            let err_msg = format!("HTTP lỗi {} từ {}", status_code, name);
            warn!(error = %err_msg, "Crawl không thành công");
            Self::handle_crawl_failure(state, provider_id, consec_fails + 1, &err_msg, ping_ms).await;
            return Err(err_msg);
        }

        let json_val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Lỗi parse JSON từ {name}: {e}"))?;

        let (models, warnings) = CkeyParser::parse_models(&json_val)?;
        let count = models.len();

        info!(
            provider = name,
            total_models = count,
            ping_ms = ping_ms,
            warnings_count = warnings.len(),
            "Parse thành công dữ liệu Ckey"
        );

        // Delta-Write SQLite
        Self::delta_write_quotes(state, provider_id, &models).await?;

        // Cập nhật trạng thái provider thành ONLINE
        let _ = sqlx::query(
            r#"
            UPDATE provider_sources
            SET status = 'ONLINE',
                consecutive_failures = 0,
                last_ping_ms = ?,
                last_model_count = ?,
                last_error_message = NULL,
                last_scanned_at_ms = ?,
                updated_at_ms = ?
            WHERE id = ?
            "#,
        )
        .bind(ping_ms)
        .bind(count as i64)
        .bind(now)
        .bind(now)
        .bind(provider_id)
        .execute(&state.db)
        .await;

        // Rebuild in-memory snapshot
        Self::rebuild_in_memory_snapshot(state).await?;

        Ok(count)
    }

    /// Xử lý khi lỗi mạng/timeout/5xx (tính toán Circuit Breaker)
    async fn handle_crawl_failure(
        state: &AppState,
        provider_id: &str,
        new_fails: i64,
        err_msg: &str,
        ping_ms: i64,
    ) {
        let now = now_utc_ms();
        let new_status = if new_fails >= 3 {
            warn!(fails = new_fails, "Kích hoạt Circuit Breaker: Tạm ngắt Provider");
            "CIRCUIT_OPEN"
        } else {
            "TIMEOUT"
        };

        let _ = sqlx::query(
            r#"
            UPDATE provider_sources
            SET status = ?,
                consecutive_failures = ?,
                last_ping_ms = ?,
                last_error_message = ?,
                last_scanned_at_ms = ?,
                circuit_opened_at_ms = CASE WHEN ? = 'CIRCUIT_OPEN' THEN ? ELSE circuit_opened_at_ms END,
                updated_at_ms = ?
            WHERE id = ?
            "#,
        )
        .bind(new_status)
        .bind(new_fails)
        .bind(ping_ms)
        .bind(err_msg)
        .bind(now)
        .bind(new_status)
        .bind(now)
        .bind(now)
        .bind(provider_id)
        .execute(&state.db)
        .await;

        // Ghi vào Dead-Letter Queue
        let dlq_id = Uuid::now_v7().to_string();
        let _ = sqlx::query(
            r#"
            INSERT INTO dead_letter_events (
                id, event_type, correlation_id, payload_json, error_reason,
                retry_count, max_retries, next_retry_at_ms, status, created_at_ms, updated_at_ms
            ) VALUES (?, 'CRAWL_PROVIDER', ?, '{}', ?, ?, 5, ?, 'PENDING', ?, ?)
            "#,
        )
        .bind(&dlq_id)
        .bind(provider_id)
        .bind(err_msg)
        .bind(new_fails)
        .bind(now + 60_000)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await;
    }

    /// Xử lý lỗi 401 Unauthorized (trip ngay Circuit Breaker)
    async fn handle_401_failure(
        state: &AppState,
        provider_id: &str,
        err_msg: &str,
        ping_ms: i64,
    ) {
        let now = now_utc_ms();
        let _ = sqlx::query(
            r#"
            UPDATE provider_sources
            SET status = 'UNAUTHORIZED_401',
                consecutive_failures = consecutive_failures + 1,
                last_ping_ms = ?,
                last_error_message = ?,
                last_scanned_at_ms = ?,
                circuit_opened_at_ms = ?,
                updated_at_ms = ?
            WHERE id = ?
            "#,
        )
        .bind(ping_ms)
        .bind(err_msg)
        .bind(now)
        .bind(now)
        .bind(now)
        .bind(provider_id)
        .execute(&state.db)
        .await;
    }

    /// Delta-Write bảng quotes vào SQLite (chỉ ghi khi hash thay đổi)
    async fn delta_write_quotes(
        state: &AppState,
        provider_id: &str,
        models: &[NormalizedCkeyModel],
    ) -> Result<(), String> {
        let now = now_utc_ms();

        for m in models {
            // Tính hash payload đại diện để so sánh delta
            let hash_input = format!(
                "{}:{}:{}:{}:{}:{}:{}",
                m.public_name,
                m.input_price_per_million_vnd,
                m.output_price_per_million_vnd,
                m.cache_read_price_per_million_vnd,
                m.price_per_request_vnd,
                m.min_charge_per_request_vnd,
                m.cache_enabled
            );
            let mut hasher = Sha256::new();
            hasher.update(hash_input.as_bytes());
            let payload_hash = format!("{:x}", hasher.finalize());

            // Tính Effective Blended cơ bản theo profile Coding/Agent 50K tokens
            let exact_price = ExactPrice {
                input_vnd: m.input_price_per_million_vnd,
                output_vnd: m.output_price_per_million_vnd,
                cache_enabled: m.cache_enabled,
                cache_read_vnd: m.cache_read_price_per_million_vnd,
                cache_write_vnd: m.cache_write_price_per_million_vnd,
                price_per_request_vnd: m.price_per_request_vnd,
                min_charge_per_request_vnd: m.min_charge_per_request_vnd,
                price_per_second_vnd: m.price_per_second_vnd,
            };
            let blended = exact_price.calculate_effective_blended(WorkloadProfile::CodingAgent, 50_000);

            let kind_str = if m.is_media_video_capable {
                "PER_SECOND_VIDEO"
            } else if m.input_price_per_million_vnd.is_zero() && m.output_price_per_million_vnd.is_zero() && !m.price_per_request_vnd.is_zero() {
                "FLAT_PER_REQUEST"
            } else {
                "PER_TOKEN"
            };

            let paths_json = serde_json::to_string(&m.supported_paths).unwrap_or_else(|_| "[]".into());
            let durations_json = serde_json::to_string(&m.video_durations_allowed).unwrap_or_else(|_| "[]".into());
            let contract_json = m.video_contract.as_ref().and_then(|c| serde_json::to_string(c).ok());

            let quote_id = Uuid::now_v7().to_string();

            // Upsert vào provider_model_quotes
            let _ = sqlx::query(
                r#"
                INSERT INTO provider_model_quotes (
                    id, provider_source_id, canonical_model_id, seller_namespace, public_name,
                    display_name, is_provider_model, pricing_model_kind, input_vnd_decimal, output_vnd_decimal,
                    cache_enabled, cache_read_vnd_decimal, cache_write_vnd_decimal, price_per_request_vnd_decimal,
                    min_charge_per_request_vnd_decimal, price_per_second_vnd_decimal, effective_blended_vnd_decimal,
                    request_rate_limit_per_minute, max_output_tokens_limit, context_tokens_limit,
                    supported_paths_json, video_durations_allowed_json, video_max_seconds, video_rate_cards_json,
                    video_contract_json, is_media_video_capable, seller_circuit_status, seller_consecutive_failures,
                    raw_payload_hash, updated_at_ms
                ) VALUES (
                    ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, 'CLOSED', 0, ?, ?
                )
                ON CONFLICT(provider_source_id, public_name) DO UPDATE SET
                    display_name = excluded.display_name,
                    pricing_model_kind = excluded.pricing_model_kind,
                    input_vnd_decimal = excluded.input_vnd_decimal,
                    output_vnd_decimal = excluded.output_vnd_decimal,
                    cache_enabled = excluded.cache_enabled,
                    cache_read_vnd_decimal = excluded.cache_read_vnd_decimal,
                    cache_write_vnd_decimal = excluded.cache_write_vnd_decimal,
                    price_per_request_vnd_decimal = excluded.price_per_request_vnd_decimal,
                    min_charge_per_request_vnd_decimal = excluded.min_charge_per_request_vnd_decimal,
                    price_per_second_vnd_decimal = excluded.price_per_second_vnd_decimal,
                    effective_blended_vnd_decimal = excluded.effective_blended_vnd_decimal,
                    request_rate_limit_per_minute = excluded.request_rate_limit_per_minute,
                    max_output_tokens_limit = excluded.max_output_tokens_limit,
                    context_tokens_limit = excluded.context_tokens_limit,
                    supported_paths_json = excluded.supported_paths_json,
                    video_durations_allowed_json = excluded.video_durations_allowed_json,
                    video_max_seconds = excluded.video_max_seconds,
                    video_contract_json = excluded.video_contract_json,
                    is_media_video_capable = excluded.is_media_video_capable,
                    raw_payload_hash = excluded.raw_payload_hash,
                    updated_at_ms = excluded.updated_at_ms
                WHERE provider_model_quotes.raw_payload_hash != excluded.raw_payload_hash
                "#,
            )
            .bind(&quote_id)
            .bind(provider_id)
            .bind(&m.seller_namespace)
            .bind(&m.public_name)
            .bind(&m.display_name)
            .bind(if m.is_provider_model { 1 } else { 0 })
            .bind(kind_str)
            .bind(m.input_price_per_million_vnd.to_string())
            .bind(m.output_price_per_million_vnd.to_string())
            .bind(if m.cache_enabled { 1 } else { 0 })
            .bind(m.cache_read_price_per_million_vnd.to_string())
            .bind(m.cache_write_price_per_million_vnd.to_string())
            .bind(m.price_per_request_vnd.to_string())
            .bind(m.min_charge_per_request_vnd.to_string())
            .bind(m.price_per_second_vnd.to_string())
            .bind(blended.to_string())
            .bind(m.request_rate_limit_per_minute)
            .bind(m.max_output_tokens_limit)
            .bind(m.context_tokens_limit)
            .bind(&paths_json)
            .bind(&durations_json)
            .bind(m.video_max_seconds)
            .bind(&contract_json)
            .bind(if m.is_media_video_capable { 1 } else { 0 })
            .bind(&payload_hash)
            .bind(now)
            .execute(&state.db)
            .await
            .map_err(|e| format!("Delta-write quote {} thất bại: {e}", m.public_name))?;
        }

        Ok(())
    }

    /// Rebuild snapshot trên RAM sau khi crawl xong
    pub async fn rebuild_in_memory_snapshot(state: &AppState) -> Result<(), String> {
        let rows = sqlx::query(
            r#"
            SELECT public_name, display_name, seller_namespace, is_provider_model,
                   input_vnd_decimal, output_vnd_decimal, cache_enabled,
                   cache_read_vnd_decimal, cache_write_vnd_decimal, price_per_request_vnd_decimal,
                   min_charge_per_request_vnd_decimal, price_per_second_vnd_decimal,
                   request_rate_limit_per_minute, max_output_tokens_limit, context_tokens_limit,
                   supported_paths_json, video_durations_allowed_json, video_max_seconds,
                   video_contract_json, is_media_video_capable
            FROM provider_model_quotes
            WHERE seller_circuit_status != 'OPEN'
            "#,
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        let mut models_map: HashMap<String, Vec<NormalizedCkeyModel>> = HashMap::new();
        let total = rows.len();

        for r in rows {
            let pub_name: String = r.try_get("public_name").map_err(|e| e.to_string())?;
            let disp_name: String = r.try_get("display_name").map_err(|e| e.to_string())?;
            let seller: String = r.try_get("seller_namespace").map_err(|e| e.to_string())?;
            let is_prov: i64 = r.try_get("is_provider_model").map_err(|e| e.to_string())?;
            let in_str: String = r.try_get("input_vnd_decimal").map_err(|e| e.to_string())?;
            let out_str: String = r.try_get("output_vnd_decimal").map_err(|e| e.to_string())?;
            let cache_en: i64 = r.try_get("cache_enabled").map_err(|e| e.to_string())?;
            let read_str: String = r.try_get("cache_read_vnd_decimal").map_err(|e| e.to_string())?;
            let write_str: String = r.try_get("cache_write_vnd_decimal").map_err(|e| e.to_string())?;
            let req_str: String = r.try_get("price_per_request_vnd_decimal").map_err(|e| e.to_string())?;
            let min_str: String = r.try_get("min_charge_per_request_vnd_decimal").map_err(|e| e.to_string())?;
            let sec_str: String = r.try_get("price_per_second_vnd_decimal").map_err(|e| e.to_string())?;
            let rpm: i64 = r.try_get("request_rate_limit_per_minute").map_err(|e| e.to_string())?;
            let max_out: i64 = r.try_get("max_output_tokens_limit").map_err(|e| e.to_string())?;
            let ctx: i64 = r.try_get("context_tokens_limit").map_err(|e| e.to_string())?;
            let paths_json: String = r.try_get("supported_paths_json").map_err(|e| e.to_string())?;
            let durs_json: String = r.try_get("video_durations_allowed_json").map_err(|e| e.to_string())?;
            let max_sec: i64 = r.try_get("video_max_seconds").map_err(|e| e.to_string())?;
            let contract_json: Option<String> = r.try_get("video_contract_json").map_err(|e| e.to_string())?;
            let is_video: i64 = r.try_get("is_media_video_capable").map_err(|e| e.to_string())?;

            let paths = serde_json::from_str(&paths_json).unwrap_or_default();
            let durs = serde_json::from_str(&durs_json).unwrap_or_default();
            let contract = contract_json.and_then(|c| serde_json::from_str(&c).ok());

            let model_item = NormalizedCkeyModel {
                public_name: pub_name,
                display_name: disp_name.clone(),
                seller_namespace: seller,
                is_provider_model: is_prov == 1,
                input_price_per_million_vnd: in_str.parse().unwrap_or(Decimal::ZERO),
                output_price_per_million_vnd: out_str.parse().unwrap_or(Decimal::ZERO),
                cache_enabled: cache_en == 1,
                cache_read_price_per_million_vnd: read_str.parse().unwrap_or(Decimal::ZERO),
                cache_write_price_per_million_vnd: write_str.parse().unwrap_or(Decimal::ZERO),
                price_per_request_vnd: req_str.parse().unwrap_or(Decimal::ZERO),
                min_charge_per_request_vnd: min_str.parse().unwrap_or(Decimal::ZERO),
                price_per_second_vnd: sec_str.parse().unwrap_or(Decimal::ZERO),
                request_rate_limit_per_minute: rpm,
                max_output_tokens_limit: max_out,
                context_tokens_limit: ctx,
                supported_paths: paths,
                video_durations_allowed: durs,
                video_max_seconds: max_sec as i32,
                video_rate_cards: None,
                video_contract: contract,
                is_media_video_capable: is_video == 1,
            };

            models_map.entry(disp_name).or_default().push(model_item);
        }

        let new_index = PriceSnapshotIndex {
            models_map,
            total_active_quotes: total,
            last_compiled_at_ms: now_utc_ms(),
        };

        state.price_index.store(std::sync::Arc::new(new_index));
        info!(total_quotes = total, "Biên dịch Snapshot Index trên RAM hoàn tất");
        Ok(())
    }

    /// Kích hoạt quét cưỡng bức toàn bộ Provider (FORCE SCAN ALL)
    pub async fn force_scan_all(state: &AppState) -> Result<usize, String> {
        let _guard = state
            .active_crawler_lock
            .try_lock()
            .map_err(|_| "Một chu kỳ quét cưỡng bức đang được thực thi. Vui lòng chờ!".to_string())?;

        let rows = sqlx::query(
            "SELECT id FROM provider_sources WHERE is_enabled = 1 AND status != 'DISABLED'",
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        let mut total_scanned = 0;
        for r in rows {
            let pid: String = r.try_get("id").map_err(|e| e.to_string())?;
            match Self::crawl_and_sync_provider(state, &pid).await {
                Ok(c) => total_scanned += c,
                Err(e) => warn!(provider_id = pid, error = %e, "Lỗi quét provider đơn lẻ"),
            }
        }

        Ok(total_scanned)
    }
}
