//! Shared Data Module

pub mod sqlite_pool;

pub use sqlite_pool::{init_sqlite_pool, run_migrations};
