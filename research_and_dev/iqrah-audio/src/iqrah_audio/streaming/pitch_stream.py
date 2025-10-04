"""
Incremental Pitch Extraction
=============================

Real-time pitch extraction with frame caching for minimal recomputation.
"""

import numpy as np
import time
from typing import Optional, Tuple, Dict
from collections import deque

from ..pitch import PitchContour, PitchExtractor


class IncrementalPitchExtractor:
    """
    Incremental pitch extraction with caching.

    Optimizations:
    - Cache computed frames to avoid recomputation
    - Only process new samples since last call
    - Sliding window approach with overlap
    - Minimal memory footprint

    Target: <10ms per frame extraction

    Usage:
        extractor = IncrementalPitchExtractor(method="yin")

        # Process new audio chunk
        pitch_frames = extractor.process_chunk(audio_chunk)

        # Get full contour so far
        contour = extractor.get_contour()
    """

    def __init__(
        self,
        method: str = "yin",
        sample_rate: int = 22050,
        hop_length: int = 512,  # ~23ms at 22050 Hz
        frame_length: int = 2048,  # ~93ms
        fmin: float = 50.0,
        fmax: float = 1000.0,
        max_cache_frames: int = 300,  # ~7s at 23ms/frame
    ):
        """
        Initialize incremental pitch extractor.

        Args:
            method: Pitch extraction method ("yin", "crepe")
            sample_rate: Audio sample rate
            hop_length: Hop length in samples
            frame_length: Frame length in samples
            fmin: Minimum frequency (Hz)
            fmax: Maximum frequency (Hz)
            max_cache_frames: Maximum frames to keep in cache
        """
        self.method = method
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.frame_length = frame_length
        self.fmin = fmin
        self.fmax = fmax
        self.max_cache_frames = max_cache_frames

        # Initialize base extractor
        self.extractor = PitchExtractor(
            method=method,
            sample_rate=sample_rate,
            hop_length=hop_length,
        )

        # Frame cache: deque for efficient append/pop
        self.f0_cache = deque(maxlen=max_cache_frames)
        self.confidence_cache = deque(maxlen=max_cache_frames)
        self.timestamp_cache = deque(maxlen=max_cache_frames)

        # Audio buffer for overlap handling
        self.audio_buffer = np.array([], dtype=np.float32)

        # Position tracking
        self.total_samples_processed = 0
        self.last_frame_idx = 0

    def process_chunk(
        self,
        audio_chunk: np.ndarray,
        return_new_only: bool = True,
    ) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        """
        Process new audio chunk incrementally.

        Args:
            audio_chunk: New audio samples
            return_new_only: If True, return only newly computed frames

        Returns:
            (f0_hz, confidence, timestamps) for new frames
        """
        # Convert to float32
        audio_chunk = np.asarray(audio_chunk, dtype=np.float32).ravel()

        if len(audio_chunk) == 0:
            return np.array([]), np.array([]), np.array([])

        # Append to buffer
        self.audio_buffer = np.concatenate([self.audio_buffer, audio_chunk])

        # Calculate how many new frames we can extract
        available_samples = len(self.audio_buffer)

        # Need at least frame_length samples for first frame
        if available_samples < self.frame_length:
            return np.array([]), np.array([]), np.array([])

        # Calculate frame indices we can extract
        max_frames = (available_samples - self.frame_length) // self.hop_length + 1

        if max_frames <= 0:
            return np.array([]), np.array([]), np.array([])

        # Extract frames we haven't processed yet
        start_frame = self.last_frame_idx
        end_frame = max_frames
        n_new_frames = end_frame - start_frame

        if n_new_frames <= 0:
            return np.array([]), np.array([]), np.array([])

        # Extract pitch for new frames
        # We extract from a window to get context
        start_sample = max(0, start_frame * self.hop_length)
        end_sample = min(available_samples, end_frame * self.hop_length + self.frame_length)

        window = self.audio_buffer[start_sample:end_sample]

        # Extract pitch
        contour = self.extractor.extract(window)

        # The contour might have more frames than we need (due to windowing)
        # We only want the new frames
        new_f0 = contour.f0_hz[-n_new_frames:]
        new_conf = contour.confidence[-n_new_frames:]

        # Calculate timestamps
        new_timestamps = (
            self.total_samples_processed +
            np.arange(start_frame, end_frame) * self.hop_length
        ) / self.sample_rate

        # Update cache
        for i in range(n_new_frames):
            self.f0_cache.append(new_f0[i])
            self.confidence_cache.append(new_conf[i])
            self.timestamp_cache.append(new_timestamps[i])

        # Update tracking
        self.last_frame_idx = end_frame
        self.total_samples_processed += len(audio_chunk)

        # Trim audio buffer to keep only necessary samples
        # Keep enough for next frame extraction (frame_length + some overlap)
        min_samples_needed = self.frame_length + self.hop_length
        if len(self.audio_buffer) > min_samples_needed * 2:
            samples_to_keep = min_samples_needed
            self.audio_buffer = self.audio_buffer[-samples_to_keep:]
            # Adjust last_frame_idx accordingly
            removed_samples = available_samples - samples_to_keep
            frames_removed = removed_samples // self.hop_length
            self.last_frame_idx = max(0, self.last_frame_idx - frames_removed)

        return new_f0, new_conf, new_timestamps

    def get_contour(self, max_frames: Optional[int] = None) -> PitchContour:
        """
        Get accumulated pitch contour.

        Args:
            max_frames: Maximum number of recent frames (None = all)

        Returns:
            PitchContour with all cached frames
        """
        if len(self.f0_cache) == 0:
            # Return empty contour
            return PitchContour(
                f0_hz=np.array([], dtype=np.float32),
                confidence=np.array([], dtype=np.float32),
                timestamps=np.array([], dtype=np.float32),
                sample_rate=self.sample_rate,
            )

        # Get frames (most recent if limited)
        if max_frames is not None and len(self.f0_cache) > max_frames:
            f0 = np.array(list(self.f0_cache)[-max_frames:], dtype=np.float32)
            conf = np.array(list(self.confidence_cache)[-max_frames:], dtype=np.float32)
            ts = np.array(list(self.timestamp_cache)[-max_frames:], dtype=np.float32)
        else:
            f0 = np.array(list(self.f0_cache), dtype=np.float32)
            conf = np.array(list(self.confidence_cache), dtype=np.float32)
            ts = np.array(list(self.timestamp_cache), dtype=np.float32)

        return PitchContour(
            f0_hz=f0,
            confidence=conf,
            timestamps=ts,
            sample_rate=self.sample_rate,
        )

    def get_latest_frames(self, n_frames: int = 10) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        """
        Get most recent N frames.

        Args:
            n_frames: Number of recent frames to get

        Returns:
            (f0_hz, confidence, timestamps)
        """
        if len(self.f0_cache) == 0:
            return np.array([]), np.array([]), np.array([])

        n = min(n_frames, len(self.f0_cache))

        f0 = np.array(list(self.f0_cache)[-n:], dtype=np.float32)
        conf = np.array(list(self.confidence_cache)[-n:], dtype=np.float32)
        ts = np.array(list(self.timestamp_cache)[-n:], dtype=np.float32)

        return f0, conf, ts

    def clear(self):
        """Clear all cached frames."""
        self.f0_cache.clear()
        self.confidence_cache.clear()
        self.timestamp_cache.clear()
        self.audio_buffer = np.array([], dtype=np.float32)
        self.total_samples_processed = 0
        self.last_frame_idx = 0

    @property
    def n_frames(self) -> int:
        """Number of frames in cache."""
        return len(self.f0_cache)

    @property
    def duration(self) -> float:
        """Duration of cached audio in seconds."""
        if len(self.timestamp_cache) == 0:
            return 0.0
        return self.timestamp_cache[-1] - self.timestamp_cache[0]

    def __repr__(self) -> str:
        return (
            f"IncrementalPitchExtractor(method={self.method}, "
            f"frames={self.n_frames}, duration={self.duration:.2f}s)"
        )


class StreamingPitchAnalyzer:
    """
    High-level streaming pitch analyzer.

    Combines buffer + incremental extraction for easy usage.
    """

    def __init__(
        self,
        method: str = "yin",
        sample_rate: int = 22050,
        buffer_size_s: float = 3.0,
        hop_length: int = 512,
    ):
        """
        Initialize streaming analyzer.

        Args:
            method: Pitch extraction method
            sample_rate: Audio sample rate
            buffer_size_s: Audio buffer size in seconds
            hop_length: Hop length for pitch extraction
        """
        from .buffer import StreamingAudioBuffer

        self.buffer = StreamingAudioBuffer(
            window_size_s=buffer_size_s,
            sample_rate=sample_rate,
        )

        self.extractor = IncrementalPitchExtractor(
            method=method,
            sample_rate=sample_rate,
            hop_length=hop_length,
        )

        self.sample_rate = sample_rate

    def push_audio(self, audio: np.ndarray) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        """
        Push new audio and get newly extracted pitch frames.

        Args:
            audio: New audio samples

        Returns:
            (f0_hz, confidence, timestamps) for new frames
        """
        # Add to buffer
        self.buffer.push_samples(audio)

        # Process chunk
        return self.extractor.process_chunk(audio)

    def get_contour(self) -> PitchContour:
        """Get full accumulated pitch contour."""
        return self.extractor.get_contour()

    def clear(self):
        """Clear buffer and cache."""
        self.buffer.clear()
        self.extractor.clear()
