-- ModelRadar Default Seed Data
-- Seed Canonical Models & Regex Rules chuẩn hóa cho Ckey.vn

-- 1. Seed danh mục Canonical Models cơ bản
INSERT OR IGNORE INTO canonical_models (id, canonical_id, family, modality, official_anchor_input_vnd_decimal, official_anchor_output_vnd_decimal, is_active, created_at_ms, updated_at_ms)
VALUES
    ('019a9f00-0001-7000-8000-000000000001', 'claude-opus-4.6', 'Anthropic', 'TEXT', '3000.0', '15000.0', 1, 1732590000000, 1732590000000),
    ('019a9f00-0002-7000-8000-000000000002', 'claude-opus-4.7', 'Anthropic', 'TEXT', '4000.0', '20000.0', 1, 1732590000000, 1732590000000),
    ('019a9f00-0003-7000-8000-000000000003', 'claude-opus-4.8', 'Anthropic', 'TEXT', '800.0', '800.0', 1, 1732590000000, 1732590000000),
    ('019a9f00-0004-7000-8000-000000000004', 'claude-sonnet-4.6', 'Anthropic', 'TEXT', '650.0', '3250.0', 1, 1732590000000, 1732590000000),
    ('019a9f00-0005-7000-8000-000000000005', 'gpt-5.1-codex', 'OpenAI', 'TEXT', '1200.0', '4800.0', 1, 1732590000000, 1732590000000),
    ('019a9f00-0006-7000-8000-000000000006', 'deepseek-r1-0528', 'DeepSeek', 'TEXT', '180.0', '540.0', 1, 1732590000000, 1732590000000),
    ('019a9f00-0007-7000-8000-000000000007', 'MiniMax-H3', 'MiniMax', 'VIDEO', '0.0', '0.0', 1, 1732590000000, 1732590000000);

-- 2. Seed các luật Regex chuẩn hóa
INSERT OR IGNORE INTO regex_alias_rules (id, rule_name, rule_type, pattern_regex, replacement_template, priority_order, is_enabled, created_at_ms, updated_at_ms)
VALUES
    ('019a9f10-0001-7000-8000-000000000001', 'Cắt tiền tố sàn người bán', 'STRIP_PREFIX', '^[^/]+/', '', 10, 1, 1732590000000, 1732590000000),
    ('019a9f10-0002-7000-8000-000000000002', 'Cắt tiền tố hãng anthropic', 'VERSION_NORMALIZE', '^anthropic-', '', 20, 1, 1732590000000, 1732590000000),
    ('019a9f10-0003-7000-8000-000000000003', 'Chuẩn hóa dấu gạch phiên bản', 'VERSION_NORMALIZE', '(\d+)-(\d+)', '$1.$2', 30, 1, 1732590000000, 1732590000000),
    ('019a9f10-0004-7000-8000-000000000004', 'Bỏ qua model embedding rác', 'BLACKLIST_IGNORE', '^(text-embedding|bge-|rerank)', 'IGNORE', 40, 1, 1732590000000, 1732590000000);
