-- Aggregated Rollback Logic for KeyForge Migrations (DATA-003 Compliance)
-- Created for review of Issue #161

-- [REVERT] 2025121200000_init.sql
DROP VIEW IF EXISTS v_active_jobs;
DROP TABLE IF EXISTS results CASCADE;
DROP TABLE IF EXISTS nodes CASCADE;
DROP TABLE IF EXISTS hardware_profiles CASCADE;
DROP TABLE IF EXISTS jobs CASCADE;
DROP TABLE IF EXISTS search_configs CASCADE;
DROP TABLE IF EXISTS scoring_profiles CASCADE;
DROP TABLE IF EXISTS keyboard_keys CASCADE;
DROP TABLE IF EXISTS keyboards CASCADE;
DROP TABLE IF EXISTS submissions CASCADE;
DROP FUNCTION IF EXISTS register_node_heartbeat;

-- [REVERT] 2025121300000_auth.sql
DROP TABLE IF EXISTS api_keys CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- [REVERT] 2025121400000_add_node_keys.sql
ALTER TABLE nodes DROP COLUMN IF EXISTS public_key;
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

-- [REVERT] 2025121500000_enterprise.sql
DROP VIEW IF EXISTS v_job_lineage;
ALTER TABLE jobs DROP COLUMN IF EXISTS owner_id;
ALTER TABLE jobs DROP COLUMN IF EXISTS priority;
ALTER TABLE jobs DROP COLUMN IF EXISTS parent_job_id;
DROP TABLE IF EXISTS audit_logs CASCADE;

-- [REVERT] 2025121800000_optimize_jobs.sql
DROP INDEX IF EXISTS idx_jobs_fetch;

-- [REVERT] 2025122000000_perf_indices.sql
DROP INDEX IF EXISTS idx_results_covering;

-- [REVERT] 2025122100000_audit_and_polish.sql
ALTER TABLE audit_logs DROP COLUMN IF EXISTS user_agent;
ALTER TABLE audit_logs DROP COLUMN IF EXISTS request_id;
ALTER TABLE audit_logs DROP COLUMN IF EXISTS status_code;
CREATE OR REPLACE VIEW v_job_lineage AS
WITH RECURSIVE lineage AS (
    SELECT id, parent_job_id, 0 as depth, ARRAY[id] as path
    FROM jobs
    WHERE parent_job_id IS NULL
    UNION ALL
    SELECT j.id, j.parent_job_id, l.depth + 1, l.path || j.id
    FROM jobs j
    JOIN lineage l ON j.parent_job_id = l.id
)
SELECT * FROM lineage;

-- [REVERT] 2025122200000_fix_recursive_view.sql
CREATE OR REPLACE VIEW v_job_lineage AS
WITH RECURSIVE lineage AS (
    SELECT id, parent_job_id, 0 as depth, ARRAY[id] as path
    FROM jobs
    WHERE parent_job_id IS NULL
    UNION ALL
    SELECT j.id, j.parent_job_id, l.depth + 1, l.path || j.id
    FROM jobs j
    JOIN lineage l ON j.parent_job_id = l.id
    WHERE l.depth < 20
    AND NOT j.id = ANY(l.path)
)
SELECT * FROM lineage;

-- [REVERT] 2025122300000_quotas.sql
DROP INDEX IF EXISTS idx_jobs_owner_created;
ALTER TABLE users DROP COLUMN IF EXISTS max_daily_jobs;
ALTER TABLE users DROP COLUMN IF EXISTS max_active_jobs;

-- [REVERT] 20251224000000_v1_foundation.sql
DROP INDEX IF EXISTS idx_users_quotas;
DROP INDEX IF EXISTS idx_jobs_public;
ALTER TABLE jobs DROP COLUMN IF EXISTS is_public;
ALTER TABLE api_keys DROP COLUMN IF EXISTS scopes;
ALTER TABLE users DROP COLUMN IF EXISTS quota_limits;

-- [REVERT] 20251225000000_fix_zombies.sql
DROP INDEX IF EXISTS idx_jobs_stale;
ALTER TABLE jobs DROP COLUMN IF EXISTS retry_count;
ALTER TABLE jobs DROP COLUMN IF EXISTS started_at;

-- [REVERT] 20251225000001_add_jobs_node_id.sql
DROP INDEX IF EXISTS idx_jobs_node_id;
ALTER TABLE jobs DROP COLUMN IF EXISTS node_id;

-- [REVERT] 20251226000000_user_scoring.sql
DROP INDEX IF EXISTS idx_jobs_reaper;
ALTER TABLE jobs DROP COLUMN IF EXISTS is_pinned;
ALTER TABLE users DROP COLUMN IF EXISTS contribution_count;
DROP INDEX IF EXISTS idx_results_user;
ALTER TABLE results DROP COLUMN IF EXISTS user_id;

-- [REVERT] 20260122000000_add_keyboard_home_row.sql
ALTER TABLE keyboards DROP COLUMN IF EXISTS home_row;

-- [REVERT] 20260122000001_add_result_raw_score.sql
ALTER TABLE results DROP COLUMN IF EXISTS raw_score;
