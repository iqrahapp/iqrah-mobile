"""
Iqrah Audio - Qur'an Recitation Analysis
========================================

SOTA pitch tracking and DTW alignment for comparing recitations to reference Qari.

Phase 2 MVP: Offline analysis (record → process → score)
Future: Real-time coaching with online-DTW
"""

__version__ = "0.1.0"

from .pitch import PitchExtractor, PitchContour
from .dtw import DTWAligner, AlignmentResult, OnlineDTWAligner
from .reference import ReferenceProcessor
from .scorer import RecitationScorer, RecitationScore
from .denoise import AudioDenoiser
from .features import (
    FeatureExtractor,
    AudioFeatures,
    extract_nasal_energy,
    detect_silence_segments,
)
from .octave import (
    OctaveCorrector,
    octave_aware_pitch_distance,
    detect_octave_errors,
    snap_to_nearest_octave,
)

__all__ = [
    # Core modules
    "PitchExtractor",
    "PitchContour",
    "DTWAligner",
    "AlignmentResult",
    "OnlineDTWAligner",
    "ReferenceProcessor",
    "RecitationScorer",
    "RecitationScore",
    "AudioDenoiser",
    # Multi-dimensional features
    "FeatureExtractor",
    "AudioFeatures",
    "extract_nasal_energy",
    "detect_silence_segments",
    # Octave correction
    "OctaveCorrector",
    "octave_aware_pitch_distance",
    "detect_octave_errors",
    "snap_to_nearest_octave",
]
