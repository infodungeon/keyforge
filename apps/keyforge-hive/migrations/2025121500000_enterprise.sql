-- Audit Logs Table
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    action TEXT NOT NULL,
    actor_id UUID, -- Nullable for system actions
    target_resource TEXT,
    details JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_audit_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_logs(actor_id);

-- Job Enhancements
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS parent_job_id TEXT REFERENCES jobs(id);
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 0; -- Higher is better
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES users(id);

CREATE INDEX IF NOT EXISTS idx_jobs_priority ON jobs(priority DESC, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_jobs_parent ON jobs(parent_job_id);

-- Recursive CTE for Lineage (View)
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
