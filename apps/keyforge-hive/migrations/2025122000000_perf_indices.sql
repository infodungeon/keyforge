-- Add covering index for results to speed up validation and duplication checks
CREATE INDEX IF NOT EXISTS idx_results_covering ON results (job_id, node_id, score);
