"""
Anchor Detection for Real-Time Alignment
=========================================

Detect alignment anchors for drift correction in online DTW.

Anchor types:
1. Silence - Pauses between words/ayat (RMS-based)
2. Plosives - Burst sounds from qalqalah letters (spectral-based)
3. Long notes - Sustained vowels like madd (F0 stability)
"""

import numpy as np
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass
from scipy.signal import find_peaks


@dataclass
class Anchor:
    """
    Alignment anchor point.

    Anchors are high-confidence points where we can correct alignment drift.
    """
    timestamp: float  # Time in seconds
    frame_idx: int    # Frame index
    anchor_type: str  # "silence", "plosive", "long_note"
    confidence: float # Confidence [0, 1]
    duration: float   # Duration of anchor in seconds (for long notes/silence)

    def __repr__(self) -> str:
        return (
            f"Anchor(t={self.timestamp:.2f}s, type={self.anchor_type}, "
            f"conf={self.confidence:.2f}, dur={self.duration:.2f}s)"
        )


class AnchorDetector:
    """
    Detect alignment anchors in real-time audio.

    Uses multiple detection methods for robustness:
    - Silence detection (RMS energy)
    - Plosive detection (spectral features)
    - Long note detection (F0 stability)
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        hop_length: int = 512,
        # Silence detection params
        silence_threshold_db: float = -40.0,
        silence_min_duration_s: float = 0.2,
        # Plosive detection params
        plosive_flatness_threshold: float = 0.6,
        plosive_min_energy_db: float = -30.0,
        # Long note detection params
        long_note_min_duration_s: float = 0.5,
        long_note_stability_cents: float = 30.0,
    ):
        """
        Initialize anchor detector.

        Args:
            sample_rate: Audio sample rate
            hop_length: Hop length for frame analysis
            silence_threshold_db: RMS threshold for silence (dB)
            silence_min_duration_s: Minimum silence duration
            plosive_flatness_threshold: Spectral flatness for plosives
            plosive_min_energy_db: Minimum energy for plosive detection
            long_note_min_duration_s: Minimum duration for long note
            long_note_stability_cents: Max pitch variation for stability
        """
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.frame_duration = hop_length / sample_rate

        # Silence params
        self.silence_threshold_db = silence_threshold_db
        self.silence_min_frames = int(silence_min_duration_s / self.frame_duration)

        # Plosive params
        self.plosive_flatness_threshold = plosive_flatness_threshold
        self.plosive_min_energy_db = plosive_min_energy_db

        # Long note params
        self.long_note_min_frames = int(long_note_min_duration_s / self.frame_duration)
        self.long_note_stability_cents = long_note_stability_cents

    def detect_silence(
        self,
        rms: np.ndarray,
        timestamps: Optional[np.ndarray] = None,
    ) -> List[Anchor]:
        """
        Detect silence regions.

        Args:
            rms: RMS energy per frame
            timestamps: Frame timestamps (optional)

        Returns:
            List of silence anchors
        """
        if timestamps is None:
            timestamps = np.arange(len(rms)) * self.frame_duration

        # Convert RMS to dB
        rms_db = 20 * np.log10(rms + 1e-8)

        # Find frames below threshold
        silence_mask = rms_db < self.silence_threshold_db

        # Find contiguous silence regions
        anchors = []
        in_silence = False
        silence_start = 0

        for i in range(len(silence_mask)):
            if silence_mask[i] and not in_silence:
                # Start of silence
                in_silence = True
                silence_start = i

            elif not silence_mask[i] and in_silence:
                # End of silence
                in_silence = False
                silence_duration_frames = i - silence_start

                if silence_duration_frames >= self.silence_min_frames:
                    # Valid silence anchor
                    anchor_frame = (silence_start + i) // 2  # Middle of silence
                    duration_s = silence_duration_frames * self.frame_duration

                    anchors.append(Anchor(
                        timestamp=timestamps[anchor_frame],
                        frame_idx=anchor_frame,
                        anchor_type="silence",
                        confidence=min(1.0, silence_duration_frames / (self.silence_min_frames * 2)),
                        duration=duration_s,
                    ))

        # Handle silence at end
        if in_silence:
            silence_duration_frames = len(silence_mask) - silence_start
            if silence_duration_frames >= self.silence_min_frames:
                anchor_frame = (silence_start + len(silence_mask)) // 2
                duration_s = silence_duration_frames * self.frame_duration

                anchors.append(Anchor(
                    timestamp=timestamps[anchor_frame],
                    frame_idx=anchor_frame,
                    anchor_type="silence",
                    confidence=min(1.0, silence_duration_frames / (self.silence_min_frames * 2)),
                    duration=duration_s,
                ))

        return anchors

    def detect_plosives(
        self,
        spectral_flatness: np.ndarray,
        rms: np.ndarray,
        timestamps: Optional[np.ndarray] = None,
    ) -> List[Anchor]:
        """
        Detect plosive bursts (qalqalah letters: ق ط ب ج د).

        Plosives have high spectral flatness (noise-like) and sudden energy.

        Args:
            spectral_flatness: Spectral flatness per frame [0, 1]
            rms: RMS energy per frame
            timestamps: Frame timestamps (optional)

        Returns:
            List of plosive anchors
        """
        if timestamps is None:
            timestamps = np.arange(len(spectral_flatness)) * self.frame_duration

        # Convert RMS to dB
        rms_db = 20 * np.log10(rms + 1e-8)

        # Find frames with high spectral flatness and sufficient energy
        plosive_mask = (
            (spectral_flatness > self.plosive_flatness_threshold) &
            (rms_db > self.plosive_min_energy_db)
        )

        # Find peaks in spectral flatness
        if np.any(plosive_mask):
            # Only search for peaks where mask is True
            masked_flatness = spectral_flatness.copy()
            masked_flatness[~plosive_mask] = 0

            # Find peaks
            peaks, properties = find_peaks(
                masked_flatness,
                height=self.plosive_flatness_threshold,
                distance=5,  # At least 5 frames apart (~115ms)
            )

            anchors = []
            for peak_idx in peaks:
                # Confidence based on how high above threshold
                flatness_above_threshold = spectral_flatness[peak_idx] - self.plosive_flatness_threshold
                confidence = min(1.0, flatness_above_threshold / 0.2)  # 0.2 range to reach max conf

                anchors.append(Anchor(
                    timestamp=timestamps[peak_idx],
                    frame_idx=int(peak_idx),
                    anchor_type="plosive",
                    confidence=confidence,
                    duration=self.frame_duration,  # Single frame
                ))

            return anchors

        return []

    def detect_long_notes(
        self,
        f0_hz: np.ndarray,
        confidence: np.ndarray,
        timestamps: Optional[np.ndarray] = None,
    ) -> List[Anchor]:
        """
        Detect long sustained notes (madd, sustained vowels).

        Long notes have stable F0 for extended duration.

        Args:
            f0_hz: Pitch in Hz per frame
            confidence: Voicing confidence per frame
            timestamps: Frame timestamps (optional)

        Returns:
            List of long note anchors
        """
        if timestamps is None:
            timestamps = np.arange(len(f0_hz)) * self.frame_duration

        # Only consider voiced frames
        voiced_mask = confidence > 0.5

        # Calculate F0 stability (variation in cents)
        anchors = []
        in_note = False
        note_start = 0

        for i in range(len(f0_hz)):
            if voiced_mask[i] and f0_hz[i] > 0:
                if not in_note:
                    # Start of potential long note
                    in_note = True
                    note_start = i

            else:
                if in_note:
                    # End of note
                    in_note = False
                    note_duration_frames = i - note_start

                    if note_duration_frames >= self.long_note_min_frames:
                        # Check stability
                        note_f0 = f0_hz[note_start:i]
                        note_f0_voiced = note_f0[note_f0 > 0]

                        if len(note_f0_voiced) > 0:
                            median_f0 = np.median(note_f0_voiced)
                            f0_cents = 1200 * np.log2(note_f0_voiced / median_f0)
                            stability_cents = np.std(f0_cents)

                            if stability_cents < self.long_note_stability_cents:
                                # Stable long note
                                anchor_frame = (note_start + i) // 2  # Middle of note
                                duration_s = note_duration_frames * self.frame_duration

                                # Higher confidence for longer and more stable notes
                                duration_conf = min(1.0, note_duration_frames / (self.long_note_min_frames * 2))
                                stability_conf = max(0.5, 1.0 - stability_cents / self.long_note_stability_cents)
                                confidence = (duration_conf + stability_conf) / 2

                                anchors.append(Anchor(
                                    timestamp=timestamps[anchor_frame],
                                    frame_idx=anchor_frame,
                                    anchor_type="long_note",
                                    confidence=confidence,
                                    duration=duration_s,
                                ))

        # Handle note at end
        if in_note:
            note_duration_frames = len(f0_hz) - note_start
            if note_duration_frames >= self.long_note_min_frames:
                note_f0 = f0_hz[note_start:]
                note_f0_voiced = note_f0[note_f0 > 0]

                if len(note_f0_voiced) > 0:
                    median_f0 = np.median(note_f0_voiced)
                    f0_cents = 1200 * np.log2(note_f0_voiced / median_f0)
                    stability_cents = np.std(f0_cents)

                    if stability_cents < self.long_note_stability_cents:
                        anchor_frame = (note_start + len(f0_hz)) // 2
                        duration_s = note_duration_frames * self.frame_duration

                        duration_conf = min(1.0, note_duration_frames / (self.long_note_min_frames * 2))
                        stability_conf = max(0.5, 1.0 - stability_cents / self.long_note_stability_cents)
                        confidence = (duration_conf + stability_conf) / 2

                        anchors.append(Anchor(
                            timestamp=timestamps[anchor_frame],
                            frame_idx=anchor_frame,
                            anchor_type="long_note",
                            confidence=confidence,
                            duration=duration_s,
                        ))

        return anchors

    def detect_all(
        self,
        f0_hz: np.ndarray,
        confidence: np.ndarray,
        rms: np.ndarray,
        spectral_flatness: np.ndarray,
        timestamps: Optional[np.ndarray] = None,
    ) -> List[Anchor]:
        """
        Detect all anchor types.

        Args:
            f0_hz: Pitch in Hz per frame
            confidence: Voicing confidence per frame
            rms: RMS energy per frame
            spectral_flatness: Spectral flatness per frame
            timestamps: Frame timestamps (optional)

        Returns:
            Combined list of all anchors, sorted by timestamp
        """
        if timestamps is None:
            n_frames = max(len(f0_hz), len(rms), len(spectral_flatness))
            timestamps = np.arange(n_frames) * self.frame_duration

        anchors = []

        # Detect each type
        anchors.extend(self.detect_silence(rms, timestamps))
        anchors.extend(self.detect_plosives(spectral_flatness, rms, timestamps))
        anchors.extend(self.detect_long_notes(f0_hz, confidence, timestamps))

        # Sort by timestamp
        anchors.sort(key=lambda a: a.timestamp)

        return anchors

    def filter_anchors(
        self,
        anchors: List[Anchor],
        min_confidence: float = 0.5,
        max_anchors: Optional[int] = None,
    ) -> List[Anchor]:
        """
        Filter anchors by confidence and limit count.

        Args:
            anchors: List of anchors
            min_confidence: Minimum confidence threshold
            max_anchors: Maximum number of anchors to keep (highest confidence)

        Returns:
            Filtered list of anchors
        """
        # Filter by confidence
        filtered = [a for a in anchors if a.confidence >= min_confidence]

        # Limit count (keep highest confidence)
        if max_anchors is not None and len(filtered) > max_anchors:
            filtered.sort(key=lambda a: a.confidence, reverse=True)
            filtered = filtered[:max_anchors]
            # Re-sort by timestamp
            filtered.sort(key=lambda a: a.timestamp)

        return filtered
