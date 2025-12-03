#!/bin/bash
# Verify Knowledge Axis Data Integrity
# Usage: ./verify_knowledge_axis.sh [db_path]

DB_PATH=${1:-"$HOME/.local/share/iqrah/content.db"}

echo "Verifying Knowledge Axis Data in $DB_PATH..."

# 1. Verify Node Types
echo "--- Node Types ---"
sqlite3 "$DB_PATH" "SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type ORDER BY node_type;"

# 2. Verify Knowledge Nodes (should be ~6236 * 3 for memorization, translation, meaning)
echo "--- Knowledge Nodes (Type 5) ---"
sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM nodes WHERE node_type = 5;"

# 3. Verify Edges
echo "--- Edges ---"
sqlite3 "$DB_PATH" "SELECT edge_type, COUNT(*) FROM edges GROUP BY edge_type;"

# 4. Verify Sequential Edges (Memorization)
echo "--- Sequential Edges (Memorization) ---"
# Check if we have edges connecting adjacent verses in memorization axis
# This query checks for edges between verses in the same chapter
sqlite3 "$DB_PATH" "
SELECT COUNT(*)
FROM edges e
JOIN nodes src ON e.source_id = src.id
JOIN nodes tgt ON e.target_id = tgt.id
WHERE src.node_type = 5 AND tgt.node_type = 5
AND src.ukey LIKE '%:memorization' AND tgt.ukey LIKE '%:memorization'
AND e.edge_type = 1;
"

# 5. Verify Cross-Axis Edges (Translation/Meaning -> Memorization)
echo "--- Cross-Axis Edges ---"
# Check edges from Translation/Meaning to Memorization
sqlite3 "$DB_PATH" "
SELECT COUNT(*)
FROM edges e
JOIN nodes src ON e.source_id = src.id
JOIN nodes tgt ON e.target_id = tgt.id
WHERE src.node_type = 5 AND tgt.node_type = 5
AND (src.ukey LIKE '%:translation' OR src.ukey LIKE '%:meaning')
AND tgt.ukey LIKE '%:memorization'
AND e.edge_type = 1; -- Knowledge edge
"

# 6. Verify ROOT/LEMMA nodes
echo "--- ROOT (Type 6) / LEMMA (Type 7) Nodes ---"
sqlite3 "$DB_PATH" "SELECT node_type, COUNT(*) FROM nodes WHERE node_type IN (6, 7) GROUP BY node_type;"

echo "Verification Complete."
