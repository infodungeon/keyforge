DROP VIEW IF EXISTS v_job_lineage;

CREATE OR REPLACE VIEW v_job_lineage AS
WITH RECURSIVE lineage AS (
    SELECT id, parent_job_id, 0 as depth, ARRAY[id] as path
    FROM jobs
    WHERE parent_job_id IS NULL
    UNION ALL
    SELECT j.id, j.parent_job_id, l.depth + 1, l.path || j.id
    FROM jobs j
    JOIN lineage l ON j.parent_job_id = l.id
    WHERE l.depth < 50
)
SELECT * FROM lineage;
