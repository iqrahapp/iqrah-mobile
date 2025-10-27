"""
Comparison Module
=================

Phase 2: Tempo-invariant rhythm, key-invariant melody, and duration comparison.
"""

from .engine import compare_recitations
from .features import extract_features, FeaturePack
from .rhythm import rhythm_score
from .melody import melody_score
from .duration import madd_score_tempo_adaptive
from .fusion import compute_overall_score
from .visualization import generate_comparison_visualizations

__all__ = [
    'compare_recitations',
    'extract_features',
    'FeaturePack',
    'rhythm_score',
    'melody_score',
    'madd_score_tempo_adaptive',
    'compute_overall_score',
    'generate_comparison_visualizations',
]
