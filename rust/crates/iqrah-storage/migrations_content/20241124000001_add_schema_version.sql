-- Add schema_version table to content.db
DROP TABLE IF EXISTS schema_version;

CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Record current schema version (v2 purist schema with scheduler v2 and knowledge graph chapters 1-3)
INSERT INTO schema_version (version, description)
VALUES ('2.0.0', 'v2 purist schema with scheduler v2 and knowledge graph chapters 1-3');
