//! Global Application State

use crate::Shared::Common::AppConfig;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use shared_contract::NormalizedCkeyModel;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Thông tin phiên đăng nhập của Operator
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub username: String,
    pub expires_at_ms: i64,
}

/// Snapshot bảng giá nạp sẵn trên RAM để đọc O(1)
#[derive(Debug, Clone, Default)]
pub struct PriceSnapshotIndex {
    /// canonical_id -> danh sách các kênh bán (đã sắp xếp từ rẻ tới đắt)
    pub models_map: HashMap<String, Vec<NormalizedCkeyModel>>,
    /// Tổng số kênh bán đang hoạt động
    pub total_active_quotes: usize,
    /// Thời điểm biên dịch snapshot gần nhất (UTC ms)
    pub last_compiled_at_ms: i64,
}

/// Trạng thái ứng dụng dùng chung xuyên suốt các controller và background workers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: SqlitePool,
    pub http_client: reqwest::Client,
    pub session_store: Arc<DashMap<String, SessionInfo>>,
    pub is_shutting_down: Arc<AtomicBool>,
    pub price_index: Arc<ArcSwap<PriceSnapshotIndex>>,
    pub active_crawler_lock: Arc<Mutex<()>>,
    pub auto_scan_enabled: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(config: Arc<AppConfig>, db: SqlitePool) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.crawler.request_timeout_ms))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .gzip(true)
            .build()
            .expect("Reqwest client khởi tạo thành công");

        let auto_scan_enabled = Arc::new(AtomicBool::new(config.crawler.auto_scan_enabled));

        Self {
            config,
            db,
            http_client,
            session_store: Arc::new(DashMap::new()),
            is_shutting_down: Arc::new(AtomicBool::new(false)),
            price_index: Arc::new(ArcSwap::from_pointee(PriceSnapshotIndex::default())),
            active_crawler_lock: Arc::new(Mutex::new(())),
            auto_scan_enabled,
        }
    }

    /// Kiểm tra hệ thống có đang trong tiến trình Graceful Shutdown không
    #[inline]
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::Relaxed)
    }

    /// Xác thực token phiên làm việc
    pub fn validate_session(&self, token: &str) -> Option<String> {
        let now = chrono::Utc::now().timestamp_millis();
        if let Some(entry) = self.session_store.get(token) {
            if entry.expires_at_ms > now {
                return Some(entry.username.clone());
            }
        }
        None
    }
}
