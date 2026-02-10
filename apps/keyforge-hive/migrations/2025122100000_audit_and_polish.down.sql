-- Restore v_job_lineage from 2025121500000_enterprise.sql
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

ALTER TABLE audit_logs DROP COLUMN IF EXISTS user_agent;
ALTER TABLE audit_logs DROP COLUMN IF EXISTS request_id;
ALTER TABLE audit_logs DROP COLUMN IF EXISTS status_code;
