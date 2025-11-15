# iqrah/graph/builder.py

from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, List, Tuple, Optional, Set
from networkx import DiGraph
from tqdm import tqdm

from iqrah.graph.identifiers import NodeIdentifierGenerator
from iqrah.morphology.corpus import QuranMorphologyCorpus
from ..morphology.enums import SegmentType
from ..quran_api.models import *


class EdgeType(Enum):
    DEPENDENCY = "dependency"
    KNOWLEDGE = "knowledge"


class NodeRegistry:
    """Manages node creation and lookup."""

    def __init__(self):
        self._nodes: Dict[str, Dict[str, Any]] = {}
        self._node_types: Set[str] = set()

    def register(self, node_id: str, node_type: str, **attributes) -> None:
        if node_id not in self._nodes:
            self._nodes[node_id] = {"type": node_type, **attributes}
            self._node_types.add(node_type)

    def get_nodes(self) -> List[Tuple[str, Dict[str, Any]]]:
        return list(self._nodes.items())

    def exists(self, node_id: str) -> bool:
        return node_id in self._nodes


class EdgeRegistry:
    """Manages edge creation and lookup."""

    def __init__(self):
        self._edges: Dict[Tuple[str, str], Dict[str, Any]] = {}

    def register(
        self, source: str, target: str, edge_type: EdgeType, **attributes
    ) -> None:
        edge = (source, target)
        if edge not in self._edges:
            self._edges[edge] = {"type": edge_type.value, **attributes}

    def get_edges(self) -> List[Tuple[str, str, Dict[str, Any]]]:
        return [(src, tgt, attr) for (src, tgt), attr in self._edges.items()]


class QuranGraphBuilder:
    """Builds a directed graph representation of Quranic text structure."""

    def __init__(self):
        self.G = DiGraph()
        self.nodes = NodeRegistry()
        self.edges = EdgeRegistry()

    def build_graph(
        self,
        quran: Quran,
        corpus: QuranMorphologyCorpus,
        chapters: Optional[List[Chapter]] = None,
        show_progress: bool = True,
    ) -> DiGraph:
        """
        Builds the Quran graph from the provided data.

        Args:
            quran: Quran object containing the text
            corpus: Morphology corpus
            chapters: Optional list of specific chapters to process
            show_progress: Whether to show progress bars

        Returns:
            DiGraph: Constructed graph
        """
        chapters_to_process = chapters or quran.chapters

        iterator = (
            tqdm(chapters_to_process, desc="Processing chapters")
            if show_progress
            else chapters_to_process
        )

        try:
            for chapter in iterator:
                self._process_chapter(chapter, corpus, show_progress)
        except Exception as e:
            raise ValueError(f"Error processing chapter: {str(e)}") from e

        # Build final graph
        self.G.add_nodes_from(self.nodes.get_nodes())
        self.G.add_edges_from(self.edges.get_edges())
        return self.G

    def _process_chapter(
        self, chapter: Chapter, corpus: QuranMorphologyCorpus, show_progress: bool
    ) -> None:
        """Process a single chapter."""
        chapter_id = NodeIdentifierGenerator.for_chapter(chapter)
        self.nodes.register(chapter_id, "chapter", name=chapter.name_simple)

        verses = (
            tqdm(chapter.verses, desc=f"Processing verses in Chapter {chapter.id}")
            if show_progress
            else chapter.verses
        )

        for verse_idx, verse in enumerate(verses):
            self._process_verse(verse, verse_idx, chapter_id, chapter, corpus)

    def _process_verse(
        self,
        verse: Verse,
        verse_idx: int,
        chapter_id: str,
        chapter: Chapter,
        corpus: QuranMorphologyCorpus,
    ) -> None:
        """Process a single verse."""
        verse_id = NodeIdentifierGenerator.for_verse(verse)
        self.nodes.register(
            verse_id,
            "verse",
        )

        self.edges.register(chapter_id, verse_id, EdgeType.DEPENDENCY)

        # Link to previous verse if exists
        if verse_idx > 0:
            prev_verse = chapter.verses[verse_idx - 1]
            prev_verse_id = NodeIdentifierGenerator.for_verse(prev_verse)
            self.edges.register(verse_id, prev_verse_id, EdgeType.DEPENDENCY)

        # Process all words except the last one (typically end marker)
        for word_idx, word in enumerate(verse.words[:-1]):
            if self._should_process_word(word):
                self._process_word(word, verse, word_idx, verse_id, chapter.id, corpus)

    def _should_process_word(self, word: Word) -> bool:
        """Determine if a word should be processed."""
        return word.char_type_name != "end"

    def _process_word(
        self,
        word: Word,
        verse: Verse,
        word_idx: int,
        verse_id: str,
        chapter_id: int,
        corpus: QuranMorphologyCorpus,
    ) -> None:
        """Process a single word."""
        # Create word instance node
        word_instance_id = NodeIdentifierGenerator.for_word_instance(word, verse)
        self.nodes.register(
            word_instance_id,
            "word_instance",
        )
        self.edges.register(verse_id, word_instance_id, EdgeType.DEPENDENCY)

        # Link to previous word instance
        if word_idx > 0:
            prev_word_instance = verse.words[word_idx - 1]
            prev_word_instance_id = NodeIdentifierGenerator.for_word_instance(
                prev_word_instance, verse
            )
            self.edges.register(
                word_instance_id,
                prev_word_instance_id,
                EdgeType.DEPENDENCY,
            )

        # Create word node
        word_id = NodeIdentifierGenerator.for_word(word)
        self.nodes.register(word_id, "word")
        self.edges.register(word_instance_id, word_id, EdgeType.DEPENDENCY)

        try:
            # Process morphological segments using chapter_id instead of verse.chapter.id
            segments = corpus[chapter_id, verse.verse_number, word.position, :]
            self._process_segments(segments, word_id)
        except Exception as e:
            raise ValueError(
                f"Error processing segments for word {word.text} in verse {verse.verse_key}: {str(e)}"
            ) from e

    def _process_segments(self, segments: List[Any], word_id: str) -> None:
        """Process morphological segments for a word."""
        for segment in segments:
            if segment.lemma:
                # Create and link lemma node
                lemma_id = NodeIdentifierGenerator.for_lemma(segment.lemma)
                self.nodes.register(lemma_id, "lemma")
                self.edges.register(word_id, lemma_id, EdgeType.DEPENDENCY)

                if segment.root:
                    # Create and link root node
                    root_id = NodeIdentifierGenerator.for_root(segment.root)
                    self.nodes.register(root_id, "root")
                    self.edges.register(lemma_id, root_id, EdgeType.DEPENDENCY)
