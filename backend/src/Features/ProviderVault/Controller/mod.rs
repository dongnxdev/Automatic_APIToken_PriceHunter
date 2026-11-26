//! ProviderVault HTTP Handlers & Router

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use shared_contract::ApiResponse;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::Features::AuthManagement::Results::json_response;
use crate::Features::ProviderVault::Model::{
    CreateProviderRequest, RevealSecretResponse, SandboxParseRequest,
};
use crate::Features::ProviderVault::Services::ProviderVaultService;

/// Lấy danh sách toàn bộ Provider
pub async fn list_providers_handler(State(state): State<AppState>) -> Response {
    let req_id = Uuid::now_v7().to_string();
    match ProviderVaultService::list_providers(&state).await {
        Ok(list) => json_response(StatusCode::OK, ApiResponse::ok(list, req_id)),
        Err(e) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<()>::err("DB_ERROR", e.to_string(), req_id),
        ),
    }
}

/// Thêm mới một Provider
pub async fn create_provider_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateProviderRequest>,
) -> Response {
    let req_id = Uuid::now_v7().to_string();
    match ProviderVaultService::create_provider(&state, payload).await {
        Ok(created) => json_response(StatusCode::CREATED, ApiResponse::ok(created, req_id)),
        Err(msg) => json_response(
            StatusCode::BAD_REQUEST,
            ApiResponse::<()>::err("VALIDATION_ERROR", msg, req_id),
        ),
    }
}

/// Mở khóa xem API Key (Bắt buộc ghi Audit Log)
pub async fn reveal_secret_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let req_id = Uuid::now_v7().to_string();
    // Actor mặc định lấy từ token session hoặc admin
    let actor = "admin";
    match ProviderVaultService::reveal_secret(&state, &id, actor).await {
        Ok(data) => json_response(StatusCode::OK, ApiResponse::ok(data, req_id)),
        Err(msg) => json_response(
            StatusCode::BAD_REQUEST,
            ApiResponse::<RevealSecretResponse>::err("REVEAL_FAILED", msg, req_id),
        ),
    }
}

/// JSON Parser Sandbox (Test Fetch & Preview thô vs chuẩn hóa)
pub async fn sandbox_parse_handler(
    State(state): State<AppState>,
    Json(payload): Json<SandboxParseRequest>,
) -> Response {
    let req_id = Uuid::now_v7().to_string();
    match ProviderVaultService::sandbox_parse(&state, payload).await {
        Ok(data) => json_response(StatusCode::OK, ApiResponse::ok(data, req_id)),
        Err(msg) => json_response(
            StatusCode::BAD_REQUEST,
            ApiResponse::<()>::err("SANDBOX_PARSE_ERROR", msg, req_id),
        ),
    }
}

/// Tạo Router cho ProviderVault
pub fn provider_vault_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/providers", get(list_providers_handler).post(create_provider_handler))
        .route("/api/v1/providers/{id}/reveal-secret", post(reveal_secret_handler))
        .route("/api/v1/providers/sandbox-parse", post(sandbox_parse_handler))
}
