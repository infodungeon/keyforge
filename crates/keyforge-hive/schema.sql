-- ===== CLEANUP (For Dev Iteration) =====
-- Cascading drop allows us to reset the schema cleanly
DROP VIEW IF EXISTS v_active_jobs;
DROP TABLE IF EXISTS results CASCADE;
DROP TABLE IF EXISTS jobs CASCADE;
DROP TABLE IF EXISTS nodes CASCADE;
DROP TABLE IF EXISTS hardware_profiles CASCADE;
DROP TABLE IF EXISTS keyboard_keys CASCADE;
DROP TABLE IF EXISTS keyboards CASCADE;
DROP TABLE IF EXISTS scoring_profiles CASCADE;
DROP TABLE IF EXISTS search_configs CASCADE;
DROP TABLE IF EXISTS submissions CASCADE;

-- ===== EXTENSIONS =====
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto"; -- Required for digest() hashing

-- ===== 1. NORMALIZED DEFINITIONS =====

-- Keyboards: Reusable hardware definitions
CREATE TABLE keyboards (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    author TEXT,
    version TEXT,
    notes TEXT,
    kb_type TEXT,
    -- Hash ensures we don't store "Corne v1" 50 times
    unique_hash TEXT GENERATED ALWAYS AS (digest(name || author || version, 'sha256')) STORED UNIQUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Physical Keys: 1-to-Many relationship
CREATE TABLE keyboard_keys (
    id SERIAL PRIMARY KEY,
    keyboard_id INTEGER NOT NULL REFERENCES keyboards(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL, -- 0-based index in array
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
CREATE INDEX idx_keys_keyboard ON keyboard_keys(keyboard_id);

-- Scoring Profiles: Unique combinations of weights
CREATE TABLE scoring_profiles (
    id SERIAL PRIMARY KEY,
    -- Physics
    penalty_sfb_base REAL NOT NULL,
    penalty_sfb_lateral REAL NOT NULL,
    penalty_sfb_lateral_weak REAL NOT NULL,
    penalty_sfb_diagonal REAL NOT NULL,
    penalty_sfb_long REAL NOT NULL,
    penalty_sfb_bottom REAL NOT NULL,
    -- Ergonomics
    penalty_sfr_weak_finger REAL NOT NULL,
    penalty_sfr_bad_row REAL NOT NULL,
    penalty_sfr_lat REAL NOT NULL,
    penalty_scissor REAL NOT NULL,
    penalty_lateral REAL NOT NULL,
    penalty_ring_pinky REAL NOT NULL,
    -- Flow
    penalty_redirect REAL NOT NULL,
    penalty_skip REAL NOT NULL,
    penalty_hand_run REAL NOT NULL,
    bonus_inward_roll REAL NOT NULL,
    bonus_bigram_roll_in REAL NOT NULL,
    bonus_bigram_roll_out REAL NOT NULL,
    -- General
    penalty_imbalance REAL NOT NULL,
    max_hand_imbalance REAL NOT NULL,
    weight_vertical_travel REAL NOT NULL,
    weight_lateral_travel REAL NOT NULL,
    weight_finger_effort REAL NOT NULL,
    
    config_hash TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Search Configs: Algorithm parameters
CREATE TABLE search_configs (
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

-- ===== 2. OPERATIONAL DATA =====

-- Jobs: The intersection of definitions and configs
CREATE TABLE jobs (
    id TEXT PRIMARY KEY, -- Provided Client Hash
    keyboard_id INTEGER NOT NULL REFERENCES keyboards(id),
    scoring_profile_id INTEGER NOT NULL REFERENCES scoring_profiles(id),
    search_config_id INTEGER NOT NULL REFERENCES search_configs(id),
    
    pinned_keys TEXT NOT NULL,
    corpus_name TEXT NOT NULL,
    cost_matrix TEXT NOT NULL,
    
    status TEXT DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Hardware Knowledge Base
CREATE TABLE hardware_profiles (
    cpu_signature TEXT PRIMARY KEY,
    architecture TEXT NOT NULL,
    l1_cache_kb INTEGER,
    l2_cache_kb INTEGER, -- Critical for Strategy Selection
    l3_cache_kb INTEGER,
    verified_ops_per_sec REAL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Worker Nodes
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    cpu_signature TEXT REFERENCES hardware_profiles(cpu_signature),
    cpu_cores INTEGER,
    performance_rating REAL,
    last_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Optimization Results
CREATE TABLE results (
    id BIGSERIAL PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id),
    layout TEXT NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    node_id TEXT NOT NULL REFERENCES nodes(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_results_job_score ON results(job_id, score ASC);

-- Community Submissions
CREATE TABLE submissions (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    layout_str TEXT NOT NULL,
    author TEXT,
    status TEXT DEFAULT 'pending',
    submitted_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ===== 3. STORED PROCEDURES (Logic Pushdown) =====

-- Procedure 1: Register Job
-- Handles deduplication of keyboards, weights, and params automatically
CREATE OR REPLACE FUNCTION register_full_job(
    p_job_id TEXT,
    p_kb_meta JSONB,
    p_kb_keys JSONB,
    p_kb_slots JSONB,
    p_weights JSONB,
    p_params JSONB,
    p_pinned TEXT,
    p_corpus TEXT,
    p_cost TEXT
) RETURNS VOID AS $$
DECLARE
    v_kb_id INTEGER;
    v_score_id INTEGER;
    v_search_id INTEGER;
    v_score_hash TEXT;
    v_search_hash TEXT;
    key_rec JSONB;
    key_idx INTEGER;
BEGIN
    -- 1. Upsert Keyboard
    INSERT INTO keyboards (name, author, version, notes, kb_type)
    VALUES (
        p_kb_meta->>'name', p_kb_meta->>'author', p_kb_meta->>'version',
        p_kb_meta->>'notes', p_kb_meta->>'type'
    )
    ON CONFLICT (unique_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
    RETURNING id INTO v_kb_id;

    -- 2. Populate Keys if new
    IF NOT EXISTS (SELECT 1 FROM keyboard_keys WHERE keyboard_id = v_kb_id LIMIT 1) THEN
        FOR key_idx IN 0 .. jsonb_array_length(p_kb_keys) - 1 LOOP
            key_rec := p_kb_keys->key_idx;
            INSERT INTO keyboard_keys (
                keyboard_id, idx, x, y, w, h, hand, finger, row_idx, col_idx, is_stretch,
                is_prime, is_med, is_low
            ) VALUES (
                v_kb_id, key_idx,
                (key_rec->>'x')::REAL, (key_rec->>'y')::REAL,
                COALESCE((key_rec->>'w')::REAL, 1.0), COALESCE((key_rec->>'h')::REAL, 1.0),
                (key_rec->>'hand')::INTEGER, (key_rec->>'finger')::INTEGER,
                (key_rec->>'row')::INTEGER, (key_rec->>'col')::INTEGER,
                COALESCE((key_rec->>'is_stretch')::BOOLEAN, false),
                (p_kb_slots->'prime_slots') @> to_jsonb(key_idx),
                (p_kb_slots->'med_slots') @> to_jsonb(key_idx),
                (p_kb_slots->'low_slots') @> to_jsonb(key_idx)
            );
        END LOOP;
    END IF;

    -- 3. Upsert Scoring Profile
    v_score_hash := digest(p_weights::TEXT, 'sha256');
    INSERT INTO scoring_profiles (
        penalty_sfb_base, penalty_sfb_lateral, penalty_sfb_lateral_weak,
        penalty_sfb_diagonal, penalty_sfb_long, penalty_sfb_bottom,
        penalty_sfr_weak_finger, penalty_sfr_bad_row, penalty_sfr_lat,
        penalty_scissor, penalty_lateral, penalty_ring_pinky,
        penalty_redirect, penalty_skip, penalty_hand_run,
        bonus_inward_roll, bonus_bigram_roll_in, bonus_bigram_roll_out,
        penalty_imbalance, max_hand_imbalance,
        weight_vertical_travel, weight_lateral_travel, weight_finger_effort,
        config_hash
    ) VALUES (
        (p_weights->>'penalty_sfb_base')::REAL, (p_weights->>'penalty_sfb_lateral')::REAL,
        (p_weights->>'penalty_sfb_lateral_weak')::REAL, (p_weights->>'penalty_sfb_diagonal')::REAL,
        (p_weights->>'penalty_sfb_long')::REAL, (p_weights->>'penalty_sfb_bottom')::REAL,
        (p_weights->>'penalty_sfr_weak_finger')::REAL, (p_weights->>'penalty_sfr_bad_row')::REAL,
        (p_weights->>'penalty_sfr_lat')::REAL, (p_weights->>'penalty_scissor')::REAL,
        (p_weights->>'penalty_lateral')::REAL, (p_weights->>'penalty_ring_pinky')::REAL,
        (p_weights->>'penalty_redirect')::REAL, (p_weights->>'penalty_skip')::REAL,
        (p_weights->>'penalty_hand_run')::REAL, (p_weights->>'bonus_inward_roll')::REAL,
        (p_weights->>'bonus_bigram_roll_in')::REAL, (p_weights->>'bonus_bigram_roll_out')::REAL,
        (p_weights->>'penalty_imbalance')::REAL, (p_weights->>'max_hand_imbalance')::REAL,
        (p_weights->>'weight_vertical_travel')::REAL, (p_weights->>'weight_lateral_travel')::REAL,
        (p_weights->>'weight_finger_effort')::REAL,
        v_score_hash
    )
    ON CONFLICT (config_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
    RETURNING id INTO v_score_id;

    -- 4. Upsert Search Config
    v_search_hash := digest(p_params::TEXT, 'sha256');
    INSERT INTO search_configs (
        search_epochs, search_steps, search_patience, search_patience_threshold,
        temp_min, temp_max, opt_limit_fast, opt_limit_slow, config_hash
    ) VALUES (
        (p_params->>'search_epochs')::INTEGER, (p_params->>'search_steps')::INTEGER,
        (p_params->>'search_patience')::INTEGER, (p_params->>'search_patience_threshold')::REAL,
        (p_params->>'temp_min')::REAL, (p_params->>'temp_max')::REAL,
        (p_params->>'opt_limit_fast')::INTEGER, (p_params->>'opt_limit_slow')::INTEGER,
        v_search_hash
    )
    ON CONFLICT (config_hash) DO UPDATE SET id = search_configs.id
    RETURNING id INTO v_search_id;

    -- 5. Link Job
    INSERT INTO jobs (id, keyboard_id, scoring_profile_id, search_config_id, pinned_keys, corpus_name, cost_matrix)
    VALUES (p_job_id, v_kb_id, v_score_id, v_search_id, p_pinned, p_corpus, p_cost)
    ON CONFLICT (id) DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- Procedure 2: Register Node Heartbeat
-- Handles CPU profile learning and node status update atomically
CREATE OR REPLACE FUNCTION register_node_heartbeat(
    p_node_id TEXT,
    p_cpu_model TEXT,
    p_arch TEXT,
    p_cores INTEGER,
    p_l2_cache INTEGER,
    p_ops_per_sec REAL
) RETURNS VOID AS $$
BEGIN
    -- 1. Upsert Hardware Profile
    INSERT INTO hardware_profiles (cpu_signature, architecture, l2_cache_kb, verified_ops_per_sec)
    VALUES (p_cpu_model, p_arch, p_l2_cache, p_ops_per_sec)
    ON CONFLICT (cpu_signature) DO UPDATE SET 
        updated_at = CURRENT_TIMESTAMP,
        verified_ops_per_sec = GREATEST(hardware_profiles.verified_ops_per_sec, EXCLUDED.verified_ops_per_sec);

    -- 2. Upsert Node Status
    INSERT INTO nodes (id, cpu_signature, cpu_cores, performance_rating, last_seen)
    VALUES (p_node_id, p_cpu_model, p_cores, p_ops_per_sec, CURRENT_TIMESTAMP)
    ON CONFLICT (id) DO UPDATE SET 
        last_seen = CURRENT_TIMESTAMP,
        performance_rating = EXCLUDED.performance_rating,
        cpu_signature = EXCLUDED.cpu_signature;
END;
$$ LANGUAGE plpgsql;

-- ===== 4. VIEWS (JSON Reconstruction) =====
CREATE OR REPLACE VIEW v_active_jobs AS
SELECT 
    j.id,
    -- Rebuild Geometry JSON object matching Rust struct
    jsonb_build_object(
        'meta', jsonb_build_object(
            'name', k.name, 'author', k.author, 'version', k.version, 
            'notes', k.notes, 'type', k.kb_type
        ),
        'geometry', jsonb_build_object(
            'home_row', 1, 
            'keys', (
                SELECT jsonb_agg(jsonb_build_object(
                    'x', kk.x, 'y', kk.y, 'w', kk.w, 'h', kk.h,
                    'row', kk.row_idx, 'col', kk.col_idx, 
                    'hand', kk.hand, 'finger', kk.finger,
                    'is_stretch', kk.is_stretch, 'id', 'k' || kk.idx
                ) ORDER BY kk.idx)
                FROM keyboard_keys kk WHERE kk.keyboard_id = k.id
            ),
            'prime_slots', (SELECT jsonb_agg(idx) FROM keyboard_keys WHERE keyboard_id = k.id AND is_prime),
            'med_slots', (SELECT jsonb_agg(idx) FROM keyboard_keys WHERE keyboard_id = k.id AND is_med),
            'low_slots', (SELECT jsonb_agg(idx) FROM keyboard_keys WHERE keyboard_id = k.id AND is_low)
        )
    ) as geometry_json,
    to_jsonb(sp) - 'id' - 'config_hash' - 'created_at' as weights_json,
    to_jsonb(sc) - 'id' - 'config_hash' as params_json,
    j.pinned_keys, j.corpus_name, j.cost_matrix, j.created_at
FROM jobs j
JOIN keyboards k ON j.keyboard_id = k.id
JOIN scoring_profiles sp ON j.scoring_profile_id = sp.id
JOIN search_configs sc ON j.search_config_id = sc.id
WHERE j.status = 'active';