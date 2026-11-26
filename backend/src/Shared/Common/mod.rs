//! Shared Common Module

pub mod config;
pub mod crypto;

pub use config::AppConfig;
pub use crypto::{decrypt_aes_gcm, encrypt_aes_gcm};
