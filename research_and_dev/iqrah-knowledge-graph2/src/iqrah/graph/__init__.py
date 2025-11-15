# iqrah/graph/__init__.py

from .builder import QuranGraphBuilder

# Visualizer requires plotly/dash (optional)
try:
    from .visualizer import GraphVisualizer
    __all__ = ["QuranGraphBuilder", "GraphVisualizer"]
except ImportError:
    __all__ = ["QuranGraphBuilder"]
