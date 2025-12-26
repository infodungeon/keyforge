-- 1. Enrich Audit Logs
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS status_code INTEGER;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS request_id UUID;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS user_agent TEXT;

-- 2. Polish v_job_lineage with depth limit and cycle detection
CREATE OR REPLACE VIEW v_job_lineage AS
WITH RECURSIVE lineage AS (
    -- Anchor member
    SELECT id, parent_job_id, 0 as depth, ARRAY[id] as path
    FROM jobs
    WHERE parent_job_id IS NULL
    
    UNION ALL
    
    -- Recursive member
    SELECT j.id, j.parent_job_id, l.depth + 1, l.path || j.id
    FROM jobs j
    JOIN lineage l ON j.parent_job_id = l.id
    WHERE l.depth < 20 -- moderate #44: prevent runaway recursion
    AND NOT j.id = ANY(l.path) -- Cycle detection
)
SELECT * FROM lineage;
