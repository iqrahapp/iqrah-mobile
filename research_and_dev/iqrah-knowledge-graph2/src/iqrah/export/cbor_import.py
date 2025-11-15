"""
CBOR Import Module

Imports knowledge graphs from compressed CBOR format.

Reads structure-only CBOR files and reconstructs NetworkX graphs.
Content data must be loaded separately from the content database.
"""

import networkx as nx
import cbor2
import zstandard as zstd
import logging
from pathlib import Path
from typing import Optional, Dict, Any
from tqdm import tqdm
import os


logger = logging.getLogger(__name__)


def import_graph_from_cbor(
    input_path: str,
    show_progress: bool = True,
) -> nx.DiGraph:
    """
    Import NetworkX graph from compressed CBOR format.

    Reads structure-only CBOR and reconstructs the graph.
    Content data (text, translations) must be loaded from content database.

    Args:
        input_path: Path to .cbor.zst file
        show_progress: Show progress bars

    Returns:
        NetworkX DiGraph with structure and scores

    Raises:
        FileNotFoundError: If file doesn't exist
        ValueError: If file format is invalid
    """
    if not os.path.exists(input_path):
        raise FileNotFoundError(f"File not found: {input_path}")

    logger.info(f"Importing graph from {input_path}")

    file_size = os.path.getsize(input_path)
    logger.info(f"File size: {file_size / 1024 / 1024:.2f} MB")

    G = nx.DiGraph()

    try:
        with open(input_path, "rb") as f:
            decompressor = zstd.ZstdDecompressor()

            with decompressor.stream_reader(f) as reader:
                decoder = cbor2.CBORDecoder(reader)

                # 1. Read header
                header = _read_header(decoder, G)
                logger.info(f"Importing v{header.get('v')} graph")

                node_count_expected = header.get("graph", {}).get("node_count", 0)
                edge_count_expected = header.get("graph", {}).get("edge_count", 0)

                # 2. Read nodes
                node_count = _import_nodes(decoder, G, node_count_expected, show_progress)
                logger.info(f"Imported {node_count} nodes")

                # 3. Read edges
                edge_count = _import_edges(decoder, G, edge_count_expected, show_progress)
                logger.info(f"Imported {edge_count} edges")

                # Validate
                if node_count != node_count_expected:
                    logger.warning(
                        f"Node count mismatch: expected {node_count_expected}, "
                        f"got {node_count}"
                    )

                if edge_count != edge_count_expected:
                    logger.warning(
                        f"Edge count mismatch: expected {edge_count_expected}, "
                        f"got {edge_count}"
                    )

    except Exception as e:
        logger.error(f"Import failed: {e}")
        raise

    logger.info(f"Import complete: {len(G.nodes)} nodes, {len(G.edges)} edges")
    return G


def _read_header(decoder: cbor2.CBORDecoder, G: nx.DiGraph) -> Dict[str, Any]:
    """
    Read and parse header from CBOR stream.

    Args:
        decoder: CBOR decoder
        G: Graph to populate metadata

    Returns:
        Header dict

    Raises:
        ValueError: If header is invalid
    """
    try:
        header = decoder.decode()

        if not isinstance(header, dict):
            raise ValueError("Header must be a dict")

        if "v" not in header:
            raise ValueError("Header missing version field")

        # Store metadata in graph
        if "metadata" in header:
            G.graph.update(header["metadata"])

        return header

    except EOFError:
        raise ValueError("Empty CBOR file")
    except Exception as e:
        raise ValueError(f"Invalid header: {e}")


def _import_nodes(
    decoder: cbor2.CBORDecoder,
    G: nx.DiGraph,
    expected_count: int,
    show_progress: bool = True,
) -> int:
    """
    Import nodes from CBOR stream.

    Args:
        decoder: CBOR decoder
        G: Graph to populate
        expected_count: Expected number of nodes (for progress)
        show_progress: Show progress bar

    Returns:
        Number of nodes imported
    """
    count = 0

    # Create progress bar
    if show_progress and expected_count > 0:
        pbar = tqdm(total=expected_count, desc="Importing nodes")
    else:
        pbar = None

    try:
        while True:
            record = decoder.decode()

            # Check if we've moved to edges
            if record.get("t") == "edge":
                # Put this record back (not possible with decoder)
                # So we'll handle this in _import_edges
                # For now, break and assume nodes are done
                logger.warning("Encountered edge while reading nodes - may be format issue")
                break

            if record.get("t") == "node":
                node_id = record["id"]
                attrs = record.get("a", {})

                G.add_node(node_id, **attrs)
                count += 1

                if pbar:
                    pbar.update(1)

            # Stop when we've read all expected nodes
            if expected_count > 0 and count >= expected_count:
                break

    except EOFError:
        # End of nodes
        pass
    finally:
        if pbar:
            pbar.close()

    return count


def _import_edges(
    decoder: cbor2.CBORDecoder,
    G: nx.DiGraph,
    expected_count: int,
    show_progress: bool = True,
) -> int:
    """
    Import edges from CBOR stream.

    Args:
        decoder: CBOR decoder
        G: Graph to populate
        expected_count: Expected number of edges (for progress)
        show_progress: Show progress bar

    Returns:
        Number of edges imported
    """
    count = 0

    # Create progress bar
    if show_progress and expected_count > 0:
        pbar = tqdm(total=expected_count, desc="Importing edges")
    else:
        pbar = None

    try:
        while True:
            record = decoder.decode()

            if record.get("t") == "edge":
                u = record["u"]
                v = record["v"]
                attrs = record.get("a", {})

                G.add_edge(u, v, **attrs)
                count += 1

                if pbar:
                    pbar.update(1)

            # Stop when we've read all expected edges
            if expected_count > 0 and count >= expected_count:
                break

    except EOFError:
        # End of edges
        pass
    finally:
        if pbar:
            pbar.close()

    return count


def augment_graph_with_content(
    G: nx.DiGraph,
    content_db_path: str,
    node_types: Optional[list] = None,
) -> None:
    """
    Augment graph nodes with content from database.

    This is a runtime operation that adds content data to the graph
    by querying the content database.

    Args:
        G: NetworkX graph (modified in place)
        content_db_path: Path to content database
        node_types: List of node types to augment (default: all)

    Raises:
        FileNotFoundError: If database doesn't exist
    """
    from ..content.database import ContentDatabase

    logger.info(f"Augmenting graph with content from {content_db_path}")

    with ContentDatabase(content_db_path) as db:
        # Default to common content node types
        if node_types is None:
            node_types = ["chapter", "verse", "word_instance"]

        # Collect node IDs by type
        nodes_to_augment = {}
        for node_id, attrs in G.nodes(data=True):
            node_type = attrs.get("type")
            if node_type in node_types:
                if node_type not in nodes_to_augment:
                    nodes_to_augment[node_type] = []
                nodes_to_augment[node_type].append(node_id)

        # Bulk query content
        all_node_ids = []
        for node_ids in nodes_to_augment.values():
            all_node_ids.extend(node_ids)

        logger.info(f"Querying content for {len(all_node_ids)} nodes...")
        content_data = db.get_content_for_nodes(all_node_ids)

        # Augment nodes
        augmented = 0
        for node_id, content in content_data.items():
            if node_id in G:
                # Add content fields to node
                G.nodes[node_id].update(content)
                augmented += 1

        logger.info(f"Augmented {augmented} nodes with content")


def load_graph_with_content(
    cbor_path: str,
    content_db_path: str,
    show_progress: bool = True,
) -> nx.DiGraph:
    """
    Convenience function to load graph and augment with content.

    Args:
        cbor_path: Path to CBOR graph file
        content_db_path: Path to content database
        show_progress: Show progress bars

    Returns:
        NetworkX DiGraph with structure and content
    """
    # Load structure
    G = import_graph_from_cbor(cbor_path, show_progress=show_progress)

    # Augment with content
    augment_graph_with_content(G, content_db_path)

    return G
