-- 1. Schema Versioning
CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO schema_version (version, description)
VALUES ('2.0.0', 'v2 schema: nodes, knowledge_nodes, and integer IDs');

-- 2. Nodes Table (The Registry)
CREATE TABLE IF NOT EXISTS nodes (
    id INTEGER PRIMARY KEY,
    ukey TEXT NOT NULL UNIQUE,
    node_type INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_nodes_ukey ON nodes(ukey);

-- 3. Knowledge Nodes Table
CREATE TABLE IF NOT EXISTS knowledge_nodes (
    node_id INTEGER PRIMARY KEY,
    content_node_id INTEGER NOT NULL,
    axis INTEGER NOT NULL,
    FOREIGN KEY(node_id) REFERENCES nodes(id),
    FOREIGN KEY(content_node_id) REFERENCES nodes(id)
) STRICT;

CREATE INDEX idx_knowledge_nodes_content ON knowledge_nodes(content_node_id);
