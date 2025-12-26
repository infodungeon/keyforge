ALTER TABLE nodes ADD COLUMN IF NOT EXISTS public_key TEXT;

CREATE OR REPLACE FUNCTION register_node_heartbeat(
        p_node_id TEXT,
        p_cpu_model TEXT,
        p_arch TEXT,
        p_cores INTEGER,
        p_l2_cache INTEGER,
        p_ops_per_sec REAL,
        p_public_key TEXT
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
        last_seen,
        public_key
    )
VALUES (
        p_node_id,
        p_cpu_model,
        p_cores,
        p_ops_per_sec,
        CURRENT_TIMESTAMP,
        p_public_key
    ) ON CONFLICT (id) DO
UPDATE
SET last_seen = CURRENT_TIMESTAMP,
    performance_rating = EXCLUDED.performance_rating,
    cpu_cores = EXCLUDED.cpu_cores,
    -- SECURITY FIX: Trust On First Use (TOFU)
    -- Keep existing key if present. Only set if NULL.
    public_key = COALESCE(nodes.public_key, EXCLUDED.public_key);
END;
$$ LANGUAGE plpgsql;
