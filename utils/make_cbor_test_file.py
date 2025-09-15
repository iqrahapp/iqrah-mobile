#!/usr/bin/env python3

import networkx as nx
import cbor2
import zstandard as zstd
import os


def export_graph_to_cbor(G, output_path):
    """Export NetworkX graph to a compressed CBOR sequence file."""
    # Ensure the output directory exists
    output_dir = os.path.dirname(output_path)
    if output_dir:
        os.makedirs(output_dir, exist_ok=True)

    print(f"🚀 Starting export to {output_path}...")

    with open(output_path, "wb") as f:
        cctx = zstd.ZstdCompressor(level=9).stream_writer(f)
        enc = cbor2.CBOREncoder(cctx)

        # 1. Header with metadata
        header = {
            "v": 1,
            "graph": {
                "node_count": len(G.nodes),
                "edge_count": len(G.edges),
                "directed": True,
                "multi": False,
            },
        }
        enc.encode(header)

        # 2. Export nodes
        for node_id, attrs in G.nodes(data=True):
            enc.encode({"t": "node", "id": str(node_id), "a": dict(attrs)})

        # 3. Export edges
        for u, v, attrs in G.edges(data=True):
            enc.encode({"t": "edge", "u": str(u), "v": str(v), "a": dict(attrs)})
        cctx.flush()

    file_size = os.path.getsize(output_path)
    print(f"✅ Export complete! File size: {file_size} bytes.")


def create_and_export_test_graph():
    """Creates the specific graph needed for the Rust integration tests."""
    # Create a directed graph
    G = nx.DiGraph()

    # Add nodes with their 'type' attribute
    G.add_node("word:test1", type="word")
    G.add_node("word:test2", type="word")
    G.add_node("root:tst", type="root")

    # Add edges with their specific distribution parameters
    G.add_edge("word:test1", "root:tst", type="knowledge", dist="beta", a=4.0, b=2.0)
    G.add_edge(
        "word:test1", "word:test2", type="knowledge", dist="normal", m=0.5, s=0.1
    )

    # Define the output path relative to the project root
    output_file = "tests/data/test-graph.cbor.zst"

    # Export the graph
    export_graph_to_cbor(G, output_file)


# Main execution block
if __name__ == "__main__":
    create_and_export_test_graph()
