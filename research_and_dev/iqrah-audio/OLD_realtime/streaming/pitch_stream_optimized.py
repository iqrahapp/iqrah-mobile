"""
Optimized Incremental Pitch Extraction
=======================================

Ultra-low latency pitch extraction with true incremental processing.
Target: <10ms per chunk (vs current ~60ms)
"""

import numpy as np
from typing import Tuple
from collections import deque

# Import low-level pitch extraction functions
try:
    import librosa
    HAS_LIBROSA = True
except ImportError:
    HAS_LIBROSA = False


class OptimizedIncrementalPitchExtractor:
    """
    Optimized incremental pitch extraction with <10ms latency.

    Key Optimizations:
    1. Frame-by-frame processing - only process new frames
    2. Direct low-level API calls - bypass overhead
    3. Minimal buffering - only keep what's needed
    4. No redundant computations

    Performance:
    - Current: ~60ms per 512-sample chunk
    - Target: <10ms per 512-sample chunk
    - Improvement: 6x faster

    The main issue with the current implementation is that it calls
    PitchExtractor.extract() on a large window for each chunk, which
    recomputes many overlapping frames. This implementation processes
    only the new frames that arrived.
    """

    def __init__(
        self,
        method: str = "yin",
        sample_rate: int = 22050,
        hop_length: int = 512,
        frame_length: int = 2048,
        fmin: float = 50.0,
        fmax: float = 1000.0,
        max_cache_frames: int = 300,
    ):
        """
        Initialize optimized incremental pitch extractor.

        Args:
            method: Pitch extraction method ("yin" only for now)
            sample_rate: Audio sample rate
            hop_length: Hop length in samples (~23ms at sr=22050, hop=512)
            frame_length: Frame length in samples (~93ms at sr=22050, len=2048)
            fmin: Minimum frequency (Hz)
            fmax: Maximum frequency (Hz)
            max_cache_frames: Maximum frames to cache
        """
        if not HAS_LIBROSA:
            raise ImportError("librosa required for optimized pitch extraction")

        if method != "yin":
            raise ValueError("OptimizedIncrementalPitchExtractor only supports YIN method")

        self.method = method
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.frame_length = frame_length
        self.fmin = fmin
        self.fmax = fmax
        self.max_cache_frames = max_cache_frames

        # Frame cache
        self.f0_cache = deque(maxlen=max_cache_frames)
        self.confidence_cache = deque(maxlen=max_cache_frames)
        self.timestamp_cache = deque(maxlen=max_cache_frames)

        # Audio buffer - only keep enough for frame extraction
        # We need frame_length samples to extract a frame
        self.audio_buffer = np.zeros(frame_length, dtype=np.float32)
        self.buffer_write_pos = 0  # Circular write position

        # Position tracking
        self.total_samples_written = 0
        self.next_frame_sample = 0  # Next sample index to extract frame from

    def process_chunk(
        self,
        audio_chunk: np.ndarray,
        return_new_only: bool = True,
    ) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        """
        Process new audio chunk with minimal latency.

        This is the optimized hot path - process only new frames.

        Args:
            audio_chunk: New audio samples
            return_new_only: Always True (for API compatibility)

        Returns:
            (f0_hz, confidence, timestamps) for new frames
        """
        audio_chunk = np.asarray(audio_chunk, dtype=np.float32).ravel()

        if len(audio_chunk) == 0:
            return np.array([]), np.array([]), np.array([])

        # Collect new frames
        new_f0 = []
        new_conf = []
        new_ts = []

        # Process each sample in the chunk
        for i in range(len(audio_chunk)):
            # Add sample to circular buffer
            self.audio_buffer[self.buffer_write_pos] = audio_chunk[i]
            self.buffer_write_pos = (self.buffer_write_pos + 1) % self.frame_length
            self.total_samples_written += 1

            # Check if we can extract a new frame
            # We extract a frame every hop_length samples
            if (self.total_samples_written - self.next_frame_sample) >= self.hop_length:
                # Extract frame at current position
                # Read frame_length samples from buffer (handle circular wrap)
                if self.buffer_write_pos >= self.frame_length:
                    # Simple case: no wrap (but this won't happen with circular buffer)
                    frame = self.audio_buffer[:self.frame_length].copy()
                else:
                    # Circular buffer case: read from write_pos backwards
                    frame = np.empty(self.frame_length, dtype=np.float32)
                    # Read backwards from current write position
                    for j in range(self.frame_length):
                        read_pos = (self.buffer_write_pos - self.frame_length + j) % self.frame_length
                        frame[j] = self.audio_buffer[read_pos]

                # Extract pitch from this frame using YIN
                f0, confidence = self._extract_frame_pitch(frame)

                # Calculate timestamp
                timestamp = self.next_frame_sample / self.sample_rate

                # Store frame
                new_f0.append(f0)
                new_conf.append(confidence)
                new_ts.append(timestamp)

                self.f0_cache.append(f0)
                self.confidence_cache.append(confidence)
                self.timestamp_cache.append(timestamp)

                # Move to next frame
                self.next_frame_sample += self.hop_length

        # Convert to arrays
        new_f0 = np.array(new_f0, dtype=np.float32)
        new_conf = np.array(new_conf, dtype=np.float32)
        new_ts = np.array(new_ts, dtype=np.float32)

        return new_f0, new_conf, new_ts

    def _extract_frame_pitch(self, frame: np.ndarray) -> Tuple[float, float]:
        """
        Extract pitch from a single frame using lightweight YIN.

        This is the core low-latency extraction function.
        Uses direct YIN implementation to avoid librosa overhead.

        Args:
            frame: Audio frame (frame_length samples)

        Returns:
            (f0_hz, confidence)
        """
        # Use optimized YIN implementation
        return self._yin_pitch(frame)

    def _yin_pitch(self, frame: np.ndarray) -> Tuple[float, float]:
        """
        Vectorized YIN algorithm for pitch detection.

        Optimized for minimal latency with numpy vectorization.

        Args:
            frame: Audio frame

        Returns:
            (f0_hz, confidence)
        """
        # Calculate period limits from frequency limits
        min_period = max(1, int(self.sample_rate / self.fmax))
        max_period = min(len(frame) // 2, int(self.sample_rate / self.fmin))

        if max_period <= min_period:
            return 0.0, 0.0

        # Vectorized difference function
        frame_length = len(frame)
        diff = np.zeros(max_period + 1, dtype=np.float32)

        for tau in range(1, max_period + 1):
            # Vectorized squared difference
            diff[tau] = np.sum((frame[:frame_length - tau] - frame[tau:frame_length]) ** 2)

        # Vectorized cumulative mean normalized difference
        cumsum = np.cumsum(diff[1:])
        # Avoid division by zero
        with np.errstate(divide='ignore', invalid='ignore'):
            cmnd = diff[1:] * np.arange(1, max_period + 1) / (cumsum + 1e-10)

        # Find first minimum below threshold
        threshold = 0.1
        tau_estimate = 0

        # Find indices where cmnd < threshold in the search range
        search_range = slice(min_period - 1, max_period - 1)
        candidates = np.where(cmnd[search_range] < threshold)[0]

        if len(candidates) > 0:
            # First candidate (add offset back)
            tau_idx = candidates[0] + (min_period - 1)

            # Parabolic interpolation for refinement
            if 0 < tau_idx < len(cmnd) - 1:
                alpha = cmnd[tau_idx - 1]
                beta = cmnd[tau_idx]
                gamma = cmnd[tau_idx + 1]

                denom = alpha - 2 * beta + gamma
                if abs(denom) > 1e-10:
                    peak_shift = 0.5 * (alpha - gamma) / denom
                    tau_estimate = (tau_idx + 1) + peak_shift  # +1 to adjust for diff offset
                else:
                    tau_estimate = tau_idx + 1
            else:
                tau_estimate = tau_idx + 1

        if tau_estimate == 0:
            # No period found
            return 0.0, 0.0

        # Convert period to frequency
        f0_hz = self.sample_rate / tau_estimate

        # Confidence from CMND value (inverted - lower CMND = higher confidence)
        tau_int = int(tau_estimate) - 1  # Adjust for diff offset
        if 0 <= tau_int < len(cmnd):
            confidence = float(1.0 - min(1.0, cmnd[tau_int]))
        else:
            confidence = 0.0

        # Sanity check
        if f0_hz < self.fmin or f0_hz > self.fmax:
            return 0.0, 0.0

        return float(f0_hz), float(confidence)

    def get_accumulated_result(self):
        """
        Get accumulated pitch result (for compatibility with AnchorDetector).

        Returns:
            Mock PitchResult with cached data
        """
        from ..pitch import PitchContour

        if len(self.f0_cache) == 0:
            return PitchContour(
                f0_hz=np.array([], dtype=np.float32),
                confidence=np.array([], dtype=np.float32),
                timestamps=np.array([], dtype=np.float32),
                sample_rate=self.sample_rate,
            )

        return PitchContour(
            f0_hz=np.array(list(self.f0_cache), dtype=np.float32),
            confidence=np.array(list(self.confidence_cache), dtype=np.float32),
            timestamps=np.array(list(self.timestamp_cache), dtype=np.float32),
            sample_rate=self.sample_rate,
        )

    def reset(self):
        """Reset extractor state."""
        self.f0_cache.clear()
        self.confidence_cache.clear()
        self.timestamp_cache.clear()
        self.audio_buffer.fill(0)
        self.buffer_write_pos = 0
        self.total_samples_written = 0
        self.next_frame_sample = 0

    @property
    def n_frames(self) -> int:
        """Number of cached frames."""
        return len(self.f0_cache)

    def __repr__(self) -> str:
        return (
            f"OptimizedIncrementalPitchExtractor(method={self.method}, "
            f"frames={self.n_frames}, sr={self.sample_rate})"
        )
