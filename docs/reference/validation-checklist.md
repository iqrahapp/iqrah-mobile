# Post-Implementation Validation Checklist

**Date**: 2025-01-25
**Status**: Verification Specification
**Purpose**: Validate integer-based node registry implementation

---

## 1. Schema Validation

### 1.1 Foreign Key Constraints

```bash
# Check for orphan edges
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) as orphan_edges FROM edges e
LEFT JOIN nodes n1 ON e.source_id = n1.id
LEFT JOIN nodes n2 ON e.target_id = n2.id
WHERE n1.id IS NULL OR n2.id IS NULL;
"
# Expected: 0
```

```bash
# Check for orphan knowledge nodes
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) as orphan_knowledge FROM knowledge_nodes kn
LEFT JOIN nodes n ON kn.node_id = n.id
WHERE n.id IS NULL;
"
# Expected: 0
```

```bash
# Check for orphan metadata
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) as orphan_metadata FROM node_metadata nm
LEFT JOIN nodes n ON nm.node_id = n.id
WHERE n.id IS NULL;
"
# Expected: 0
```

```bash
# Check for orphan goals
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) as orphan_goals FROM node_goals ng
LEFT JOIN nodes n ON ng.node_id = n.id
WHERE n.id IS NULL;
"
# Expected: 0
```

**Status**: [ ] PASS / [ ] FAIL

---

### 1.2 Node Registry Completeness

```bash
# Count nodes by type
sqlite3 ~/.local/share/iqrah/content.db "
SELECT node_type, COUNT(*) as count
FROM nodes
GROUP BY node_type;
"
# Expected for chapters 1-3:
#   0 (Verse): ~143
#   1 (Chapter): 3
#   2 (Word): 0 (unused)
#   3 (Knowledge): ~572 (143 verses × 4 axes)
#   4 (WordInstance): ~500+
```

```bash
# Total node count
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) as total_nodes FROM nodes;
"
# Expected: ~1200+
```

**Status**: [ ] PASS / [ ] FAIL

---

### 1.3 Knowledge Node Integrity

```bash
# All knowledge nodes have base nodes
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) FROM knowledge_nodes kn
LEFT JOIN nodes n ON kn.base_node_id = n.id
WHERE n.id IS NULL;
"
# Expected: 0
```

```bash
# Knowledge nodes are registered
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) FROM knowledge_nodes kn
JOIN nodes n ON kn.node_id = n.id
WHERE n.node_type = 3;
"
# Expected: Same as knowledge_nodes row count
```

```bash
# Axis values are valid (0-5)
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) FROM knowledge_nodes
WHERE axis < 0 OR axis > 5;
"
# Expected: 0
```

**Status**: [ ] PASS / [ ] FAIL

---

### 1.4 Edge Validation

```bash
# Edge type values are valid (0-1)
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) FROM edges
WHERE edge_type < 0 OR edge_type > 1;
"
# Expected: 0
```

```bash
# Edge weights are valid (0.0-1.0)
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) FROM edges
WHERE weight < 0.0 OR weight > 1.0;
"
# Expected: 0
```

```bash
# No self-loops
sqlite3 ~/.local/share/iqrah/content.db "
SELECT COUNT(*) FROM edges
WHERE source_id = target_id;
"
# Expected: 0 (unless intentional)
```

**Status**: [ ] PASS / [ ] FAIL

---

## 2. Rust Implementation Validation

### 2.1 Compilation Checks

```bash
cd rust

# Build with warnings as errors
RUSTFLAGS="-D warnings" cargo build --all-features

# Clippy with strict linting
cargo clippy --all-features --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check
```

**Status**: [ ] PASS / [ ] FAIL

---

### 2.2 Unit Tests

```bash
cd rust

# Test all features
cargo test --all-features

# Test specific modules
cargo test --package iqrah-storage node_registry
cargo test --package iqrah-core domain::models
```

**Expected**:
- All tests pass
- No warnings
- Coverage for NodeRegistry, enum conversions

**Status**: [ ] PASS / [ ] FAIL

---

### 2.3 NodeRegistry Functionality

```rust
// Test in Rust REPL or integration test
use iqrah_storage::NodeRegistry;

// Test: String → Int lookup
let id = registry.get_id("VERSE:1:1:memorization").await?;
assert!(id > 0);

// Test: Int → String lookup
let ukey = registry.get_ukey(id).await?;
assert_eq!(ukey, "VERSE:1:1:memorization");

// Test: Cache hit (second lookup should be faster)
let id2 = registry.get_id("VERSE:1:1:memorization").await?;
assert_eq!(id, id2);

// Test: Batch lookup
let ukeys = vec!["VERSE:1:1".to_string(), "VERSE:1:2".to_string()];
let ids = registry.get_ids(&ukeys).await?;
assert_eq!(ids.len(), 2);
```

**Status**: [ ] PASS / [ ] FAIL

---

## 3. Python Generator Validation

### 3.1 Build Process

```bash
cd research_and_dev/iqrah-knowledge-graph2

# Generate graph for chapters 1-3
python -m iqrah_cli build knowledge-graph --chapters 1-3 --format both

# Check output files
ls -lh output/
# Expected:
#   knowledge_graph_1_3.sql (~5-10 MB)
#   knowledge_graph_1_3.cbor (~500 KB - 1 MB)
```

**Status**: [ ] PASS / [ ] FAIL

---

### 3.2 SQL Import

```bash
# Import generated SQL
sqlite3 test_content.db < output/knowledge_graph_1_3.sql

# Verify import
sqlite3 test_content.db "SELECT COUNT(*) FROM nodes;"
sqlite3 test_content.db "SELECT COUNT(*) FROM edges;"
sqlite3 test_content.db "SELECT COUNT(*) FROM knowledge_nodes;"
```

**Expected**: No SQL errors, counts match generator output

**Status**: [ ] PASS / [ ] FAIL

---

### 3.3 Python Tests

```bash
cd research_and_dev/iqrah-knowledge-graph2

# Run test suite
python -m pytest tests/ -v

# Run validation checks
python -m iqrah_cli validate --graph-file output/knowledge_graph_1_3.cbor
```

**Status**: [ ] PASS / [ ] FAIL

---

## 4. Integration Testing

### 4.1 End-to-End Test

```bash
cd rust

# Initialize app with new schema
cargo run --bin iqrah-cli -- init --force

# Verify database created
ls -lh ~/.local/share/iqrah/
# Expected: content.db, user.db

# Schedule learning session
cargo run --bin iqrah-cli -- schedule --axis memorization --limit 5
# Expected: 5 nodes scheduled, no errors
```

**Status**: [ ] PASS / [ ] FAIL

---

### 4.2 Node Lookup Performance

```bash
# Benchmark node lookups
cargo bench --bench node_lookup

# Expected: <1ms per lookup with cache
```

**Performance Targets**:
- Cold lookup (no cache): <10ms
- Warm lookup (cached): <0.1ms
- Batch lookup (10 nodes): <5ms

**Status**: [ ] PASS / [ ] FAIL

---

### 4.3 Graph Traversal Performance

```bash
# Benchmark graph traversal
cargo bench --bench graph_traversal

# Expected: 10-100x faster than string-based
```

**Performance Targets**:
- BFS traversal (1000 nodes): <50ms
- DFS traversal (1000 nodes): <50ms
- Edge iteration (10k edges): <100ms

**Status**: [ ] PASS / [ ] FAIL

---

## 5. User Data Stability

### 5.1 Content Update Test

```bash
# 1. Create user state
cargo run --bin iqrah-cli -- init
cargo run --bin iqrah-cli -- schedule --axis memorization --limit 10
# Complete some items, generate user state

# 2. Replace content.db (simulate monthly update)
rm ~/.local/share/iqrah/content.db
python -m iqrah_cli build knowledge-graph --chapters 1-3
cp output/knowledge_graph_1_3.sql ~/.local/share/iqrah/content.db

# 3. Verify user state preserved
sqlite3 ~/.local/share/iqrah/user.db "
SELECT COUNT(*) FROM user_memory_states;
"
# Expected: Same count as before

# 4. Verify lookups still work
cargo run --bin iqrah-cli -- schedule --axis memorization --limit 5
# Expected: No errors, uses existing user state
```

**Status**: [ ] PASS / [ ] FAIL

---

## 6. Documentation Consistency

### 6.1 Task Document Updates

Check all production-ready tasks:

```bash
cd docs/todo/production-ready-tasks

# Verify no references to "virtual nodes"
grep -r "virtual" *.md
# Expected: No matches (or only in historical context)

# Verify INTEGER ID usage
grep -r "node_id INTEGER" *.md
# Expected: Multiple matches in code examples

# Verify NodeRegistry mentions
grep -r "NodeRegistry" *.md
# Expected: Mentioned in tasks 1.4, 2.1, 2.2
```

**Files to Check**:
- [ ] task-1.1-architecture-documentation.md
- [ ] task-1.4-repository-refactoring.md
- [ ] task-2.1-generate-full-knowledge-graph.md
- [ ] task-2.2-verify-knowledge-axis-end-to-end.md
- [ ] task-2.4-cross-axis-propagation-verification.md
- [ ] task-3.3-graph-update-mechanism.md
- [ ] AGENT_PROMPT_TEMPLATE.md

**Status**: [ ] PASS / [ ] FAIL

---

## 7. CI/CD Pipeline

### 7.1 Pre-Commit Checks

```bash
cd rust

# Run all pre-commit checks
RUSTFLAGS="-D warnings" cargo build --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo fmt --all -- --check
```

**Status**: [ ] PASS / [ ] FAIL

---

### 7.2 GitHub Actions

```bash
# Push to feature branch
git push origin feature/integer-node-registry

# Monitor CI
# https://github.com/your-org/iqrah-mobile/actions

# Expected: All checks pass (build, test, clippy, fmt)
```

**Status**: [ ] PASS / [ ] FAIL

---

## 8. Final Verification

### 8.1 Database Integrity

```bash
# Run PRAGMA checks
sqlite3 ~/.local/share/iqrah/content.db "PRAGMA foreign_key_check;"
# Expected: No output (all FKs valid)

sqlite3 ~/.local/share/iqrah/content.db "PRAGMA integrity_check;"
# Expected: ok

sqlite3 ~/.local/share/iqrah/content.db "PRAGMA quick_check;"
# Expected: ok
```

**Status**: [ ] PASS / [ ] FAIL

---

### 8.2 Enum Consistency

```bash
# Verify enum values match across languages

# Rust
grep -A10 "pub enum NodeType" rust/crates/iqrah-core/src/domain/models.rs

# Python
grep -A10 "class NodeType" research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py

# Compare manually - values must match exactly
```

**Status**: [ ] PASS / [ ] FAIL

---

## 9. Performance Benchmarks

### 9.1 Baseline Comparison

Run benchmarks and compare to baseline:

```bash
cargo bench --bench all > benchmark_results.txt

# Compare key metrics:
# - Node lookup: Should be 10-100x faster
# - Edge traversal: Should be 10-100x faster
# - Memory usage: Should be 50-70% smaller
```

**Expected Improvements**:
- [ ] Node lookup: >10x faster
- [ ] Graph traversal: >10x faster
- [ ] Memory per node: <8 bytes (was 20-50 bytes)

**Status**: [ ] PASS / [ ] FAIL

---

## 10. Rollback Plan

### If Critical Issues Found:

1. **Revert schema**:
   ```bash
   git checkout main -- rust/migrations_content/
   cargo run --bin iqrah-cli -- init --force
   ```

2. **Restore user data**:
   ```bash
   # user.db is unchanged, no restoration needed
   ```

3. **Revert code**:
   ```bash
   git revert <commit-hash>
   git push origin main
   ```

---

## Summary

**Total Checks**: 40+

**Critical Checks** (must pass):
- [ ] Foreign key integrity
- [ ] Rust compilation (no warnings)
- [ ] All unit tests pass
- [ ] Python graph generation succeeds
- [ ] SQL import succeeds
- [ ] End-to-end integration test passes
- [ ] User data stability test passes

**Performance Checks** (should improve):
- [ ] Node lookup performance >10x
- [ ] Graph traversal performance >10x

**Documentation Checks**:
- [ ] All task documents updated
- [ ] No "virtual node" references remain

---

## Sign-Off

**Implementation Lead**: ___________________ Date: __________

**QA Lead**: ___________________ Date: __________

**Project Architect**: ___________________ Date: __________

---

## References

- [Schema Design](../implementation/schema-design.md) - Expected database structure
- [Rust Implementation Guide](../implementation/rust-implementation-guide.md) - Code expectations
- [Python Generator Guide](../implementation/python-generator-guide.md) - Generation process
- [Enum Mappings](enum-mappings.md) - Canonical integer values
