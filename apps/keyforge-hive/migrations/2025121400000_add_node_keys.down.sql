DROP FUNCTION IF EXISTS register_node_heartbeat(TEXT, TEXT, TEXT, INTEGER, INTEGER, REAL, TEXT);

-- Restore previous register_node_heartbeat
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

ALTER TABLE nodes DROP COLUMN IF EXISTS public_key;
