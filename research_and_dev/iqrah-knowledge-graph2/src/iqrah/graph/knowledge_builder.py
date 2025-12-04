"""
Knowledge Graph Builder

Production-ready module for building knowledge graphs with learning edges.
Refactored from the experimental notebook code.
"""

import networkx as nx
import logging
from typing import List, Tuple, Optional, Set
from collections import Counter
import time

from .identifiers import NodeIdentifierGenerator as NIG, NodeIdentifierParser as NIP
from .knowledge import Distribution, KnowledgeEdgeManager
from .node_manager import NodeManager
from ..quran_offline import Quran


logger = logging.getLogger(__name__)


class KnowledgeGraphBuilder:
    """
    Builds a knowledge graph for Quranic learning.

    Creates knowledge edges on top of a dependency graph to model:
    - Memorization relationships (hierarchical and contextual)
    - Translation understanding (word-to-verse-to-chapter)
    - Grammar connections (word-lemma-root)
    - Cross-dimensional learning (translation helps memorization)

    Usage:
        builder = KnowledgeGraphBuilder(dependency_graph, quran)
        builder.build_memorization_edges()
        builder.build_translation_edges()
        builder.build_grammar_edges()
        builder.compile()
        builder.save("output.graphml")
    """

    def __init__(self, graph: nx.DiGraph, quran: Quran):
        """
        Initialize the knowledge graph builder.

        Args:
            graph: Base dependency graph (must contain chapter/verse/word nodes)
            quran: Quran data model with offline data
        """
        self.G = graph
        self.quran = quran
        self.edge_manager = KnowledgeEdgeManager(graph)
        self.node_manager = NodeManager(graph)
        self._is_compiled = False

        # Statistics tracking
        self.stats = Counter()

        logger.info(
            f"Initialized KnowledgeGraphBuilder: {len(graph.nodes)} nodes, "
            f"{len(graph.edges)} edges"
        )

    # -------------------------------------------------------------------------
    # Helper Methods
    # -------------------------------------------------------------------------

    def get_nodes_by_type(self, node_type: str) -> Set[str]:
        """Get all nodes of a specific type."""
        return self.node_manager.get_nodes_by_type(node_type)

    def get_verse_words(self, verse_id: str) -> List[str]:
        """Get all word instance IDs for a verse."""
        return self.node_manager.get_verse_words(verse_id)

    def get_chapter_verses(self, chapter_id: str) -> List[str]:
        """Get all verse IDs for a chapter."""
        return self.node_manager.get_chapter_verses(chapter_id)

    def get_word_root(self, word_id: str, cutoff: int = 3) -> Optional[str]:
        """Get root of a word by traversing the graph."""
        for path in nx.all_simple_paths(
            self.G,
            word_id,
            self.node_manager.get_nodes_by_type("root"),
            cutoff=cutoff
        ):
            return path[-1]  # Return first found root
        return None

    def get_all_translatable_nodes(self) -> Set[str]:
        """Get all nodes that can have translation knowledge."""
        return (
            self.node_manager.get_nodes_by_type("word_instance") |
            self.node_manager.get_nodes_by_type("verse")
        )

    def get_duplicated_verses(self) -> List[Tuple[str, List[str]]]:
        """
        Find verses with identical text for connecting.

        Returns:
            List of (text, [verse_keys]) tuples, sorted by frequency
        """
        verse_map = {}
        for verse_id in self.get_nodes_by_type("verse"):
            verse_key = NIP.get_verse_key(verse_id)
            verse = self.quran[verse_key]

            text = verse.text_uthmani_simple
            if text is None:
                continue

            if text not in verse_map:
                verse_map[text] = []
            verse_map[text].append(verse.verse_key)

        # Filter to only duplicates
        duplicates = {text: verses for text, verses in verse_map.items() if len(verses) > 1}

        return sorted(duplicates.items(), key=lambda x: len(x[1]), reverse=True)

    def has_tajweed_rules(self, word_id: str) -> bool:
        """
        Check if word has tajweed rules.

        Note: This is a placeholder for future tajweed integration.
        Currently returns False for all words.
        """
        node = self.G.nodes.get(word_id)
        if not node:
            return False
        return node.get('has_tajweed', False)

    # -------------------------------------------------------------------------
    # Knowledge Edge Building Methods
    # -------------------------------------------------------------------------

    def build_memorization_edges(self) -> int:
        """
        Build standard memorization edges:
        - Words -> Verse (weighted by word length)
        - Verse -> Chapter (weighted by verse length)
        - Context windows for words (Gaussian distribution)

        Returns:
            Number of edges created
        """
        logger.info("Building memorization edges...")
        edges_before = self.stats["edges_created"]

        for chapter_id in self.get_nodes_by_type("chapter"):
            chapter_key = NIP.get_chapter_key(chapter_id)
            chapter = self.quran[chapter_key]

            for verse in chapter.verses:
                verse_id = NIG.for_verse(verse)

                # Verse -> Chapter memorization
                self.edge_manager.add_knowledge_edge(
                    f"{verse_id}:memorization",
                    f"{chapter_id}:memorization",
                    Distribution.auto(weight=verse.get_letters_count())
                )
                self.stats["edges_created"] += 1

                # Collect non-end words
                word_nodes = []
                for word in verse.words:
                    if word.is_end_word():
                        continue

                    word_id = NIG.for_word_instance(word, verse)
                    word_nodes.append(word_id)

                    # Word -> Verse memorization
                    self.edge_manager.add_knowledge_edge(
                        f"{word_id}:memorization",
                        f"{verse_id}:memorization",
                        Distribution.auto(weight=word.get_letters_count())
                    )
                    self.stats["edges_created"] += 1

                # Contextual memorization (Gaussian windows)
                if word_nodes:
                    window_edges = self.edge_manager.add_gaussian_window_edges(
                        [f"{w}:memorization" for w in word_nodes],
                        window_size=3,
                        base_weight=0.5,
                        std_scale=0.15
                    )
                    self.stats["edges_created"] += window_edges

            # Sequential verse memorization
            verse_nodes = [NIG.for_verse(v) for v in chapter.verses]
            if verse_nodes:
                verse_edges = self.edge_manager.add_gaussian_window_edges(
                    [f"{v}:memorization" for v in verse_nodes],
                    window_size=1, # Connect to immediate neighbors
                    base_weight=0.7, # Stronger connection for verses
                    std_scale=0.1
                )
                self.stats["edges_created"] += verse_edges

        edges_created = self.stats["edges_created"] - edges_before
        logger.info(f"Created {edges_created} memorization edges")
        return edges_created

    def build_tajweed_edges(self) -> int:
        """
        Build tajweed learning edges:
        - Tajweed rules -> Memorization (strong impact)
        - Neighboring tajweed connections (for rules spanning words)

        Note: Currently a placeholder. Requires tajweed rule detection.

        Returns:
            Number of edges created
        """
        logger.info("Building tajweed edges...")
        edges_before = self.stats["edges_created"]

        for verse_id in self.get_nodes_by_type("verse"):
            verse_key = NIP.get_verse_key(verse_id)
            verse = self.quran[verse_key]

            words = [w for w in verse.words if not w.is_end_word()]

            for i, word in enumerate(words):
                word_id = NIG.for_word_instance(word, verse)

                if self.has_tajweed_rules(word_id):
                    # Tajweed -> Memorization (strong impact)
                    self.edge_manager.add_knowledge_edge(
                        f"{word_id}:tajweed",
                        f"{word_id}:memorization",
                        Distribution.normal(mean=0.7, std=0.1)
                    )
                    self.stats["edges_created"] += 1

                    # Connect neighboring tajweed nodes
                    if i + 1 < len(words):
                        next_word = words[i + 1]
                        next_word_id = NIG.for_word_instance(next_word, verse)

                        self.edge_manager.add_knowledge_edge(
                            f"{word_id}:tajweed",
                            f"{next_word_id}:tajweed",
                            Distribution.normal(mean=0.3, std=0.1)
                        )
                        self.stats["edges_created"] += 1

        edges_created = self.stats["edges_created"] - edges_before
        logger.info(f"Created {edges_created} tajweed edges")
        return edges_created

    def build_translation_edges(self) -> int:
        """
        Build translation understanding edges:
        - Word -> Verse -> Chapter (hierarchical understanding)
        - Word instance -> Word type (same word learning)
        - Duplicate verse connections (identical verses)
        - Translation -> Memorization (understanding aids memory)

        Returns:
            Number of edges created
        """
        logger.info("Building translation edges...")
        edges_before = self.stats["edges_created"]

        for chapter_id in self.get_nodes_by_type("chapter"):
            chapter_key = NIP.get_chapter_key(chapter_id)
            chapter = self.quran[chapter_key]

            for verse in chapter.verses:
                verse_id = NIG.for_verse(verse)

                # Verse -> Chapter translation
                self.edge_manager.add_knowledge_edge(
                    f"{verse_id}:translation",
                    f"{chapter_id}:translation",
                    Distribution.auto(weight=verse.get_words_count())
                )
                self.stats["edges_created"] += 1

                # Word processing
                for word in verse.words:
                    if word.is_end_word():
                        continue

                    word_instance_id = NIG.for_word_instance(word, verse)
                    word_type_id = NIG.for_word(word)

                    # Word -> Verse translation
                    self.edge_manager.add_knowledge_edge(
                        f"{word_instance_id}:translation",
                        f"{verse_id}:translation",
                        Distribution.auto(weight=word.get_letters_count())
                    )
                    self.stats["edges_created"] += 1

                    # Word instance -> Word type (very high impact)
                    self.edge_manager.add_knowledge_edge(
                        f"{word_instance_id}:translation",
                        f"{word_type_id}:translation",
                        Distribution.normal(mean=0.9, std=0.1)
                    )
                    self.stats["edges_created"] += 1

        # Translation helps memorization
        for node_id in self.get_all_translatable_nodes():
            self.edge_manager.add_knowledge_edge(
                f"{node_id}:translation",
                f"{node_id}:memorization",
                Distribution.normal(mean=0.4, std=0.15)
            )
            self.stats["edges_created"] += 1

        # Connect duplicate verses (identical text)
        duplicate_pairs = 0
        for text, verse_keys in self.get_duplicated_verses():
            for i in range(len(verse_keys)):
                for j in range(i + 1, len(verse_keys)):
                    v1, v2 = verse_keys[i], verse_keys[j]
                    self.edge_manager.add_bidirectional_knowledge_edge(
                        f"{NIG.for_verse(v1)}:translation",
                        f"{NIG.for_verse(v2)}:translation",
                        Distribution.normal(mean=0.9, std=0.1)
                    )
                    self.stats["edges_created"] += 2
                    duplicate_pairs += 1

        logger.info(f"Connected {duplicate_pairs} duplicate verse pairs")

        edges_created = self.stats["edges_created"] - edges_before
        logger.info(f"Created {edges_created} translation edges")
        return edges_created

    def build_grammar_edges(self) -> int:
        """
        Build grammar connection edges:
        - Word <-> Lemma (bidirectional, weighted by lemma length)
        - Lemma <-> Root (bidirectional, strong positive skew)

        Returns:
            Number of edges created
        """
        logger.info("Building grammar edges...")
        edges_before = self.stats["edges_created"]

        for word_id in self.get_nodes_by_type("word"):
            lemma_ids = self.node_manager.get_related_nodes(word_id, successor_type="lemma")

            for lemma_id in lemma_ids:
                _, lemma = lemma_id.split(":", 1)

                # Word <-> Lemma bidirectional translation
                self.edge_manager.add_bidirectional_knowledge_edge(
                    f"{word_id}:translation",
                    f"{lemma_id}:translation",
                    Distribution.auto(weight=len(lemma))
                )
                self.stats["edges_created"] += 2

                # Lemma -> Root
                root_ids = self.node_manager.get_related_nodes(lemma_id, successor_type="root")

                for root_id in root_ids:
                    # Lemma <-> Root meaning
                    self.edge_manager.add_bidirectional_knowledge_edge(
                        f"{lemma_id}:translation",
                        f"{root_id}:meaning",
                        Distribution.beta(alpha=4, beta=2)
                    )
                    self.stats["edges_created"] += 2

        edges_created = self.stats["edges_created"] - edges_before
        logger.info(f"Created {edges_created} grammar edges")
        return edges_created

    def build_deep_understanding_edges(self) -> int:
        """
        Build deep understanding edges:
        - Translation -> Tafsir (for verses with tafsir)
        - Root meaning -> Lemma translation (strong positive skew)

        Returns:
            Number of edges created
        """
        logger.info("Building deep understanding edges...")
        edges_before = self.stats["edges_created"]

        # Translation aids tafsir
        verses_with_tafsir = (
            self.get_nodes_by_type("verse") &
            self.node_manager.get_nodes_by_metadata("has_tafsir")
        )

        for verse_id in verses_with_tafsir:
            self.edge_manager.add_knowledge_edge(
                f"{verse_id}:translation",
                f"{verse_id}:tafsir",
                Distribution.normal(mean=0.3, std=0.1)
            )
            self.stats["edges_created"] += 1

        # Root meanings impact word understanding
        for lemma_id in self.get_nodes_by_type("lemma"):
            root_id = self.get_word_root(lemma_id)
            if root_id:
                self.edge_manager.add_knowledge_edge(
                    f"{root_id}:meaning",
                    f"{lemma_id}:translation",
                    Distribution.beta(alpha=4, beta=2)
                )
                self.stats["edges_created"] += 1

        edges_created = self.stats["edges_created"] - edges_before
        logger.info(f"Created {edges_created} deep understanding edges")
        return edges_created

    # -------------------------------------------------------------------------
    # Builder Workflow
    # -------------------------------------------------------------------------

    def build_all(
        self,
        include_memorization: bool = True,
        include_tajweed: bool = False,
        include_translation: bool = True,
        include_grammar: bool = True,
        include_deep_understanding: bool = False,
    ) -> None:
        """
        Build all knowledge edges based on configuration.

        Args:
            include_memorization: Build memorization edges
            include_tajweed: Build tajweed edges (placeholder)
            include_translation: Build translation edges
            include_grammar: Build grammar edges
            include_deep_understanding: Build deep understanding edges

        Raises:
            RuntimeError: If already compiled
        """
        if self._is_compiled:
            raise RuntimeError("Knowledge graph already compiled. Cannot add more edges.")

        start_time = time.time()
        logger.info("Building knowledge graph...")

        if include_memorization:
            self.build_memorization_edges()

        if include_tajweed:
            self.build_tajweed_edges()

        if include_translation:
            self.build_translation_edges()

        if include_grammar:
            self.build_grammar_edges()

        if include_deep_understanding:
            self.build_deep_understanding_edges()

        elapsed = time.time() - start_time
        logger.info(
            f"Knowledge graph built in {elapsed:.2f}s. "
            f"Created {self.stats['edges_created']} knowledge edges."
        )

    def compile(self) -> None:
        """
        Finalize the knowledge graph by computing all pending weights.

        Must be called after all edge building and before saving.

        Raises:
            RuntimeError: If already compiled
            ValueError: If edge configuration is invalid
        """
        if self._is_compiled:
            raise RuntimeError("Knowledge graph has already been compiled")

        logger.info("Compiling knowledge graph...")

        # Compute all pending edge weights
        self.edge_manager.compile()

        # Validate final graph
        self._validate_compiled_graph()

        # Update metadata
        self.G.graph['knowledge_edges'] = self.stats["edges_created"]
        self.G.graph['knowledge_compiled'] = True

        self._is_compiled = True
        logger.info("Knowledge graph compiled successfully")

    def _validate_compiled_graph(self) -> None:
        """
        Validate the compiled graph.

        Raises:
            ValueError: If edge is missing weight distribution
        """
        for src, dst, data in self.G.edges(data=True):
            if "dist" not in data:
                # Allow dependency edges to be missing weights
                is_dependency = data.get("type") == "dependency"
                if not is_dependency:
                    raise ValueError(
                        f"Edge missing weight distribution after compilation: "
                        f"{src} -> {dst} {{{data}}}"
                    )

    def save(self, filename: str) -> None:
        """
        Save the compiled knowledge graph to GraphML.

        Args:
            filename: Path to save file (must end with .graphml)

        Raises:
            RuntimeError: If not yet compiled
            ValueError: If file format is unsupported
        """
        if not self._is_compiled:
            raise RuntimeError(
                "Cannot save uncompiled knowledge graph. Call compile() first."
            )

        if not filename.endswith(".graphml"):
            raise ValueError("Only .graphml format is supported")

        logger.info(f"Saving knowledge graph to {filename}")

        # Add metadata
        import datetime
        self.G.graph['created_at'] = datetime.datetime.now().isoformat()
        self.G.graph['node_count'] = len(self.G.nodes)
        self.G.graph['edge_count'] = len(self.G.edges)

        # Save
        nx.write_graphml(self.G, filename)

        logger.info(
            f"Saved knowledge graph: {len(self.G.nodes)} nodes, "
            f"{len(self.G.edges)} edges"
        )
