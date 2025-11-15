"""
Tests for graph statistics and validation.

Tests comprehensive statistics computation and validation metrics.
"""

import pytest
import networkx as nx
import tempfile
import json
from pathlib import Path

from iqrah.graph.statistics import GraphStatistics, compute_graph_statistics
from iqrah.quran_offline import load_quran_offline
from iqrah.morphology.corpus import QuranicArabicCorpus
from iqrah.graph.builder import QuranGraphBuilder
from iqrah.graph.knowledge_builder import KnowledgeGraphBuilder
from iqrah.graph.scoring import calculate_knowledge_scores


class TestGraphStatistics:
    """Test graph statistics computation."""

    @pytest.fixture
    def simple_graph(self):
        """Create a simple test graph."""
        G = nx.DiGraph()

        # Add nodes with types and scores
        G.add_node("CHAPTER:1", type="chapter", foundational_score=0.9, influence_score=0.8)
        G.add_node("VERSE:1:1", type="verse", foundational_score=0.7, influence_score=0.6)
        G.add_node("WORD_INSTANCE:1:1:1", type="word_instance", foundational_score=0.5, influence_score=0.4)
        G.add_node("WORD:test", type="word", foundational_score=0.6, influence_score=0.5)
        G.add_node("LEMMA:test", type="lemma", foundational_score=0.8, influence_score=0.7)
        G.add_node("ROOT:tst", type="root", foundational_score=0.95, influence_score=0.9)

        # Add edges with distributions
        G.add_edge("WORD_INSTANCE:1:1:1", "VERSE:1:1", type="dependency")
        G.add_edge("VERSE:1:1", "CHAPTER:1", type="dependency")
        G.add_edge("WORD_INSTANCE:1:1:1:memorization", "VERSE:1:1:memorization",
                   dist="auto", weight=0.5)
        G.add_edge("WORD_INSTANCE:1:1:1:translation", "VERSE:1:1:translation",
                   dist="normal", m=0.7, s=0.1)
        G.add_edge("WORD:test", "LEMMA:test", dist="auto", weight=1.0)

        return G

    @pytest.fixture
    def real_graph(self, morphology_path):
        """Create a real graph from first chapter."""
        if not morphology_path.exists():
            pytest.skip("Morphology corpus not found")

        quran = load_quran_offline()
        corpus = QuranicArabicCorpus()
        corpus.load_data(str(morphology_path))

        chapters = [quran.chapters[0]]
        builder = QuranGraphBuilder()
        G = builder.build_graph(quran, corpus, chapters, show_progress=False)

        # Add knowledge edges
        kb = KnowledgeGraphBuilder(G, quran)
        kb.build_all(include_memorization=True, include_translation=True)
        kb.compile()

        # Add scores
        calculate_knowledge_scores(G, max_iter=1000)

        return G

    @pytest.fixture
    def morphology_path(self):
        """Get morphology path."""
        return Path("research_and_dev/data/quranic-arabic-corpus-morphology.csv")

    def test_basic_stats_computation(self, simple_graph):
        """Test basic statistics computation."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        assert stats is not None
        assert "basic_stats" in stats
        assert "node_stats" in stats
        assert "edge_stats" in stats

        basic = stats["basic_stats"]
        assert basic["total_nodes"] == 6
        assert basic["total_edges"] == 5
        assert basic["directed"] == True

    def test_node_stats_by_type(self, simple_graph):
        """Test node statistics by type."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        node_stats = stats["node_stats"]["by_type"]

        assert node_stats["chapter"] == 1
        assert node_stats["verse"] == 1
        assert node_stats["word_instance"] == 1
        assert node_stats["word"] == 1
        assert node_stats["lemma"] == 1
        assert node_stats["root"] == 1

    def test_edge_stats_by_type(self, simple_graph):
        """Test edge statistics by type."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        edge_stats = stats["edge_stats"]["by_type"]

        assert edge_stats["dependency"] == 2

    def test_score_stats(self, simple_graph):
        """Test score statistics."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        score_stats = stats["score_stats"]

        assert "foundational" in score_stats
        assert "influence" in score_stats

        f = score_stats["foundational"]
        assert f["count"] == 6
        assert 0 <= f["mean"] <= 1
        assert 0 <= f["min"] <= 1
        assert 0 <= f["max"] <= 1
        assert f["min"] <= f["mean"] <= f["max"]

    def test_top_nodes_computation(self, simple_graph):
        """Test top nodes computation."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        top_nodes = stats["top_nodes"]

        # Check foundational ranking
        top_foundational = top_nodes["by_foundational_score"]
        assert len(top_foundational) > 0

        # Root should be most foundational
        assert top_foundational[0]["node_id"] == "ROOT:tst"
        assert top_foundational[0]["score"] == 0.95

        # Check top chapters
        top_chapters = top_nodes["top_chapters"]
        assert len(top_chapters) == 1
        assert top_chapters[0]["node_id"] == "CHAPTER:1"

    def test_validation_metrics(self, simple_graph):
        """Test validation metrics."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        validation = stats["validation"]

        assert "checks" in validation
        assert "errors" in validation
        assert "warnings" in validation

        # Simple graph should pass basic checks
        assert validation["checks"]["all_nodes_have_type"] == True

    def test_validation_catches_errors(self):
        """Test validation catches common errors."""
        G = nx.DiGraph()

        # Add node without type
        G.add_node("BAD_NODE")

        # Add knowledge edge without distribution
        G.add_node("N1", type="word")
        G.add_node("N2", type="verse")
        G.add_edge("N1", "N2")  # No dist or weight

        stats_calc = GraphStatistics(G)
        stats = stats_calc.compute_all()

        validation = stats["validation"]

        # Should detect missing type
        assert validation["checks"]["all_nodes_have_type"] == False
        assert validation["total_warnings"] > 0

    def test_validation_catches_invalid_scores(self):
        """Test validation catches invalid score ranges."""
        G = nx.DiGraph()

        G.add_node("N1", type="word", foundational_score=1.5)  # Out of range
        G.add_node("N2", type="verse", influence_score=-0.1)  # Out of range

        stats_calc = GraphStatistics(G)
        stats = stats_calc.compute_all()

        validation = stats["validation"]

        assert validation["checks"]["all_scores_in_range"] == False
        assert validation["total_errors"] > 0

    def test_export_to_json(self, simple_graph):
        """Test JSON export."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = Path(tmpdir) / "stats.json"

            stats_calc = GraphStatistics(simple_graph)
            stats_calc.compute_all()
            stats_calc.export_to_json(str(output_path))

            assert output_path.exists()

            # Verify JSON is valid
            with open(output_path) as f:
                loaded_stats = json.load(f)

            assert loaded_stats is not None
            assert "basic_stats" in loaded_stats
            assert loaded_stats["basic_stats"]["total_nodes"] == 6

    def test_print_summary(self, simple_graph, capsys):
        """Test summary printing."""
        stats_calc = GraphStatistics(simple_graph)
        stats_calc.compute_all()
        stats_calc.print_summary()

        captured = capsys.readouterr()
        assert "KNOWLEDGE GRAPH STATISTICS SUMMARY" in captured.out
        assert "Basic Statistics" in captured.out
        assert "Top 10 Most Foundational Nodes" in captured.out

    def test_convenience_function(self, simple_graph):
        """Test convenience function."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = Path(tmpdir) / "stats.json"

            stats = compute_graph_statistics(
                simple_graph,
                export_path=str(output_path),
                print_summary=False
            )

            assert stats is not None
            assert output_path.exists()

    def test_real_graph_statistics(self, real_graph):
        """Test statistics on real graph."""
        stats_calc = GraphStatistics(real_graph)
        stats = stats_calc.compute_all()

        # Basic checks
        basic = stats["basic_stats"]
        assert basic["total_nodes"] > 0
        assert basic["total_edges"] > 0

        # Node types should include all expected types
        node_types = stats["node_stats"]["by_type"]
        assert "chapter" in node_types
        assert "verse" in node_types
        assert "word_instance" in node_types

        # Should have knowledge edges
        knowledge_stats = stats["knowledge_edge_stats"]
        assert knowledge_stats["total_knowledge_edges"] > 0

        # Should have scores
        score_stats = stats["score_stats"]
        assert score_stats["scored_nodes"] > 0

        # Validation should pass
        validation = stats["validation"]
        assert validation["passed_checks"] == validation["total_checks"]
        assert validation["total_errors"] == 0

    def test_knowledge_edge_axis_detection(self, simple_graph):
        """Test detection of knowledge edge axes."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        knowledge_stats = stats["knowledge_edge_stats"]
        axes = knowledge_stats["by_axis"]

        # Should detect memorization and translation axes
        assert "memorization" in axes
        assert "translation" in axes

    def test_distribution_type_detection(self, simple_graph):
        """Test detection of distribution types."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        knowledge_stats = stats["knowledge_edge_stats"]
        dist_types = knowledge_stats["by_distribution"]

        assert "auto" in dist_types
        assert "normal" in dist_types

    def test_connectivity_stats(self, simple_graph):
        """Test connectivity statistics."""
        stats_calc = GraphStatistics(simple_graph)
        stats = stats_calc.compute_all()

        conn = stats["connectivity"]

        assert "avg_in_degree" in conn
        assert "avg_out_degree" in conn
        assert "max_in_degree" in conn
        assert "max_out_degree" in conn

        assert conn["avg_in_degree"] >= 0
        assert conn["avg_out_degree"] >= 0

    def test_expected_ratios_validation(self, real_graph):
        """Test validation of expected node ratios."""
        stats_calc = GraphStatistics(real_graph)
        stats = stats_calc.compute_all()

        validation = stats["validation"]

        # Words per verse should be in expected range
        if "words_per_verse_ratio" in validation["checks"]:
            # First chapter has ~31 words on average
            assert validation["checks"]["words_per_verse_ratio"] == True


class TestStatisticsTracking:
    """Test statistics tracking over time."""

    def test_stats_comparison(self):
        """Test comparing statistics from different builds."""
        # Create two similar graphs
        G1 = nx.DiGraph()
        G1.add_node("N1", type="word", foundational_score=0.5)
        G1.add_node("N2", type="verse", foundational_score=0.6)
        G1.add_edge("N1", "N2", dist="auto", weight=1.0)

        G2 = nx.DiGraph()
        G2.add_node("N1", type="word", foundational_score=0.55)  # Slightly different score
        G2.add_node("N2", type="verse", foundational_score=0.6)
        G2.add_node("N3", type="chapter", foundational_score=0.8)  # Additional node
        G2.add_edge("N1", "N2", dist="auto", weight=1.0)
        G2.add_edge("N2", "N3", dist="auto", weight=1.0)

        stats1 = compute_graph_statistics(G1, print_summary=False)
        stats2 = compute_graph_statistics(G2, print_summary=False)

        # Compare
        assert stats1["basic_stats"]["total_nodes"] == 2
        assert stats2["basic_stats"]["total_nodes"] == 3

        assert stats1["basic_stats"]["total_edges"] == 1
        assert stats2["basic_stats"]["total_edges"] == 2

    def test_stats_regression_detection(self):
        """Test detecting regressions in graph quality."""
        # Graph with issues
        G_bad = nx.DiGraph()
        G_bad.add_node("N1")  # Missing type - should fail validation
        G_bad.add_node("N2", type="verse", foundational_score=2.0)  # Invalid score

        stats = compute_graph_statistics(G_bad, print_summary=False)
        validation = stats["validation"]

        # Should detect issues
        assert validation["total_errors"] > 0 or validation["total_warnings"] > 0
        assert validation["passed_checks"] < validation["total_checks"]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
