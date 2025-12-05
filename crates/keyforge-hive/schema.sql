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
    unique_hash TEXT GENERATED ALWAYS AS (digest(name || author || version, 'sha256')) STORED UNIQUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS keyboard_keys (
    id SERIAL PRIMARY KEY,
    keyboard_id INTEGER NOT NULL REFERENCES keyboards(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    x REAL NOT NULL, y REAL NOT NULL, w REAL DEFAULT 1.0, h REAL DEFAULT 1.0, r REAL DEFAULT 0.0,
    hand INTEGER NOT NULL, finger INTEGER NOT NULL, row_idx INTEGER NOT NULL, col_idx INTEGER NOT NULL,
    is_stretch BOOLEAN DEFAULT FALSE,
    is_prime BOOLEAN DEFAULT FALSE, is_med BOOLEAN DEFAULT FALSE, is_low BOOLEAN DEFAULT FALSE
);
CREATE INDEX IF NOT EXISTS idx_keys_keyboard ON keyboard_keys(keyboard_id);

CREATE TABLE IF NOT EXISTS scoring_profiles (
    id SERIAL PRIMARY KEY,
    penalty_sfb_base REAL NOT NULL, penalty_sfb_lateral REAL NOT NULL, penalty_sfb_lateral_weak REAL NOT NULL,
    penalty_sfb_diagonal REAL NOT NULL, penalty_sfb_long REAL NOT NULL, penalty_sfb_bottom REAL NOT NULL,
    penalty_sfr_weak_finger REAL NOT NULL, penalty_sfr_bad_row REAL NOT NULL, penalty_sfr_lat REAL NOT NULL,
    penalty_scissor REAL NOT NULL, penalty_lateral REAL NOT NULL, penalty_ring_pinky REAL NOT NULL,
    penalty_redirect REAL NOT NULL, penalty_skip REAL NOT NULL, penalty_hand_run REAL NOT NULL,
    bonus_inward_roll REAL NOT NULL, bonus_bigram_roll_in REAL NOT NULL, bonus_bigram_roll_out REAL NOT NULL,
    penalty_imbalance REAL NOT NULL, max_hand_imbalance REAL NOT NULL,
    weight_vertical_travel REAL NOT NULL, weight_lateral_travel REAL NOT NULL, weight_finger_effort REAL NOT NULL,
    
    finger_penalty_scale TEXT NOT NULL,
    comfortable_scissors TEXT NOT NULL,
    
    loader_trigram_limit INTEGER DEFAULT 3000,
    corpus_scale REAL DEFAULT 200000000.0,
    default_cost_ms REAL DEFAULT 120.0,
    threshold_sfb_long_row_diff INTEGER DEFAULT 2,
    threshold_scissor_row_diff INTEGER DEFAULT 2,
    penalty_sfb_outward_adder REAL DEFAULT 10.0,
    weight_weak_finger_sfb REAL DEFAULT 2.7,
    penalty_high_in_med REAL DEFAULT 12.0,
    penalty_high_in_low REAL DEFAULT 20.0,
    penalty_med_in_prime REAL DEFAULT 2.0,
    penalty_med_in_low REAL DEFAULT 2.0,
    penalty_low_in_prime REAL DEFAULT 15.0,
    penalty_low_in_med REAL DEFAULT 2.0,

    config_hash TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS search_configs (
    id SERIAL PRIMARY KEY,
    search_epochs INTEGER NOT NULL, search_steps INTEGER NOT NULL,
    search_patience INTEGER NOT NULL, search_patience_threshold REAL NOT NULL,
    temp_min REAL NOT NULL, temp_max REAL NOT NULL,
    opt_limit_fast INTEGER NOT NULL, opt_limit_slow INTEGER NOT NULL,
    config_hash TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    keyboard_id INTEGER NOT NULL REFERENCES keyboards(id),
    scoring_profile_id INTEGER NOT NULL REFERENCES scoring_profiles(id),
    search_config_id INTEGER NOT NULL REFERENCES search_configs(id),
    pinned_keys TEXT NOT NULL, corpus_name TEXT NOT NULL, cost_matrix TEXT NOT NULL,
    status TEXT DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS hardware_profiles (
    cpu_signature TEXT PRIMARY KEY, architecture TEXT NOT NULL,
    l1_cache_kb INTEGER, l2_cache_kb INTEGER, l3_cache_kb INTEGER,
    verified_ops_per_sec REAL, updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    cpu_signature TEXT REFERENCES hardware_profiles(cpu_signature),
    cpu_cores INTEGER, performance_rating REAL, last_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS results (
    id BIGSERIAL PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id),
    layout TEXT NOT NULL, score DOUBLE PRECISION NOT NULL,
    node_id TEXT NOT NULL REFERENCES nodes(id), created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_results_job_score ON results(job_id, score ASC);

CREATE TABLE IF NOT EXISTS submissions (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL, layout_str TEXT NOT NULL, author TEXT,
    status TEXT DEFAULT 'pending', submitted_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ===== VIEWS =====

CREATE OR REPLACE VIEW v_active_jobs AS
SELECT 
    j.id,
    j.pinned_keys,
    j.corpus_name,
    j.cost_matrix,
    j.created_at,
    jsonb_build_object(
        'meta', jsonb_build_object(
            'name', k.name, 'author', k.author, 'version', k.version, 'notes', k.notes, 'type', k.kb_type
        ),
        'geometry', jsonb_build_object(
            'keys', (
                SELECT jsonb_agg(jsonb_build_object(
                    'id', 'k' || kk.idx, 
                    'hand', kk.hand, 'finger', kk.finger, 'row', kk.row_idx, 'col', kk.col_idx,
                    'x', kk.x, 'y', kk.y, 'w', kk.w, 'h', kk.h, 'is_stretch', kk.is_stretch
                ) ORDER BY kk.idx)
                FROM keyboard_keys kk WHERE kk.keyboard_id = k.id
            ),
            'prime_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_prime),
            'med_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_med),
            'low_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_low),
            'home_row', 1
        )
    ) AS geometry_json,
    (to_jsonb(sp) - 'id' - 'config_hash' - 'created_at') AS weights_json,
    (to_jsonb(sc) - 'id' - 'config_hash') AS params_json
FROM jobs j
JOIN keyboards k ON j.keyboard_id = k.id
JOIN scoring_profiles sp ON j.scoring_profile_id = sp.id
JOIN search_configs sc ON j.search_config_id = sc.id
WHERE j.status = 'active';

-- ===== PROCEDURES =====

-- Safely drop old versions of the function
DROP FUNCTION IF EXISTS register_node_heartbeat(text, text, text, integer, integer, double precision);
DROP FUNCTION IF EXISTS register_node_heartbeat(text, text, text, integer, integer, real);

CREATE OR REPLACE FUNCTION register_full_job(
    p_job_id TEXT, p_kb_meta JSONB, p_kb_keys JSONB, p_kb_slots JSONB,
    p_weights JSONB, p_params JSONB,
    p_pinned TEXT, p_corpus TEXT, p_cost TEXT
) RETURNS VOID AS $$
DECLARE
    v_kb_id INTEGER; v_score_id INTEGER; v_search_id INTEGER;
    v_score_hash TEXT; v_search_hash TEXT; key_rec JSONB; key_idx INTEGER;
BEGIN
    INSERT INTO keyboards (name, author, version, notes, kb_type)
    VALUES (p_kb_meta->>'name', p_kb_meta->>'author', p_kb_meta->>'version', p_kb_meta->>'notes', p_kb_meta->>'type')
    ON CONFLICT (unique_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
    RETURNING id INTO v_kb_id;

    IF NOT EXISTS (SELECT 1 FROM keyboard_keys WHERE keyboard_id = v_kb_id LIMIT 1) THEN
        FOR key_idx IN 0 .. jsonb_array_length(p_kb_keys) - 1 LOOP
            key_rec := p_kb_keys->key_idx;
            INSERT INTO keyboard_keys (
                keyboard_id, idx, x, y, w, h, hand, finger, row_idx, col_idx, is_stretch, is_prime, is_med, is_low
            ) VALUES (
                v_kb_id, key_idx, (key_rec->>'x')::REAL, (key_rec->>'y')::REAL,
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
        finger_penalty_scale, comfortable_scissors,
        
        loader_trigram_limit, corpus_scale, default_cost_ms,
        threshold_sfb_long_row_diff, threshold_scissor_row_diff,
        penalty_sfb_outward_adder, weight_weak_finger_sfb,
        penalty_high_in_med, penalty_high_in_low,
        penalty_med_in_prime, penalty_med_in_low,
        penalty_low_in_prime, penalty_low_in_med,
        
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
        (p_weights->>'finger_penalty_scale'), (p_weights->>'comfortable_scissors'),
        
        COALESCE((p_weights->>'loader_trigram_limit')::INTEGER, 3000),
        COALESCE((p_weights->>'corpus_scale')::REAL, 200000000.0),
        COALESCE((p_weights->>'default_cost_ms')::REAL, 120.0),
        COALESCE((p_weights->>'threshold_sfb_long_row_diff')::INTEGER, 2),
        COALESCE((p_weights->>'threshold_scissor_row_diff')::INTEGER, 2),
        COALESCE((p_weights->>'penalty_sfb_outward_adder')::REAL, 10.0),
        COALESCE((p_weights->>'weight_weak_finger_sfb')::REAL, 2.7),
        COALESCE((p_weights->>'penalty_high_in_med')::REAL, 12.0),
        COALESCE((p_weights->>'penalty_high_in_low')::REAL, 20.0),
        COALESCE((p_weights->>'penalty_med_in_prime')::REAL, 2.0),
        COALESCE((p_weights->>'penalty_med_in_low')::REAL, 2.0),
        COALESCE((p_weights->>'penalty_low_in_prime')::REAL, 15.0),
        COALESCE((p_weights->>'penalty_low_in_med')::REAL, 2.0),

        v_score_hash
    )
    ON CONFLICT (config_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
    RETURNING id INTO v_score_id;

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

    INSERT INTO jobs (id, keyboard_id, scoring_profile_id, search_config_id, pinned_keys, corpus_name, cost_matrix)
    VALUES (p_job_id, v_kb_id, v_score_id, v_search_id, p_pinned, p_corpus, p_cost)
    ON CONFLICT (id) DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- UPDATED FUNCTION: Uses REAL to match Rust f32
CREATE OR REPLACE FUNCTION register_node_heartbeat(
    p_node_id TEXT, p_cpu_model TEXT, p_arch TEXT, p_cores INTEGER, p_l2_cache INTEGER, p_ops_per_sec REAL
) RETURNS VOID AS $$
BEGIN
    INSERT INTO hardware_profiles (
        cpu_signature, architecture, l2_cache_kb, verified_ops_per_sec, updated_at
    ) VALUES (
        p_cpu_model, p_arch, p_l2_cache, p_ops_per_sec, CURRENT_TIMESTAMP
    )
    ON CONFLICT (cpu_signature) DO UPDATE SET
        verified_ops_per_sec = GREATEST(hardware_profiles.verified_ops_per_sec, EXCLUDED.verified_ops_per_sec),
        l2_cache_kb = COALESCE(EXCLUDED.l2_cache_kb, hardware_profiles.l2_cache_kb),
        updated_at = CURRENT_TIMESTAMP;

    INSERT INTO nodes (
        id, cpu_signature, cpu_cores, performance_rating, last_seen
    ) VALUES (
        p_node_id, p_cpu_model, p_cores, p_ops_per_sec, CURRENT_TIMESTAMP
    )
    ON CONFLICT (id) DO UPDATE SET
        last_seen = CURRENT_TIMESTAMP,
        performance_rating = EXCLUDED.performance_rating,
        cpu_cores = EXCLUDED.cpu_cores;
END;
$$ LANGUAGE plpgsql;