# src/iqrah_cli/commands/visualize.py

import sys
from pathlib import Path
import networkx as nx
from iqrah.graph.visualizer import GraphVisualizer


def setup_parser(subparsers):
    parser = subparsers.add_parser(
        "visualize", help="Visualize a graph with interactive layout"
    )
    parser.add_argument("input_file", type=str, help="Path to the input graph file")
    parser.add_argument(
        "--format",
        type=str,
        default="graphml",
        choices=["graphml", "gexf", "gml", "pajek"],
        help="Input file format (default: graphml)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8050,
        help="Port to run the web server on (default: 8050)",
    )
    parser.add_argument(
        "--host",
        type=str,
        default="0.0.0.0",
        help="Host to run the web server on (default: 0.0.0.0)",
    )
    parser.add_argument(
        "--reverse", action="store_true", help="Reverse the graph before visualization"
    )


def run(args):
    """Execute the visualize command."""
    # Check if input file exists
    input_path = Path(args.input_file)
    if not input_path.exists():
        print(f"Error: Input file '{args.input_file}' does not exist.")
        sys.exit(1)

    # Load the graph
    try:
        if args.format == "graphml":
            G = nx.read_graphml(args.input_file)
        elif args.format == "gexf":
            G = nx.read_gexf(args.input_file)
        elif args.format == "gml":
            G = nx.read_gml(args.input_file)
        elif args.format == "pajek":
            G = nx.read_pajek(args.input_file)
    except Exception as e:
        print(f"Error loading graph: {str(e)}")
        sys.exit(1)

    if args.reverse:
        G = G.reverse()

    # Create visualizer and prepare layout
    visualizer = GraphVisualizer(G)
    visualizer.prepare_layout()

    # Run the server
    print(f"Starting server at http://{args.host}:{args.port}")
    visualizer.run_server(host=args.host, port=args.port)
