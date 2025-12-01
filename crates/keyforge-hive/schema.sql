-- Stores unique search configurations
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,          -- The SHA256 Hash of the config
    geometry_json TEXT NOT NULL,  -- Physical layout
    weights_json TEXT NOT NULL,   -- Scoring weights
    pinned_keys TEXT NOT NULL,    -- Constraints
    corpus_name TEXT NOT NULL,    -- Data source ID
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Stores the best results found for specific jobs
CREATE TABLE IF NOT EXISTS results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL,
    layout TEXT NOT NULL,
    score REAL NOT NULL,
    node_id TEXT NOT NULL,        -- Who found it?
    submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(job_id) REFERENCES jobs(id)
);

-- Index for fast leaderboard lookups
CREATE INDEX IF NOT EXISTS idx_results_score ON results(job_id, score ASC);