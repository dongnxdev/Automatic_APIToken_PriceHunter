-- ModelRadar Database Schema Initialization (SQLite 3 WAL Mode)
-- Quy chuẩn: Khóa chính UUIDv7 (TEXT), Thời gian UTC Milliseconds (INTEGER),
-- Toàn bộ đơn vị giá là Exact Decimal lưu dạng TEXT (CẤM LÀM TRÒN).

PRAGMA foreign_keys = ON;

-- 1. Nguồn Crawl & API Key
CREATE TABLE IF NOT EXISTS provider_sources (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    adapter_type TEXT NOT NULL CHECK (adapter_type IN ('CKEY_LLM', 'OPENAI_COMPAT', 'CUSTOM_JSON')),
    auth_mode TEXT NOT NULL DEFAULT 'QUERY_KEY' CHECK (auth_mode IN ('QUERY_KEY', 'BEARER_HEADER', 'X_API_KEY_HEADER')),
    auth_param_name TEXT NOT NULL DEFAULT 'key',
    models_endpoint_url TEXT NOT NULL,
    inference_base_url TEXT NOT NULL,
    api_key_ciphertext TEXT NOT NULL,
    api_key_masked TEXT NOT NULL,
    proxy_url_ciphertext TEXT,
    fx_currency TEXT NOT NULL DEFAULT 'VND' CHECK (fx_currency IN ('VND', 'USD', 'CNY')),
    fx_rate_to_vnd_decimal TEXT NOT NULL DEFAULT '1.0',
    price_multiplier_decimal TEXT NOT NULL DEFAULT '1.0',
    custom_mapping_json TEXT,
    scan_interval_seconds INTEGER NOT NULL DEFAULT 180 CHECK (scan_interval_seconds >= 15),
    is_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    status TEXT NOT NULL DEFAULT 'ONLINE' CHECK (status IN ('ONLINE', 'UNAUTHORIZED_401', 'RATE_LIMITED_429', 'TIMEOUT', 'CIRCUIT_OPEN', 'DISABLED')),
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    last_ping_ms INTEGER,
    last_model_count INTEGER NOT NULL DEFAULT 0,
    last_error_message TEXT,
    last_scanned_at_ms INTEGER,
    circuit_opened_at_ms INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

-- 2. Danh mục Model chuẩn
CREATE TABLE IF NOT EXISTS canonical_models (
    id TEXT PRIMARY KEY NOT NULL,
    canonical_id TEXT NOT NULL UNIQUE,
    family TEXT NOT NULL,
    modality TEXT NOT NULL DEFAULT 'TEXT' CHECK (modality IN ('TEXT', 'IMAGE', 'AUDIO', 'VIDEO', 'EMBEDDING')),
    official_anchor_input_vnd_decimal TEXT,
    official_anchor_output_vnd_decimal TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

-- 3. Bảng ánh xạ Model Alias
CREATE TABLE IF NOT EXISTS model_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    provider_source_id TEXT NOT NULL REFERENCES provider_sources(id) ON DELETE CASCADE,
    raw_public_name TEXT NOT NULL,
    raw_display_name TEXT NOT NULL,
    normalized_lookup_key TEXT NOT NULL,
    canonical_model_id TEXT REFERENCES canonical_models(id) ON DELETE SET NULL,
    mapping_status TEXT NOT NULL CHECK (mapping_status IN ('AUTO_MAPPED', 'MANUAL_MAPPED', 'UNMAPPED', 'BLACKLISTED')),
    first_seen_at_ms INTEGER NOT NULL,
    last_seen_at_ms INTEGER NOT NULL,
    UNIQUE(provider_source_id, raw_public_name)
);

CREATE INDEX IF NOT EXISTS idx_model_aliases_status ON model_aliases(mapping_status);

-- 4. Bảng luật Regex chuẩn hóa
CREATE TABLE IF NOT EXISTS regex_alias_rules (
    id TEXT PRIMARY KEY NOT NULL,
    rule_name TEXT NOT NULL,
    rule_type TEXT NOT NULL CHECK (rule_type IN ('STRIP_PREFIX', 'VERSION_NORMALIZE', 'EXACT_ALIAS', 'BLACKLIST_IGNORE')),
    pattern_regex TEXT NOT NULL,
    replacement_template TEXT NOT NULL,
    priority_order INTEGER NOT NULL DEFAULT 100,
    is_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

-- 5. Bảng giá & Capability chuẩn hóa theo từng kênh bán
CREATE TABLE IF NOT EXISTS provider_model_quotes (
    id TEXT PRIMARY KEY NOT NULL,
    provider_source_id TEXT NOT NULL REFERENCES provider_sources(id) ON DELETE CASCADE,
    canonical_model_id TEXT REFERENCES canonical_models(id) ON DELETE SET NULL,
    seller_namespace TEXT NOT NULL,
    public_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    is_provider_model INTEGER NOT NULL CHECK (is_provider_model IN (0, 1)),
    pricing_model_kind TEXT NOT NULL CHECK (pricing_model_kind IN ('PER_TOKEN', 'FLAT_PER_REQUEST', 'PER_SECOND_VIDEO')),
    input_vnd_decimal TEXT NOT NULL,
    output_vnd_decimal TEXT NOT NULL,
    cache_enabled INTEGER NOT NULL DEFAULT 0 CHECK (cache_enabled IN (0, 1)),
    cache_read_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    cache_write_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    price_per_request_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    min_charge_per_request_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    price_per_second_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    effective_blended_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    request_rate_limit_per_minute INTEGER NOT NULL DEFAULT 0,
    max_output_tokens_limit INTEGER NOT NULL DEFAULT 0,
    context_tokens_limit INTEGER NOT NULL DEFAULT 0,
    supported_paths_json TEXT NOT NULL DEFAULT '[]',
    video_durations_allowed_json TEXT NOT NULL DEFAULT '[]',
    video_max_seconds INTEGER NOT NULL DEFAULT 0,
    video_rate_cards_json TEXT,
    video_contract_json TEXT,
    is_media_video_capable INTEGER NOT NULL DEFAULT 0 CHECK (is_media_video_capable IN (0, 1)),
    seller_circuit_status TEXT NOT NULL DEFAULT 'CLOSED' CHECK (seller_circuit_status IN ('CLOSED', 'OPEN', 'HALF_OPEN')),
    seller_consecutive_failures INTEGER NOT NULL DEFAULT 0,
    raw_payload_hash TEXT NOT NULL,
    last_live_test_status TEXT CHECK (last_live_test_status IN ('MATCHED', 'MISMATCHED', 'FAILED') OR last_live_test_status IS NULL),
    last_live_test_ttft_ms INTEGER,
    last_live_test_tps_decimal TEXT,
    updated_at_ms INTEGER NOT NULL,
    UNIQUE(provider_source_id, public_name)
);

CREATE INDEX IF NOT EXISTS idx_quotes_canonical ON provider_model_quotes(canonical_model_id);
CREATE INDEX IF NOT EXISTS idx_quotes_seller ON provider_model_quotes(seller_namespace);
CREATE INDEX IF NOT EXISTS idx_quotes_blended ON provider_model_quotes(canonical_model_id, effective_blended_vnd_decimal);

-- 6. Chuỗi thời gian lịch sử giá
CREATE TABLE IF NOT EXISTS price_history_points (
    id TEXT PRIMARY KEY NOT NULL,
    provider_source_id TEXT NOT NULL REFERENCES provider_sources(id) ON DELETE CASCADE,
    canonical_model_id TEXT NOT NULL REFERENCES canonical_models(id) ON DELETE CASCADE,
    public_name TEXT NOT NULL,
    input_vnd_decimal TEXT NOT NULL,
    output_vnd_decimal TEXT NOT NULL,
    cache_read_vnd_decimal TEXT NOT NULL,
    blended_vnd_decimal TEXT NOT NULL,
    recorded_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_price_history_lookup
    ON price_history_points(canonical_model_id, recorded_at_ms DESC, provider_source_id);

-- 7. Luật định tuyến
CREATE TABLE IF NOT EXISTS routing_policies (
    id TEXT PRIMARY KEY NOT NULL,
    canonical_model_id TEXT REFERENCES canonical_models(id) ON DELETE CASCADE,
    routing_mode TEXT NOT NULL CHECK (routing_mode IN ('CHEAPEST', 'CHEAPEST_WITH_LATENCY_CAP', 'PINNED_PROVIDER')),
    max_latency_ms INTEGER NOT NULL DEFAULT 500,
    pinned_provider_id TEXT REFERENCES provider_sources(id) ON DELETE SET NULL,
    pinned_public_name TEXT,
    failover_chain_json TEXT NOT NULL DEFAULT '[]',
    price_ceiling_vnd_decimal TEXT,
    is_price_breaker_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_price_breaker_enabled IN (0, 1)),
    updated_at_ms INTEGER NOT NULL
);

-- 8. Khóa nội bộ IDE/CLI
CREATE TABLE IF NOT EXISTS internal_api_keys (
    id TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    key_prefix TEXT NOT NULL,
    key_last4 TEXT NOT NULL,
    key_hash_sha256 TEXT NOT NULL UNIQUE,
    total_tokens_used INTEGER NOT NULL DEFAULT 0,
    total_requests_count INTEGER NOT NULL DEFAULT 0,
    total_saved_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    last_used_at_ms INTEGER,
    created_at_ms INTEGER NOT NULL
);

-- 9. Log sử dụng qua Smart Gateway
CREATE TABLE IF NOT EXISTS gateway_usage_logs (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL,
    internal_api_key_id TEXT NOT NULL REFERENCES internal_api_keys(id) ON DELETE CASCADE,
    canonical_model_id TEXT NOT NULL,
    selected_provider_id TEXT NOT NULL,
    selected_seller_namespace TEXT NOT NULL,
    selected_public_name TEXT NOT NULL,
    failover_attempts INTEGER NOT NULL DEFAULT 0,
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    cached_tokens INTEGER NOT NULL DEFAULT 0,
    actual_cost_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    anchor_cost_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    saved_vnd_decimal TEXT NOT NULL DEFAULT '0.0',
    ttft_ms INTEGER,
    total_latency_ms INTEGER NOT NULL,
    http_status INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_gateway_usage_created ON gateway_usage_logs(created_at_ms DESC);

-- 10. Luật báo động
CREATE TABLE IF NOT EXISTS alert_rules (
    id TEXT PRIMARY KEY NOT NULL,
    rule_name TEXT NOT NULL,
    rule_type TEXT NOT NULL CHECK (rule_type IN ('PRICE_BELOW', 'PROVIDER_FAILURE_STREAK', 'PROVIDER_AUTH_401', 'MODEL_UNMAPPED', 'PRICE_SPIKE')),
    canonical_model_id TEXT REFERENCES canonical_models(id) ON DELETE CASCADE,
    provider_source_id TEXT REFERENCES provider_sources(id) ON DELETE CASCADE,
    threshold_vnd_decimal TEXT,
    failure_streak_threshold INTEGER,
    channels_json TEXT NOT NULL,
    cooldown_seconds INTEGER NOT NULL DEFAULT 300,
    trigger_count_today INTEGER NOT NULL DEFAULT 0,
    last_triggered_at_ms INTEGER,
    is_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    created_at_ms INTEGER NOT NULL
);

-- 11. Lịch sử gửi tin cảnh báo
CREATE TABLE IF NOT EXISTS alert_deliveries (
    id TEXT PRIMARY KEY NOT NULL,
    alert_rule_id TEXT NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    channel_type TEXT NOT NULL CHECK (channel_type IN ('TELEGRAM', 'DISCORD', 'WEBHOOK')),
    payload_summary TEXT NOT NULL,
    delivery_status TEXT NOT NULL CHECK (delivery_status IN ('SENT', 'RETRYING', 'DEAD_LETTER')),
    is_read INTEGER NOT NULL DEFAULT 0 CHECK (is_read IN (0, 1)),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at_ms INTEGER NOT NULL
);

-- 12. Hàng đợi Dead-Letter Queue
CREATE TABLE IF NOT EXISTS dead_letter_events (
    id TEXT PRIMARY KEY NOT NULL,
    event_type TEXT NOT NULL CHECK (event_type IN ('CRAWL_PROVIDER', 'DISPATCH_ALERT', 'RECORD_USAGE')),
    correlation_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    error_reason TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 5,
    next_retry_at_ms INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('PENDING', 'EXHAUSTED', 'REPLAYED_OK')),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

-- 13. Audit Log hệ thống
CREATE TABLE IF NOT EXISTS system_audit_logs (
    id TEXT PRIMARY KEY NOT NULL,
    action TEXT NOT NULL,
    actor TEXT NOT NULL,
    details_json TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL
);
