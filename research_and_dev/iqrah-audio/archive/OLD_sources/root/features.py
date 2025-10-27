"""
Multi-Dimensional Feature Extraction
=====================================

Extract rich features for improved alignment and analysis:
- F0 (pitch)
- Mel-spectrogram (timbre)
- Chroma (octave-invariant pitch)
- Energy (RMS, spectral centroid)
- Spectral features (flatness, ZCR)
"""

import numpy as np
from dataclasses import dataclass
from typing import Optional, Tuple
import librosa


@dataclass
class AudioFeatures:
    """
    Multi-dimensional audio features for alignment and analysis.

    Attributes:
        f0_hz: Fundamental frequency (pitch)
        f0_confidence: Voicing confidence
        mel_spec: Log-mel spectrogram (timbre)
        chroma: Chroma features (octave-invariant pitch)
        rms: Root mean square energy
        spectral_centroid: Spectral centroid (brightness)
        spectral_flatness: Spectral flatness (noisiness)
        zcr: Zero crossing rate (voicing proxy)
        timestamps: Time axis
        sample_rate: Audio sample rate
    """
    f0_hz: np.ndarray  # (n_frames,)
    f0_confidence: np.ndarray  # (n_frames,)
    mel_spec: np.ndarray  # (n_mels, n_frames)
    chroma: Optional[np.ndarray] = None  # (12, n_frames)
    rms: Optional[np.ndarray] = None  # (n_frames,)
    spectral_centroid: Optional[np.ndarray] = None  # (n_frames,)
    spectral_flatness: Optional[np.ndarray] = None  # (n_frames,)
    zcr: Optional[np.ndarray] = None  # (n_frames,)
    timestamps: Optional[np.ndarray] = None  # (n_frames,)
    sample_rate: int = 22050

    @property
    def n_frames(self) -> int:
        """Number of frames."""
        return len(self.f0_hz)

    @property
    def f0_cents(self) -> np.ndarray:
        """Convert F0 to cents relative to A4."""
        with np.errstate(divide='ignore', invalid='ignore'):
            cents = 1200 * np.log2(self.f0_hz / 440.0)
            cents[self.f0_hz == 0] = 0
        return cents

    def to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        data = {
            "f0_hz": self.f0_hz.tolist(),
            "f0_confidence": self.f0_confidence.tolist(),
            "mel_spec": self.mel_spec.tolist(),
            "sample_rate": self.sample_rate,
        }

        # Optional features
        if self.chroma is not None:
            data["chroma"] = self.chroma.tolist()
        if self.rms is not None:
            data["rms"] = self.rms.tolist()
        if self.spectral_centroid is not None:
            data["spectral_centroid"] = self.spectral_centroid.tolist()
        if self.spectral_flatness is not None:
            data["spectral_flatness"] = self.spectral_flatness.tolist()
        if self.zcr is not None:
            data["zcr"] = self.zcr.tolist()
        if self.timestamps is not None:
            data["timestamps"] = self.timestamps.tolist()

        return data

    @classmethod
    def from_dict(cls, data: dict) -> "AudioFeatures":
        """Load from dictionary."""
        return cls(
            f0_hz=np.array(data["f0_hz"], dtype=np.float32),
            f0_confidence=np.array(data["f0_confidence"], dtype=np.float32),
            mel_spec=np.array(data["mel_spec"], dtype=np.float32),
            chroma=np.array(data["chroma"], dtype=np.float32) if "chroma" in data else None,
            rms=np.array(data["rms"], dtype=np.float32) if "rms" in data else None,
            spectral_centroid=np.array(data["spectral_centroid"], dtype=np.float32)
                if "spectral_centroid" in data else None,
            spectral_flatness=np.array(data["spectral_flatness"], dtype=np.float32)
                if "spectral_flatness" in data else None,
            zcr=np.array(data["zcr"], dtype=np.float32) if "zcr" in data else None,
            timestamps=np.array(data["timestamps"], dtype=np.float32)
                if "timestamps" in data else None,
            sample_rate=data["sample_rate"],
        )


class FeatureExtractor:
    """
    Extract multi-dimensional features from audio.

    Combines pitch, timbre, and energy features for robust alignment.
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        hop_length: int = 512,
        n_fft: int = 2048,
        n_mels: int = 80,
        extract_chroma: bool = True,
        extract_energy: bool = True,
        extract_spectral: bool = True,
    ):
        """
        Initialize feature extractor.

        Args:
            sample_rate: Audio sample rate
            hop_length: Hop length in samples
            n_fft: FFT window size
            n_mels: Number of mel bands
            extract_chroma: Extract chroma features
            extract_energy: Extract RMS energy
            extract_spectral: Extract spectral features
        """
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.n_fft = n_fft
        self.n_mels = n_mels
        self.should_extract_chroma = extract_chroma
        self.should_extract_energy = extract_energy
        self.should_extract_spectral = extract_spectral

    def extract_mel_spectrogram(
        self,
        audio: np.ndarray,
        log: bool = True,
    ) -> np.ndarray:
        """
        Extract mel-spectrogram (timbre features).

        Args:
            audio: Audio signal
            log: Apply log scaling

        Returns:
            Mel-spectrogram (n_mels, n_frames)
        """
        mel_spec = librosa.feature.melspectrogram(
            y=audio,
            sr=self.sample_rate,
            n_fft=self.n_fft,
            hop_length=self.hop_length,
            n_mels=self.n_mels,
            fmin=50,  # Lower bound for Quranic recitation
            fmax=8000,  # Upper bound
        )

        if log:
            # Convert to log scale (dB)
            mel_spec = librosa.power_to_db(mel_spec, ref=np.max)

        return mel_spec.astype(np.float32)

    def extract_chroma(self, audio: np.ndarray) -> np.ndarray:
        """
        Extract chroma features (octave-invariant pitch).

        Useful for detecting octave errors and matching across registers.

        Args:
            audio: Audio signal

        Returns:
            Chroma features (12, n_frames)
        """
        chroma = librosa.feature.chroma_cqt(
            y=audio,
            sr=self.sample_rate,
            hop_length=self.hop_length,
            n_chroma=12,
        )

        return chroma.astype(np.float32)

    def extract_energy(self, audio: np.ndarray) -> np.ndarray:
        """
        Extract RMS energy.

        Args:
            audio: Audio signal

        Returns:
            RMS energy per frame (n_frames,)
        """
        rms = librosa.feature.rms(
            y=audio,
            frame_length=self.n_fft,
            hop_length=self.hop_length,
        )[0]

        return rms.astype(np.float32)

    def extract_spectral_centroid(self, audio: np.ndarray) -> np.ndarray:
        """
        Extract spectral centroid (brightness).

        Args:
            audio: Audio signal

        Returns:
            Spectral centroid per frame (n_frames,)
        """
        centroid = librosa.feature.spectral_centroid(
            y=audio,
            sr=self.sample_rate,
            n_fft=self.n_fft,
            hop_length=self.hop_length,
        )[0]

        return centroid.astype(np.float32)

    def extract_spectral_flatness(self, audio: np.ndarray) -> np.ndarray:
        """
        Extract spectral flatness (noisiness).

        High flatness = noise-like (good for detecting qalqalah bursts).
        Low flatness = tonal.

        Args:
            audio: Audio signal

        Returns:
            Spectral flatness per frame (n_frames,)
        """
        flatness = librosa.feature.spectral_flatness(
            y=audio,
            n_fft=self.n_fft,
            hop_length=self.hop_length,
        )[0]

        return flatness.astype(np.float32)

    def extract_zcr(self, audio: np.ndarray) -> np.ndarray:
        """
        Extract zero-crossing rate (voicing proxy).

        Args:
            audio: Audio signal

        Returns:
            ZCR per frame (n_frames,)
        """
        zcr = librosa.feature.zero_crossing_rate(
            y=audio,
            frame_length=self.n_fft,
            hop_length=self.hop_length,
        )[0]

        return zcr.astype(np.float32)

    def extract_all(
        self,
        audio: np.ndarray,
        pitch_contour: "PitchContour",  # From pitch.py
    ) -> AudioFeatures:
        """
        Extract all features from audio.

        Args:
            audio: Audio signal
            pitch_contour: Pre-computed pitch contour

        Returns:
            AudioFeatures object with all features
        """
        # Mel-spectrogram (always extracted)
        mel_spec = self.extract_mel_spectrogram(audio)

        # Optional features
        chroma = self.extract_chroma(audio) if self.should_extract_chroma else None
        rms = self.extract_energy(audio) if self.should_extract_energy else None

        spectral_centroid = None
        spectral_flatness = None
        zcr = None

        if self.should_extract_spectral:
            spectral_centroid = self.extract_spectral_centroid(audio)
            spectral_flatness = self.extract_spectral_flatness(audio)
            zcr = self.extract_zcr(audio)

        # Create features object
        features = AudioFeatures(
            f0_hz=pitch_contour.f0_hz,
            f0_confidence=pitch_contour.confidence,
            mel_spec=mel_spec,
            chroma=chroma,
            rms=rms,
            spectral_centroid=spectral_centroid,
            spectral_flatness=spectral_flatness,
            zcr=zcr,
            timestamps=pitch_contour.timestamps,
            sample_rate=self.sample_rate,
        )

        return features

    def compute_similarity(
        self,
        features_a: AudioFeatures,
        features_b: AudioFeatures,
        frame_a: int,
        frame_b: int,
        weights: Optional[dict] = None,
    ) -> float:
        """
        Compute similarity between two frames.

        This is the core cost function for DTW alignment.

        Args:
            features_a: Features from audio A
            features_b: Features from audio B
            frame_a: Frame index in audio A
            frame_b: Frame index in audio B
            weights: Feature weights (default: balanced)

        Returns:
            Similarity score [0, 1] (higher = more similar)
        """
        if weights is None:
            weights = {
                "f0": 0.5,  # Pitch is important
                "timbre": 0.3,  # Timbre for consonants/vowels
                "energy": 0.1,  # Energy matching
                "chroma": 0.1,  # Octave-invariant backup
            }

        # Ensure weights sum to 1
        total_weight = sum(weights.values())
        weights = {k: v / total_weight for k, v in weights.items()}

        similarity = 0.0

        # 1. F0 similarity (cents-based)
        if "f0" in weights and weights["f0"] > 0:
            f0_a = features_a.f0_cents[frame_a]
            f0_b = features_b.f0_cents[frame_b]

            # Only compare if both voiced
            if (features_a.f0_confidence[frame_a] > 0.5 and
                features_b.f0_confidence[frame_b] > 0.5):

                # Convert cents error to similarity (50 cents = 50% similarity)
                error_cents = abs(f0_a - f0_b)
                f0_sim = max(0, 1 - error_cents / 100.0)  # 100 cents = semitone
                similarity += weights["f0"] * f0_sim
            else:
                # Both unvoiced = similar; one voiced = dissimilar
                both_unvoiced = (features_a.f0_confidence[frame_a] < 0.5 and
                                features_b.f0_confidence[frame_b] < 0.5)
                similarity += weights["f0"] * (1.0 if both_unvoiced else 0.0)

        # 2. Timbre similarity (mel-spectrogram cosine similarity)
        if "timbre" in weights and weights["timbre"] > 0:
            mel_a = features_a.mel_spec[:, frame_a]
            mel_b = features_b.mel_spec[:, frame_b]

            # Cosine similarity
            cos_sim = np.dot(mel_a, mel_b) / (np.linalg.norm(mel_a) * np.linalg.norm(mel_b) + 1e-8)
            cos_sim = np.clip(cos_sim, 0, 1)  # Clamp to [0, 1]

            similarity += weights["timbre"] * cos_sim

        # 3. Energy similarity
        if "energy" in weights and weights["energy"] > 0 and features_a.rms is not None:
            rms_a = features_a.rms[frame_a]
            rms_b = features_b.rms[frame_b]

            # Normalize and compare
            rms_diff = abs(rms_a - rms_b) / (max(rms_a, rms_b) + 1e-8)
            energy_sim = 1 - rms_diff

            similarity += weights["energy"] * energy_sim

        # 4. Chroma similarity (octave-invariant)
        if ("chroma" in weights and weights["chroma"] > 0 and
            features_a.chroma is not None):

            chroma_a = features_a.chroma[:, frame_a]
            chroma_b = features_b.chroma[:, frame_b]

            # Cosine similarity
            cos_sim = np.dot(chroma_a, chroma_b) / (
                np.linalg.norm(chroma_a) * np.linalg.norm(chroma_b) + 1e-8
            )
            cos_sim = np.clip(cos_sim, 0, 1)

            similarity += weights["chroma"] * cos_sim

        return float(similarity)

    def compute_cost_matrix(
        self,
        features_a: AudioFeatures,
        features_b: AudioFeatures,
        weights: Optional[dict] = None,
    ) -> np.ndarray:
        """
        Compute full cost matrix for DTW.

        Args:
            features_a: Features from audio A
            features_b: Features from audio B
            weights: Feature weights

        Returns:
            Cost matrix (n_frames_a, n_frames_b)
        """
        n_frames_a = features_a.n_frames
        n_frames_b = features_b.n_frames

        cost_matrix = np.zeros((n_frames_a, n_frames_b), dtype=np.float32)

        for i in range(n_frames_a):
            for j in range(n_frames_b):
                similarity = self.compute_similarity(
                    features_a, features_b, i, j, weights
                )
                # Convert similarity to cost (distance)
                cost_matrix[i, j] = 1 - similarity

        return cost_matrix


def extract_nasal_energy(
    audio: np.ndarray,
    sample_rate: int = 22050,
    hop_length: int = 512,
    freq_range: Tuple[float, float] = (200, 400),
) -> np.ndarray:
    """
    Extract energy in nasal frequency band for ghunna detection.

    Args:
        audio: Audio signal
        sample_rate: Sample rate
        hop_length: Hop length
        freq_range: Frequency range for nasal band (Hz)

    Returns:
        Nasal energy per frame
    """
    # Compute STFT
    stft = librosa.stft(audio, hop_length=hop_length)
    mag = np.abs(stft)

    # Frequency axis
    freqs = librosa.fft_frequencies(sr=sample_rate)

    # Find bins in nasal range
    nasal_bins = (freqs >= freq_range[0]) & (freqs <= freq_range[1])

    # Sum energy in nasal band
    nasal_energy = np.sum(mag[nasal_bins, :], axis=0)

    return nasal_energy.astype(np.float32)


def detect_silence_segments(
    audio: np.ndarray,
    sample_rate: int = 22050,
    hop_length: int = 512,
    threshold_db: float = -40.0,
    min_duration_ms: float = 200.0,
) -> list[Tuple[float, float]]:
    """
    Detect silence segments for anchor-based alignment.

    Args:
        audio: Audio signal
        sample_rate: Sample rate
        hop_length: Hop length
        threshold_db: Silence threshold (dB)
        min_duration_ms: Minimum silence duration (ms)

    Returns:
        List of (start_time, end_time) for silence segments
    """
    # Compute RMS energy
    rms = librosa.feature.rms(y=audio, hop_length=hop_length)[0]

    # Convert to dB
    rms_db = librosa.amplitude_to_db(rms, ref=np.max)

    # Detect silence frames
    silent_frames = rms_db < threshold_db

    # Convert to time
    times = librosa.frames_to_time(
        np.arange(len(rms)),
        sr=sample_rate,
        hop_length=hop_length
    )

    # Group into segments
    segments = []
    in_segment = False
    start_time = 0.0

    for i, is_silent in enumerate(silent_frames):
        if is_silent and not in_segment:
            start_time = times[i]
            in_segment = True
        elif not is_silent and in_segment:
            end_time = times[i]
            duration_ms = (end_time - start_time) * 1000

            if duration_ms >= min_duration_ms:
                segments.append((start_time, end_time))

            in_segment = False

    # Close final segment
    if in_segment:
        end_time = times[-1]
        duration_ms = (end_time - start_time) * 1000
        if duration_ms >= min_duration_ms:
            segments.append((start_time, end_time))

    return segments
