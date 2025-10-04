"""
Streaming Audio Buffer
======================

Efficient ring buffer for real-time audio processing.
"""

import numpy as np
from threading import Lock
from typing import Optional


class StreamingAudioBuffer:
    """
    Thread-safe ring buffer for streaming audio.

    Features:
    - Fixed-size circular buffer
    - Thread-safe push/read operations
    - Efficient numpy implementation
    - Zero-copy windowing

    Usage:
        buffer = StreamingAudioBuffer(window_size_s=3.0, sample_rate=22050)
        buffer.push_samples(new_audio)
        window = buffer.get_window()
    """

    def __init__(
        self,
        window_size_s: float = 3.0,
        sample_rate: int = 22050,
        dtype: np.dtype = np.float32,
    ):
        """
        Initialize streaming buffer.

        Args:
            window_size_s: Window size in seconds (default 3.0)
            sample_rate: Audio sample rate
            dtype: Data type for samples
        """
        self.window_size_s = window_size_s
        self.sample_rate = sample_rate
        self.dtype = dtype

        # Calculate buffer size
        self.buffer_size = int(window_size_s * sample_rate)

        # Create circular buffer
        self.buffer = np.zeros(self.buffer_size, dtype=dtype)

        # Write position (circular)
        self.write_pos = 0

        # Total samples written (for tracking)
        self.total_samples = 0

        # Thread safety
        self.lock = Lock()

        # Status
        self.is_full = False

    def push_samples(self, samples: np.ndarray) -> int:
        """
        Add new audio samples to buffer.

        Args:
            samples: Audio samples to add (1D array)

        Returns:
            Number of samples actually written
        """
        with self.lock:
            samples = np.asarray(samples, dtype=self.dtype).ravel()
            n_samples = len(samples)

            if n_samples == 0:
                return 0

            # Handle wrap-around
            if self.write_pos + n_samples <= self.buffer_size:
                # Simple case: no wrap
                self.buffer[self.write_pos:self.write_pos + n_samples] = samples
            else:
                # Wrap-around case
                first_chunk = self.buffer_size - self.write_pos
                self.buffer[self.write_pos:] = samples[:first_chunk]
                self.buffer[:n_samples - first_chunk] = samples[first_chunk:]

            # Update write position
            self.write_pos = (self.write_pos + n_samples) % self.buffer_size

            # Update tracking
            self.total_samples += n_samples

            # Mark as full once we've written enough
            if self.total_samples >= self.buffer_size:
                self.is_full = True

            return n_samples

    def get_window(self, size: Optional[int] = None) -> np.ndarray:
        """
        Get current window of audio.

        Args:
            size: Window size in samples (None = full buffer)

        Returns:
            Audio window (copy)
        """
        with self.lock:
            if size is None:
                size = self.buffer_size

            size = min(size, self.buffer_size)

            if not self.is_full:
                # Not enough data yet, return what we have
                return self.buffer[:self.write_pos].copy()

            # Read backwards from write position
            if size <= self.write_pos:
                # Simple case: no wrap
                return self.buffer[self.write_pos - size:self.write_pos].copy()
            else:
                # Wrap-around case
                first_chunk = self.write_pos
                second_chunk = size - first_chunk

                window = np.empty(size, dtype=self.dtype)
                window[:second_chunk] = self.buffer[-second_chunk:]
                window[second_chunk:] = self.buffer[:first_chunk]

                return window

    def get_latest_samples(self, n_samples: int) -> np.ndarray:
        """
        Get most recent N samples.

        Args:
            n_samples: Number of samples to retrieve

        Returns:
            Latest samples (copy)
        """
        return self.get_window(size=n_samples)

    def clear(self):
        """Clear buffer and reset."""
        with self.lock:
            self.buffer.fill(0)
            self.write_pos = 0
            self.total_samples = 0
            self.is_full = False

    @property
    def available_samples(self) -> int:
        """Number of samples currently available."""
        with self.lock:
            return min(self.total_samples, self.buffer_size)

    @property
    def duration_s(self) -> float:
        """Duration of available audio in seconds."""
        return self.available_samples / self.sample_rate

    def __len__(self) -> int:
        """Buffer size in samples."""
        return self.buffer_size

    def __repr__(self) -> str:
        return (
            f"StreamingAudioBuffer(window={self.window_size_s}s, "
            f"sr={self.sample_rate}, available={self.duration_s:.2f}s, "
            f"full={self.is_full})"
        )
