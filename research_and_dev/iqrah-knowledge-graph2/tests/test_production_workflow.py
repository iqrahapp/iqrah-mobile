"""
Integration tests for production knowledge graph workflow.

Tests the complete pipeline from offline data to CBOR export.
"""

import pytest
import tempfile
import shutil
from pathlib import Path

from iqrah.quran_offline import load_quran_offline
from iqrah.morphology.corpus import QuranicArabicCorpus
from iqrah.graph.builder import QuranGraphBuilder
from iqrah.graph.knowledge_builder import KnowledgeGraphBuilder
from iqrah.graph.scoring import calculate_knowledge_scores
from iqrah.content.builder import ContentDatabaseBuilder
from iqrah.content.database import ContentDatabase
from iqrah.export import export_graph_to_cbor, import_graph_from_cbor, inspect_cbor_graph
from iqrah.config import KnowledgeGraphConfig, load_preset


class TestProductionWorkflow:
    """Test complete production workflow."""

    @pytest.fixture
    def temp_dir(self):
        """Create temporary directory for test outputs."""
        temp_dir = tempfile.mkdtemp()
        yield Path(temp_dir)
        shutil.rmtree(temp_dir)

    @pytest.fixture
    def morphology_path(self):
        """Get path to morphology corpus."""
        # Adjust path as needed
        return Path("research_and_dev/data/quranic-arabic-corpus-morphology.csv")

    def test_config_system(self):
        """Test configuration loading and parsing."""
        # Test default config
        config = KnowledgeGraphConfig()
        assert config.name == "default"
        assert config.memorization.enabled == True

        # Test chapter parsing
        config.chapters = "1-3,5"
        chapters = config.parse_chapters()
        assert chapters == [1, 2, 3, 5]

        # Test preset loading
        preset = load_preset("basic")
        assert preset.name == "basic"
        assert preset.memorization.enabled == True

    def test_content_database_creation(self, temp_dir, morphology_path):
        """Test content database creation."""
        if not morphology_path.exists():
            pytest.skip("Morphology corpus not found")

        db_path = temp_dir / "content.db"

        # Build database
        builder = ContentDatabaseBuilder()
        builder.build(
            output_path=str(db_path),
            morphology_corpus_path=str(morphology_path),
            show_progress=False
        )

        # Verify database exists and has data
        assert db_path.exists()

        with ContentDatabase(str(db_path)) as db:
            stats = db.get_statistics()

            # Should have chapters
            assert stats["chapters"] == 114

            # Should have verses
            assert stats["verses"] == 6236

            # Should have words
            assert stats["words"] > 0

            # Test querying
            chapter = db.get_chapter("CHAPTER:1")
            assert chapter is not None
            assert chapter["chapter_number"] == 1

            verse = db.get_verse("VERSE:1:1")
            assert verse is not None
            assert verse["verse_key"] == "1:1"

    def test_dependency_graph_building(self, morphology_path):
        """Test dependency graph building."""
        if not morphology_path.exists():
            pytest.skip("Morphology corpus not found")

        # Load data
        quran = load_quran_offline()
        corpus = QuranicArabicCorpus()
        corpus.load_data(str(morphology_path))

        # Build graph for first chapter only (faster)
        chapters = [quran.chapters[0]]

        builder = QuranGraphBuilder()
        G = builder.build_graph(
            quran=quran,
            corpus=corpus,
            chapters=chapters,
            show_progress=False
        )

        # Verify graph structure
        assert len(G.nodes) > 0
        assert len(G.edges) > 0

        # Should have different node types
        node_types = {data.get("type") for _, data in G.nodes(data=True)}
        assert "chapter" in node_types
        assert "verse" in node_types
        assert "word_instance" in node_types

    def test_knowledge_graph_building(self, morphology_path):
        """Test knowledge graph building with edges."""
        if not morphology_path.exists():
            pytest.skip("Morphology corpus not found")

        # Load data
        quran = load_quran_offline()
        corpus = QuranicArabicCorpus()
        corpus.load_data(str(morphology_path))

        # Build dependency graph
        chapters = [quran.chapters[0]]  # First chapter only
        builder = QuranGraphBuilder()
        G = builder.build_graph(quran, corpus, chapters, show_progress=False)

        initial_edges = len(G.edges)

        # Build knowledge edges
        kb = KnowledgeGraphBuilder(G, quran)
        kb.build_all(
            include_memorization=True,
            include_translation=True,
            include_grammar=False,  # Skip for speed
        )
        kb.compile()

        # Should have added knowledge edges
        assert len(G.edges) > initial_edges

        # Verify edge attributes
        for _, _, data in list(G.edges(data=True))[:10]:
            if data.get("type") != "dependency":
                # Knowledge edges should have distribution info
                assert "dist" in data or "weight" in data

    def test_cbor_export_import(self, temp_dir, morphology_path):
        """Test CBOR export and import (structure-only)."""
        if not morphology_path.exists():
            pytest.skip("Morphology corpus not found")

        # Build minimal graph
        quran = load_quran_offline()
        corpus = QuranicArabicCorpus()
        corpus.load_data(str(morphology_path))

        chapters = [quran.chapters[0]]
        builder = QuranGraphBuilder()
        G = builder.build_graph(quran, corpus, chapters, show_progress=False)

        # Add some knowledge edges
        kb = KnowledgeGraphBuilder(G, quran)
        kb.build_memorization_edges()
        kb.compile()

        # Export to CBOR (structure only)
        cbor_path = temp_dir / "test.cbor.zst"
        export_graph_to_cbor(
            G,
            str(cbor_path),
            show_progress=False
        )

        assert cbor_path.exists()

        # Inspect exported graph
        stats = inspect_cbor_graph(str(cbor_path), sample_size=5, show_full_sample=False)
        assert stats["header"] is not None
        assert stats["header"]["v"] == 2  # Version 2 (structure-only)
        assert stats["header"]["format"] == "structure_only"

        # Import graph
        G_imported = import_graph_from_cbor(str(cbor_path), show_progress=False)

        assert len(G_imported.nodes) == len(G.nodes)
        assert len(G_imported.edges) == len(G.edges)

        # Verify no content in imported graph
        for node_id, data in list(G_imported.nodes(data=True))[:10]:
            # Should NOT have content fields
            assert "arabic" not in data
            assert "translation" not in data
            assert "transliteration" not in data

            # Should have structural fields
            if data.get("type") == "word_instance":
                assert "verse_key" in data or "position" in data

    def test_scoring(self, morphology_path):
        """Test PageRank scoring."""
        if not morphology_path.exists():
            pytest.skip("Morphology corpus not found")

        # Build small graph
        quran = load_quran_offline()
        corpus = QuranicArabicCorpus()
        corpus.load_data(str(morphology_path))

        chapters = [quran.chapters[0]]
        builder = QuranGraphBuilder()
        G = builder.build_graph(quran, corpus, chapters, show_progress=False)

        kb = KnowledgeGraphBuilder(G, quran)
        kb.build_memorization_edges()
        kb.build_translation_edges()
        kb.compile()

        # Calculate scores
        calculate_knowledge_scores(
            G,
            alpha=0.85,
            max_iter=1000,  # Low for testing
        )

        # Verify scores exist
        scored_nodes = 0
        for node_id, data in G.nodes(data=True):
            if "foundational_score" in data:
                scored_nodes += 1
                # Scores should be in [0, 1]
                assert 0.0 <= data["foundational_score"] <= 1.0
                assert 0.0 <= data["influence_score"] <= 1.0

        assert scored_nodes > 0

    def test_complete_pipeline(self, temp_dir, morphology_path):
        """Test complete pipeline: dependency -> knowledge -> score -> export."""
        if not morphology_path.exists():
            pytest.skip("Morphology corpus not found")

        # 1. Build dependency graph
        quran = load_quran_offline()
        corpus = QuranicArabicCorpus()
        corpus.load_data(str(morphology_path))

        chapters = [quran.chapters[0]]  # First chapter for speed
        builder = QuranGraphBuilder()
        G = builder.build_graph(quran, corpus, chapters, show_progress=False)

        # 2. Build knowledge graph
        kb = KnowledgeGraphBuilder(G, quran)
        kb.build_all(
            include_memorization=True,
            include_translation=True,
            include_grammar=False,
        )
        kb.compile()

        # 3. Score
        calculate_knowledge_scores(G, max_iter=1000)

        # 4. Export
        cbor_path = temp_dir / "complete.cbor.zst"
        export_graph_to_cbor(G, str(cbor_path), show_progress=False)

        # 5. Verify export
        assert cbor_path.exists()
        assert cbor_path.stat().st_size > 0

        # 6. Re-import and verify
        G_imported = import_graph_from_cbor(str(cbor_path), show_progress=False)
        assert len(G_imported.nodes) == len(G.nodes)

        # Check scores were preserved
        for node_id in list(G_imported.nodes())[:5]:
            if "foundational_score" in G.nodes[node_id]:
                assert "foundational_score" in G_imported.nodes[node_id]
                assert "influence_score" in G_imported.nodes[node_id]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
