-- ===== EXTENSIONS =====
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ===== TABLES =====
CREATE TABLE IF NOT EXISTS keyboards (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    author TEXT,
    version TEXT,
    notes TEXT,
    kb_type TEXT,
    unique_hash TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS keyboard_keys (
    id SERIAL PRIMARY KEY,
    keyboard_id INTEGER NOT NULL REFERENCES keyboards(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    x REAL NOT NULL,
    y REAL NOT NULL,
    w REAL DEFAULT 1.0,
    h REAL DEFAULT 1.0,
    r REAL DEFAULT 0.0,
    hand INTEGER NOT NULL,
    finger INTEGER NOT NULL,
    row_idx INTEGER NOT NULL,
    col_idx INTEGER NOT NULL,
    is_stretch BOOLEAN DEFAULT FALSE,
    is_prime BOOLEAN DEFAULT FALSE,
    is_med BOOLEAN DEFAULT FALSE,
    is_low BOOLEAN DEFAULT FALSE
);
CREATE INDEX IF NOT EXISTS idx_keys_keyboard ON keyboard_keys(keyboard_id);

CREATE TABLE IF NOT EXISTS scoring_profiles (
    id SERIAL PRIMARY KEY,
    weights JSONB NOT NULL,
    config_hash TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS search_configs (
    id SERIAL PRIMARY KEY,
    search_epochs INTEGER NOT NULL,
    search_steps INTEGER NOT NULL,
    search_patience INTEGER NOT NULL,
    search_patience_threshold REAL NOT NULL,
    temp_min REAL NOT NULL,
    temp_max REAL NOT NULL,
    opt_limit_fast INTEGER NOT NULL,
    opt_limit_slow INTEGER NOT NULL,
    config_hash TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    keyboard_id INTEGER NOT NULL REFERENCES keyboards(id),
    scoring_profile_id INTEGER NOT NULL REFERENCES scoring_profiles(id),
    search_config_id INTEGER NOT NULL REFERENCES search_configs(id),
    pinned_keys TEXT NOT NULL,
    corpus_name TEXT NOT NULL,
    cost_matrix TEXT NOT NULL,
    status TEXT DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS hardware_profiles (
    cpu_signature TEXT PRIMARY KEY,
    architecture TEXT NOT NULL,
    l1_cache_kb INTEGER,
    l2_cache_kb INTEGER,
    l3_cache_kb INTEGER,
    verified_ops_per_sec REAL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    cpu_signature TEXT REFERENCES hardware_profiles(cpu_signature),
    cpu_cores INTEGER,
    performance_rating REAL,
    last_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS results (
    id BIGSERIAL PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    layout TEXT NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_results_job_score ON results(job_id, score ASC);

CREATE TABLE IF NOT EXISTS submissions (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    layout_str TEXT NOT NULL,
    author TEXT,
    status TEXT DEFAULT 'pending',
    submitted_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ===== VIEWS =====
CREATE OR REPLACE VIEW v_active_jobs AS
SELECT j.id,
    j.pinned_keys,
    j.corpus_name,
    j.cost_matrix,
    j.created_at,
    jsonb_build_object(
        'meta',
        jsonb_build_object(
            'name',
            k.name,
            'author',
            k.author,
            'version',
            k.version,
            'notes',
            k.notes,
            'type',
            k.kb_type
        ),
        'geometry',
        jsonb_build_object(
            'keys',
            (
                SELECT jsonb_agg(
                        jsonb_build_object(
                            'id',
                            'k' || kk.idx,
                            'hand',
                            kk.hand,
                            'finger',
                            kk.finger,
                            'row',
                            kk.row_idx,
                            'col',
                            kk.col_idx,
                            'x',
                            kk.x,
                            'y',
                            kk.y,
                            'w',
                            kk.w,
                            'h',
                            kk.h,
                            'is_stretch',
                            kk.is_stretch,
                            'r',
                            kk.r
                        )
                        ORDER BY kk.idx
                    )
                FROM keyboard_keys kk
                WHERE kk.keyboard_id = k.id
            ),
            'prime_slots',
            (
                SELECT coalesce(jsonb_agg(idx), '[]'::jsonb)
                FROM keyboard_keys
                WHERE keyboard_id = k.id
                    AND is_prime
            ),
            'med_slots',
            (
                SELECT coalesce(jsonb_agg(idx), '[]'::jsonb)
                FROM keyboard_keys
                WHERE keyboard_id = k.id
                    AND is_med
            ),
            'low_slots',
            (
                SELECT coalesce(jsonb_agg(idx), '[]'::jsonb)
                FROM keyboard_keys
                WHERE keyboard_id = k.id
                    AND is_low
            ),
            'home_row',
            1
        )
    ) AS geometry_json,
    sp.weights AS weights_json,
    (to_jsonb(sc) - 'id' - 'config_hash') AS params_json
FROM jobs j
    JOIN keyboards k ON j.keyboard_id = k.id
    JOIN scoring_profiles sp ON j.scoring_profile_id = sp.id
    JOIN search_configs sc ON j.search_config_id = sc.id
WHERE j.status = 'active';

-- ===== PROCEDURES =====
CREATE OR REPLACE FUNCTION register_node_heartbeat(
        p_node_id TEXT,
        p_cpu_model TEXT,
        p_arch TEXT,
        p_cores INTEGER,
        p_l2_cache INTEGER,
        p_ops_per_sec REAL
    ) RETURNS VOID AS $$ BEGIN
INSERT INTO hardware_profiles (
        cpu_signature,
        architecture,
        l2_cache_kb,
        verified_ops_per_sec,
        updated_at
    )
VALUES (
        p_cpu_model,
        p_arch,
        p_l2_cache,
        p_ops_per_sec,
        CURRENT_TIMESTAMP
    ) ON CONFLICT (cpu_signature) DO
UPDATE
SET verified_ops_per_sec = GREATEST(
        hardware_profiles.verified_ops_per_sec,
        EXCLUDED.verified_ops_per_sec
    ),
    l2_cache_kb = COALESCE(
        EXCLUDED.l2_cache_kb,
        hardware_profiles.l2_cache_kb
    ),
    updated_at = CURRENT_TIMESTAMP;
INSERT INTO nodes (
        id,
        cpu_signature,
        cpu_cores,
        performance_rating,
        last_seen
    )
VALUES (
        p_node_id,
        p_cpu_model,
        p_cores,
        p_ops_per_sec,
        CURRENT_TIMESTAMP
    ) ON CONFLICT (id) DO
UPDATE
SET last_seen = CURRENT_TIMESTAMP,
    performance_rating = EXCLUDED.performance_rating,
    cpu_cores = EXCLUDED.cpu_cores;
END;
$$ LANGUAGE plpgsql;

-- ===== OPTIMIZATION INDEXES =====
CREATE INDEX IF NOT EXISTS idx_results_node_id ON results(node_id);
CREATE INDEX IF NOT EXISTS idx_results_created_at ON results(created_at);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);
CREATE INDEX IF NOT EXISTS idx_nodes_last_seen ON nodes(last_seen);
CREATE INDEX IF NOT EXISTS idx_submissions_submitted_at ON submissions(submitted_at);
