# iqrah/graph/__init__.py

from .builder import QuranGraphBuilder
from .knowledge_builder import KnowledgeGraphBuilder
from .scoring import calculate_knowledge_scores, KnowledgeGraphScoring
from .statistics import compute_graph_statistics, GraphStatistics

# Visualizer requires plotly/dash (optional)
try:
    from .visualizer import GraphVisualizer
    __all__ = [
        "QuranGraphBuilder",
        "KnowledgeGraphBuilder",
        "calculate_knowledge_scores",
        "KnowledgeGraphScoring",
        "compute_graph_statistics",
        "GraphStatistics",
        "GraphVisualizer"
    ]
except ImportError:
    __all__ = [
        "QuranGraphBuilder",
        "KnowledgeGraphBuilder",
        "calculate_knowledge_scores",
        "KnowledgeGraphScoring",
        "compute_graph_statistics",
        "GraphStatistics"
    ]
