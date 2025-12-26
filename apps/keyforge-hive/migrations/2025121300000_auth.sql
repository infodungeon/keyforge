-- Users table (Owners of keys)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT UNIQUE NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    -- 'admin', 'user', 'agent'
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- API Keys table (Hashed storage)
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    key_hash TEXT NOT NULL UNIQUE,
    -- SHA256 hash of the raw key
    label TEXT,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
-- Insert default admin user if not exists (Bootstrap)
INSERT INTO users (username, role)
VALUES ('admin', 'admin') ON CONFLICT (username) DO NOTHING;