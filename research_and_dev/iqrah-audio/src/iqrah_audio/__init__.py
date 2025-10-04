"""
Iqrah Audio - Qur'an Recitation Analysis
========================================

SOTA pitch tracking and DTW alignment for comparing recitations to reference Qari.

Phase 2 MVP: Offline analysis (record → process → score)
Future: Real-time coaching with online-DTW
"""

__version__ = "0.1.0"

from .pitch import PitchExtractor, PitchContour
from .dtw import DTWAligner, AlignmentResult
from .reference import ReferenceProcessor
from .scorer import RecitationScorer, RecitationScore
from .denoise import AudioDenoiser

__all__ = [
    "PitchExtractor",
    "PitchContour",
    "DTWAligner",
    "AlignmentResult",
    "ReferenceProcessor",
    "RecitationScorer",
    "RecitationScore",
    "AudioDenoiser",
]
