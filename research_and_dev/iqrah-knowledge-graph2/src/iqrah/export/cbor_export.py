"""
CBOR Export Module

Exports knowledge graphs to compressed CBOR format.

CRITICAL ARCHITECTURE CHANGE:
- Exports ONLY graph structure (nodes, edges, scores, attributes)
- Does NOT include content (Arabic text, translations, transliterations)
- Content is stored separately in SQLite database
- Content is augmented at runtime via database queries

This separation allows:
- Smaller CBOR files
- Independent content updates
- Better separation of concerns
- Faster graph loading
"""

import networkx as nx
import cbor2
import zstandard as zstd
import logging
from pathlib import Path
from typing import Optional, Dict, Any
from tqdm import tqdm
import datetime
import os


logger = logging.getLogger(__name__)


# Structural attributes to include (NOT content)
STRUCTURAL_ATTRIBUTES = {
    "type",  # Node type (word_instance, verse, chapter, etc.)
    "verse_key",  # Reference key
    "chapter_number",  # Reference
    "verse_number",  # Reference
    "position",  # Word position
    "word_key",  # Reference
    "foundational_score",  # PageRank score
    "influence_score",  # PageRank score
    "knowledge_axis",  # Knowledge dimension
    "dist",  # Edge distribution type
    "m",  # Distribution mean
    "s",  # Distribution std
    "a",  # Distribution alpha
    "b",  # Distribution beta
    "weight",  # Edge weight
    "knowledge_type",  # Edge knowledge type
}


def export_graph_to_cbor(
    G: nx.DiGraph,
    output_path: str,
    compression_level: int = 9,
    show_progress: bool = True,
    include_metadata: bool = True,
) -> None:
    """
    Export NetworkX graph to compressed CBOR format (structure only).

    IMPORTANT: This exports ONLY the graph structure and scores.
    Content data (text, translations) must be loaded from the content database.

    Args:
        G: NetworkX DiGraph to export
        output_path: Path to output .cbor.zst file
        compression_level: Zstandard compression level (1-22, default 9)
        show_progress: Show progress bars
        include_metadata: Include metadata header

    Raises:
        ValueError: If graph is invalid
        IOError: If file cannot be written
    """
    if not G.nodes():
        raise ValueError("Cannot export empty graph")

    logger.info(f"Exporting graph to {output_path}")
    logger.info(f"Graph: {len(G.nodes)} nodes, {len(G.edges)} edges")

    # Ensure output directory exists
    output_file = Path(output_path)
    output_file.parent.mkdir(parents=True, exist_ok=True)

    try:
        with open(output_path, "wb") as f:
            # Create compression context with streaming
            compressor = zstd.ZstdCompressor(level=compression_level)

            with compressor.stream_writer(f) as writer:
                encoder = cbor2.CBOREncoder(writer)

                # 1. Write header
                if include_metadata:
                    header = _create_header(G)
                    encoder.encode(header)
                    logger.info(f"Header written: v{header['v']}")

                # 2. Write nodes (structure only)
                node_count = _export_nodes(encoder, G, show_progress)
                logger.info(f"Exported {node_count} nodes (structure only)")

                # 3. Write edges
                edge_count = _export_edges(encoder, G, show_progress)
                logger.info(f"Exported {edge_count} edges")

        # Report file size
        file_size = os.path.getsize(output_path)
        logger.info(
            f"Export complete: {output_path} "
            f"({file_size / 1024 / 1024:.2f} MB)"
        )

    except Exception as e:
        logger.error(f"Export failed: {e}")
        # Clean up partial file
        if output_file.exists():
            output_file.unlink()
        raise


def _create_header(G: nx.DiGraph) -> Dict[str, Any]:
    """Create metadata header for CBOR export."""
    return {
        "v": 2,  # Version 2.0 - structure-only format
        "format": "structure_only",  # Marker for new architecture
        "created_at": datetime.datetime.now().isoformat(),
        "graph": {
            "directed": G.is_directed(),
            "multi": G.is_multigraph(),
            "node_count": len(G.nodes),
            "edge_count": len(G.edges),
        },
        "metadata": dict(G.graph) if G.graph else {},
    }


def _export_nodes(
    encoder: cbor2.CBOREncoder,
    G: nx.DiGraph,
    show_progress: bool = True,
) -> int:
    """
    Export nodes with structure-only attributes.

    CRITICAL: Does NOT include content (text, translations, transliterations).
    Only includes:
    - Node ID
    - Type
    - Reference keys (verse_key, word_key, etc.)
    - Scores (foundational, influence)
    - Structural metadata

    Returns:
        Number of nodes exported
    """
    nodes = list(G.nodes(data=True))
    iterator = tqdm(nodes, desc="Exporting nodes", disable=not show_progress)

    for node_id, attrs in iterator:
        # Filter to structural attributes only
        structural_attrs = _filter_structural_attributes(attrs)

        encoder.encode({
            "t": "node",
            "id": str(node_id),
            "a": structural_attrs,
        })

    return len(nodes)


def _export_edges(
    encoder: cbor2.CBOREncoder,
    G: nx.DiGraph,
    show_progress: bool = True,
) -> int:
    """
    Export edges with all attributes.

    Includes:
    - Edge endpoints
    - Distribution parameters
    - Knowledge type
    - Weights

    Returns:
        Number of edges exported
    """
    edges = list(G.edges(data=True))
    iterator = tqdm(edges, desc="Exporting edges", disable=not show_progress)

    for u, v, attrs in iterator:
        # Filter to structural attributes
        structural_attrs = _filter_structural_attributes(attrs)

        encoder.encode({
            "t": "edge",
            "u": str(u),
            "v": str(v),
            "a": structural_attrs,
        })

    return len(edges)


def _filter_structural_attributes(attrs: Dict[str, Any]) -> Dict[str, Any]:
    """
    Filter node/edge attributes to include only structural data.

    REMOVES:
    - arabic
    - text_uthmani
    - text_uthmani_simple
    - translation
    - transliteration
    - audio_url
    - Any other content fields

    KEEPS:
    - type, verse_key, position (structural)
    - foundational_score, influence_score (computed)
    - dist, m, s, a, b, weight (edge params)
    """
    return {
        k: v
        for k, v in attrs.items()
        if k in STRUCTURAL_ATTRIBUTES or k.endswith("_score")
    }


def inspect_cbor_graph(
    file_path: str,
    sample_size: int = 10,
    show_full_sample: bool = False,
) -> Dict[str, Any]:
    """
    Inspect CBOR graph file and return statistics.

    Args:
        file_path: Path to .cbor.zst file
        sample_size: Number of records to sample
        show_full_sample: Print full sample records

    Returns:
        Dict with statistics and sample data

    Raises:
        FileNotFoundError: If file doesn't exist
        ValueError: If file is invalid
    """
    if not os.path.exists(file_path):
        raise FileNotFoundError(f"File not found: {file_path}")

    logger.info(f"Inspecting {file_path}")

    file_size = os.path.getsize(file_path)
    logger.info(f"File size: {file_size / 1024 / 1024:.2f} MB")

    stats = {
        "file_size": file_size,
        "node_types": {},
        "edge_attrs": {},
        "header": None,
        "sample_nodes": [],
        "sample_edges": [],
    }

    try:
        with open(file_path, "rb") as f:
            decompressor = zstd.ZstdDecompressor()

            with decompressor.stream_reader(f) as reader:
                decoder = cbor2.CBORDecoder(reader)

                # Read header
                try:
                    header = decoder.decode()
                    stats["header"] = header
                    logger.info(f"Version: {header.get('v')}")
                    logger.info(f"Format: {header.get('format', 'unknown')}")
                    logger.info(
                        f"Nodes: {header.get('graph', {}).get('node_count', 'N/A')}"
                    )
                    logger.info(
                        f"Edges: {header.get('graph', {}).get('edge_count', 'N/A')}"
                    )
                except Exception as e:
                    logger.warning(f"Could not read header: {e}")

                # Sample records
                count = 0
                try:
                    while True:
                        record = decoder.decode()
                        count += 1

                        if record.get("t") == "node":
                            node_type = record.get("a", {}).get("type", "unknown")
                            stats["node_types"][node_type] = (
                                stats["node_types"].get(node_type, 0) + 1
                            )

                            if len(stats["sample_nodes"]) < sample_size:
                                stats["sample_nodes"].append(record)

                                if show_full_sample:
                                    logger.info(f"Node: {record['id']}")
                                    logger.info(f"  Attrs: {record['a']}")

                        elif record.get("t") == "edge":
                            attrs = record.get("a", {})
                            for key in attrs.keys():
                                stats["edge_attrs"][key] = (
                                    stats["edge_attrs"].get(key, 0) + 1
                                )

                            if len(stats["sample_edges"]) < sample_size:
                                stats["sample_edges"].append(record)

                                if show_full_sample:
                                    logger.info(f"Edge: {record['u']} -> {record['v']}")
                                    logger.info(f"  Attrs: {attrs}")

                except EOFError:
                    logger.info(f"Processed {count} records")

    except Exception as e:
        logger.error(f"Inspection failed: {e}")
        raise

    # Print summary
    logger.info("Node types:")
    for node_type, count in sorted(stats["node_types"].items()):
        logger.info(f"  {node_type}: {count}")

    logger.info("Edge attribute keys:")
    for attr, count in sorted(stats["edge_attrs"].items()):
        logger.info(f"  {attr}: {count}")

    return stats
