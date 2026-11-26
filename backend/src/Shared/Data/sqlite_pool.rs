//! Module Khởi Tạo SQLite Connection Pool & Chạy Migrations (WAL Mode)

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;
use tracing::info;

/// Khởi tạo SQLite Connection Pool tối ưu cho môi trường Production (WAL Mode)
pub async fn init_sqlite_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Nếu là file SQLite cục bộ, tự động tạo thư mục chứa nếu chưa tồn tại
    if let Some(file_path) = database_url.strip_prefix("sqlite://") {
        let clean_path = file_path.split('?').next().unwrap_or(file_path);
        let path = Path::new(clean_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
    }

    let connect_opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_millis(5000))
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(16)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_millis(5000))
        .connect_with(connect_opts)
        .await?;

    info!("SQLite Connection Pool khởi tạo thành công (WAL mode active)");
    Ok(pool)
}

/// Thực thi toàn bộ SQL Migrations có versioning trong thư mục migrations/
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    info!("Bắt đầu kiểm tra và thực thi SQLite Migrations...");
    sqlx::migrate!("../migrations").run(pool).await?;
    info!("SQLite Migrations hoàn tất thành công (100% up-to-date)");
    Ok(())
}
