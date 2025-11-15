# iqrah/graph/visualizer.py
import networkx as nx
import plotly.graph_objects as go
import dash
from dash import dcc, html
from dash.dependencies import Input, Output
from typing import Set, Optional


class GraphVisualizer:
    def __init__(self, graph: nx.DiGraph, reverse: bool = False):
        self.graph = graph.reverse() if reverse else graph
        self.pos = None
        self.node_data = []
        self.edge_data = []

        # Updated colors for node types
        self.node_colors = {
            "default": "gray",  # Gray for default nodes
            "word_instance": "red",  # Red for word-related nodes
            "verse": "green",  # Green for verse-related nodes
            "lemma": "blue",  # Blue for lemma-related nodes
            "chapter": "cyan",  # Cyan for chapter-related nodes
            "word": "orange",  # Orange for general word nodes
        }

        # Updated colors for edge types
        self.edge_colors = {
            "default": "gray",  # Gray for default edges
            "fake_edge": "gray",  # Gray for auxiliary or placeholder edges
            "has_root": "darkred",  # Dark red to signify root connection
            "has_lemma": "blue",  # Blue for lemma-related edges (aligns with lemma node)
            "has_verse": "green",  # Green for verse-related edges (aligns with verse node)
            "has_word_instance": "red",  # Red for word-instance-related edges (aligns with word_instance node)
            "is_word": "orange",  # Orange for general word relations (aligns with word node)
            "next_chapter": "cyan",  # Cyan for chapter navigation (aligns with chapter node)
            "prev_verse": "lightgreen",  # Light green for previous verse (similar to verse node color)
            "prev_word_instance": "pink",  # Pink as an auxiliary relation for previous word instance
        }

    def prepare_layout(self):
        """Prepare graph layout using topological generations"""
        # Create a copy of the graph for modification
        G = self.graph.copy()

        # Assign layers based on topological generations
        for layer, nodes in enumerate(nx.topological_generations(G)):
            for node in nodes:
                G.nodes[node]["layer"] = layer

        # Get the multipartite layout
        self.pos = nx.multipartite_layout(G, subset_key="layer")

        # Prepare node data
        self._prepare_node_data()

        # Prepare edge data
        self._prepare_edge_data()

    def _prepare_node_data(self):
        """Prepare node data for visualization"""
        self.node_data = []
        for node, (x, y) in self.pos.items():
            # Determine node type based on connectivity
            node_type = self.graph.nodes[node].get("type", "word")

            self.node_data.append(
                {
                    "id": node,
                    "x": x,
                    "y": y,
                    "color": self.node_colors.get(node_type, "gray"),
                    "size": 5
                    + 2 * (self.graph.in_degree(node) + self.graph.out_degree(node)),
                    "text": f"Node: {node}<br>Type: {node_type}",
                    "type": node_type,
                }
            )

    def _prepare_edge_data(self):
        """Prepare edge data for visualization"""
        self.edge_data = []
        for u, v, data in self.graph.edges(data=True):
            edge_type = data.get("type", "default")
            x0, y0 = self.pos[u]
            x1, y1 = self.pos[v]
            self.edge_data.append(
                {
                    "source": u,
                    "target": v,
                    "x": [x0, x1, None],
                    "y": [y0, y1, None],
                    "color": self.edge_colors.get(edge_type, "gray"),
                    "type": edge_type,
                }
            )

    def find_connected_nodes(self, start_node: str) -> Set[str]:
        """Find all nodes connected to the start node"""
        descendants = nx.descendants(self.graph, start_node)
        ancestors = nx.ancestors(self.graph, start_node)
        return descendants.union(ancestors).union({start_node})

    def create_figure(self, highlight_node: Optional[str] = None):
        """Create the plotly figure for visualization"""
        fig = go.Figure()
        connected_nodes = set()

        # Handle edge visualization
        if highlight_node is None:
            # Normal state
            for edge in self.edge_data:
                fig.add_trace(
                    go.Scatter(
                        x=edge["x"],
                        y=edge["y"],
                        line=dict(width=0.5, color=edge["color"]),
                        hoverinfo="none",
                        mode="lines",
                    )
                )
        else:
            # Highlighted state
            connected_nodes = self.find_connected_nodes(highlight_node)
            connected_edges = {
                (u, v)
                for u, v in self.graph.edges()
                if u in connected_nodes and v in connected_nodes
            }

            for edge in self.edge_data:
                color = (
                    edge["color"]
                    if (edge["source"], edge["target"]) in connected_edges
                    else "rgba(200,200,200,0.3)"
                )
                width = (
                    1.5 if (edge["source"], edge["target"]) in connected_edges else 0.5
                )
                fig.add_trace(
                    go.Scatter(
                        x=edge["x"],
                        y=edge["y"],
                        line=dict(width=width, color=color),
                        hoverinfo="none",
                        mode="lines",
                    )
                )

        # Handle node visualization
        node_x = [node["x"] for node in self.node_data]
        node_y = [node["y"] for node in self.node_data]
        node_colors = []
        node_sizes = []

        for node in self.node_data:
            if highlight_node is None:
                node_colors.append(node["color"])
                node_sizes.append(node["size"])
            else:
                is_connected = node["id"] in connected_nodes
                if node["id"] == highlight_node:
                    node_colors.append(node["color"])
                    node_sizes.append(node["size"] * 1.5)
                elif is_connected:
                    node_colors.append(node["color"])
                    node_sizes.append(node["size"] * 1.2)
                else:
                    node_colors.append("rgba(200,200,200,0.3)")
                    node_sizes.append(node["size"])

        fig.add_trace(
            go.Scatter(
                x=node_x,
                y=node_y,
                mode="markers",
                marker=dict(
                    size=node_sizes,
                    color=node_colors,
                    line=dict(width=0.5, color="black"),
                ),
                hoverinfo="text",
                text=[node["text"] for node in self.node_data],
                customdata=[node["id"] for node in self.node_data],
            )
        )

        fig.update_layout(
            title="Interactive Graph Visualization",
            showlegend=False,
            margin=dict(l=40, r=40, t=40, b=40),
            xaxis=dict(showgrid=False, zeroline=False, visible=False),
            yaxis=dict(showgrid=False, zeroline=False, visible=False),
            height=1000,
            hovermode="closest",
        )

        return fig

    def run_server(self, host: str = "0.0.0.0", port: int = 8050, debug: bool = False):
        """Run the visualization server"""
        app = dash.Dash(__name__)

        app.layout = html.Div(
            [
                dcc.Graph(
                    id="graph-visualization",
                    figure=self.create_figure(),
                    clear_on_unhover=True,
                )
            ]
        )

        @app.callback(
            Output("graph-visualization", "figure"),
            [Input("graph-visualization", "hoverData")],
        )
        def update_highlight(hoverData):
            if hoverData is None:
                return self.create_figure()
            point_index = hoverData["points"][0]["pointIndex"]
            hovered_node = self.node_data[point_index]["id"]
            return self.create_figure(highlight_node=hovered_node)

        app.run_server(debug=debug, host=host, port=port)
