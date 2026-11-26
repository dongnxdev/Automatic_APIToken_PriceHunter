//! Integration Tests Cho ProviderVault & CrawlerEngine
//!
//! Phát lại fixture thực tế docs/api_test.json (650 models Ckey.vn) qua WireMock

use modelradar_server::app_state::AppState;
use modelradar_server::Features::CrawlerEngine::CkeyCrawlerService;
use modelradar_server::Features::ProviderVault::Model::{
    CreateProviderRequest, SandboxParseRequest,
};
use modelradar_server::Features::ProviderVault::ProviderVaultService;
use modelradar_server::Shared::Common::AppConfig;
use modelradar_server::Shared::Data::{init_sqlite_pool, run_migrations};
use sqlx::Row;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn find_file_in_workspace(rel_path: &str) -> std::path::PathBuf {
    if std::path::Path::new(rel_path).exists() {
        std::path::PathBuf::from(rel_path)
    } else {
        std::path::PathBuf::from("..").join(rel_path)
    }
}

async fn setup_test_app_state() -> AppState {
    let db_path = find_file_in_workspace("target").join(format!("test_modelradar_{}.db", uuid::Uuid::now_v7()));
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = init_sqlite_pool(&db_url).await.expect("Init test db pool");
    run_migrations(&pool).await.expect("Run migrations");

    let cfg_path = find_file_in_workspace("appsettings.json");
    let cfg_path_example = find_file_in_workspace("appsettings.example.json");

    let config = AppConfig::load_from_file(&cfg_path)
        .or_else(|_| AppConfig::load_from_file(&cfg_path_example))
        .expect("Load config");

    AppState::new(Arc::new(config), pool)
}

#[tokio::test]
async fn test_ckey_crawler_and_vault_full_flow_exact_decimal() {
    // 1. Dựng WireMock Server
    let mock_server = MockServer::start().await;
    let fixture_path = find_file_in_workspace("docs/api_test.json");
    let fixture_content = std::fs::read_to_string(&fixture_path)
        .expect("Đọc fixture docs/api_test.json thành công");

    // Mock endpoint /api/llm/models?key=test_ckey_key_123
    Mock::given(method("GET"))
        .and(path("/api/llm/models"))
        .and(query_param("key", "test_ckey_key_123"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(fixture_content.clone(), "application/json"),
        )
        .mount(&mock_server)
        .await;

    let state = setup_test_app_state().await;

    // 2. Tạo Provider trỏ vào MockServer
    let endpoint_url = format!("{}/api/llm/models?key=test_ckey_key_123", mock_server.uri());
    let created_provider = ProviderVaultService::create_provider(
        &state,
        CreateProviderRequest {
            name: "Mock Ckey Test".into(),
            adapter_type: Some("CKEY_LLM".into()),
            auth_mode: Some("QUERY_KEY".into()),
            auth_param_name: Some("key".into()),
            models_endpoint_url: endpoint_url,
            inference_base_url: Some(format!("{}/v1", mock_server.uri())),
            api_key: "".into(), // Key đã nằm trong URL và sẽ được tự động bóc tách
            proxy_url: None,
            fx_currency: Some("VND".into()),
            fx_rate_to_vnd_decimal: Some("1.0".into()),
            price_multiplier_decimal: Some("1.0".into()),
            scan_interval_seconds: Some(60),
        },
    )
    .await
    .expect("Tạo provider thành công");

    assert_eq!(created_provider.models_endpoint_url, format!("{}/api/llm/models", mock_server.uri()));
    assert_eq!(created_provider.api_key_masked, "test••••_123");

    // 3. Test mở khóa Secret (Reveal Secret)
    let secret = ProviderVaultService::reveal_secret(&state, &created_provider.id, "test_admin")
        .await
        .expect("Mở khóa key thành công");
    assert_eq!(secret.api_key_plaintext, "test_ckey_key_123");

    // Verify Audit Log đã được ghi
    let audit_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM system_audit_logs WHERE action = 'REVEAL_SECRET'")
        .fetch_one(&state.db)
        .await
        .expect("Query audit log");
    assert!(audit_count.0 >= 1);

    // 4. Test Sandbox Parse (dry-run)
    let sandbox_res = ProviderVaultService::sandbox_parse(
        &state,
        SandboxParseRequest {
            adapter_type: "CKEY_LLM".into(),
            auth_mode: "QUERY_KEY".into(),
            auth_param_name: Some("key".into()),
            endpoint_url: format!("{}/api/llm/models", mock_server.uri()),
            api_key: "test_ckey_key_123".into(),
            proxy_url: None,
        },
    )
    .await
    .expect("Sandbox parse thành công");

    assert_eq!(sandbox_res.http_status, 200);
    assert_eq!(sandbox_res.total_models_found, 650);
    assert_eq!(sandbox_res.normalized_preview.len(), 10);

    // 5. Chạy Crawl & Sync Provider thực tế
    let count = CkeyCrawlerService::crawl_and_sync_provider(&state, &created_provider.id)
        .await
        .expect("Crawl provider thành công");

    assert_eq!(count, 650, "Bắt buộc phải crawl đủ 650 models từ Ckey fixture");

    // 6. Kiểm tra tính chính xác tuyệt đối của dữ liệu trong SQLite (Zero Rounding)
    // 6.1. Kiểm tra sypham98/claude-opus-4-6
    let row_sypham = sqlx::query(
        r#"
        SELECT input_vnd_decimal, output_vnd_decimal, cache_read_vnd_decimal,
               cache_write_vnd_decimal, min_charge_per_request_vnd_decimal,
               effective_blended_vnd_decimal, seller_namespace
        FROM provider_model_quotes
        WHERE provider_source_id = ? AND public_name = 'sypham98/claude-opus-4-6'
        "#,
    )
    .bind(&created_provider.id)
    .fetch_one(&state.db)
    .await
    .expect("Query quote sypham98");

    let in_vnd: String = row_sypham.get("input_vnd_decimal");
    let out_vnd: String = row_sypham.get("output_vnd_decimal");
    let read_vnd: String = row_sypham.get("cache_read_vnd_decimal");
    let min_charge: String = row_sypham.get("min_charge_per_request_vnd_decimal");
    let blended: String = row_sypham.get("effective_blended_vnd_decimal");
    let seller: String = row_sypham.get("seller_namespace");

    use std::str::FromStr;
    use rust_decimal::Decimal;

    assert_eq!(Decimal::from_str(&in_vnd).unwrap(), Decimal::from_str("526.5").unwrap());
    assert_eq!(Decimal::from_str(&out_vnd).unwrap(), Decimal::from_str("1170").unwrap());
    assert_eq!(Decimal::from_str(&read_vnd).unwrap(), Decimal::from_str("253.5").unwrap());
    assert_eq!(Decimal::from_str(&min_charge).unwrap(), Decimal::from_str("5.2").unwrap());
    assert_eq!(Decimal::from_str(&blended).unwrap(), Decimal::from_str("508.95").unwrap());
    assert_eq!(seller, "sypham98");

    // 6.2. Kiểm tra số lẻ 4-6 chữ số: anhyeu00m/claude-opus-5
    let row_anhyeu = sqlx::query(
        r#"
        SELECT input_vnd_decimal, cache_read_vnd_decimal, cache_write_vnd_decimal
        FROM provider_model_quotes
        WHERE provider_source_id = ? AND public_name = 'anhyeu00m/claude-opus-5'
        "#,
    )
    .bind(&created_provider.id)
    .fetch_one(&state.db)
    .await
    .expect("Query quote anhyeu00m");

    let anhyeu_read: String = row_anhyeu.get("cache_read_vnd_decimal");
    let anhyeu_write: String = row_anhyeu.get("cache_write_vnd_decimal");
    assert_eq!(Decimal::from_str(&anhyeu_read).unwrap(), Decimal::from_str("323.4816").unwrap(), "Bắt buộc giữ nguyên vẹn 4 chữ số thập phân");
    assert_eq!(Decimal::from_str(&anhyeu_write).unwrap(), Decimal::from_str("1779.1488").unwrap(), "Bắt buộc giữ nguyên vẹn 4 chữ số thập phân");

    // 6.3. Kiểm tra phụ phí request lẻ: tdsang1999/claude-opus-4.8
    let row_tdsang = sqlx::query(
        r#"
        SELECT price_per_request_vnd_decimal, min_charge_per_request_vnd_decimal
        FROM provider_model_quotes
        WHERE provider_source_id = ? AND public_name = 'tdsang1999/claude-opus-4.8'
        "#,
    )
    .bind(&created_provider.id)
    .fetch_one(&state.db)
    .await
    .expect("Query quote tdsang1999");

    let tdsang_req: String = row_tdsang.get("price_per_request_vnd_decimal");
    assert_eq!(Decimal::from_str(&tdsang_req).unwrap(), Decimal::from_str("88.37").unwrap(), "Phụ phí request giữ đúng 88.37 đ");

    // 6.4. Kiểm tra mô hình giá khoán: thanhnhan9023/claude-opus-4-6
    let row_flat = sqlx::query(
        r#"
        SELECT pricing_model_kind, price_per_request_vnd_decimal, effective_blended_vnd_decimal
        FROM provider_model_quotes
        WHERE provider_source_id = ? AND public_name = 'thanhnhan9023/claude-opus-4-6'
        "#,
    )
    .bind(&created_provider.id)
    .fetch_one(&state.db)
    .await
    .expect("Query flat quote");

    let flat_kind: String = row_flat.get("pricing_model_kind");
    let flat_req: String = row_flat.get("price_per_request_vnd_decimal");
    let flat_blended: String = row_flat.get("effective_blended_vnd_decimal");
    assert_eq!(flat_kind, "FLAT_PER_REQUEST");
    assert_eq!(Decimal::from_str(&flat_req).unwrap(), Decimal::from_str("65").unwrap());
    assert_eq!(Decimal::from_str(&flat_blended).unwrap(), Decimal::from_str("1300").unwrap());

    // 6.5. Kiểm tra mô hình video: MiniMax-H3
    let row_video = sqlx::query(
        r#"
        SELECT pricing_model_kind, price_per_second_vnd_decimal, is_media_video_capable
        FROM provider_model_quotes
        WHERE provider_source_id = ? AND public_name = 'MiniMax-H3'
        "#,
    )
    .bind(&created_provider.id)
    .fetch_one(&state.db)
    .await
    .expect("Query video quote");

    let video_kind: String = row_video.get("pricing_model_kind");
    let video_sec: String = row_video.get("price_per_second_vnd_decimal");
    let is_video: i64 = row_video.get("is_media_video_capable");
    assert_eq!(video_kind, "PER_SECOND_VIDEO");
    assert_eq!(Decimal::from_str(&video_sec).unwrap(), Decimal::from_str("1200").unwrap());
    assert_eq!(is_video, 1);

    // 7. Kiểm tra Snapshot Index trên RAM được cập nhật
    let snapshot = state.price_index.load();
    assert_eq!(snapshot.total_active_quotes, 650);
    assert!(snapshot.models_map.len() > 100);
}

#[tokio::test]
async fn test_circuit_breaker_on_401_and_server_errors() {
    let mock_server = MockServer::start().await;
    let state = setup_test_app_state().await;

    // 1. Test 401 Unauthorized -> Trip Provider ngay lập tức
    Mock::given(method("GET"))
        .and(path("/api/llm/models"))
        .and(query_param("key", "invalid_key"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "success": false,
            "status": 401,
            "message": "API key không hợp lệ"
        })))
        .mount(&mock_server)
        .await;

    let p401 = ProviderVaultService::create_provider(
        &state,
        CreateProviderRequest {
            name: "Ckey 401 Test".into(),
            adapter_type: Some("CKEY_LLM".into()),
            auth_mode: Some("QUERY_KEY".into()),
            auth_param_name: Some("key".into()),
            models_endpoint_url: format!("{}/api/llm/models", mock_server.uri()),
            inference_base_url: None,
            api_key: "invalid_key".into(),
            proxy_url: None,
            fx_currency: None,
            fx_rate_to_vnd_decimal: None,
            price_multiplier_decimal: None,
            scan_interval_seconds: None,
        },
    )
    .await
    .expect("Tạo provider 401");

    let crawl_res = CkeyCrawlerService::crawl_and_sync_provider(&state, &p401.id).await;
    assert!(crawl_res.is_err());

    let row = sqlx::query("SELECT status, consecutive_failures FROM provider_sources WHERE id = ?")
        .bind(&p401.id)
        .fetch_one(&state.db)
        .await
        .expect("Query provider status");

    let status: String = row.get("status");
    let consec: i64 = row.get("consecutive_failures");
    assert_eq!(status, "UNAUTHORIZED_401");
    assert_eq!(consec, 1);
}
