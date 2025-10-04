"""
Real-Time Pipeline
==================

Complete real-time Quranic recitation analysis pipeline.
Target: <100ms end-to-end latency.
"""

import numpy as np
import time
from typing import Optional, List, Callable
from dataclasses import dataclass

from .buffer import StreamingAudioBuffer
from .pitch_stream import IncrementalPitchExtractor
from .anchors import AnchorDetector, Anchor
from .online_dtw import EnhancedOnlineDTW, OnlineAlignmentState
from .feedback import LiveFeedback, RealtimeHints
from ..pitch import PitchExtractor


@dataclass
class PipelineConfig:
    """
    Configuration for real-time pipeline.

    Audio Settings:
    - sample_rate: Audio sample rate (Hz)
    - hop_length: Hop length for pitch extraction
    - buffer_size_s: Audio buffer window size (seconds)

    Pitch Extraction:
    - pitch_method: Pitch extraction method ("yin", "pyin", "crepe")
    - min_freq: Minimum pitch frequency (Hz)
    - max_freq: Maximum pitch frequency (Hz)

    Alignment:
    - dtw_window_size: DTW window size (frames)
    - dtw_band_width: DTW band width (frames)
    - confidence_threshold: Confidence gating threshold

    Feedback:
    - update_rate_hz: Feedback update rate (Hz)
    - on_note_threshold_cents: On-note threshold (cents)
    - smoothing_alpha: EMA smoothing factor

    Anchors:
    - enable_anchors: Enable anchor detection
    - anchor_min_confidence: Minimum anchor confidence
    """

    # Audio settings
    sample_rate: int = 22050
    hop_length: int = 512
    buffer_size_s: float = 3.0

    # Pitch extraction
    pitch_method: str = "yin"
    min_freq: float = 80.0
    max_freq: float = 800.0

    # Alignment
    dtw_window_size: int = 300
    dtw_band_width: int = 50
    confidence_threshold: float = 0.3  # Lowered for real speech (has unvoiced segments)

    # Feedback
    update_rate_hz: float = 15.0
    on_note_threshold_cents: float = 50.0
    smoothing_alpha: float = 0.3

    # Anchors
    enable_anchors: bool = True
    anchor_min_confidence: float = 0.7


@dataclass
class PipelineStats:
    """
    Real-time pipeline statistics.

    Tracks latency and processing metrics for performance monitoring.
    """

    total_frames_processed: int = 0
    total_audio_duration_s: float = 0.0

    # Latency tracking (ms)
    pitch_latency_ms: float = 0.0
    anchor_latency_ms: float = 0.0
    dtw_latency_ms: float = 0.0
    feedback_latency_ms: float = 0.0
    total_latency_ms: float = 0.0

    # Processing stats
    hints_generated: int = 0
    anchors_detected: int = 0

    def update_latency(self, pitch_ms: float, anchor_ms: float,
                      dtw_ms: float, feedback_ms: float):
        """Update latency estimates with exponential moving average."""
        alpha = 0.1  # EMA smoothing

        self.pitch_latency_ms = alpha * pitch_ms + (1 - alpha) * self.pitch_latency_ms
        self.anchor_latency_ms = alpha * anchor_ms + (1 - alpha) * self.anchor_latency_ms
        self.dtw_latency_ms = alpha * dtw_ms + (1 - alpha) * self.dtw_latency_ms
        self.feedback_latency_ms = alpha * feedback_ms + (1 - alpha) * self.feedback_latency_ms

        self.total_latency_ms = (
            self.pitch_latency_ms +
            self.anchor_latency_ms +
            self.dtw_latency_ms +
            self.feedback_latency_ms
        )


class RealtimePipeline:
    """
    Complete real-time Quranic recitation analysis pipeline.

    Combines all streaming components for end-to-end processing:
    1. Audio buffering (StreamingAudioBuffer)
    2. Incremental pitch extraction (IncrementalPitchExtractor)
    3. Anchor detection (AnchorDetector)
    4. Online alignment (EnhancedOnlineDTW)
    5. Live feedback generation (LiveFeedback)

    Target: <100ms end-to-end latency

    Usage:
        # Create pipeline
        pipeline = RealtimePipeline(reference_audio)

        # Process streaming audio
        for chunk in audio_stream:
            hints = pipeline.process_chunk(chunk)
            if hints:
                display_feedback(hints)

        # Get statistics
        stats = pipeline.get_stats()
        print(f"Total latency: {stats.total_latency_ms:.2f}ms")
    """

    def __init__(
        self,
        reference_audio: np.ndarray,
        config: Optional[PipelineConfig] = None,
        on_hints_callback: Optional[Callable[[RealtimeHints], None]] = None,
    ):
        """
        Initialize real-time pipeline.

        Args:
            reference_audio: Reference audio (1D float32 array)
            config: Pipeline configuration (default: PipelineConfig())
            on_hints_callback: Optional callback for hints (called when hints generated)
        """
        self.config = config or PipelineConfig()
        self.on_hints_callback = on_hints_callback

        # Extract reference pitch
        print("Extracting reference pitch...")
        pitch_extractor = PitchExtractor(
            method=self.config.pitch_method,
            sample_rate=self.config.sample_rate,
            hop_length=self.config.hop_length,
        )

        self.reference_pitch = pitch_extractor.extract_stable_pitch(reference_audio)
        print(f"✓ Reference: {len(self.reference_pitch.f0_hz)} frames")

        # Detect reference anchors
        self.reference_anchors = []
        if self.config.enable_anchors:
            print("Detecting reference anchors...")
            from ..features import FeatureExtractor

            # Extract features for anchor detection
            feat_extractor = FeatureExtractor(
                sample_rate=self.config.sample_rate,
                extract_chroma=False,
                extract_energy=True,
                extract_spectral=True,
            )
            features = feat_extractor.extract_all(reference_audio, self.reference_pitch)

            anchor_detector = AnchorDetector(
                sample_rate=self.config.sample_rate,
                hop_length=self.config.hop_length,
            )

            self.reference_anchors = anchor_detector.detect_all(
                f0_hz=self.reference_pitch.f0_hz,
                confidence=self.reference_pitch.confidence,
                rms=features.rms,
                spectral_flatness=features.spectral_flatness,
                timestamps=self.reference_pitch.timestamps,
            )

            # Filter by confidence
            self.reference_anchors = [
                a for a in self.reference_anchors
                if a.confidence >= self.config.anchor_min_confidence
            ]

            print(f"✓ Anchors: {len(self.reference_anchors)} detected")

        # Create pipeline components
        self.audio_buffer = StreamingAudioBuffer(
            window_size_s=self.config.buffer_size_s,
            sample_rate=self.config.sample_rate,
        )

        # Use optimized pitch extractor for low latency
        from .pitch_stream_optimized import OptimizedIncrementalPitchExtractor

        self.pitch_extractor = OptimizedIncrementalPitchExtractor(
            method=self.config.pitch_method,
            sample_rate=self.config.sample_rate,
            hop_length=self.config.hop_length,
        )

        self.anchor_detector = AnchorDetector(
            sample_rate=self.config.sample_rate,
            hop_length=self.config.hop_length,
        )

        self.online_dtw = EnhancedOnlineDTW(
            window_size=self.config.dtw_window_size,
            band_width=self.config.dtw_band_width,
            confidence_threshold=self.config.confidence_threshold,
            sample_rate=self.config.sample_rate,
            hop_length=self.config.hop_length,
        )

        # Set reference anchors for DTW
        if self.reference_anchors:
            self.online_dtw.set_reference_anchors(self.reference_anchors)

        self.feedback_generator = LiveFeedback(
            update_rate_hz=self.config.update_rate_hz,
            on_note_threshold_cents=self.config.on_note_threshold_cents,
            smoothing_alpha=self.config.smoothing_alpha,
        )

        # Statistics
        self.stats = PipelineStats()

        print("✓ Pipeline ready")

    def process_chunk(self, audio_chunk: np.ndarray) -> Optional[RealtimeHints]:
        """
        Process a chunk of streaming audio.

        This is the main entry point for real-time processing.

        Args:
            audio_chunk: Audio chunk (1D float32 array)

        Returns:
            RealtimeHints if feedback generated, None otherwise
        """
        t0 = time.perf_counter()

        # 1. Buffer audio
        self.audio_buffer.push_samples(audio_chunk)

        # 2. Extract pitch incrementally
        t1 = time.perf_counter()
        f0_frames, conf_frames, time_frames = self.pitch_extractor.process_chunk(audio_chunk)
        t2 = time.perf_counter()
        pitch_ms = (t2 - t1) * 1000

        if len(f0_frames) == 0:
            return None

        # Process each new pitch frame
        hints = None
        for i in range(len(f0_frames)):
            f0_hz = f0_frames[i]
            confidence = conf_frames[i]
            # 3. Detect anchors (if enabled)
            t3 = time.perf_counter()
            query_anchor = None
            # Note: Anchor detection in streaming is complex and would require
            # feature extraction on the streaming audio. For now, we skip this
            # in the streaming path and rely only on reference anchors for drift correction.
            # This can be optimized in Phase 1 along with pitch extraction.

            t4 = time.perf_counter()
            anchor_ms = (t4 - t3) * 1000

            # 4. Update alignment
            t5 = time.perf_counter()
            alignment_state = self.online_dtw.update(
                query_frame=f0_hz,
                query_confidence=confidence,
                reference=self.reference_pitch.f0_hz,
                query_anchor=query_anchor,
            )
            t6 = time.perf_counter()
            dtw_ms = (t6 - t5) * 1000

            # 5. Generate feedback
            t7 = time.perf_counter()
            hints = self.feedback_generator.generate_hints(
                alignment_state=alignment_state,
                current_pitch_hz=f0_hz,
                current_confidence=confidence,
                reference_pitch_hz=self.reference_pitch.f0_hz,
            )
            t8 = time.perf_counter()
            feedback_ms = (t8 - t7) * 1000

            # Update stats
            self.stats.update_latency(pitch_ms, anchor_ms, dtw_ms, feedback_ms)
            self.stats.total_frames_processed += 1

            if hints:
                self.stats.hints_generated += 1

                # Call callback if provided
                if self.on_hints_callback:
                    self.on_hints_callback(hints)

        # Update audio duration
        self.stats.total_audio_duration_s += len(audio_chunk) / self.config.sample_rate

        return hints

    def reset(self):
        """Reset pipeline state (for new recitation)."""
        self.audio_buffer.clear()
        self.pitch_extractor.reset()
        # Note: DTW and feedback keep their state for continuity

    def get_stats(self) -> PipelineStats:
        """Get pipeline statistics."""
        return self.stats

    def get_alignment_state(self) -> OnlineAlignmentState:
        """Get current alignment state."""
        return self.online_dtw.state

    def __repr__(self) -> str:
        return (
            f"RealtimePipeline("
            f"reference_frames={len(self.reference_pitch.f0_hz)}, "
            f"anchors={len(self.reference_anchors)}, "
            f"processed={self.stats.total_frames_processed}, "
            f"latency={self.stats.total_latency_ms:.2f}ms)"
        )
