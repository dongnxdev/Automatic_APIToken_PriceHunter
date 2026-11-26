//! ModelRadar Server Library Core

#[allow(non_snake_case)]
pub mod Features;
#[allow(non_snake_case)]
pub mod Shared;
pub mod app_state;

use app_state::AppState;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Dựng Axum Router hoàn chỉnh từ AppState
pub fn build_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(Features::AuthManagement::auth_router())
        .merge(Features::ProviderVault::provider_vault_router())
        .merge(Features::CrawlerEngine::crawler_router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
