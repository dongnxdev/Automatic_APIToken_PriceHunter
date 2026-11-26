//! AuthManagement Business Logic Service

use crate::app_state::{AppState, SessionInfo};
use crate::Features::AuthManagement::Common::verify_password;
use crate::Features::AuthManagement::Model::{LoginRequest, LoginResponseData};
use crate::Shared::Utils::now_utc_ms;
use uuid::Uuid;

pub struct AuthService;

impl AuthService {
    /// Xử lý đăng nhập của Operator
    pub fn login(
        state: &AppState,
        payload: LoginRequest,
    ) -> Result<LoginResponseData, &'static str> {
        let expected_user = &state.config.management.username;
        let expected_pass = &state.config.management.password_hash;

        if payload.username.trim() != expected_user {
            return Err("Tên đăng nhập hoặc mật khẩu không chính xác");
        }

        if !verify_password(&payload.password, expected_pass) {
            return Err("Tên đăng nhập hoặc mật khẩu không chính xác");
        }

        let token = format!("mr_sess_{}", Uuid::now_v7());
        let ttl_ms = (state.config.management.session_ttl_seconds as i64) * 1000;
        let expires_at_ms = now_utc_ms() + ttl_ms;

        state.session_store.insert(
            token.clone(),
            SessionInfo {
                username: payload.username,
                expires_at_ms,
            },
        );

        Ok(LoginResponseData {
            username: expected_user.clone(),
            token,
            expires_at_ms,
        })
    }

    /// Đăng xuất và thu hồi session token
    pub fn logout(state: &AppState, token: &str) {
        state.session_store.remove(token);
    }
}
