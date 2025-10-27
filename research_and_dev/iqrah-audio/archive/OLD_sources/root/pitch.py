"""
Pitch Extraction Module
=======================

SOTA pitch tracking using CREPE (neural network) with YIN fallback.
Optimized for Qur'anic recitation with noise robustness.
"""

import numpy as np
from dataclasses import dataclass
from typing import Optional, Tuple
import warnings

try:
    import crepe
    CREPE_AVAILABLE = True
except ImportError:
    CREPE_AVAILABLE = False
    warnings.warn("CREPE not available, falling back to librosa YIN")

import librosa


@dataclass
class PitchContour:
    """
    Pitch contour representation optimized for mobile.

    Attributes:
        f0_hz: Fundamental frequency in Hz (0 = unvoiced)
        confidence: Voicing confidence [0, 1]
        timestamps: Time in seconds
        sample_rate: Original audio sample rate
    """
    f0_hz: np.ndarray  # Shape: (n_frames,)
    confidence: np.ndarray  # Shape: (n_frames,)
    timestamps: np.ndarray  # Shape: (n_frames,)
    sample_rate: int

    @property
    def f0_cents(self) -> np.ndarray:
        """Convert Hz to cents relative to A4 (440 Hz)."""
        with np.errstate(divide='ignore', invalid='ignore'):
            cents = 1200 * np.log2(self.f0_hz / 440.0)
            cents[self.f0_hz == 0] = 0  # Unvoiced frames
        return cents

    @property
    def f0_semitones(self) -> np.ndarray:
        """Convert Hz to semitones relative to A4."""
        return self.f0_cents / 100.0

    @property
    def duration(self) -> float:
        """Total duration in seconds."""
        return self.timestamps[-1] if len(self.timestamps) > 0 else 0.0

    def get_voiced_segments(self, confidence_threshold: float = 0.5) -> list[Tuple[float, float]]:
        """
        Extract voiced segments as (start_time, end_time) pairs.

        Args:
            confidence_threshold: Minimum confidence for voiced frames

        Returns:
            List of (start, end) time pairs for voiced segments
        """
        voiced = self.confidence >= confidence_threshold
        segments = []

        in_segment = False
        start_time = 0.0

        for i, is_voiced in enumerate(voiced):
            if is_voiced and not in_segment:
                start_time = self.timestamps[i]
                in_segment = True
            elif not is_voiced and in_segment:
                end_time = self.timestamps[i - 1]
                segments.append((start_time, end_time))
                in_segment = False

        # Close final segment if still open
        if in_segment:
            segments.append((start_time, self.timestamps[-1]))

        return segments

    def to_dict(self) -> dict:
        """Convert to dictionary for CBOR serialization."""
        return {
            "f0_hz": self.f0_hz.tolist(),
            "confidence": self.confidence.tolist(),
            "timestamps": self.timestamps.tolist(),
            "sample_rate": self.sample_rate,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "PitchContour":
        """Load from dictionary (CBOR deserialization)."""
        return cls(
            f0_hz=np.array(data["f0_hz"], dtype=np.float32),
            confidence=np.array(data["confidence"], dtype=np.float32),
            timestamps=np.array(data["timestamps"], dtype=np.float32),
            sample_rate=data["sample_rate"],
        )


class PitchExtractor:
    """
    Pitch extraction using CREPE (SOTA) with YIN fallback.

    CREPE: Convolutional neural network, very noise-robust
    YIN: Classic autocorrelation method, lightweight fallback
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        hop_length: int = 512,
        method: str = "auto",  # "crepe", "yin", "auto"
        crepe_model: str = "tiny",  # "tiny", "small", "medium", "large", "full"
    ):
        """
        Initialize pitch extractor.

        Args:
            sample_rate: Target sample rate for analysis
            hop_length: Hop length in samples (controls time resolution)
            method: Extraction method ("crepe", "yin", or "auto")
            crepe_model: CREPE model size (tiny=fastest, full=most accurate)
        """
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.crepe_model = crepe_model

        # Auto-select best available method
        if method == "auto":
            self.method = "crepe" if CREPE_AVAILABLE else "yin"
        else:
            self.method = method

        if self.method == "crepe" and not CREPE_AVAILABLE:
            warnings.warn("CREPE requested but not available, using YIN")
            self.method = "yin"

    def extract(self, audio: np.ndarray, sr: Optional[int] = None) -> PitchContour:
        """
        Extract pitch contour from audio.

        Args:
            audio: Audio signal (1D array)
            sr: Sample rate of input audio (resamples if different from self.sample_rate)

        Returns:
            PitchContour object with f0, confidence, and timestamps
        """
        # Resample if needed
        if sr is not None and sr != self.sample_rate:
            import resampy
            audio = resampy.resample(audio, sr, self.sample_rate)

        if self.method == "crepe":
            return self._extract_crepe(audio)
        else:
            return self._extract_yin(audio)

    def _extract_crepe(self, audio: np.ndarray) -> PitchContour:
        """Extract pitch using CREPE neural network."""
        # CREPE expects specific sample rate
        crepe_sr = 16000
        if self.sample_rate != crepe_sr:
            import resampy
            audio_resampled = resampy.resample(audio, self.sample_rate, crepe_sr)
        else:
            audio_resampled = audio

        # Run CREPE
        time, frequency, confidence, activation = crepe.predict(
            audio_resampled,
            crepe_sr,
            viterbi=True,  # Smooth pitch trajectory
            model_capacity=self.crepe_model,
            step_size=int(1000 * self.hop_length / self.sample_rate),  # ms
        )

        return PitchContour(
            f0_hz=frequency.astype(np.float32),
            confidence=confidence.astype(np.float32),
            timestamps=time.astype(np.float32),
            sample_rate=self.sample_rate,
        )

    def _extract_yin(self, audio: np.ndarray) -> PitchContour:
        """Extract pitch using librosa's pYIN algorithm."""
        f0, voiced_flag, voiced_probs = librosa.pyin(
            audio,
            fmin=librosa.note_to_hz('C2'),  # ~65 Hz (male bass)
            fmax=librosa.note_to_hz('C7'),  # ~2093 Hz (high soprano)
            sr=self.sample_rate,
            hop_length=self.hop_length,
        )

        # Handle NaNs (unvoiced frames)
        f0 = np.nan_to_num(f0, nan=0.0)
        confidence = np.nan_to_num(voiced_probs, nan=0.0)

        # Generate timestamps
        n_frames = len(f0)
        timestamps = librosa.frames_to_time(
            np.arange(n_frames),
            sr=self.sample_rate,
            hop_length=self.hop_length
        )

        return PitchContour(
            f0_hz=f0.astype(np.float32),
            confidence=confidence.astype(np.float32),
            timestamps=timestamps.astype(np.float32),
            sample_rate=self.sample_rate,
        )

    def extract_stable_pitch(
        self,
        audio: np.ndarray,
        sr: Optional[int] = None,
        median_filter_size: int = 5
    ) -> PitchContour:
        """
        Extract pitch with additional stabilization.

        Applies median filtering to reduce octave errors and jitter.

        Args:
            audio: Audio signal
            sr: Sample rate
            median_filter_size: Size of median filter (odd number)

        Returns:
            Stabilized pitch contour
        """
        from scipy.signal import medfilt

        contour = self.extract(audio, sr)

        # Apply median filter to f0 (preserving 0s for unvoiced)
        voiced_mask = contour.f0_hz > 0
        f0_filtered = contour.f0_hz.copy()

        if np.any(voiced_mask):
            # Only filter voiced regions
            f0_filtered[voiced_mask] = medfilt(
                contour.f0_hz[voiced_mask],
                kernel_size=median_filter_size
            )

        return PitchContour(
            f0_hz=f0_filtered,
            confidence=contour.confidence,
            timestamps=contour.timestamps,
            sample_rate=contour.sample_rate,
        )
