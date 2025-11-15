"""
Knowledge Graph Statistics and Validation

Provides comprehensive statistics and validation for knowledge graphs.
Tracks metrics for quality assurance and change detection.
"""

import networkx as nx
import logging
from typing import Dict, List, Any, Optional, Tuple
from collections import Counter, defaultdict
import json
from pathlib import Path
from datetime import datetime

from .identifiers import NodeIdentifierParser, NodeType


logger = logging.getLogger(__name__)


class GraphStatistics:
    """
    Comprehensive statistics and validation for knowledge graphs.

    Tracks:
    - Node/edge counts by type
    - Score distributions
    - Top-ranked nodes
    - Edge type distributions
    - Graph connectivity metrics
    - Validation metrics
    """

    def __init__(self, graph: nx.DiGraph):
        """
        Initialize statistics calculator.

        Args:
            graph: NetworkX DiGraph to analyze
        """
        self.G = graph
        self.stats = {}
        self._computed = False

    def compute_all(self) -> Dict[str, Any]:
        """
        Compute all statistics.

        Returns:
            Dict with all statistics
        """
        logger.info("Computing graph statistics...")

        self.stats = {
            "metadata": self._compute_metadata(),
            "basic_stats": self._compute_basic_stats(),
            "node_stats": self._compute_node_stats(),
            "edge_stats": self._compute_edge_stats(),
            "knowledge_edge_stats": self._compute_knowledge_edge_stats(),
            "score_stats": self._compute_score_stats(),
            "top_nodes": self._compute_top_nodes(),
            "connectivity": self._compute_connectivity_stats(),
            "validation": self._compute_validation_metrics(),
        }

        self._computed = True
        logger.info("Statistics computation complete")

        return self.stats

    def _compute_metadata(self) -> Dict[str, Any]:
        """Compute metadata about the graph."""
        return {
            "timestamp": datetime.now().isoformat(),
            "directed": self.G.is_directed(),
            "multigraph": self.G.is_multigraph(),
            "graph_attributes": dict(self.G.graph),
        }

    def _compute_basic_stats(self) -> Dict[str, Any]:
        """Compute basic graph statistics."""
        return {
            "total_nodes": len(self.G.nodes),
            "total_edges": len(self.G.edges),
            "density": nx.density(self.G),
            "is_connected": nx.is_weakly_connected(self.G) if self.G.is_directed() else nx.is_connected(self.G),
        }

    def _compute_node_stats(self) -> Dict[str, Any]:
        """Compute node statistics by type."""
        node_types = Counter()
        node_attrs = defaultdict(list)

        for node_id, data in self.G.nodes(data=True):
            node_type = data.get("type", "unknown")
            node_types[node_type] += 1

            # Track attributes per type
            for key in data.keys():
                node_attrs[f"{node_type}_{key}"].append(1)

        return {
            "by_type": dict(node_types),
            "attribute_coverage": {
                key: len(values) for key, values in node_attrs.items()
            },
        }

    def _compute_edge_stats(self) -> Dict[str, Any]:
        """Compute edge statistics."""
        edge_types = Counter()
        edge_attrs = defaultdict(list)

        for u, v, data in self.G.edges(data=True):
            edge_type = data.get("type", "unknown")
            edge_types[edge_type] += 1

            # Track attributes
            for key, value in data.items():
                edge_attrs[key].append(value)

        return {
            "by_type": dict(edge_types),
            "attribute_coverage": {
                key: len(values) for key, values in edge_attrs.items()
            },
        }

    def _compute_knowledge_edge_stats(self) -> Dict[str, Any]:
        """Compute knowledge edge statistics (by axis/dimension)."""
        knowledge_axes = Counter()
        distribution_types = Counter()

        for u, v, data in self.G.edges(data=True):
            # Parse knowledge axis from edge ID
            if ":" in str(u) and ":" in str(v):
                # Extract axis from node IDs (e.g., "VERSE:1:1:memorization")
                u_parts = str(u).split(":")
                v_parts = str(v).split(":")

                if len(u_parts) > 1 and len(v_parts) > 1:
                    # Last part might be the axis
                    u_axis = u_parts[-1] if u_parts[-1] in [
                        "memorization", "translation", "tafsir", "tajweed",
                        "contextual_memorization", "meaning", "grammar"
                    ] else None

                    v_axis = v_parts[-1] if v_parts[-1] in [
                        "memorization", "translation", "tafsir", "tajweed",
                        "contextual_memorization", "meaning", "grammar"
                    ] else None

                    if u_axis:
                        knowledge_axes[u_axis] += 1
                    if v_axis and v_axis != u_axis:
                        knowledge_axes[v_axis] += 1

            # Distribution types
            dist_type = data.get("dist")
            if dist_type:
                distribution_types[dist_type] += 1

        return {
            "by_axis": dict(knowledge_axes),
            "by_distribution": dict(distribution_types),
            "total_knowledge_edges": sum(knowledge_axes.values()),
        }

    def _compute_score_stats(self) -> Dict[str, Any]:
        """Compute score distribution statistics."""
        import numpy as np

        foundational_scores = []
        influence_scores = []

        for node_id, data in self.G.nodes(data=True):
            if "foundational_score" in data:
                foundational_scores.append(data["foundational_score"])
            if "influence_score" in data:
                influence_scores.append(data["influence_score"])

        def compute_distribution(scores: List[float]) -> Dict[str, float]:
            if not scores:
                return {}
            arr = np.array(scores)
            return {
                "count": len(scores),
                "mean": float(np.mean(arr)),
                "std": float(np.std(arr)),
                "min": float(np.min(arr)),
                "max": float(np.max(arr)),
                "median": float(np.median(arr)),
                "q25": float(np.percentile(arr, 25)),
                "q75": float(np.percentile(arr, 75)),
            }

        return {
            "foundational": compute_distribution(foundational_scores),
            "influence": compute_distribution(influence_scores),
            "scored_nodes": len(foundational_scores),
            "scored_percentage": len(foundational_scores) / len(self.G.nodes) * 100 if self.G.nodes else 0,
        }

    def _compute_top_nodes(self, n: int = 20) -> Dict[str, Any]:
        """Compute top-ranked nodes by various metrics."""
        # Top by foundational score
        foundational_ranked = sorted(
            [
                (node_id, data.get("foundational_score", 0), data.get("type", "unknown"))
                for node_id, data in self.G.nodes(data=True)
            ],
            key=lambda x: x[1],
            reverse=True
        )[:n]

        # Top by influence score
        influence_ranked = sorted(
            [
                (node_id, data.get("influence_score", 0), data.get("type", "unknown"))
                for node_id, data in self.G.nodes(data=True)
            ],
            key=lambda x: x[1],
            reverse=True
        )[:n]

        # Top by degree
        degree_ranked = sorted(
            [
                (node_id, self.G.degree(node_id), self.G.nodes[node_id].get("type", "unknown"))
                for node_id in self.G.nodes()
            ],
            key=lambda x: x[1],
            reverse=True
        )[:n]

        # Extract top chapters, verses
        top_chapters = self._get_top_by_type("chapter", n)
        top_verses = self._get_top_by_type("verse", n)
        top_words = self._get_top_by_type("word", n)
        top_lemmas = self._get_top_by_type("lemma", n)
        top_roots = self._get_top_by_type("root", n)

        return {
            "by_foundational_score": [
                {"node_id": nid, "score": score, "type": ntype}
                for nid, score, ntype in foundational_ranked
            ],
            "by_influence_score": [
                {"node_id": nid, "score": score, "type": ntype}
                for nid, score, ntype in influence_ranked
            ],
            "by_degree": [
                {"node_id": nid, "degree": degree, "type": ntype}
                for nid, degree, ntype in degree_ranked
            ],
            "top_chapters": top_chapters,
            "top_verses": top_verses,
            "top_words": top_words,
            "top_lemmas": top_lemmas,
            "top_roots": top_roots,
        }

    def _get_top_by_type(self, node_type: str, n: int = 10) -> List[Dict[str, Any]]:
        """Get top N nodes of a specific type by foundational score."""
        nodes_of_type = [
            (node_id, data.get("foundational_score", 0))
            for node_id, data in self.G.nodes(data=True)
            if data.get("type") == node_type
        ]

        ranked = sorted(nodes_of_type, key=lambda x: x[1], reverse=True)[:n]

        return [
            {"node_id": nid, "score": score}
            for nid, score in ranked
        ]

    def _compute_connectivity_stats(self) -> Dict[str, Any]:
        """Compute graph connectivity statistics."""
        # In-degree and out-degree distributions
        in_degrees = dict(self.G.in_degree())
        out_degrees = dict(self.G.out_degree())

        import numpy as np

        in_degree_values = list(in_degrees.values())
        out_degree_values = list(out_degrees.values())

        return {
            "avg_in_degree": float(np.mean(in_degree_values)) if in_degree_values else 0,
            "avg_out_degree": float(np.mean(out_degree_values)) if out_degree_values else 0,
            "max_in_degree": max(in_degree_values) if in_degree_values else 0,
            "max_out_degree": max(out_degree_values) if out_degree_values else 0,
            "weakly_connected_components": nx.number_weakly_connected_components(self.G) if self.G.is_directed() else nx.number_connected_components(self.G),
        }

    def _compute_validation_metrics(self) -> Dict[str, Any]:
        """Compute validation metrics for quality assurance."""
        validation = {
            "errors": [],
            "warnings": [],
            "checks": {},
        }

        # Check 1: All nodes have types
        nodes_without_type = [
            node_id for node_id, data in self.G.nodes(data=True)
            if "type" not in data
        ]
        validation["checks"]["all_nodes_have_type"] = len(nodes_without_type) == 0
        if nodes_without_type:
            validation["warnings"].append(
                f"{len(nodes_without_type)} nodes missing 'type' attribute"
            )

        # Check 2: All knowledge edges have distributions
        edges_without_dist = [
            (u, v) for u, v, data in self.G.edges(data=True)
            if data.get("type") != "dependency" and "dist" not in data and "weight" not in data
        ]
        validation["checks"]["all_knowledge_edges_have_dist"] = len(edges_without_dist) == 0
        if edges_without_dist:
            validation["errors"].append(
                f"{len(edges_without_dist)} knowledge edges missing distribution/weight"
            )

        # Check 3: Scores are in valid range [0, 1]
        invalid_scores = []
        for node_id, data in self.G.nodes(data=True):
            for score_key in ["foundational_score", "influence_score"]:
                if score_key in data:
                    score = data[score_key]
                    if not (0 <= score <= 1):
                        invalid_scores.append((node_id, score_key, score))

        validation["checks"]["all_scores_in_range"] = len(invalid_scores) == 0
        if invalid_scores:
            validation["errors"].append(
                f"{len(invalid_scores)} scores out of range [0, 1]"
            )

        # Check 4: Graph is weakly connected
        is_connected = nx.is_weakly_connected(self.G) if self.G.is_directed() else nx.is_connected(self.G)
        validation["checks"]["graph_is_connected"] = is_connected
        if not is_connected:
            validation["warnings"].append("Graph is not weakly connected")

        # Check 5: Expected node type ratios
        node_counts = self.stats.get("node_stats", {}).get("by_type", {})
        word_instances = node_counts.get("word_instance", 0)
        verses = node_counts.get("verse", 0)

        # Average ~20-30 words per verse in Quran
        if verses > 0:
            words_per_verse = word_instances / verses
            validation["checks"]["words_per_verse_ratio"] = 15 <= words_per_verse <= 35
            if not (15 <= words_per_verse <= 35):
                validation["warnings"].append(
                    f"Unusual words/verse ratio: {words_per_verse:.1f} (expected 15-35)"
                )

        # Summary
        validation["total_errors"] = len(validation["errors"])
        validation["total_warnings"] = len(validation["warnings"])
        validation["passed_checks"] = sum(1 for v in validation["checks"].values() if v)
        validation["total_checks"] = len(validation["checks"])

        return validation

    def export_to_json(self, output_path: str) -> None:
        """
        Export statistics to JSON file.

        Args:
            output_path: Path to output JSON file
        """
        if not self._computed:
            self.compute_all()

        output_file = Path(output_path)
        output_file.parent.mkdir(parents=True, exist_ok=True)

        with open(output_file, 'w') as f:
            json.dump(self.stats, f, indent=2)

        logger.info(f"Statistics exported to {output_path}")

    def print_summary(self) -> None:
        """Print a human-readable summary of statistics."""
        if not self._computed:
            self.compute_all()

        print("\n" + "=" * 80)
        print("KNOWLEDGE GRAPH STATISTICS SUMMARY")
        print("=" * 80)

        # Basic stats
        basic = self.stats["basic_stats"]
        print(f"\n📊 Basic Statistics:")
        print(f"  Total Nodes: {basic['total_nodes']:,}")
        print(f"  Total Edges: {basic['total_edges']:,}")
        print(f"  Density: {basic['density']:.6f}")
        print(f"  Connected: {basic['is_connected']}")

        # Node stats
        node_stats = self.stats["node_stats"]["by_type"]
        print(f"\n📦 Nodes by Type:")
        for node_type, count in sorted(node_stats.items(), key=lambda x: x[1], reverse=True):
            print(f"  {node_type:20s}: {count:,}")

        # Edge stats
        edge_stats = self.stats["edge_stats"]["by_type"]
        print(f"\n🔗 Edges by Type:")
        for edge_type, count in sorted(edge_stats.items(), key=lambda x: x[1], reverse=True):
            print(f"  {edge_type:20s}: {count:,}")

        # Knowledge edge stats
        knowledge_stats = self.stats["knowledge_edge_stats"]
        print(f"\n🧠 Knowledge Edges by Axis:")
        for axis, count in sorted(knowledge_stats["by_axis"].items(), key=lambda x: x[1], reverse=True):
            print(f"  {axis:20s}: {count:,}")

        print(f"\n📈 Distribution Types:")
        for dist_type, count in sorted(knowledge_stats["by_distribution"].items(), key=lambda x: x[1], reverse=True):
            print(f"  {dist_type:20s}: {count:,}")

        # Score stats
        score_stats = self.stats["score_stats"]
        if score_stats.get("foundational"):
            print(f"\n⭐ Foundational Scores:")
            f = score_stats["foundational"]
            print(f"  Count: {f['count']:,}")
            print(f"  Mean: {f['mean']:.4f}")
            print(f"  Std: {f['std']:.4f}")
            print(f"  Range: [{f['min']:.4f}, {f['max']:.4f}]")
            print(f"  Median: {f['median']:.4f}")

        if score_stats.get("influence"):
            print(f"\n🎯 Influence Scores:")
            i = score_stats["influence"]
            print(f"  Count: {i['count']:,}")
            print(f"  Mean: {i['mean']:.4f}")
            print(f"  Std: {i['std']:.4f}")
            print(f"  Range: [{i['min']:.4f}, {i['max']:.4f}]")
            print(f"  Median: {i['median']:.4f}")

        # Top nodes
        top_nodes = self.stats["top_nodes"]

        print(f"\n🏆 Top 10 Most Foundational Nodes:")
        for i, node in enumerate(top_nodes["by_foundational_score"][:10], 1):
            print(f"  {i:2d}. {node['node_id']:40s} ({node['type']:15s}) - {node['score']:.4f}")

        print(f"\n🌟 Top 10 Most Influential Nodes:")
        for i, node in enumerate(top_nodes["by_influence_score"][:10], 1):
            print(f"  {i:2d}. {node['node_id']:40s} ({node['type']:15s}) - {node['score']:.4f}")

        print(f"\n📖 Top 5 Most Important Chapters:")
        for i, node in enumerate(top_nodes["top_chapters"][:5], 1):
            print(f"  {i}. {node['node_id']:20s} - {node['score']:.4f}")

        print(f"\n📜 Top 5 Most Important Verses:")
        for i, node in enumerate(top_nodes["top_verses"][:5], 1):
            print(f"  {i}. {node['node_id']:20s} - {node['score']:.4f}")

        print(f"\n🔤 Top 5 Most Important Roots:")
        for i, node in enumerate(top_nodes["top_roots"][:5], 1):
            print(f"  {i}. {node['node_id']:20s} - {node['score']:.4f}")

        # Connectivity
        conn = self.stats["connectivity"]
        print(f"\n🔗 Connectivity:")
        print(f"  Avg In-Degree: {conn['avg_in_degree']:.2f}")
        print(f"  Avg Out-Degree: {conn['avg_out_degree']:.2f}")
        print(f"  Max In-Degree: {conn['max_in_degree']}")
        print(f"  Max Out-Degree: {conn['max_out_degree']}")
        print(f"  Weakly Connected Components: {conn['weakly_connected_components']}")

        # Validation
        validation = self.stats["validation"]
        print(f"\n✅ Validation:")
        print(f"  Checks Passed: {validation['passed_checks']}/{validation['total_checks']}")
        print(f"  Errors: {validation['total_errors']}")
        print(f"  Warnings: {validation['total_warnings']}")

        if validation["errors"]:
            print(f"\n  ❌ Errors:")
            for error in validation["errors"]:
                print(f"     - {error}")

        if validation["warnings"]:
            print(f"\n  ⚠️  Warnings:")
            for warning in validation["warnings"]:
                print(f"     - {warning}")

        print("\n" + "=" * 80 + "\n")


def compute_graph_statistics(
    graph: nx.DiGraph,
    export_path: Optional[str] = None,
    print_summary: bool = True,
) -> Dict[str, Any]:
    """
    Convenience function to compute and optionally export graph statistics.

    Args:
        graph: NetworkX DiGraph to analyze
        export_path: Optional path to export JSON statistics
        print_summary: Whether to print summary to console

    Returns:
        Dict with all statistics
    """
    stats_calculator = GraphStatistics(graph)
    stats = stats_calculator.compute_all()

    if print_summary:
        stats_calculator.print_summary()

    if export_path:
        stats_calculator.export_to_json(export_path)

    return stats
