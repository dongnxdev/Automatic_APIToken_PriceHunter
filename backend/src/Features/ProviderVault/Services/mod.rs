//! ProviderVault Business Services

use crate::app_state::AppState;
use crate::Features::ProviderVault::Common::{clean_endpoint_and_extract_key, slugify};
use crate::Features::ProviderVault::Model::{
    CreateProviderRequest, ProviderDto, RevealSecretResponse, SandboxParseRequest,
    SandboxParseResponse,
};
use crate::Shared::Common::{decrypt_aes_gcm, encrypt_aes_gcm};
use crate::Shared::Utils::{mask_secret, now_utc_ms};
use sqlx::Row;
use uuid::Uuid;

pub struct ProviderVaultService;

impl ProviderVaultService {
    /// Lấy danh sách toàn bộ Provider
    pub async fn list_providers(state: &AppState) -> Result<Vec<ProviderDto>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, slug, adapter_type, auth_mode, auth_param_name,
                   models_endpoint_url, inference_base_url, api_key_masked,
                   proxy_url_ciphertext, fx_currency, fx_rate_to_vnd_decimal,
                   price_multiplier_decimal, scan_interval_seconds, is_enabled,
                   status, last_ping_ms, last_model_count, last_error_message, last_scanned_at_ms
            FROM provider_sources
            ORDER BY created_at_ms ASC
            "#,
        )
        .fetch_all(&state.db)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let proxy_cipher: Option<String> = r.try_get("proxy_url_ciphertext")?;
            list.push(ProviderDto {
                id: r.try_get("id")?,
                name: r.try_get("name")?,
                slug: r.try_get("slug")?,
                adapter_type: r.try_get("adapter_type")?,
                auth_mode: r.try_get("auth_mode")?,
                auth_param_name: r.try_get("auth_param_name")?,
                models_endpoint_url: r.try_get("models_endpoint_url")?,
                inference_base_url: r.try_get("inference_base_url")?,
                api_key_masked: r.try_get("api_key_masked")?,
                has_proxy: proxy_cipher.is_some(),
                fx_currency: r.try_get("fx_currency")?,
                fx_rate_to_vnd_decimal: r.try_get("fx_rate_to_vnd_decimal")?,
                price_multiplier_decimal: r.try_get("price_multiplier_decimal")?,
                scan_interval_seconds: r.try_get("scan_interval_seconds")?,
                is_enabled: r.try_get::<i64, _>("is_enabled")? == 1,
                status: r.try_get("status")?,
                last_ping_ms: r.try_get("last_ping_ms")?,
                last_model_count: r.try_get("last_model_count")?,
                last_error_message: r.try_get("last_error_message")?,
                last_scanned_at_ms: r.try_get("last_scanned_at_ms")?,
            });
        }

        Ok(list)
    }

    /// Tạo mới một Provider (mã hóa API Key và Proxy URL bằng AES-256-GCM)
    pub async fn create_provider(
        state: &AppState,
        req: CreateProviderRequest,
    ) -> Result<ProviderDto, String> {
        let id = Uuid::now_v7().to_string();
        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err("Tên Provider không được để trống".into());
        }
        let slug = slugify(&name);

        let param_name = req.auth_param_name.unwrap_or_else(|| "key".into());
        let (clean_url, extracted_key) =
            clean_endpoint_and_extract_key(&req.models_endpoint_url, &param_name);

        let final_key = if !req.api_key.trim().is_empty() {
            req.api_key.trim().to_string()
        } else if let Some(k) = extracted_key {
            k
        } else {
            return Err("API Key bắt buộc phải có".into());
        };

        let key_cipher = encrypt_aes_gcm(&final_key, &state.config.master_key)
            .map_err(|e| format!("Mã hóa API Key thất bại: {e}"))?;
        let key_masked = mask_secret(&final_key);

        let proxy_cipher = if let Some(p) = req.proxy_url.filter(|s| !s.trim().is_empty()) {
            Some(
                encrypt_aes_gcm(p.trim(), &state.config.master_key)
                    .map_err(|e| format!("Mã hóa Proxy URL thất bại: {e}"))?,
            )
        } else {
            None
        };

        let adapter_type = req.adapter_type.unwrap_or_else(|| "CKEY_LLM".into());
        let auth_mode = req.auth_mode.unwrap_or_else(|| "QUERY_KEY".into());
        let inference_base_url = req
            .inference_base_url
            .unwrap_or_else(|| "https://ckey.vn/v1".into());
        let fx_currency = req.fx_currency.unwrap_or_else(|| "VND".into());
        let fx_rate = req.fx_rate_to_vnd_decimal.unwrap_or_else(|| "1.0".into());
        let multiplier = req.price_multiplier_decimal.unwrap_or_else(|| "1.0".into());
        let interval = req.scan_interval_seconds.unwrap_or(180).max(15);
        let now = now_utc_ms();

        sqlx::query(
            r#"
            INSERT INTO provider_sources (
                id, name, slug, adapter_type, auth_mode, auth_param_name,
                models_endpoint_url, inference_base_url, api_key_ciphertext, api_key_masked,
                proxy_url_ciphertext, fx_currency, fx_rate_to_vnd_decimal, price_multiplier_decimal,
                scan_interval_seconds, is_enabled, status, created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, 'ONLINE', ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&name)
        .bind(&slug)
        .bind(&adapter_type)
        .bind(&auth_mode)
        .bind(&param_name)
        .bind(&clean_url)
        .bind(&inference_base_url)
        .bind(&key_cipher)
        .bind(&key_masked)
        .bind(&proxy_cipher)
        .bind(&fx_currency)
        .bind(&fx_rate)
        .bind(&multiplier)
        .bind(interval)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Lưu Provider vào Database thất bại: {e}"))?;

        Ok(ProviderDto {
            id,
            name,
            slug,
            adapter_type,
            auth_mode,
            auth_param_name: param_name,
            models_endpoint_url: clean_url,
            inference_base_url,
            api_key_masked: key_masked,
            has_proxy: proxy_cipher.is_some(),
            fx_currency,
            fx_rate_to_vnd_decimal: fx_rate,
            price_multiplier_decimal: multiplier,
            scan_interval_seconds: interval,
            is_enabled: true,
            status: "ONLINE".into(),
            last_ping_ms: None,
            last_model_count: 0,
            last_error_message: None,
            last_scanned_at_ms: None,
        })
    }

    /// Mở khóa xem API Key gốc (Bắt buộc ghi Audit Log)
    pub async fn reveal_secret(
        state: &AppState,
        provider_id: &str,
        actor: &str,
    ) -> Result<RevealSecretResponse, String> {
        let row = sqlx::query(
            r#"
            SELECT api_key_ciphertext, proxy_url_ciphertext
            FROM provider_sources
            WHERE id = ?
            "#,
        )
        .bind(provider_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Không tìm thấy Provider".to_string())?;

        let key_cipher: String = row.try_get("api_key_ciphertext").map_err(|e| e.to_string())?;
        let proxy_cipher: Option<String> = row.try_get("proxy_url_ciphertext").map_err(|e| e.to_string())?;

        let key_plain = decrypt_aes_gcm(&key_cipher, &state.config.master_key)
            .map_err(|e| format!("Giải mã API Key thất bại: {e}"))?;

        let proxy_plain = if let Some(p) = proxy_cipher {
            Some(
                decrypt_aes_gcm(&p, &state.config.master_key)
                    .map_err(|e| format!("Giải mã Proxy URL thất bại: {e}"))?,
            )
        } else {
            None
        };

        // Ghi Audit Log bắt buộc
        let audit_id = Uuid::now_v7().to_string();
        let audit_details = serde_json::json!({
            "action": "REVEAL_SECRET",
            "provider_id": provider_id,
            "timestamp_ms": now_utc_ms(),
        });

        let _ = sqlx::query(
            r#"
            INSERT INTO system_audit_logs (id, action, actor, details_json, created_at_ms)
            VALUES (?, 'REVEAL_SECRET', ?, ?, ?)
            "#,
        )
        .bind(&audit_id)
        .bind(actor)
        .bind(audit_details.to_string())
        .bind(now_utc_ms())
        .execute(&state.db)
        .await;

        Ok(RevealSecretResponse {
            provider_id: provider_id.to_string(),
            api_key_plaintext: key_plain,
            proxy_url_plaintext: proxy_plain,
        })
    }

    /// JSON Parser Sandbox: Fetch live và trả về mẫu raw + normalized (dry-run, không lưu DB)
    pub async fn sandbox_parse(
        state: &AppState,
        req: SandboxParseRequest,
    ) -> Result<SandboxParseResponse, String> {
        let start = std::time::Instant::now();
        let param_name = req.auth_param_name.unwrap_or_else(|| "key".into());
        let (clean_url, _) = clean_endpoint_and_extract_key(&req.endpoint_url, &param_name);

        let mut req_builder = state.http_client.get(&clean_url);
        if req.auth_mode == "QUERY_KEY" {
            req_builder = req_builder.query(&[(&param_name, &req.api_key)]);
        } else if req.auth_mode == "X_API_KEY_HEADER" {
            req_builder = req_builder.header("x-api-key", &req.api_key);
        } else {
            req_builder = req_builder.bearer_auth(&req.api_key);
        }

        let resp = req_builder
            .send()
            .await
            .map_err(|e| format!("Lỗi kết nối tới endpoint: {e}"))?;

        let status = resp.status().as_u16();
        let ping_ms = start.elapsed().as_millis() as i64;

        let json_val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Lỗi parse JSON phản hồi: {e}"))?;

        // Áp dụng Ckey Parser
        let (normalized, warnings) = crate::Features::CrawlerEngine::Services::CkeyParser::parse_models(&json_val)?;

        // Lấy 10 phần tử đầu tiên làm sample preview
        let raw_sample = if let Some(arr) = json_val
            .get("data")
            .and_then(|d| d.get("models"))
            .and_then(|m| m.as_array())
        {
            serde_json::Value::Array(arr.iter().take(10).cloned().collect())
        } else {
            json_val.clone()
        };

        let total = normalized.len();
        let preview_subset = normalized.into_iter().take(10).collect();

        Ok(SandboxParseResponse {
            http_status: status,
            ping_ms,
            total_models_found: total,
            raw_sample_json: raw_sample,
            normalized_preview: preview_subset,
            parser_warnings: warnings,
        })
    }
}
