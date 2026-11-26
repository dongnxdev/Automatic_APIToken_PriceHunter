//! CrawlerEngine HTTP Handlers & Router

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};
use shared_contract::ApiResponse;
use std::sync::atomic::Ordering;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::Features::AuthManagement::Results::json_response;
use crate::Features::CrawlerEngine::Services::CkeyCrawlerService;

/// Handler kích hoạt quét cưỡng bức toàn bộ Provider (FORCE SCAN ALL)
pub async fn force_scan_all_handler(State(state): State<AppState>) -> Response {
    let req_id = Uuid::now_v7().to_string();
    match CkeyCrawlerService::force_scan_all(&state).await {
        Ok(count) => json_response(
            StatusCode::OK,
            ApiResponse::ok(
                serde_json::json!({
                    "total_models_scanned": count,
                    "message": "Quét toàn bộ Provider thành công"
                }),
                req_id,
            ),
        ),
        Err(msg) => json_response(
            StatusCode::CONFLICT,
            ApiResponse::<()>::err("SCAN_LOCKED", msg, req_id),
        ),
    }
}

/// Handler bật/tắt AUTO-SCAN toàn cục
pub async fn toggle_auto_scan_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let req_id = Uuid::now_v7().to_string();
    let enabled = payload
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    state.auto_scan_enabled.store(enabled, Ordering::SeqCst);

    json_response(
        StatusCode::OK,
        ApiResponse::ok(
            serde_json::json!({
                "auto_scan_enabled": enabled
            }),
            req_id,
        ),
    )
}

/// Tạo Router cho CrawlerEngine
pub fn crawler_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/scanner/force-all", post(force_scan_all_handler))
        .route("/api/v1/scanner/auto-scan", post(toggle_auto_scan_handler))
}
