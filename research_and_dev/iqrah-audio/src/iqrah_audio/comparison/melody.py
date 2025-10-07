"""
Melody Comparison with Key-Invariant Contour Matching
======================================================

Implements key-invariant melodic comparison using ΔF0 (pitch contour).
Falls back to HPCP/chroma when F0 is unreliable.

Based on SOTA report recommendations.
"""

import numpy as np
from typing import Tuple
from .features import FeaturePack, estimate_pitch_shift
from .rhythm import soft_dtw_divergence


def melody_score(
    student: FeaturePack,
    reference: FeaturePack,
    gamma: float = 0.15
) -> dict:
    """
    Compute melody similarity score using key-invariant contour matching.

    Primary method: ΔF0 (first difference of semitones)
    Fallback: HPCP/chroma (not yet implemented)

    Args:
        student: Student feature pack
        reference: Reference (Qari) feature pack
        gamma: Soft-DTW temperature

    Returns:
        Dictionary with:
            - score: 0-100
            - pitch_shift_cents: Estimated key difference
            - contour_similarity: DTW-based similarity
            - notes: List of feedback notes
    """
    # Estimate pitch shift (key difference)
    pitch_shift = estimate_pitch_shift(student, reference)

    # Extract ΔF0 sequences (already z-normalized per phrase)
    student_df0 = student.df0
    ref_df0 = reference.df0

    # Handle NaNs (replace with 0 for comparison)
    student_df0_clean = np.nan_to_num(student_df0, nan=0.0)
    ref_df0_clean = np.nan_to_num(ref_df0, nan=0.0)

    # Reshape for DTW
    student_seq = student_df0_clean.reshape(-1, 1)
    ref_seq = ref_df0_clean.reshape(-1, 1)

    # Compute Soft-DTW divergence on contour
    divergence, path = soft_dtw_divergence(
        student_seq,
        ref_seq,
        gamma=gamma,
        bandwidth=int(0.15 * max(len(student_seq), len(ref_seq)))
    )

    # Convert to similarity score
    # Lower divergence = better contour match
    scale = 2.0  # Tunable
    contour_similarity = 100 * np.exp(-divergence / scale)
    contour_similarity = max(0, min(100, contour_similarity))

    # Compute pitch range coverage
    student_range = compute_pitch_range(student.f0_semitones)
    ref_range = compute_pitch_range(reference.f0_semitones)
    range_ratio = student_range / ref_range if ref_range > 0 else 1.0

    # Overall melody score (weight contour more than range)
    melody_score_val = 0.8 * contour_similarity + 0.2 * min(100, range_ratio * 100)

    # Generate feedback notes
    notes = []

    # Pitch shift feedback
    semitone_shift = abs(pitch_shift) / 100.0
    if semitone_shift < 1:
        notes.append(f"Reciting in similar key (shift: {pitch_shift:+.0f} cents)")
    elif semitone_shift < 3:
        notes.append(f"Reciting {semitone_shift:.1f} semitones {'higher' if pitch_shift > 0 else 'lower'}")
    else:
        notes.append(f"Significant key difference: {semitone_shift:.1f} semitones {'higher' if pitch_shift > 0 else 'lower'}")

    # Contour feedback
    if contour_similarity >= 90:
        notes.append("Excellent melodic contour - very close to reference")
    elif contour_similarity >= 75:
        notes.append("Good melodic contour with minor deviations")
    elif contour_similarity >= 60:
        notes.append("Melodic contour needs improvement")
    else:
        notes.append("Melodic contour significantly differs from reference")

    # Range feedback
    if range_ratio < 0.7:
        notes.append(f"Pitch range too narrow ({student_range:.1f} vs {ref_range:.1f} semitones)")
    elif range_ratio > 1.3:
        notes.append(f"Pitch range too wide ({student_range:.1f} vs {ref_range:.1f} semitones)")

    return {
        'score': round(melody_score_val, 1),
        'pitch_shift_cents': round(pitch_shift, 1),
        'contour_similarity': round(contour_similarity, 1),
        'student_range_semitones': round(student_range, 1),
        'reference_range_semitones': round(ref_range, 1),
        'range_ratio': round(range_ratio, 2),
        'notes': notes
    }


def compute_pitch_range(f0_semitones: np.ndarray) -> float:
    """
    Compute pitch range in semitones (90th percentile - 10th percentile).

    Uses percentiles to be robust to outliers.

    Args:
        f0_semitones: F0 in semitones (may contain NaNs)

    Returns:
        Range in semitones
    """
    # Filter out NaNs and zeros
    voiced = f0_semitones[~np.isnan(f0_semitones)]

    if len(voiced) < 10:
        return 0.0

    # Use 10th to 90th percentile for robustness
    p10 = np.percentile(voiced, 10)
    p90 = np.percentile(voiced, 90)

    return float(p90 - p10)


def compute_melodic_correlation(
    student: FeaturePack,
    reference: FeaturePack
) -> float:
    """
    Compute Pearson correlation of normalized pitch trajectories.

    This is an additional metric that complements DTW divergence.

    Returns:
        Correlation coefficient (-1 to 1)
    """
    # Get voiced segments
    student_voiced = student.f0_semitones[~np.isnan(student.f0_semitones)]
    ref_voiced = reference.f0_semitones[~np.isnan(reference.f0_semitones)]

    if len(student_voiced) < 10 or len(ref_voiced) < 10:
        return 0.0

    # Resample to same length
    min_len = min(len(student_voiced), len(ref_voiced))
    student_resampled = np.interp(
        np.linspace(0, 1, min_len),
        np.linspace(0, 1, len(student_voiced)),
        student_voiced
    )
    ref_resampled = np.interp(
        np.linspace(0, 1, min_len),
        np.linspace(0, 1, len(ref_voiced)),
        ref_voiced
    )

    # Z-normalize both
    student_znorm = (student_resampled - np.mean(student_resampled)) / (np.std(student_resampled) + 1e-8)
    ref_znorm = (ref_resampled - np.mean(ref_resampled)) / (np.std(ref_resampled) + 1e-8)

    # Compute correlation
    correlation = np.corrcoef(student_znorm, ref_znorm)[0, 1]

    return float(correlation)
