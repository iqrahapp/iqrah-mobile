from collections import defaultdict
from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, List, Optional, Union
import networkx as nx
import numpy as np
from scipy.stats import norm, beta

from .builder import EdgeType


class KnowledgeAxis(Enum):
    """Valid knowledge axes for different node types"""

    MEMORIZATION = "memorization"
    TRANSLATION = "translation"
    TAFSIR = "tafsir"
    TAJWEED = "tajweed"
    CONTEXTUAL_MEMORIZATION = "contextual_memorization"
    MEANING = "meaning"


class NodeType(Enum):
    """Valid node types with their allowed knowledge axes"""

    CHAPTER = {
        KnowledgeAxis.MEMORIZATION,
        KnowledgeAxis.TRANSLATION,
        KnowledgeAxis.TAFSIR,
    }
    VERSE = {
        KnowledgeAxis.MEMORIZATION,
        KnowledgeAxis.TRANSLATION,
        KnowledgeAxis.TAFSIR,
        KnowledgeAxis.TAJWEED,
        KnowledgeAxis.CONTEXTUAL_MEMORIZATION,
    }
    WORD_INSTANCE = {
        KnowledgeAxis.MEMORIZATION,
        KnowledgeAxis.TRANSLATION,
        KnowledgeAxis.TAJWEED,
        KnowledgeAxis.CONTEXTUAL_MEMORIZATION,
    }
    WORD = {KnowledgeAxis.TRANSLATION}
    LEMMA = {KnowledgeAxis.TRANSLATION}
    ROOT = {KnowledgeAxis.MEANING}


@dataclass
class Distribution:
    type: str
    params: Dict[str, float]

    @staticmethod
    def auto(weight: float | None = None) -> "Distribution":
        """
        Create an auto-weighted distribution.

        Args:
            weight: Optional relative weight for this edge. If None,
                   equal weights will be used for all neighboring edges.
        """
        params = {"weight": weight} if weight is not None else {}
        return Distribution("auto", params)

    @staticmethod
    def normal(mean: float, std: float) -> "Distribution":
        return Distribution("normal", {"m": mean, "s": std})

    @staticmethod
    def beta(alpha: float, beta: float) -> "Distribution":
        return Distribution("beta", {"a": alpha, "b": beta})

    def to_dict(self) -> Dict[str, Union[str, float]]:
        return {"dist": self.type, **self.params}


class KnowledgeEdgeManager:
    def __init__(self, graph: nx.DiGraph):
        self.G = graph
        self.pending_edges: Dict[str, List[tuple[str, str, Optional[float]]]] = (
            defaultdict(list)
        )
        self._is_compiled = False

    def _validate_knowledge_node(self, node_id: str) -> tuple[str, str, NodeType]:
        """Validates knowledge node format and type."""
        if self._is_compiled:
            raise RuntimeError("Cannot modify graph after compilation")

        try:
            *base_parts, axis = node_id.split(":")
            base_node = ":".join(base_parts)
        except ValueError:
            raise ValueError(
                f"Invalid node format: {node_id}. Expected format: BASE_NODE:axis"
            )

        if not self.G.has_node(base_node):
            raise ValueError(f"Parent node {base_node} does not exist")

        parent_data = self.G.nodes[base_node]
        try:
            node_type = NodeType[parent_data.get("type", "").upper()]
        except KeyError:
            raise ValueError(
                f"Invalid node type for {base_node}: {parent_data.get('type')}. "
                f"Valid types: {[t.name for t in NodeType]}"
            )

        try:
            knowledge_axis = KnowledgeAxis(axis)
        except ValueError:
            raise ValueError(
                f"Invalid knowledge axis: {axis}. "
                f"Valid axes: {[a.value for a in KnowledgeAxis]}"
            )

        if knowledge_axis not in node_type.value:
            raise ValueError(
                f"Knowledge axis '{axis}' not allowed for node type '{node_type.name}'. "
                f"Allowed axes: {[a.value for a in node_type.value]}"
            )

        return base_node, axis, node_type

    def _ensure_knowledge_node(self, node_id: str) -> None:
        """Ensures knowledge axis node exists and is connected to its parent."""
        if self.G.has_node(node_id):
            return

        base_node, axis, _ = self._validate_knowledge_node(node_id)

        if self.G.has_edge(node_id, base_node):
            raise ValueError(
                f"Dependency edge already exists between {node_id} and {base_node}"
            )

        self.G.add_node(node_id, type="knowledge", axis=axis, parent_node=base_node)
        self.G.add_edge(node_id, base_node, type=EdgeType.DEPENDENCY.value)
        self.pending_edges[base_node].append((node_id, base_node, None))

    def add_knowledge_edge(
        self,
        source: str,
        target: str,
        distribution: Distribution = Distribution.auto(),
        **attributes,
    ) -> None:
        """
        Add a knowledge propagation edge, ensuring knowledge axis nodes exist.

        If the edge already exists, it will be silently skipped.

        Args:
            source: Source node ID (e.g., "VERSE:1:2:memorization")
            target: Target node ID
            distribution: Edge weight distribution (defaults to auto)
            **attributes: Additional edge attributes

        Raises:
            ValueError: If node format/type is invalid
            RuntimeError: If graph is already compiled
        """
        if self._is_compiled:
            raise RuntimeError("Cannot add edges after compilation")

        # Skip if edge already exists (can happen when multiple edge builders add same edge)
        if self.G.has_edge(source, target):
            return

        self._ensure_knowledge_node(source)
        self._ensure_knowledge_node(target)

        self.G.add_edge(
            source,
            target,
            type=EdgeType.KNOWLEDGE.value,
            **attributes,
        )

        if distribution.type == "auto":
            weight = distribution.params.get("weight")
            self.pending_edges[target].append((source, target, weight))
        else:
            self.G.edges[source, target].update(distribution.to_dict())

    def compile(self, strict: bool = True) -> None:
        """
        Compile the graph by computing all pending weights and freeze it.

        Args:
            strict: If True, requires consistent weight specification (all or none)
                   If False, uses mean of specified weights for unspecified ones

        Raises:
            RuntimeError: If already compiled
            ValueError: In strict mode, if weight specification is inconsistent
        """
        if self._is_compiled:
            raise RuntimeError("Graph is already compiled")

        for target, edges in self.pending_edges.items():
            weights = [w for _, _, w in edges]
            has_weights = [w is not None for w in weights]

            if not any(has_weights):
                # No weights specified - use equal distribution
                weight = 1.0 / len(edges)
                normalized_weights = [weight] * len(edges)

            elif all(has_weights):
                # All edges weighted - normalize
                total = sum(w for w in weights if w is not None)
                normalized_weights = [w / total for w in weights]

            elif strict:
                raise ValueError(
                    f"Inconsistent weight specification for target {target} in strict mode: "
                    f"either all edges should have weights or none should. Found: {edges}"
                )

            else:
                # Mixed weights in non-strict mode - use mean for unspecified
                specified_weights = [w for w in weights if w is not None]
                mean_weight = sum(specified_weights) / len(specified_weights)
                filled_weights = [w if w is not None else mean_weight for w in weights]
                total = sum(filled_weights)
                normalized_weights = [w / total for w in filled_weights]

            # Set edge distributions
            for (source, target, _), weight in zip(edges, normalized_weights):
                self.G.edges[source, target].update(
                    Distribution.normal(weight, 0.1).to_dict()
                )

        self.pending_edges.clear()
        self._is_compiled = True

    def is_compiled(self) -> bool:
        """Return whether the graph has been compiled."""
        return self._is_compiled

    def get_stats(self) -> dict[str, Any]:
        """Get statistics about the graph state."""
        return {
            "is_compiled": self._is_compiled,
            "pending_edges": sum(len(edges) for edges in self.pending_edges.values()),
            "nodes_with_pending": len(self.pending_edges),
            "total_nodes": len(self.G.nodes),
        }

    def add_bidirectional_knowledge_edge(
        self,
        node1: str,
        node2: str,
        distribution: Distribution = Distribution.auto(),
    ) -> None:
        """Add bidirectional knowledge edges between nodes."""
        self.add_knowledge_edge(node1, node2, distribution)
        self.add_knowledge_edge(node2, node1, distribution)

    def add_gaussian_window_edges(
        self,
        nodes: List[str],
        window_size: int,
        base_weight: float = 0.5,
        std_scale: float = 0.1,
    ) -> int:
        """
        Add bidirectional edges with gaussian-weighted distributions.

        Args:
            nodes: List of node IDs in sequence
            window_size: Size of the context window (in each direction)
            base_weight: Base weight for edges
            std_scale: Scale for the standard deviation
        """
        weights = self._gaussian_weights(window_size)

        total_edges = 0

        # For each node
        for i, node in enumerate(nodes):
            # Look backwards (left context)
            for j, weight in enumerate(weights):
                left_idx = i - (j + 1)  # j+1 because we don't want self-edge
                if left_idx >= 0:
                    self.add_knowledge_edge(
                        node,
                        nodes[left_idx],
                        Distribution.normal(weight * base_weight, weight * std_scale),
                    )
                    total_edges += 1

            # Look forwards (right context)
            for j, weight in enumerate(weights):
                right_idx = i + j + 1  # j+1 because we don't want self-edge
                if right_idx < len(nodes):
                    self.add_knowledge_edge(
                        node,
                        nodes[right_idx],
                        Distribution.normal(weight * base_weight, weight * std_scale),
                    )
                    total_edges += 1

        return total_edges

    @staticmethod
    def _gaussian_weights(window_size: int) -> np.ndarray:
        """
        Generate gaussian weights for window.
        The weights are normalized so that closest word has weight 1.0.
        """
        x = np.arange(window_size)
        return norm.pdf(x, loc=0, scale=window_size / 3) / norm.pdf(
            0, loc=0, scale=window_size / 3
        )
