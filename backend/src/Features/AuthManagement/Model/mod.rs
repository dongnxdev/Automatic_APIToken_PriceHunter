//! AuthManagement Model DTOs

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponseData {
    pub username: String,
    pub token: String,
    pub expires_at_ms: i64,
}

#[derive(Debug, Serialize)]
pub struct HealthResponseData {
    pub status: &'static str,
    pub timestamp_ms: i64,
}
