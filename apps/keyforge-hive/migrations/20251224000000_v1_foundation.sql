-- KeyForge v1.0 Foundation Schema
-- Enhances existing structures for quotas, scopes, and public sharing.

-- 1. Users: Add JSONB quotas (Rate Limiting & Resource Caps)
ALTER TABLE users ADD COLUMN IF NOT EXISTS quota_limits JSONB DEFAULT '{"daily_jobs": 50, "concurrent_jobs": 5}'::jsonb;

-- 2. API Keys: Add Scopes (RBAC)
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS scopes JSONB DEFAULT '["worker"]'::jsonb;

-- 3. Jobs: Add Visibility Toggle
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS is_public BOOLEAN DEFAULT FALSE;

-- 4. Audit Logs: Ensure structural integrity (Idempotent check)
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    action TEXT NOT NULL,
    actor_id UUID,
    target_resource TEXT,
    details JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 5. Ensure indexes exist for performance
CREATE INDEX IF NOT EXISTS idx_jobs_public ON jobs(is_public) WHERE is_public = TRUE;
CREATE INDEX IF NOT EXISTS idx_users_quotas ON users USING gin (quota_limits);
