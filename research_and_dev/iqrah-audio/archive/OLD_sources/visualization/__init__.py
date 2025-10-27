"""
Visualization module for Iqrah Audio Recitation Comparison.

Provides interactive visualizations for:
- DTW path over onset grid (rhythm timing divergence)
- ΔF0 contour comparison (melody)
- Madd duration bars (expected vs actual)
- Pronunciation timeline with error highlights
"""

from .dtw_path import plot_dtw_path
from .melody_contour import plot_melody_contour
from .duration_bars import plot_duration_bars
from .pronunciation_timeline import plot_pronunciation_timeline
from .html_viewer import create_interactive_viewer

__all__ = [
    'plot_dtw_path',
    'plot_melody_contour',
    'plot_duration_bars',
    'plot_pronunciation_timeline',
    'create_interactive_viewer',
]
