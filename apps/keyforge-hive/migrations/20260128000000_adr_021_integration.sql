-- apps/keyforge-hive/migrations/20260128000000_adr_021_integration.sql

-- 1. User Profiles & Preferences
CREATE TABLE IF NOT EXISTS user_profiles (
    id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    biometric_status TEXT NOT NULL DEFAULT 'empty',
    preferences JSONB NOT NULL DEFAULT '{
        "space_hand": "right",
        "use_personal_biometrics": false,
        "theme": "dark"
    }',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 2. Biometric Profiles
CREATE TABLE IF NOT EXISTS biometric_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    performance_index REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id)
);

-- 3. Biometric Latencies (Sparse storage for bigrams)
CREATE TABLE IF NOT EXISTS biometric_latencies (
    profile_id UUID NOT NULL REFERENCES biometric_profiles(id) ON DELETE CASCADE,
    key1_code INTEGER NOT NULL,
    key2_code INTEGER NOT NULL,
    median_ms REAL NOT NULL,
    std_dev REAL NOT NULL,
    sample_count INTEGER NOT NULL,
    PRIMARY KEY (profile_id, key1_code, key2_code)
);

-- 4. Layout Submissions (Community)
CREATE TABLE IF NOT EXISTS layout_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    keyboard_id TEXT NOT NULL,
    layout_data JSONB NOT NULL,
    score INTEGER NOT NULL,
    tags TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 5. Analysis Sessions
CREATE TABLE IF NOT EXISTS analysis_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    keyboard_id TEXT NOT NULL,
    corpus_id TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS analysis_session_history (
    id BIGSERIAL PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES analysis_sessions(id) ON DELETE CASCADE,
    layout_id TEXT NOT NULL,
    score INTEGER NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 6. Enhanced Token Tracking (Research Metrics)
CREATE TABLE IF NOT EXISTS research_metrics (
    id BIGSERIAL PRIMARY KEY,
    session_id UUID REFERENCES analysis_sessions(id) ON DELETE SET NULL,
    query TEXT,
    mode TEXT,
    phase TEXT,
    response_ms INTEGER,
    success BOOLEAN DEFAULT TRUE,
    error_message TEXT,
    search_engine TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indices for performance
CREATE INDEX IF NOT EXISTS idx_biometric_user ON biometric_profiles(user_id);
CREATE INDEX IF NOT EXISTS idx_layout_author ON layout_submissions(author_id);
CREATE INDEX IF NOT EXISTS idx_analysis_user ON analysis_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_research_session ON research_metrics(session_id);
