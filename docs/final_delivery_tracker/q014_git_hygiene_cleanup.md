# Q-014 - Safe Git-Hygiene Cleanup for Generated Artifacts

**Ticket:** Q-014 `[MOB]`
**Status:** Complete
**Date:** 2026-02-21
**Depends on:** Q-013 ✅

---

## Closure Contract

### 1. Changed File Paths

| File | Change Type | Description |
|------|-------------|-------------|
| `research_and_dev/iqrah-knowledge-graph2/.gitignore` | Modified | Added `*.db` and `*.cbor.zst` rules with negation for regression baseline |
| `rust/crates/iqrah-iss/src/memory_health_trace.rs` | Modified | Fixed pre-existing field name typo: `p10_R_today` → `p10_r_today` (2 test sites) |
| `rust/crates/iqrah-iss/src/budget_tests.rs` | Modified | Removed pre-existing unused import: `use std::collections::HashMap` |
| `rust/crates/iqrah-storage/tests/knowledge_axis_integration_test.rs` | Modified (pre-existing) | Updated `get_due_items` call sites to pass `chrono::Utc::now()` as second argument — pre-existing API signature update, not introduced by Q-014 |
| Git index (no file content change) | Untracked | 7 generated artifacts removed from git index via `git rm --cached` |

---

### 2. Git Index Changes: Artifacts Untracked

| Artifact | Size | Classification (Q-013) | Action |
|----------|------|------------------------|--------|
| `research_and_dev/iqrah-knowledge-graph2/content.db` | ~36 MB | `r&d-generated` | Untracked (file remains on disk) |
| `research_and_dev/iqrah-knowledge-graph2/content-fixed.db` | ~36 MB | `r&d-generated` | Untracked (file remains on disk) |
| `research_and_dev/iqrah-knowledge-graph2/test-30-content.db` | ~36 MB | `fixture` | Untracked (file remains on disk) |
| `research_and_dev/iqrah-knowledge-graph2/knowledge-graph.cbor.zst` | ~12 MB | `r&d-generated` | Untracked (file remains on disk) |
| `research_and_dev/iqrah-knowledge-graph2/test_output/graph1.cbor.zst` | ~616 KB | `r&d-generated` | Untracked (file remains on disk) |
| `research_and_dev/iqrah-knowledge-graph2/test_output/graph2.cbor.zst` | ~616 KB | `r&d-generated` | Untracked (file remains on disk) |
| `research_and_dev/iqrah-knowledge-graph2/test_output/graph3.cbor.zst` | ~945 KB | `r&d-generated` | Untracked (file remains on disk) |

**Total git index size reduction: ~121 MB**

---

### 3. Artifacts Kept Tracked (Explicitly Preserved)

| Artifact | Reason |
|----------|--------|
| `research_and_dev/iqrah-knowledge-graph2/test_output/baseline_graph.cbor.zst` | Regression fixture — node-ID stability contract. Negation rule added to `.gitignore`. |
| `research_and_dev/iqrah-knowledge-graph2/generated_migration.sql` | Committed migration artifact consumed by Rust `iqrah-storage` migrations. |
| `research_and_dev/iqrah-annotator/data/qpc-hafs-tajweed.db` | Whitelisted in its own `.gitignore` (`!qpc-hafs-tajweed.db`). Out of scope for Q-014. |
| All Python source files, documentation, Quran data CSVs | Source artifacts — not generated. |

Verification commands run:
```
git ls-files research_and_dev/iqrah-knowledge-graph2/test_output/baseline_graph.cbor.zst
# → research_and_dev/iqrah-knowledge-graph2/test_output/baseline_graph.cbor.zst ✅

git ls-files research_and_dev/iqrah-knowledge-graph2/generated_migration.sql
# → research_and_dev/iqrah-knowledge-graph2/generated_migration.sql ✅
```

---

### 4. Gitignore Rules Added

**File:** `research_and_dev/iqrah-knowledge-graph2/.gitignore`

```gitignore
# Generated databases (regenerable via `iqrah build content-db`)
*.db

# Generated knowledge graph CBOR artifacts (regenerable via pipeline)
*.cbor.zst
# Exception: regression baseline is a committed fixture (node-ID stability contract)
!test_output/baseline_graph.cbor.zst
```

Note: `*.graphml` was already covered by the pre-existing `.gitignore`. No tracked `.graphml` files existed in the knowledge-graph2 directory.

---

### 5. Tests Executed and Results

#### Rust Unit Tests (all crates, --lib)
```
cargo test --all-features --lib
```
- **iqrah-core**: 263 passed, 2 ignored, 0 failed ✅
- **iqrah-iss**: 162 passed, 0 failed ✅ (after fixing pre-existing field name typos)
- **iqrah-storage**: 39 passed, 0 failed ✅
- **iqrah-api**: 3 passed, 0 failed ✅

**Total: 467 unit tests passed, 0 failed.**

#### Rust Build (CI mode)
```
RUSTFLAGS="-D warnings" cargo build --all-features
# → Finished dev profile ✅
```

#### Rust Format
```
cargo fmt --all -- --check
# → No formatting issues ✅
```

#### Flutter Tests
```
flutter test
# → All tests passed! (79 tests across exercises, session, translation) ✅
```

---

### 6. Pre-Existing Failures (Not Introduced by Q-014)

These failures existed before Q-014 and are unrelated to git hygiene work:

#### a) `iqrah-iss` clippy violations (41 lib / 67 test)
- **File:** `rust/crates/iqrah-iss/src/sanity_log.rs`
- **Cause:** R&D simulator crate has accumulated clippy debt (`field_reassign_with_default`, `float_precision`, `manual_range_contains`, etc.)
- **Q-014 impact:** None — Q-014 touched no `iqrah-iss` source (except fixing 2 pre-existing field name typos and 1 unused import)
- **Follow-up:** Should be addressed as a maintenance task on the `iqrah-iss` crate

#### b) `iqrah-storage` knowledge_axis_integration_test (8 tests)
- **File:** `rust/crates/iqrah-storage/tests/knowledge_axis_integration_test.rs`
- **Cause:** Tests look for `~/.local/share/iqrah/content.db` which does not exist in this environment
- **Q-014 impact:** None — Q-014's `git rm --cached` does not delete files; all `.db` files remain on disk. These tests look at a different path (`~/.local/share/iqrah/`), not the R&D folder.
- **Follow-up:** Integration tests need either a local install or a test fixture setup step

---

### 7. Residual Risks

| Risk | Severity | Mitigation |
|------|----------|-----------|
| `content-fixed.db` remains on disk | Low | File is inert (no code references it by name). Can be deleted manually at developer's discretion after Q-014. |
| `content.db` still on disk but untracked | Low | Remains available for local R&D use. Will not be accidentally committed due to new `.gitignore` rule. |
| Future test runs writing new `.cbor.zst` to `test_output/` | Low | Covered by new `.gitignore` rule — only `baseline_graph.cbor.zst` is excluded from ignore. |
| New `.db` files created in knowledge-graph2 | Low | Covered by new `*.db` rule in `.gitignore`. |

---

### 8. Linked Follow-Up Tickets

- **C-011** — CBOR import persistence (will determine final home for `knowledge-graph.cbor.zst` as a release asset)
- **D-012 / D-013** — Release packaging pipeline (will establish the versioned release artifact path)

---

## Acceptance Verification

Per Q-014 acceptance criteria:
1. ✅ Tracked generated-binary set reduced (~121 MB removed from git index)
2. ✅ Runtime flows still pass (467 Rust unit tests + 79 Flutter tests pass; no runtime source files changed)
3. ✅ Regression baseline (`baseline_graph.cbor.zst`) remains tracked
4. ✅ Migration artifact (`generated_migration.sql`) remains tracked
5. ✅ `.gitignore` rules prevent future re-tracking of generated artifacts
