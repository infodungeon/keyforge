-- Add node tracking to jobs for worker assignment and zombie reaping
--
-- NOTE: results already track node_id; jobs.node_id tracks the currently assigned worker.

ALTER TABLE jobs
  ADD COLUMN IF NOT EXISTS node_id TEXT REFERENCES nodes(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_jobs_node_id ON jobs(node_id);
