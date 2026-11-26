//! Shared Utilities Module

pub mod masking;
pub mod time_text;

pub use masking::{mask_secret, mask_url_secrets};
pub use time_text::{normalize_lookup_key, normalize_nfc_trim, now_utc_iso, now_utc_ms};
