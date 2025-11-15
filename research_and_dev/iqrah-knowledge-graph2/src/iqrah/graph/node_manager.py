# iqrah/graph/node_manager.py

from typing import Set, Dict, List, Any, Optional, Callable
import networkx as nx
from collections import defaultdict


class NodeManager:
    """Manages node queries and metadata."""

    def __init__(self, graph: nx.DiGraph):
        self.G = graph
        self._type_index: Dict[str, Set[str]] = defaultdict(set)
        self._axis_index: Dict[str, Set[str]] = defaultdict(set)
        self._metadata_index: Dict[str, Dict[str, Set[str]]] = defaultdict(
            lambda: defaultdict(set)
        )
        self._build_indices()

    def _build_indices(self) -> None:
        """Build indices for efficient querying."""
        for node, data in self.G.nodes(data=True):
            # Index by type
            if "type" in data:
                self._type_index[data["type"]].add(node)

            # Index by knowledge axis
            if "axis" in data:
                self._axis_index[data["axis"]].add(node)

            # Index other metadata
            for key, value in data.items():
                if isinstance(value, (str, int, float, bool)):
                    self._metadata_index[key][str(value)].add(node)

    def get_nodes_by_type(self, node_type: str) -> Set[str]:
        """Get all nodes of a specific type."""
        return self._type_index.get(node_type, set())

    def get_nodes_by_axis(self, axis: str) -> Set[str]:
        """Get all nodes of a specific knowledge axis."""
        return self._axis_index.get(axis, set())

    def get_nodes_by_metadata(self, key: str, value: Any = None) -> Set[str]:
        from itertools import chain

        """Get all nodes with specific metadata value."""
        by_key = self._metadata_index.get(key, {})

        if value is None:
            # Flatten all sets in by_key and return as a single set
            return set(chain.from_iterable(by_key.values()))

        return by_key.get(str(value), set())

    def get_verse_words(self, verse_id: str) -> List[str]:
        """Get all word instances for a verse."""
        return sorted(
            [n for n in self.G.successors(verse_id) if n.startswith("WORD_INSTANCE")]
        )

    def get_chapter_verses(self, chapter_id: str) -> List[str]:
        """Get all verses for a chapter."""
        return sorted(
            [n for n in self.G.successors(chapter_id) if n.startswith("VERSE")]
        )

    def filter_nodes(self, predicate: Callable[[str, Dict], bool]) -> Set[str]:
        """Get nodes that satisfy a predicate function."""
        return {node for node, data in self.G.nodes(data=True) if predicate(node, data)}

    def get_related_nodes(
        self,
        node_id: str,
        successor_type: str | None = None,
        edge_type: str | None = None,
    ) -> Set[str]:
        successors = self.G.successors(node_id)

        if successor_type:
            successors = {
                n
                for n in successors
                if self.G.nodes[n].get("type").lower() == successor_type.lower()
            }

        if edge_type:
            successors = {
                n
                for n in successors
                if self.G.edges[node_id, n].get("type").lower() == edge_type.lower()
            }

        return set(successors)
