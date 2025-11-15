"""
Knowledge Graph Scoring Module

Calculates foundational and influence scores for nodes using PageRank.

Foundational Score:
    Measures how fundamental a concept is to the knowledge graph.
    Uses personalized PageRank with higher weights for roots, lemmas, chapters.

Influence Score:
    Measures how much a concept influences other concepts.
    Uses reverse PageRank to measure downstream impact.

Both scores are log01-normalized to [0, 1] range for consistent interpretation.
"""

import networkx as nx
import numpy as np
import logging
from typing import Dict, List, Optional
from tqdm import tqdm

from .identifiers import NodeIdentifierParser, NodeType


logger = logging.getLogger(__name__)


# Default weights for personalization by node type
DEFAULT_NODE_TYPE_WEIGHTS = {
    NodeType.ROOT: 3.0,
    NodeType.LEMMA: 2.5,
    NodeType.CHAPTER: 2.0,
    NodeType.VERSE: 1.5,
    NodeType.WORD: 1.0,
    NodeType.WORD_INSTANCE: 0.5,
}


class KnowledgeGraphScoring:
    """
    Calculate foundational and influence scores for knowledge graphs.

    Uses personalized PageRank to compute scores that reflect:
    - Foundational importance (how fundamental a concept is)
    - Influence (how much impact a concept has on others)
    """

    def __init__(
        self,
        graph: nx.DiGraph,
        node_type_weights: Optional[Dict[NodeType, float]] = None,
    ):
        """
        Initialize scoring with a knowledge graph.

        Args:
            graph: NetworkX DiGraph (should be compiled knowledge graph)
            node_type_weights: Custom weights for node types (default: DEFAULT_NODE_TYPE_WEIGHTS)
        """
        self.G = graph
        self.node_type_weights = node_type_weights or DEFAULT_NODE_TYPE_WEIGHTS

        logger.info(f"Initialized scoring for graph: {len(graph.nodes)} nodes, {len(graph.edges)} edges")

    def calculate_scores(
        self,
        alpha: float = 0.85,
        max_iter: int = 50000,
        personalize_foundational: bool = True,
        personalize_influence: bool = False,
    ) -> None:
        """
        Calculate and store foundational and influence scores.

        Modifies graph in place by adding:
        - G.nodes[n]["foundational_score"]
        - G.nodes[n]["influence_score"]

        Args:
            alpha: PageRank damping factor (0.85 default)
            max_iter: Maximum iterations for PageRank
            personalize_foundational: Use personalized PageRank for foundational
            personalize_influence: Use personalized PageRank for influence

        Raises:
            ValueError: If graph has no edges
        """
        if not self.G.edges():
            raise ValueError("Cannot score graph with no edges")

        logger.info("Calculating knowledge scores...")

        # 1. Build knowledge-only graph with expected weights
        knowledge_graph = self._build_knowledge_graph()

        # 2. Create personalization vectors
        nodes_list = list(knowledge_graph.nodes())

        if personalize_foundational:
            pers_foundational = self._create_personalized_vector(nodes_list)
        else:
            pers_foundational = None

        if personalize_influence:
            pers_influence = self._create_personalized_vector(nodes_list)
        else:
            pers_influence = None

        # 3. Calculate PageRank scores
        logger.info("Running foundational PageRank...")
        pr_foundational = nx.pagerank(
            knowledge_graph,
            alpha=alpha,
            personalization=pers_foundational,
            dangling=pers_foundational,
            weight="weight",
            max_iter=max_iter,
        )

        logger.info("Running influence PageRank (reverse graph)...")
        pr_influence = nx.pagerank(
            knowledge_graph.reverse(copy=False),
            alpha=alpha,
            personalization=pers_influence,
            dangling=pers_influence,
            weight="weight",
            max_iter=max_iter,
        )

        # 4. Normalize scores to [0, 1] using log01
        f_arr = np.array([float(pr_foundational.get(n, 0.0)) for n in nodes_list], dtype=float)
        i_arr = np.array([float(pr_influence.get(n, 0.0)) for n in nodes_list], dtype=float)

        f_norm = self._log01_normalize(f_arr)
        i_norm = self._log01_normalize(i_arr)

        # 5. Write scores to graph
        logger.info("Writing scores to graph...")
        for n, f, i in zip(nodes_list, f_norm, i_norm):
            self.G.nodes[n]["foundational_score"] = float(f)
            self.G.nodes[n]["influence_score"] = float(i)

        logger.info("Knowledge scores calculated successfully")

        # Log statistics
        self._log_score_statistics()

    def _build_knowledge_graph(self) -> nx.DiGraph:
        """
        Build knowledge-only subgraph with expected edge weights.

        Filters to only knowledge edges and computes expected weights
        from distribution parameters.

        Returns:
            NetworkX DiGraph with knowledge edges and weights
        """
        logger.info("Building knowledge subgraph...")

        knowledge_edges = []

        for u, v, data in tqdm(
            self.G.edges(data=True),
            desc="Building knowledge graph",
            leave=False
        ):
            # Compute expected weight from distribution
            weight = self._expected_edge_weight(data)

            if weight > 0.0:
                knowledge_edges.append((u, v, {"weight": weight}))

        knowledge_graph = nx.DiGraph()
        knowledge_graph.add_nodes_from(self.G.nodes())
        knowledge_graph.add_edges_from(knowledge_edges)

        logger.info(
            f"Knowledge graph: {len(knowledge_graph.nodes)} nodes, "
            f"{len(knowledge_graph.edges)} edges"
        )

        return knowledge_graph

    def _expected_edge_weight(self, data: dict) -> float:
        """
        Calculate expected edge weight from distribution parameters.

        Args:
            data: Edge data dict with distribution info

        Returns:
            Expected weight (0.0-1.0)
        """
        dist = data.get("dist")

        if dist == "normal":
            m = float(data.get("m", 0.0))
            return np.clip(m, 0.0, 1.0)

        elif dist == "beta":
            a = float(data.get("a", 1.0))
            b = float(data.get("b", 1.0))
            denom = a + b
            return (a / denom) if denom > 0 else 0.0

        elif dist in ("auto", "constant"):
            weight = float(data.get("weight", 1.0))
            # Normalize if probability-like
            if data.get("probability_like", True):
                return np.clip(weight, 0.0, 1.0)
            else:
                return max(0.0, weight)

        else:
            # Unknown distribution, default to 1.0
            return 1.0

    def _create_personalized_vector(
        self,
        nodes_list: List[str]
    ) -> Dict[str, float]:
        """
        Create personalized PageRank vector based on node types.

        Higher weights for:
        - Roots (most fundamental)
        - Lemmas
        - Chapters

        Lower weights for:
        - Word instances (most specific)

        Args:
            nodes_list: List of node IDs

        Returns:
            Dict mapping node_id -> personalization weight (normalized)
        """
        node_weights = {}
        total = 0.0

        for node_id in nodes_list:
            try:
                node_type, _ = NodeIdentifierParser.parse(node_id)
                weight = self.node_type_weights.get(node_type, 1.0)
            except Exception:
                weight = 1.0

            # Ensure valid weight
            if weight < 0 or not np.isfinite(weight):
                weight = 0.0

            node_weights[node_id] = weight
            total += weight

        # Normalize
        if total == 0.0:
            uniform = 1.0 / max(1, len(node_weights))
            return {n: uniform for n in node_weights}

        return {n: w / total for n, w in node_weights.items()}

    def _log01_normalize(self, arr: np.ndarray, scale: Optional[float] = None) -> np.ndarray:
        """
        Normalize array to [0, 1] using log transformation.

        Process:
        1. Clip to >= 0
        2. Apply log1p(arr * scale)
        3. Min-max normalize to [0, 1]

        Args:
            arr: NumPy array to normalize
            scale: Scaling factor (default: 1/median)

        Returns:
            Normalized array in [0, 1] range
        """
        arr = np.clip(arr, 0.0, None)

        if arr.size == 0:
            return arr

        # Auto-scale based on median to avoid collapsing distribution
        if scale is None:
            positives = arr[arr > 0]
            med = np.median(positives) if positives.size else 0.0
            scale = (1.0 / med) if med > 0 else 1e9

        # Log transformation
        x = np.log1p(arr * scale)

        # Min-max normalization
        xmin = np.min(x)
        denom = np.ptp(x)  # peak-to-peak (max - min)

        if not np.isfinite(xmin) or not np.isfinite(denom) or denom == 0:
            return np.zeros_like(arr)

        return (x - xmin) / denom

    def _log_score_statistics(self) -> None:
        """Log statistics about computed scores."""
        foundational_scores = []
        influence_scores = []

        for node_id, data in self.G.nodes(data=True):
            if "foundational_score" in data:
                foundational_scores.append(data["foundational_score"])
            if "influence_score" in data:
                influence_scores.append(data["influence_score"])

        if foundational_scores:
            f_arr = np.array(foundational_scores)
            logger.info(
                f"Foundational scores: "
                f"mean={f_arr.mean():.4f}, "
                f"std={f_arr.std():.4f}, "
                f"min={f_arr.min():.4f}, "
                f"max={f_arr.max():.4f}"
            )

        if influence_scores:
            i_arr = np.array(influence_scores)
            logger.info(
                f"Influence scores: "
                f"mean={i_arr.mean():.4f}, "
                f"std={i_arr.std():.4f}, "
                f"min={i_arr.min():.4f}, "
                f"max={i_arr.max():.4f}"
            )

    def get_top_foundational_nodes(self, n: int = 10) -> List[tuple]:
        """
        Get top N most foundational nodes.

        Args:
            n: Number of nodes to return

        Returns:
            List of (node_id, score) tuples, sorted by score descending
        """
        nodes_with_scores = [
            (node_id, data.get("foundational_score", 0.0))
            for node_id, data in self.G.nodes(data=True)
        ]

        return sorted(nodes_with_scores, key=lambda x: x[1], reverse=True)[:n]

    def get_top_influential_nodes(self, n: int = 10) -> List[tuple]:
        """
        Get top N most influential nodes.

        Args:
            n: Number of nodes to return

        Returns:
            List of (node_id, score) tuples, sorted by score descending
        """
        nodes_with_scores = [
            (node_id, data.get("influence_score", 0.0))
            for node_id, data in self.G.nodes(data=True)
        ]

        return sorted(nodes_with_scores, key=lambda x: x[1], reverse=True)[:n]


def calculate_knowledge_scores(
    graph: nx.DiGraph,
    alpha: float = 0.85,
    max_iter: int = 50000,
    node_type_weights: Optional[Dict[NodeType, float]] = None,
    personalize_foundational: bool = True,
    personalize_influence: bool = False,
) -> None:
    """
    Convenience function to calculate knowledge scores.

    Modifies graph in place by adding foundational_score and influence_score
    to all nodes.

    Args:
        graph: NetworkX DiGraph (compiled knowledge graph)
        alpha: PageRank damping factor
        max_iter: Maximum PageRank iterations
        node_type_weights: Custom node type weights
        personalize_foundational: Use personalized PageRank for foundational
        personalize_influence: Use personalized PageRank for influence
    """
    scorer = KnowledgeGraphScoring(graph, node_type_weights)
    scorer.calculate_scores(
        alpha=alpha,
        max_iter=max_iter,
        personalize_foundational=personalize_foundational,
        personalize_influence=personalize_influence,
    )
