"""
Real-Time Streaming Module
===========================

Components for real-time Quranic recitation analysis with <100ms latency.

Classes:
- StreamingAudioBuffer: Ring buffer for streaming audio
- IncrementalPitchExtractor: Incremental pitch extraction with caching
- AnchorDetector: Detect alignment anchors (silence, plosives, long notes)
- EnhancedOnlineDTW: Online DTW with anchors and confidence gating
- LiveFeedback: Generate real-time coaching feedback
- RealtimePipeline: Complete real-time analysis pipeline
"""

from .buffer import StreamingAudioBuffer
from .pitch_stream import IncrementalPitchExtractor
from .anchors import AnchorDetector
from .feedback import LiveFeedback, RealtimeHints

__all__ = [
    "StreamingAudioBuffer",
    "IncrementalPitchExtractor",
    "AnchorDetector",
    "LiveFeedback",
    "RealtimeHints",
]
