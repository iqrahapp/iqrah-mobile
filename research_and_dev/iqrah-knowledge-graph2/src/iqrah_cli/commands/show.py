from pyvis.network import Network
import networkx as nx
from typing import Optional, List, Set, Dict
import argparse
from pathlib import Path
from iqrah.graph.builder import EdgeType
import webbrowser
import os


def create_interactive_view(
    G: nx.DiGraph,
    output_path: str,
    filter_types: Optional[List[str]] = None,
    filter_axes: Optional[List[str]] = None,
    show_dependency: bool = True,
    show_knowledge: bool = True,
    height: str = "900px",
) -> None:
    """
    Create interactive visualization of the graph.

    Args:
        G: NetworkX graph
        output_path: Where to save the HTML
        filter_types: Only show these node types
        filter_axes: Only show these knowledge axes
        show_dependency: Show dependency edges
        show_knowledge: Show knowledge edges
        height: Height of visualization
    """
    # Initialize network
    net = Network(
        height=height,
        width="100%",
        directed=True,
        bgcolor="#ffffff",
        font_color="#000000",
    )

    # Configure physics
    net.force_atlas_2based()
    net.show_buttons(filter_=["physics"])

    # Node colors by type
    colors = {
        "chapter": "#FF9999",
        "verse": "#99FF99",
        "word_instance": "#9999FF",
        "word": "#FFFF99",
        "lemma": "#FF99FF",
        "root": "#99FFFF",
        "knowledge": "#CCCCCC",
    }

    # Filter nodes
    nodes_to_show = set()
    if filter_types or filter_axes:
        for node, data in G.nodes(data=True):
            node_type = data.get("type")
            axis = data.get("axis")
            if (not filter_types or node_type in filter_types) and (
                not filter_axes or axis in filter_axes
            ):
                nodes_to_show.add(node)
    else:
        nodes_to_show = set(G.nodes())

    # Add nodes with styling
    for node in nodes_to_show:
        data = G.nodes[node]
        node_type = data.get("type", "unknown")

        # Create detailed tooltip
        tooltip = "<br>".join(
            [
                f"<b>{k}:</b> {v}"
                for k, v in data.items()
                if k not in ["hidden", "physics"]
            ]
        )

        # Determine node size
        size = 25 if node_type in ["chapter"] else 20 if node_type in ["verse"] else 15

        # Add node
        net.add_node(
            node,
            label=node.split(":")[-1],
            title=tooltip,
            color=colors.get(node_type, "#CCCCCC"),
            size=size,
            borderWidth=2,
            borderWidthSelected=4,
        )

    # Add edges with styling
    edge_colors = {
        EdgeType.DEPENDENCY.value: {"color": "#666666", "width": 1},
        EdgeType.KNOWLEDGE.value: {"color": "#ff0000", "width": 2},
    }

    for source, target, data in G.edges(data=True):
        if source in nodes_to_show and target in nodes_to_show:
            edge_type = data.get("type", EdgeType.DEPENDENCY.value)

            # Skip if edge type is filtered
            if (edge_type == EdgeType.DEPENDENCY.value and not show_dependency) or (
                edge_type == EdgeType.KNOWLEDGE.value and not show_knowledge
            ):
                continue

            # Create edge tooltip
            tooltip = "<br>".join(
                [
                    f"<b>{k}:</b> {v}"
                    for k, v in data.items()
                    if k not in ["hidden", "physics"]
                ]
            )

            # Add edge
            style = edge_colors.get(edge_type, {"color": "#999999", "width": 1})
            net.add_edge(
                source,
                target,
                title=tooltip,
                color=style["color"],
                width=style["width"],
                arrowStrikethrough=False,
            )

    # Save and show
    net.save(output_path)
    webbrowser.open(f"file://{os.path.abspath(output_path)}")


def main():
    parser = argparse.ArgumentParser(description="Visualize Quran Knowledge Graph")
    parser.add_argument("graph_path", type=str, help="Path to GraphML file")
    parser.add_argument(
        "--output", type=str, default="graph.html", help="Output HTML path"
    )
    parser.add_argument("--types", nargs="+", help="Filter node types")
    parser.add_argument("--axes", nargs="+", help="Filter knowledge axes")
    parser.add_argument(
        "--no-dependency", action="store_true", help="Hide dependency edges"
    )
    parser.add_argument(
        "--no-knowledge", action="store_true", help="Hide knowledge edges"
    )
    parser.add_argument(
        "--height", type=str, default="900px", help="Visualization height"
    )

    args = parser.parse_args()

    # Load graph
    G = nx.read_graphml(args.graph_path)
    print(
        f"Loaded graph with {G.number_of_nodes()} nodes and {G.number_of_edges()} edges"
    )

    # Print summary
    print("\nNode types:")
    node_types = set(data["type"] for _, data in G.nodes(data=True) if "type" in data)
    for ntype in sorted(node_types):
        count = sum(1 for _, data in G.nodes(data=True) if data.get("type") == ntype)
        print(f"  {ntype}: {count} nodes")

    print("\nKnowledge axes:")
    axes = set(data["axis"] for _, data in G.nodes(data=True) if "axis" in data)
    for axis in sorted(axes):
        count = sum(1 for _, data in G.nodes(data=True) if data.get("axis") == axis)
        print(f"  {axis}: {count} nodes")

    print("\nEdge types:")
    edge_types = set(
        data["type"] for _, _, data in G.edges(data=True) if "type" in data
    )
    for etype in sorted(edge_types):
        count = sum(1 for _, _, data in G.edges(data=True) if data.get("type") == etype)
        print(f"  {etype}: {count} edges")

    # Create visualization
    create_interactive_view(
        G,
        args.output,
        filter_types=args.types,
        filter_axes=args.axes,
        show_dependency=not args.no_dependency,
        show_knowledge=not args.no_knowledge,
        height=args.height,
    )

    print(f"\nVisualization saved to {args.output}")


if __name__ == "__main__":
    main()
