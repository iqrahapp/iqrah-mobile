# Task 1.5: Integrate Node ID Stability Validation into Build Pipeline

## Metadata
- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 1 day
- **Dependencies:** None (Python-side only, but Task 1.1 architecture doc helpful)
- **Agent Type:** Implementation (Python)
- **Parallelizable:** Yes (with tasks 1.1, 1.2, 1.3)

## Goal

Integrate the existing node ID stability validation script into the Python knowledge graph build pipeline to prevent accidental node ID changes that would break user progress.

## Context

**The Problem:**
User progress (`user_memory_states` table) is keyed by `node_id`. If a node ID changes between graph versions, the user loses all progress for that node.

**Example Breaking Change:**
```
Version 1: node_id = "VERSE:1:1"
Version 2: node_id = "CHAPTER:1:VERSE:1" (NEW FORMAT)
Result: User progress for "VERSE:1:1" orphaned
```

**Why This Can Happen:**
- Refactoring graph builder code
- Changing ID generation logic
- Adding new node types
- Typos or bugs in ID construction

**Solution:**
Before generating a new graph, validate that all previous node IDs still exist. New nodes are fine, removed/changed nodes are breaking changes that require explicit migration.

## Current State

**Python R&D Project:**
- **Location:** `research_and_dev/iqrah-knowledge-graph2/`
- **Graph Library:** NetworkX (`import networkx as nx`)
- **Graph Format:** CBOR binary with Zstandard compression (`.cbor.zst`)
- **Builder:** `src/iqrah/graph/builder.py`, `knowledge_builder.py`
- **Export Module:** `src/iqrah/export/cbor_export.py`
- **CLI:** `src/iqrah_cli/commands/build.py`

**Validation Script (ALREADY EXISTS):**
- **File:** `research_and_dev/iqrah-knowledge-graph2/validate_stability.py` (196 lines)
- **Status:** Fully functional standalone script
- **Features:**
  - Loads CBOR graph files
  - Extracts node IDs from both dict and list-based CBOR formats
  - Compares old vs new graph node IDs
  - Reports missing IDs with sample output
  - Returns exit code 0 (pass) or 1 (fail)
  - CLI with argparse (proper usage, help text)

**Current Workflow (Manual):**
```bash
cd research_and_dev/iqrah-knowledge-graph2

# Build graph
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology ../data/morphology.csv \
    --preset full \
    -o output/knowledge_graph.cbor.zst

# Manual validation (optional, not enforced)
python validate_stability.py old_graph.cbor.zst output/knowledge_graph.cbor.zst
```

**The Problem:**
Validation is manual and optional. Nothing prevents developers from accidentally breaking node IDs.

## Target State

### Automated Validation in Build Pipeline

**Workflow:**
1. Build new graph
2. Export to CBOR
3. **Automatically validate** against previous version (if exists)
4. **Exit with error** if validation fails
5. Save current graph as baseline for next build

**Integration Points:**
- CLI command: Add `--validate` flag (default: true)
- Build script: Call `validate_stability.py` automatically
- CI/CD: Validation failure blocks merge

## Implementation Steps

### Step 1: Test Existing Validation Script (30 min)

**Verify the script works correctly:**

```bash
cd research_and_dev/iqrah-knowledge-graph2

# If you have sample graphs, test it:
python validate_stability.py \
    /path/to/old_graph.cbor.zst \
    /path/to/new_graph.cbor.zst

# Check exit code
echo $?  # Should be 0 (pass) or 1 (fail)
```

**Expected output (on success):**
```
Loading old graph: old_graph.cbor.zst
Loading new graph: new_graph.cbor.zst

✅ ID stability validated
   Old graph: 11,234 nodes
   New graph: 11,450 nodes
   Added: 216 new nodes

   Sample new node IDs (showing first 5):
     + VERSE:4:1:memorization
     + VERSE:4:2:memorization
     ...

✅ PASSED: Graph update is safe - no breaking changes detected
   User progress will be preserved.
```

**Expected output (on failure):**
```
Loading old graph: old_graph.cbor.zst
Loading new graph: new_graph.cbor.zst

❌ ERROR: Node IDs removed in new graph version!
   Missing IDs count: 5

   Sample missing IDs (showing first 10):
     - VERSE:1:1:memorization
     - VERSE:1:2:memorization
     ...

⚠️  FAILED: Breaking changes detected!
   User progress will be lost if this graph is released.

   To fix:
   1. Ensure node IDs are never changed or removed
   2. Only ADD new nodes, never modify/remove existing ones
   3. If IDs must change, provide a migration mapping
```

**Verify:**
- [ ] Script runs without errors
- [ ] Correctly detects missing node IDs
- [ ] Exit code 0 on success, 1 on failure
- [ ] Output is clear and actionable

### Step 2: Create Build Pipeline Wrapper (1-2 hours)

**File:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah/validation/pipeline_validation.py` (NEW)

```python
"""
Pipeline validation integration for knowledge graph builds.

This module integrates the standalone validate_stability.py script
into the automated build pipeline.
"""

import subprocess
import sys
from pathlib import Path
from typing import Optional
from loguru import logger


class GraphValidationError(Exception):
    """Raised when graph validation fails."""
    pass


def validate_graph_stability(
    new_graph_path: Path,
    baseline_path: Optional[Path] = None,
    skip_validation: bool = False,
) -> bool:
    """
    Validate graph stability against baseline.

    Args:
        new_graph_path: Path to newly built graph (.cbor.zst)
        baseline_path: Path to baseline graph (if None, uses default)
        skip_validation: If True, skip validation (dangerous!)

    Returns:
        True if validation passed or skipped

    Raises:
        GraphValidationError: If validation fails
    """
    if skip_validation:
        logger.warning("⚠️  VALIDATION SKIPPED (--skip-validation flag)")
        return True

    # Determine baseline path
    if baseline_path is None:
        baseline_path = new_graph_path.parent / "baseline_graph.cbor.zst"

    # If no baseline exists, this is the first build
    if not baseline_path.exists():
        logger.info("No baseline graph found - first build detected")
        logger.info(f"Saving baseline: {baseline_path}")
        _save_baseline(new_graph_path, baseline_path)
        return True

    # Run validation script
    logger.info(f"Validating against baseline: {baseline_path}")

    validate_script = Path(__file__).parent.parent.parent.parent / "validate_stability.py"

    if not validate_script.exists():
        raise FileNotFoundError(f"Validation script not found: {validate_script}")

    result = subprocess.run(
        [sys.executable, str(validate_script), str(baseline_path), str(new_graph_path)],
        capture_output=True,
        text=True,
    )

    # Print validation output
    print(result.stdout)

    if result.returncode != 0:
        # Validation failed
        logger.error("❌ Graph validation FAILED!")
        logger.error(result.stderr)
        raise GraphValidationError(
            "Node ID stability check failed. "
            "Use --skip-validation to override (not recommended)."
        )

    logger.success("✅ Graph validation PASSED")

    # Update baseline for next build
    logger.info(f"Updating baseline: {baseline_path}")
    _save_baseline(new_graph_path, baseline_path)

    return True


def _save_baseline(source: Path, target: Path) -> None:
    """Copy graph file to baseline location."""
    import shutil
    shutil.copy2(source, target)
    logger.info(f"Baseline saved: {target}")
```

### Step 3: Integrate into CLI Build Command (1 hour)

**File:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah_cli/commands/build.py`

Add validation step after graph export:

```python
# Around line 460, after CBOR export

from iqrah.validation.pipeline_validation import validate_graph_stability, GraphValidationError

def build_knowledge_graph(args) -> nx.DiGraph:
    # ... existing code ...

    # Save graph (existing code)
    if output_format in ("cbor", "both"):
        cbor_path = args.output if output_format == "cbor" else args.output + ".cbor.zst"
        export_graph_to_cbor(
            G,
            cbor_path,
            compression_level=config.export.compression_level,
            show_progress=not args.no_progress,
        )
        logger.success(f"CBOR export saved: {cbor_path}")

        # NEW: Validate graph stability
        try:
            validate_graph_stability(
                new_graph_path=Path(cbor_path),
                baseline_path=args.baseline if hasattr(args, 'baseline') else None,
                skip_validation=args.skip_validation if hasattr(args, 'skip_validation') else False,
            )
        except GraphValidationError as e:
            logger.error(str(e))
            sys.exit(1)

    # ... rest of function ...
```

**Add CLI arguments** (around line 230):

```python
def _setup_knowledge_graph_parser(subparsers):
    # ... existing arguments ...

    # NEW: Validation arguments
    parser.add_argument(
        "--skip-validation",
        action="store_true",
        help="Skip node ID stability validation (DANGEROUS - only for major version bumps)"
    )

    parser.add_argument(
        "--baseline",
        type=str,
        help="Path to baseline graph for validation (default: auto-detected)"
    )
```

### Step 4: Update Documentation (30 min)

**File:** `research_and_dev/iqrah-knowledge-graph2/README.md`

Add section:

```markdown
## Node ID Stability Validation

All graph builds are automatically validated to prevent breaking user progress.

### How It Works

1. **First Build:** Graph is saved as baseline (`baseline_graph.cbor.zst`)
2. **Subsequent Builds:** New graph is validated against baseline
3. **Validation Checks:** All node IDs from baseline must exist in new graph
4. **On Success:** New graph becomes the baseline
5. **On Failure:** Build exits with error

### Running Builds

```bash
# Normal build (validation enabled by default)
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology path/to/corpus.csv \
    --preset full \
    -o output/graph.cbor.zst

# Skip validation (DANGEROUS - only for major version bumps)
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology path/to/corpus.csv \
    --skip-validation \
    -o output/graph.cbor.zst

# Use custom baseline
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology path/to/corpus.csv \
    --baseline path/to/baseline.cbor.zst \
    -o output/graph.cbor.zst
```

### Manual Validation

You can also run validation manually:

```bash
python validate_stability.py old_graph.cbor.zst new_graph.cbor.zst
```

### Handling Breaking Changes

If you MUST remove/change node IDs:

1. Document the change in `CHANGELOG.md`
2. Bump major version (2.0.0 → 3.0.0)
3. Create migration mapping (see `docs/migration-strategy.md`)
4. Run with `--skip-validation` flag
5. Communicate breaking change to users

### CI/CD Integration

In CI pipeline:

```yaml
- name: Build and validate knowledge graph
  run: |
    cd research_and_dev/iqrah-knowledge-graph2
    python -m iqrah_cli build knowledge-graph \
      --from-scratch \
      --morphology data/corpus.csv \
      --preset ci-test \
      -o output/graph.cbor.zst
    # Validation runs automatically - will fail build if broken
```

Validation failures will cause CI to fail, preventing merge.
```

### Step 5: Add Tests (1 hour)

**File:** `research_and_dev/iqrah-knowledge-graph2/tests/test_pipeline_validation.py` (NEW)

```python
import pytest
from pathlib import Path
import tempfile
import networkx as nx
from iqrah.validation.pipeline_validation import validate_graph_stability, GraphValidationError
from iqrah.export.cbor_export import export_graph_to_cbor


def test_first_build_no_baseline(tmp_path):
    """First build should succeed and create baseline."""
    graph = nx.DiGraph()
    graph.add_node("VERSE:1:1", type="verse")
    graph.add_node("VERSE:1:2", type="verse")

    graph_path = tmp_path / "test_graph.cbor.zst"
    export_graph_to_cbor(graph, str(graph_path), show_progress=False)

    baseline_path = tmp_path / "baseline_graph.cbor.zst"

    # Should succeed (no baseline yet)
    result = validate_graph_stability(
        new_graph_path=graph_path,
        baseline_path=baseline_path,
    )

    assert result is True
    assert baseline_path.exists()


def test_validation_passes_with_added_nodes(tmp_path):
    """Validation should pass when only adding nodes."""
    # Create baseline
    old_graph = nx.DiGraph()
    old_graph.add_node("VERSE:1:1", type="verse")
    old_graph.add_node("VERSE:1:2", type="verse")

    baseline_path = tmp_path / "baseline.cbor.zst"
    export_graph_to_cbor(old_graph, str(baseline_path), show_progress=False)

    # Create new graph with added nodes
    new_graph = nx.DiGraph()
    new_graph.add_node("VERSE:1:1", type="verse")
    new_graph.add_node("VERSE:1:2", type="verse")
    new_graph.add_node("VERSE:1:3", type="verse")  # NEW

    new_path = tmp_path / "new_graph.cbor.zst"
    export_graph_to_cbor(new_graph, str(new_path), show_progress=False)

    # Should succeed
    result = validate_graph_stability(
        new_graph_path=new_path,
        baseline_path=baseline_path,
    )

    assert result is True


def test_validation_fails_with_removed_nodes(tmp_path):
    """Validation should fail when nodes are removed."""
    # Create baseline
    old_graph = nx.DiGraph()
    old_graph.add_node("VERSE:1:1", type="verse")
    old_graph.add_node("VERSE:1:2", type="verse")
    old_graph.add_node("VERSE:1:3", type="verse")

    baseline_path = tmp_path / "baseline.cbor.zst"
    export_graph_to_cbor(old_graph, str(baseline_path), show_progress=False)

    # Create new graph with removed node
    new_graph = nx.DiGraph()
    new_graph.add_node("VERSE:1:1", type="verse")
    new_graph.add_node("VERSE:1:2", type="verse")
    # VERSE:1:3 removed!

    new_path = tmp_path / "new_graph.cbor.zst"
    export_graph_to_cbor(new_graph, str(new_path), show_progress=False)

    # Should fail
    with pytest.raises(GraphValidationError):
        validate_graph_stability(
            new_graph_path=new_path,
            baseline_path=baseline_path,
        )


def test_skip_validation_flag(tmp_path):
    """Skip validation should bypass checks."""
    # Create graphs with breaking change
    old_graph = nx.DiGraph()
    old_graph.add_node("VERSE:1:1", type="verse")

    baseline_path = tmp_path / "baseline.cbor.zst"
    export_graph_to_cbor(old_graph, str(baseline_path), show_progress=False)

    new_graph = nx.DiGraph()
    new_graph.add_node("VERSE:1:2", type="verse")  # Different node

    new_path = tmp_path / "new_graph.cbor.zst"
    export_graph_to_cbor(new_graph, str(new_path), show_progress=False)

    # Should succeed with skip flag
    result = validate_graph_stability(
        new_graph_path=new_path,
        baseline_path=baseline_path,
        skip_validation=True,
    )

    assert result is True
```

Run tests:
```bash
cd research_and_dev/iqrah-knowledge-graph2
pytest tests/test_pipeline_validation.py -v
```

## Verification Plan

### Unit Tests

```bash
cd research_and_dev/iqrah-knowledge-graph2
pytest tests/test_pipeline_validation.py -v
```

- [ ] `test_first_build_no_baseline` passes
- [ ] `test_validation_passes_with_added_nodes` passes
- [ ] `test_validation_fails_with_removed_nodes` passes (raises error correctly)
- [ ] `test_skip_validation_flag` passes

### Integration Test

```bash
cd research_and_dev/iqrah-knowledge-graph2

# First build (creates baseline)
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology ../data/morphology.csv \
    --preset basic \
    --chapters "1-2" \
    -o test_output/graph1.cbor.zst

# Should succeed, create baseline_graph.cbor.zst

# Second build (no changes)
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology ../data/morphology.csv \
    --preset basic \
    --chapters "1-2" \
    -o test_output/graph2.cbor.zst

# Should succeed (validation passes)

# Third build (with changes - add chapter 3)
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology ../data/morphology.csv \
    --preset basic \
    --chapters "1-3" \
    -o test_output/graph3.cbor.zst

# Should succeed (added nodes OK)
```

### Manual Verification

- [ ] First build creates baseline file
- [ ] Baseline file is valid CBOR
- [ ] Second build validates against baseline
- [ ] Validation output is printed to console
- [ ] Adding nodes passes validation
- [ ] Removing nodes fails validation (test manually by editing graph)
- [ ] `--skip-validation` flag works
- [ ] Exit code is 1 on validation failure (for CI)
- [ ] Error messages are clear and actionable

### CI Test

Create a breaking change intentionally:

```bash
# Manually edit builder to remove a node, then build
python -m iqrah_cli build knowledge-graph ... -o test.cbor.zst

# Should exit with code 1 and error message
echo $?  # Should be 1
```

## Scope Limits & Safeguards

### ✅ MUST DO

- Integrate existing `validate_stability.py` script into build pipeline
- Add CLI flags for validation control
- Create wrapper module for pipeline integration
- Add comprehensive tests
- Document validation workflow
- Ensure validation runs by default
- Exit with code 1 on validation failure (for CI)

### ❌ DO NOT

- Modify `validate_stability.py` itself (it already works)
- Change CBOR format or export logic
- Touch Rust code
- Implement automatic migration (out of scope)
- Add overly complex validation rules

### ⚠️ If Uncertain

- If baseline path unclear → Use `baseline_graph.cbor.zst` in output directory
- If validation script path unclear → Use relative path from validation module
- If subprocess call fails → Check Python executable path
- If tests don't run → Verify pytest is installed
- If unsure about integration point → Add validation right after CBOR export

## Success Criteria

- [ ] `pipeline_validation.py` module created
- [ ] Integration added to `build.py` CLI command
- [ ] `--skip-validation` flag works
- [ ] `--baseline` flag works
- [ ] Tests pass (4+ test cases)
- [ ] First build creates baseline
- [ ] Second build validates against baseline
- [ ] Adding nodes passes validation
- [ ] Removing nodes fails validation (exit code 1)
- [ ] README documents validation workflow
- [ ] Error messages are clear
- [ ] CI will fail on validation errors

## Related Files

**Create These Files:**
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/validation/__init__.py`
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/validation/pipeline_validation.py`
- `/research_and_dev/iqrah-knowledge-graph2/tests/test_pipeline_validation.py`

**Modify These Files:**
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah_cli/commands/build.py` - Add validation call
- `/research_and_dev/iqrah-knowledge-graph2/README.md` - Document validation

**Use Existing (No Changes):**
- `/research_and_dev/iqrah-knowledge-graph2/validate_stability.py` - Standalone validation script
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/export/cbor_export.py` - CBOR export

**Will Create:**
- `/research_and_dev/iqrah-knowledge-graph2/output/baseline_graph.cbor.zst` - Baseline for validation (generated)

## Notes

### Why This Matters

The existing `validate_stability.py` script is excellent but requires manual execution. By integrating it into the build pipeline:

1. **Automatic enforcement** - No way to accidentally skip validation
2. **CI integration** - Validation failures block merges
3. **Developer experience** - Clear error messages when mistakes happen
4. **Production safety** - User progress is protected

### Technical Approach

We're using a **wrapper + subprocess** approach:
- Keep `validate_stability.py` standalone (works independently)
- Create thin wrapper (`pipeline_validation.py`) for integration
- Call validation script via subprocess (clean separation)
- Capture output and exit codes for CI integration

### Baseline Management

The baseline graph serves as the "source of truth" for node IDs:
- Stored as `baseline_graph.cbor.zst` next to output
- Updated automatically on successful validation
- Can be overridden with `--baseline` flag
- First build creates initial baseline

### Escape Hatch

The `--skip-validation` flag exists for:
- Major version bumps with intentional breaking changes
- Migration scenarios with documented mapping
- Emergency situations (use with extreme caution)

Always document why validation was skipped.
