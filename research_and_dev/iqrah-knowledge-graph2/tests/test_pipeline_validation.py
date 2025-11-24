import pytest
from pathlib import Path
import networkx as nx
from iqrah.validation.pipeline_validation import validate_graph_stability, GraphValidationError
from iqrah.export.cbor_export import export_graph_to_cbor
import zstandard as zstd
import os

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
