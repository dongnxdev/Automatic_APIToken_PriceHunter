//! AuthManagement HTTP Handlers & Router

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use shared_contract::ApiResponse;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::Features::AuthManagement::Model::LoginRequest;
use crate::Features::AuthManagement::Results::{json_response, login_cookie_header};
use crate::Features::AuthManagement::Services::AuthService;
use crate::Shared::Utils::now_utc_ms;

/// Handler đăng nhập Management
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    let req_id = Uuid::now_v7().to_string();
    match AuthService::login(&state, payload) {
        Ok(data) => {
            let cookie_headers = login_cookie_header(&data.token);
            let mut res = json_response(StatusCode::OK, ApiResponse::ok(data, req_id));
            res.headers_mut().extend(cookie_headers);
            res
        }
        Err(msg) => json_response(
            StatusCode::UNAUTHORIZED,
            ApiResponse::<()>::err("UNAUTHORIZED", msg, req_id),
        ),
    }
}

/// Handler đăng xuất
pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let req_id = Uuid::now_v7().to_string();
    if let Some(cookie_hdr) = headers.get(header::COOKIE).and_then(|h| h.to_str().ok()) {
        for cookie in cookie_hdr.split(';') {
            let trimmed = cookie.trim();
            if let Some(token) = trimmed.strip_prefix("modelradar_session=") {
                AuthService::logout(&state, token);
            }
        }
    }
    json_response(StatusCode::OK, ApiResponse::ok(true, req_id))
}

/// Liveness probe: báo hệ thống còn sống
pub async fn health_live_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ALIVE",
        "timestamp_ms": now_utc_ms()
    })))
}

/// Readiness probe: kiểm tra sẵn sàng nhận traffic
pub async fn health_ready_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    if state.is_shutting_down() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "SHUTTING_DOWN",
                "timestamp_ms": now_utc_ms()
            })),
        );
    }

    // Kiểm tra kết nối tới SQLite Database
    let query_check: Result<(i64,), sqlx::Error> =
        sqlx::query_as("SELECT 1").fetch_one(&state.db).await;

    match query_check {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "READY",
                "database": "OK",
                "timestamp_ms": now_utc_ms()
            })),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "DB_UNAVAILABLE",
                "error": e.to_string(),
                "timestamp_ms": now_utc_ms()
            })),
        ),
    }
}

/// Prometheus metrics endpoint
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.price_index.load();
    let total_quotes = snapshot.total_active_quotes;
    let total_models = snapshot.models_map.len();
    let is_shutting_down = if state.is_shutting_down() { 1 } else { 0 };

    let body = format!(
        "# HELP modelradar_active_quotes_total Tổng số báo giá đang active\n\
         # TYPE modelradar_active_quotes_total gauge\n\
         modelradar_active_quotes_total {total_quotes}\n\
         # HELP modelradar_canonical_models_total Tổng số model chuẩn trong snapshot\n\
         # TYPE modelradar_canonical_models_total gauge\n\
         modelradar_canonical_models_total {total_models}\n\
         # HELP modelradar_shutting_down Trạng thái tắt hệ thống (1 nếu đang shutdown)\n\
         # TYPE modelradar_shutting_down gauge\n\
         modelradar_shutting_down {is_shutting_down}\n"
    );

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        body,
    )
}

/// Tạo Router cho AuthManagement
pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/auth/login", post(login_handler))
        .route("/api/v1/auth/logout", post(logout_handler))
        .route("/health/live", get(health_live_handler))
        .route("/health/ready", get(health_ready_handler))
        .route("/metrics", get(metrics_handler))
}
