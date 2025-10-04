"""
Octave Error Correction
========================

Correct octave errors in pitch tracking using multiple strategies.
"""

import numpy as np
from typing import Optional, Tuple
from scipy.signal import medfilt


def correct_octave_errors_simple(
    f0_hz: np.ndarray,
    confidence: np.ndarray,
    confidence_threshold: float = 0.5,
    median_filter_size: int = 11,  # Not used - kept for API compatibility
) -> np.ndarray:
    """
    Simple octave error correction using median-based strategy.

    Computes a global median pitch and snaps each frame to the nearest octave
    relative to this global median. This corrects octave jumps effectively.

    Args:
        f0_hz: Pitch in Hz
        confidence: Voicing confidence
        confidence_threshold: Threshold for voiced frames
        median_filter_size: Deprecated (kept for API compatibility)

    Returns:
        Corrected F0
    """
    f0_corrected = f0_hz.copy()
    voiced_mask = confidence >= confidence_threshold

    if np.sum(voiced_mask) == 0:
        return f0_corrected

    voiced_f0 = f0_hz[voiced_mask]

    # Compute global median as reference
    # This is more robust than local median for octave jump detection
    global_median = np.median(voiced_f0[voiced_f0 > 0])

    if global_median == 0:
        return f0_corrected

    # Snap each frame to nearest octave relative to global median
    corrected_voiced = voiced_f0.copy()
    for i in range(len(voiced_f0)):
        if voiced_f0[i] == 0:
            continue

        # Find nearest octave shift
        best_f0 = voiced_f0[i]
        min_error = float('inf')

        for octave_shift in range(-2, 3):  # Try ±2 octaves
            shifted_f0 = voiced_f0[i] * (2 ** octave_shift)
            error = abs(np.log2(shifted_f0 / global_median))

            if error < min_error:
                min_error = error
                best_f0 = shifted_f0

        corrected_voiced[i] = best_f0

    f0_corrected[voiced_mask] = corrected_voiced
    return f0_corrected


def snap_to_nearest_octave(
    f0_hz: np.ndarray,
    reference_f0_hz: np.ndarray,
    max_octave_shift: int = 2,
) -> np.ndarray:
    """
    Snap pitch to nearest octave relative to reference.

    Corrects octave errors by finding the octave shift that minimizes
    the distance to the reference pitch.

    Args:
        f0_hz: Query pitch in Hz
        reference_f0_hz: Reference pitch in Hz (aligned)
        max_octave_shift: Maximum octaves to shift

    Returns:
        Corrected F0
    """
    f0_corrected = f0_hz.copy()

    for i in range(len(f0_hz)):
        if f0_hz[i] == 0 or reference_f0_hz[i] == 0:
            continue

        # Try different octave shifts
        best_error = float('inf')
        best_f0 = f0_hz[i]

        for octave_shift in range(-max_octave_shift, max_octave_shift + 1):
            shifted_f0 = f0_hz[i] * (2 ** octave_shift)

            # Calculate error in cents
            error = abs(1200 * np.log2(shifted_f0 / reference_f0_hz[i]))

            if error < best_error:
                best_error = error
                best_f0 = shifted_f0

        f0_corrected[i] = best_f0

    return f0_corrected


def detect_octave_errors(
    f0_cents: np.ndarray,
    reference_cents: np.ndarray,
    threshold_cents: float = 600.0,  # Half octave
) -> np.ndarray:
    """
    Detect frames with likely octave errors.

    Args:
        f0_cents: Query pitch in cents
        reference_cents: Reference pitch in cents
        threshold_cents: Error threshold (default: half octave)

    Returns:
        Boolean array: True where octave error likely
    """
    error_cents = np.abs(f0_cents - reference_cents)
    octave_errors = error_cents > threshold_cents

    return octave_errors


def octave_aware_pitch_distance(
    f0_a_hz: float,
    f0_b_hz: float,
    max_octaves: int = 2,
) -> float:
    """
    Calculate octave-aware pitch distance.

    Finds the minimum distance considering octave shifts.

    Args:
        f0_a_hz: First pitch (Hz)
        f0_b_hz: Second pitch (Hz)
        max_octaves: Maximum octaves to consider

    Returns:
        Minimum distance in cents
    """
    if f0_a_hz == 0 or f0_b_hz == 0:
        return 1200.0  # Large penalty for unvoiced

    min_distance = float('inf')

    for octave_shift in range(-max_octaves, max_octaves + 1):
        shifted_f0_a = f0_a_hz * (2 ** octave_shift)

        # Distance in cents
        distance = abs(1200 * np.log2(shifted_f0_a / f0_b_hz))

        min_distance = min(min_distance, distance)

    return min_distance


def correct_using_chroma(
    f0_hz: np.ndarray,
    chroma: np.ndarray,
    confidence: np.ndarray,
    base_octave: int = 4,  # A4 = 440 Hz
) -> np.ndarray:
    """
    Correct pitch using chroma features (octave-invariant).

    Uses chroma to determine pitch class, then selects appropriate octave.

    Args:
        f0_hz: Pitch in Hz
        chroma: Chroma features (12, n_frames)
        confidence: Voicing confidence
        base_octave: Base octave for correction

    Returns:
        Corrected F0
    """
    f0_corrected = f0_hz.copy()
    n_frames = len(f0_hz)

    # Reference frequencies for each pitch class (octave 4)
    pitch_class_freqs = np.array([
        # C4,  C#4, D4,  D#4, E4,  F4,  F#4, G4,  G#4, A4,  A#4, B4
        261.63, 277.18, 293.66, 311.13, 329.63, 349.23,
        369.99, 392.00, 415.30, 440.00, 466.16, 493.88
    ])

    for i in range(n_frames):
        if confidence[i] < 0.5:
            continue

        # Get dominant chroma bin
        chroma_frame = chroma[:, i]
        pitch_class = np.argmax(chroma_frame)

        # Get reference frequency for this pitch class
        ref_freq = pitch_class_freqs[pitch_class]

        # Find closest octave
        current_f0 = f0_hz[i]

        # Try octaves ±2
        best_octave = 0
        min_error = float('inf')

        for octave_shift in range(-2, 3):
            candidate_f0 = ref_freq * (2 ** octave_shift)
            error = abs(np.log2(current_f0 / candidate_f0))

            if error < min_error:
                min_error = error
                best_octave = octave_shift

        f0_corrected[i] = ref_freq * (2 ** best_octave)

    return f0_corrected


class OctaveCorrector:
    """
    Comprehensive octave error correction.

    Combines multiple strategies for robust correction.
    """

    def __init__(
        self,
        strategy: str = "median",  # "median", "snap", "chroma", "hybrid"
        max_octave_shift: int = 2,
        confidence_threshold: float = 0.5,
    ):
        """
        Initialize octave corrector.

        Args:
            strategy: Correction strategy
            max_octave_shift: Maximum octaves to shift
            confidence_threshold: Voicing threshold
        """
        self.strategy = strategy
        self.max_octave_shift = max_octave_shift
        self.confidence_threshold = confidence_threshold

    def correct(
        self,
        f0_hz: np.ndarray,
        confidence: np.ndarray,
        reference_f0_hz: Optional[np.ndarray] = None,
        chroma: Optional[np.ndarray] = None,
    ) -> np.ndarray:
        """
        Correct octave errors.

        Args:
            f0_hz: Pitch in Hz
            confidence: Voicing confidence
            reference_f0_hz: Reference pitch (for "snap" strategy)
            chroma: Chroma features (for "chroma" strategy)

        Returns:
            Corrected F0
        """
        if self.strategy == "median":
            return correct_octave_errors_simple(
                f0_hz,
                confidence,
                self.confidence_threshold
            )

        elif self.strategy == "snap" and reference_f0_hz is not None:
            return snap_to_nearest_octave(
                f0_hz,
                reference_f0_hz,
                self.max_octave_shift
            )

        elif self.strategy == "chroma" and chroma is not None:
            return correct_using_chroma(
                f0_hz,
                chroma,
                confidence
            )

        elif self.strategy == "hybrid":
            # First apply median filtering
            f0_median = correct_octave_errors_simple(
                f0_hz,
                confidence,
                self.confidence_threshold
            )

            # Then snap to reference if available
            if reference_f0_hz is not None:
                f0_snapped = snap_to_nearest_octave(
                    f0_median,
                    reference_f0_hz,
                    self.max_octave_shift
                )
                return f0_snapped

            return f0_median

        else:
            return f0_hz  # No correction


def calculate_octave_confidence(
    f0_hz: np.ndarray,
    confidence: np.ndarray,
    chroma: Optional[np.ndarray] = None,
) -> np.ndarray:
    """
    Calculate confidence that pitch is in correct octave.

    Args:
        f0_hz: Pitch in Hz
        confidence: Voicing confidence
        chroma: Chroma features (optional)

    Returns:
        Octave confidence per frame [0, 1]
    """
    octave_confidence = np.ones(len(f0_hz))

    # Low confidence if voicing is low
    octave_confidence *= confidence

    # Check for consistency with chroma if available
    if chroma is not None:
        # Calculate pitch class from F0
        with np.errstate(divide='ignore', invalid='ignore'):
            pitch_class_from_f0 = (12 * np.log2(f0_hz / 440.0)) % 12

        # Compare with dominant chroma bin
        for i in range(len(f0_hz)):
            if confidence[i] > 0.5:
                dominant_chroma = np.argmax(chroma[:, i])

                # Distance to nearest chroma bin
                pc_distance = min(
                    abs(pitch_class_from_f0[i] - dominant_chroma),
                    12 - abs(pitch_class_from_f0[i] - dominant_chroma)
                )

                # Confidence decreases with distance
                chroma_conf = max(0, 1 - pc_distance / 6.0)
                octave_confidence[i] *= chroma_conf

    return octave_confidence
