# Task 1.5: Add Node ID Stability Validation to Python Pipeline

## Metadata
- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 1 day
- **Dependencies:** None (Python-side only, but Task 1.1 architecture doc helpful)
- **Agent Type:** Implementation (Python)
- **Parallelizable:** Yes (with tasks 1.1, 1.2, 1.3)

## Goal

Implement validation in the Python knowledge graph build pipeline to prevent accidental node ID changes that would break user progress, enforcing the node ID stability policy.

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
Before generating a new graph migration, validate that all previous node IDs still exist. New nodes are fine, removed/changed nodes are breaking changes that require explicit migration.

## Current State

**Python R&D Project:**
- **Location:** `research_and_dev/iqrah-knowledge-graph2/`
- **Graph Builder:** `src/iqrah/graph/builder.py`, `knowledge_builder.py`
- **Export Script:** `score_and_extract.py` (generates SQL migrations)

**No Validation:**
- Nothing prevents node IDs from changing
- Nothing compares old vs new graph
- Could accidentally break user data

**Example Build Process (Current):**
```bash
cd research_and_dev/iqrah-knowledge-graph2
python score_and_extract.py --output ../iqrah-mobile/rust/crates/iqrah-storage/migrations_content/new_graph.sql
```

No checks, just overwrites.

## Target State

### Validation Script

**File:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah/validation/node_stability.py`

```python
def validate_node_stability(old_graph: Graph, new_graph: Graph) -> ValidationResult:
    """
    Validate that all node IDs from old graph still exist in new graph.

    Returns:
        ValidationResult with:
        - is_valid: bool
        - missing_nodes: Set[str] (node IDs removed)
        - added_nodes: Set[str] (new node IDs - informational)
        - summary: str
    """
```

### Integration into Build Pipeline

**File:** `research_and_dev/iqrah-knowledge-graph2/score_and_extract.py`

```python
if __name__ == "__main__":
    # Build new graph
    new_graph = build_knowledge_graph()

    # Load previous version (if exists)
    if Path("previous_graph.json").exists():
        old_graph = load_graph("previous_graph.json")

        # Validate stability
        result = validate_node_stability(old_graph, new_graph)

        if not result.is_valid:
            print("ERROR: Node ID stability check FAILED!")
            print(f"Missing node IDs: {result.missing_nodes}")
            sys.exit(1)

        print(f"✅ Node ID stability validated ({len(result.added_nodes)} new nodes)")

    # Save current graph for next validation
    save_graph(new_graph, "previous_graph.json")

    # Export to SQL
    export_to_sql(new_graph, output_path)
```

### CI Integration (Documentation)

**File:** `research_and_dev/iqrah-knowledge-graph2/README.md`

Document how to run validation in CI:
```bash
# Before merging graph updates:
python score_and_extract.py --validate-only

# Should exit with code 1 if node IDs removed
```

## Implementation Steps

### Step 1: Create Validation Module (2 hours)

**File:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah/validation/__init__.py`

```python
"""Validation utilities for knowledge graph stability."""
```

**File:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah/validation/node_stability.py`

```python
from typing import Set, NamedTuple
from ..graph.models import Graph

class ValidationResult(NamedTuple):
    is_valid: bool
    missing_nodes: Set[str]
    added_nodes: Set[str]
    summary: str

def validate_node_stability(
    old_graph: Graph,
    new_graph: Graph,
    allow_removals: bool = False
) -> ValidationResult:
    """
    Validate that all node IDs from old graph still exist in new graph.

    Args:
        old_graph: Previous version of the graph
        new_graph: New version of the graph
        allow_removals: If True, only warn about removals (don't fail)

    Returns:
        ValidationResult with validation status and details
    """
    old_nodes = set(old_graph.nodes.keys())
    new_nodes = set(new_graph.nodes.keys())

    missing_nodes = old_nodes - new_nodes
    added_nodes = new_nodes - old_nodes

    is_valid = len(missing_nodes) == 0 or allow_removals

    summary = f"""
Node Stability Validation:
- Old graph: {len(old_nodes)} nodes
- New graph: {len(new_nodes)} nodes
- Added: {len(added_nodes)} nodes
- Removed: {len(missing_nodes)} nodes
- Status: {'✅ PASS' if is_valid else '❌ FAIL'}
"""

    if missing_nodes:
        summary += f"\nMissing node IDs:\n"
        for node_id in sorted(missing_nodes)[:10]:  # Show first 10
            summary += f"  - {node_id}\n"
        if len(missing_nodes) > 10:
            summary += f"  ... and {len(missing_nodes) - 10} more\n"

    return ValidationResult(
        is_valid=is_valid,
        missing_nodes=missing_nodes,
        added_nodes=added_nodes,
        summary=summary
    )

def validate_node_id_formats(graph: Graph) -> ValidationResult:
    """
    Validate that all node IDs follow the documented format specification.

    Checks:
    - Prefixes match expected types (CHAPTER, VERSE, WORD, WORD_INSTANCE)
    - Chapter numbers in range 1-114
    - Verse numbers >= 1
    - Knowledge axis names are valid

    Returns:
        ValidationResult with validation status
    """
    invalid_nodes = set()
    valid_prefixes = {"CHAPTER", "VERSE", "WORD", "WORD_INSTANCE"}
    valid_axes = {"memorization", "translation", "tafsir", "tajweed", "contextual_memorization", "meaning"}

    for node_id in graph.nodes.keys():
        parts = node_id.split(":")

        if len(parts) < 1:
            invalid_nodes.add(node_id)
            continue

        # Check if it's a knowledge node (ends with axis)
        if len(parts) >= 2 and parts[-1] in valid_axes:
            # Valid knowledge node
            continue

        # Check content node prefix
        prefix = parts[0]

        # Handle unprefixed verse IDs like "1:1"
        if prefix.isdigit() and len(parts) == 2:
            # Likely unprefixed verse
            try:
                ch = int(parts[0])
                v = int(parts[1])
                if not (1 <= ch <= 114) or v < 1:
                    invalid_nodes.add(node_id)
            except ValueError:
                invalid_nodes.add(node_id)
            continue

        # Check prefixed nodes
        if prefix not in valid_prefixes and prefix != "WORD":  # WORD is sometimes numeric
            invalid_nodes.add(node_id)
            continue

        # Validate chapter numbers if present
        if prefix in ["CHAPTER", "VERSE", "WORD_INSTANCE"]:
            try:
                ch_idx = 1
                if prefix == "CHAPTER" and len(parts) >= 2:
                    ch = int(parts[1])
                elif prefix in ["VERSE", "WORD_INSTANCE"] and len(parts) >= 3:
                    ch = int(parts[1])
                else:
                    continue  # Can't validate structure

                if not (1 <= ch <= 114):
                    invalid_nodes.add(node_id)
            except (ValueError, IndexError):
                invalid_nodes.add(node_id)

    is_valid = len(invalid_nodes) == 0

    summary = f"""
Node ID Format Validation:
- Total nodes: {len(graph.nodes)}
- Invalid formats: {len(invalid_nodes)}
- Status: {'✅ PASS' if is_valid else '❌ FAIL'}
"""

    if invalid_nodes:
        summary += "\nInvalid node IDs:\n"
        for node_id in sorted(invalid_nodes)[:10]:
            summary += f"  - {node_id}\n"
        if len(invalid_nodes) > 10:
            summary += f"  ... and {len(invalid_nodes) - 10} more\n"

    return ValidationResult(
        is_valid=is_valid,
        missing_nodes=set(),
        added_nodes=invalid_nodes,
        summary=summary
    )
```

### Step 2: Add Graph Serialization (1 hour)

**File:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/serialization.py`

```python
import json
from pathlib import Path
from .models import Graph

def save_graph_snapshot(graph: Graph, path: str) -> None:
    """Save a snapshot of node IDs for validation."""
    snapshot = {
        "node_ids": sorted(graph.nodes.keys()),
        "version": "2.0.0",  # Graph schema version
        "node_count": len(graph.nodes),
    }

    with open(path, 'w') as f:
        json.dump(snapshot, f, indent=2)

def load_graph_snapshot(path: str) -> Set[str]:
    """Load node IDs from snapshot."""
    with open(path, 'r') as f:
        snapshot = json.load(f)
    return set(snapshot["node_ids"])
```

### Step 3: Integrate into Build Script (1-2 hours)

**File:** `research_and_dev/iqrah-knowledge-graph2/score_and_extract.py`

Add validation logic:

```python
import sys
from pathlib import Path
from iqrah.validation.node_stability import validate_node_stability, validate_node_id_formats
from iqrah.graph.serialization import save_graph_snapshot, load_graph_snapshot

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    parser.add_argument("--validate-only", action="store_true", help="Only run validation, don't generate")
    parser.add_argument("--skip-validation", action="store_true", help="Skip stability validation (dangerous!)")
    args = parser.parse_args()

    # Build new graph
    print("Building knowledge graph...")
    new_graph = build_knowledge_graph()

    # Validate node ID formats
    print("\nValidating node ID formats...")
    format_result = validate_node_id_formats(new_graph)
    print(format_result.summary)

    if not format_result.is_valid:
        print("ERROR: Node ID format validation failed!")
        sys.exit(1)

    # Validate node stability (if previous version exists)
    snapshot_path = Path("graph_snapshot.json")

    if snapshot_path.exists() and not args.skip_validation:
        print("\nValidating node ID stability...")
        old_node_ids = load_graph_snapshot(snapshot_path)

        # Create minimal old graph structure
        old_graph = Graph()
        for node_id in old_node_ids:
            old_graph.add_node(node_id)  # Simplified

        stability_result = validate_node_stability(old_graph, new_graph)
        print(stability_result.summary)

        if not stability_result.is_valid:
            print("\n❌ BREAKING CHANGE DETECTED!")
            print("Node IDs have been removed, which will break user progress.")
            print("If this is intentional, you must:")
            print("1. Create a migration mapping (see docs/migration-strategy.md)")
            print("2. Bump major version (2.0.0 -> 3.0.0)")
            print("3. Run with --skip-validation (not recommended)")
            sys.exit(1)

        print(f"✅ Node stability validated: {len(stability_result.added_nodes)} new nodes added")

    if args.validate_only:
        print("\nValidation complete (--validate-only mode)")
        return

    # Save snapshot for next validation
    print("\nSaving graph snapshot...")
    save_graph_snapshot(new_graph, snapshot_path)

    # Export to SQL
    print(f"\nExporting to {args.output}...")
    export_to_sql(new_graph, args.output)

    print("✅ Graph generation complete!")

if __name__ == "__main__":
    main()
```

### Step 4: Add Tests (1-2 hours)

**File:** `research_and_dev/iqrah-knowledge-graph2/tests/test_node_stability.py`

```python
import pytest
from iqrah.graph.models import Graph
from iqrah.validation.node_stability import validate_node_stability, validate_node_id_formats

def test_no_changes():
    """Test that identical graphs pass validation."""
    graph = Graph()
    graph.add_node("VERSE:1:1")
    graph.add_node("VERSE:1:2")

    result = validate_node_stability(graph, graph)
    assert result.is_valid
    assert len(result.missing_nodes) == 0
    assert len(result.added_nodes) == 0

def test_added_nodes():
    """Test that adding nodes is allowed."""
    old_graph = Graph()
    old_graph.add_node("VERSE:1:1")

    new_graph = Graph()
    new_graph.add_node("VERSE:1:1")
    new_graph.add_node("VERSE:1:2")

    result = validate_node_stability(old_graph, new_graph)
    assert result.is_valid
    assert len(result.added_nodes) == 1
    assert "VERSE:1:2" in result.added_nodes

def test_removed_nodes():
    """Test that removing nodes fails validation."""
    old_graph = Graph()
    old_graph.add_node("VERSE:1:1")
    old_graph.add_node("VERSE:1:2")

    new_graph = Graph()
    new_graph.add_node("VERSE:1:1")

    result = validate_node_stability(old_graph, new_graph)
    assert not result.is_valid
    assert len(result.missing_nodes) == 1
    assert "VERSE:1:2" in result.missing_nodes

def test_valid_node_formats():
    """Test that valid node IDs pass format validation."""
    graph = Graph()
    graph.add_node("CHAPTER:1")
    graph.add_node("VERSE:1:1")
    graph.add_node("WORD:123")
    graph.add_node("WORD_INSTANCE:1:1:3")
    graph.add_node("VERSE:1:1:memorization")
    graph.add_node("1:1")  # Unprefixed verse (legacy)

    result = validate_node_id_formats(graph)
    assert result.is_valid

def test_invalid_node_formats():
    """Test that invalid node IDs fail format validation."""
    graph = Graph()
    graph.add_node("INVALID:1:1")
    graph.add_node("CHAPTER:115")  # Out of range

    result = validate_node_id_formats(graph)
    assert not result.is_valid
```

Run tests:
```bash
cd research_and_dev/iqrah-knowledge-graph2
pytest tests/test_node_stability.py
```

### Step 5: Document Usage (30 min)

**File:** `research_and_dev/iqrah-knowledge-graph2/README.md`

Add section:
```markdown
## Node ID Stability Validation

To prevent breaking user progress, all graph updates must pass node ID stability validation.

### Running Validation

```bash
# Generate graph with validation (default):
python score_and_extract.py --output migrations/new_graph.sql

# Validate only (don't generate):
python score_and_extract.py --output dummy.sql --validate-only

# Skip validation (DANGEROUS - only for major version bumps):
python score_and_extract.py --output migrations/new_graph.sql --skip-validation
```

### What Gets Validated

1. **Node ID Stability:** All node IDs from previous version must exist
2. **Format Validation:** Node IDs follow documented format specification

### Handling Breaking Changes

If you MUST remove/change node IDs:

1. Document the change in `CHANGELOG.md`
2. Bump major version (2.0.0 → 3.0.0)
3. Create migration mapping (see `docs/migration-strategy.md`)
4. Use `--skip-validation` flag with extreme caution
```

## Verification Plan

### Unit Tests

```bash
cd research_and_dev/iqrah-knowledge-graph2
pytest tests/test_node_stability.py -v
```

- [ ] `test_no_changes` passes (identical graphs)
- [ ] `test_added_nodes` passes (new nodes OK)
- [ ] `test_removed_nodes` fails validation correctly
- [ ] `test_valid_node_formats` passes
- [ ] `test_invalid_node_formats` fails correctly

### Integration Test

```bash
# First run (no previous snapshot):
python score_and_extract.py --output test_output.sql
# Should succeed, create graph_snapshot.json

# Second run (with snapshot):
python score_and_extract.py --output test_output.sql
# Should succeed (no changes)

# Test breaking change:
# Manually edit graph builder to remove a node
python score_and_extract.py --output test_output.sql
# Should FAIL with node stability error
```

### Manual Verification

- [ ] First run creates `graph_snapshot.json`
- [ ] Snapshot contains node IDs as JSON array
- [ ] Second run validates against snapshot
- [ ] Removing a node triggers validation failure
- [ ] Exit code is 1 on validation failure (for CI)
- [ ] Error message clearly explains the issue

## Scope Limits & Safeguards

### ✅ MUST DO

- Implement node stability validation (compare old vs new)
- Implement node ID format validation
- Integrate into `score_and_extract.py`
- Add comprehensive tests
- Document usage in README
- Exit with code 1 on validation failure (for CI)

### ❌ DO NOT

- Modify graph generation logic (only add validation)
- Change existing node IDs (this is validation, not refactoring)
- Touch Rust code (Python-side only)
- Implement automatic migration (out of scope)
- Add overly complex validation rules (keep it simple)

### ⚠️ If Uncertain

- If graph structure is complex → simplify by only tracking node IDs (not full graph)
- If snapshot format unclear → use simple JSON with node ID array
- If validation seems too strict → add `--skip-validation` flag as escape hatch
- If tests don't run → check pytest installation and project structure

## Success Criteria

- [ ] `node_stability.py` module exists with validation functions
- [ ] `score_and_extract.py` runs validation by default
- [ ] Validation compares old vs new node IDs
- [ ] Format validation checks ID structure
- [ ] Tests pass (5+ test cases)
- [ ] First run succeeds and creates snapshot
- [ ] Second run validates against snapshot
- [ ] Removing node triggers failure (exit code 1)
- [ ] README documents validation usage
- [ ] `--validate-only` flag works
- [ ] `--skip-validation` escape hatch exists

## Related Files

**Create These Files:**
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/validation/__init__.py`
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/validation/node_stability.py`
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/serialization.py`
- `/research_and_dev/iqrah-knowledge-graph2/tests/test_node_stability.py`

**Modify These Files:**
- `/research_and_dev/iqrah-knowledge-graph2/score_and_extract.py` - Add validation
- `/research_and_dev/iqrah-knowledge-graph2/README.md` - Document usage

**Will Create:**
- `/research_and_dev/iqrah-knowledge-graph2/graph_snapshot.json` - Node ID snapshot (generated)

## Notes

### Why This Matters

Without this validation, it's easy to accidentally:
- Rename node IDs during refactoring
- Change ID format (e.g., "1:1" → "VERSE:1:1")
- Remove nodes during testing
- Break user progress silently

This validation acts as a **safety net** to prevent production incidents.

### CI Integration

In CI/CD pipeline, add:
```yaml
- name: Validate knowledge graph stability
  run: |
    cd research_and_dev/iqrah-knowledge-graph2
    python score_and_extract.py --output /tmp/test.sql
```

This ensures every graph update is validated before merge.

### Escape Hatch

The `--skip-validation` flag exists for legitimate breaking changes (e.g., major version bump with migration plan). Document its use clearly and require explicit user confirmation.
