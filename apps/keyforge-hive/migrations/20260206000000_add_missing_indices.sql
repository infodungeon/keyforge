-- Optimize common queries for submissions and nodes
CREATE INDEX IF NOT EXISTS idx_submissions_status ON submissions(status);
CREATE INDEX IF NOT EXISTS idx_nodes_cpu_signature ON nodes(cpu_signature);
