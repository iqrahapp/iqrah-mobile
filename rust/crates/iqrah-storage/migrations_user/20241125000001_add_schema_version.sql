-- Add schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Record current schema version
INSERT OR IGNORE INTO schema_version (version, description)
VALUES ('1.0.0', 'Initial user schema with FSRS, propagation tracking, and scheduler v2 bandit');
