//! ModelRadar Server Binary Entrypoint
//!
//! Production-Ready Price API Hunter & Smart Gateway

use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

use modelradar_server::app_state::AppState;
use modelradar_server::build_app;
use modelradar_server::Features::ProviderVault::Model::CreateProviderRequest;
use modelradar_server::Features::ProviderVault::ProviderVaultService;
use modelradar_server::Shared::Common::AppConfig;
use modelradar_server::Shared::Data::{init_sqlite_pool, run_migrations};
use modelradar_server::Shared::Utils::now_utc_iso;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Khởi tạo Structured Logger
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,modelradar_server=debug".into()),
        )
        .json()
        .init();

    info!(
        started_at = now_utc_iso(),
        "=== Khởi động ModelRadar (Price API Hunter & Smart Gateway) ==="
    );

    // Xác định đường dẫn file cấu hình appsettings.json
    let config_path = if Path::new("appsettings.json").exists() {
        "appsettings.json"
    } else if Path::new("appsettings.example.json").exists() {
        "appsettings.example.json"
    } else {
        "appsettings.json"
    };

    info!(config_file = config_path, "Đang nạp cấu hình hệ thống...");
    let config = match AppConfig::load_from_file(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!(error = %e, "Lỗi kiểm định cấu hình khởi động (Fail-fast)");
            std::process::exit(1);
        }
    };

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);

    // Khởi tạo SQLite Pool và chạy Migrations
    let pool = init_sqlite_pool(&config.server.database_url).await?;
    run_migrations(&pool).await?;

    let state = AppState::new(Arc::new(config.clone()), pool);

    // Đồng bộ danh sách Provider khởi tạo từ appsettings.json vào DB nếu chưa có
    let existing_providers = ProviderVaultService::list_providers(&state).await?;
    for p in &config.providers {
        if !existing_providers.iter().any(|ep| ep.name == p.name) {
            info!(provider = p.name, "Đồng bộ Provider ban đầu từ appsettings.json vào DB...");
            let _ = ProviderVaultService::create_provider(
                &state,
                CreateProviderRequest {
                    name: p.name.clone(),
                    adapter_type: Some(p.adapter_type.clone()),
                    auth_mode: Some(p.auth_mode.clone()),
                    auth_param_name: Some(p.query_param_name.clone()),
                    models_endpoint_url: p.endpoint.clone(),
                    inference_base_url: Some(p.inference_base_url.clone()),
                    api_key: p.api_key.clone(),
                    proxy_url: None,
                    fx_currency: Some(p.fx_currency.clone()),
                    fx_rate_to_vnd_decimal: Some("1.0".into()),
                    price_multiplier_decimal: Some("1.0".into()),
                    scan_interval_seconds: Some(config.crawler.default_interval_seconds as i64),
                },
            )
            .await;
        }
    }

    let app = build_app(state.clone());
    let listener = TcpListener::bind(&bind_addr).await?;
    info!(
        addr = bind_addr,
        "Server HTTP ModelRadar đang lắng nghe và sẵn sàng phục vụ..."
    );

    // Lắng nghe tín hiệu Graceful Shutdown (SIGINT / Ctrl+C)
    let shutdown_state = state.clone();
    let graceful_shutdown = async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Lắng nghe tín hiệu Ctrl+C thất bại");
        info!("Nhận tín hiệu dừng (SIGINT/Ctrl+C). Bắt đầu Graceful Shutdown...");
        shutdown_state.is_shutting_down.store(true, Ordering::SeqCst);
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(graceful_shutdown)
        .await?;

    info!("Server đã đóng toàn bộ kết nối và tắt an toàn (Graceful Shutdown hoàn tất).");
    Ok(())
}
