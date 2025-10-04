"""
Recitation Scoring Module
=========================

Score user recitation against reference qari.
Multiple metrics: on-note %, pitch stability, tempo, etc.
"""

import numpy as np
from dataclasses import dataclass
from typing import Optional, Tuple

from .pitch import PitchContour
from .dtw import DTWAligner, AlignmentResult


@dataclass
class RecitationScore:
    """
    Complete recitation scoring result.

    Attributes:
        overall_score: Overall score [0, 100]
        alignment_score: DTW alignment score [0, 100]
        on_note_percent: Percentage of frames within pitch threshold
        pitch_stability: Pitch stability score [0, 100]
        tempo_score: Tempo matching score [0, 100]
        voiced_ratio: Ratio of voiced frames
        metrics: Detailed metrics dict
    """
    overall_score: float
    alignment_score: float
    on_note_percent: float
    pitch_stability: float
    tempo_score: float
    voiced_ratio: float
    metrics: dict

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            "overall_score": self.overall_score,
            "alignment_score": self.alignment_score,
            "on_note_percent": self.on_note_percent,
            "pitch_stability": self.pitch_stability,
            "tempo_score": self.tempo_score,
            "voiced_ratio": self.voiced_ratio,
            "metrics": self.metrics,
        }


class RecitationScorer:
    """
    Score recitation quality against reference.

    Implements multiple scoring metrics used in the design spec.
    """

    def __init__(
        self,
        on_note_threshold_cents: float = 50.0,  # ±50 cents = within pitch
        stability_window: int = 5,  # Frames for stability calculation
    ):
        """
        Initialize scorer.

        Args:
            on_note_threshold_cents: Pitch threshold for "on-note" (cents)
            stability_window: Window size for pitch stability
        """
        self.on_note_threshold_cents = on_note_threshold_cents
        self.stability_window = stability_window
        self.aligner = DTWAligner(window=None)  # No window constraint for offline

    def score(
        self,
        user_contour: PitchContour,
        reference_contour: PitchContour,
        alignment: Optional[AlignmentResult] = None
    ) -> RecitationScore:
        """
        Score user recitation.

        Args:
            user_contour: User pitch contour
            reference_contour: Reference pitch contour
            alignment: Pre-computed alignment (will compute if None)

        Returns:
            RecitationScore with all metrics
        """
        # Align if not provided
        if alignment is None:
            alignment = self.aligner.align(
                user_contour.f0_cents,
                reference_contour.f0_cents
            )

        # 1. Alignment score (DTW similarity)
        alignment_score = alignment.alignment_score * 100

        # 2. On-note percentage
        on_note_pct = self._calculate_on_note_percent(
            user_contour,
            reference_contour,
            alignment
        )

        # 3. Pitch stability
        pitch_stability = self._calculate_pitch_stability(user_contour)

        # 4. Tempo score
        tempo_score = self._calculate_tempo_score(
            user_contour,
            reference_contour,
            alignment
        )

        # 5. Voiced ratio
        voiced_ratio = self._calculate_voiced_ratio(user_contour)

        # Overall score (weighted average)
        overall_score = (
            0.4 * alignment_score +
            0.3 * on_note_pct +
            0.2 * pitch_stability +
            0.1 * tempo_score
        )

        # Detailed metrics
        metrics = {
            "dtw_distance": alignment.distance,
            "dtw_normalized": alignment.normalized_distance,
            "mean_pitch_error_cents": self._mean_pitch_error(
                user_contour, reference_contour, alignment
            ),
            "user_duration": user_contour.duration,
            "reference_duration": reference_contour.duration,
            "tempo_ratio": user_contour.duration / reference_contour.duration
                          if reference_contour.duration > 0 else 1.0,
        }

        return RecitationScore(
            overall_score=float(overall_score),
            alignment_score=float(alignment_score),
            on_note_percent=float(on_note_pct),
            pitch_stability=float(pitch_stability),
            tempo_score=float(tempo_score),
            voiced_ratio=float(voiced_ratio),
            metrics=metrics,
        )

    def _calculate_on_note_percent(
        self,
        user: PitchContour,
        reference: PitchContour,
        alignment: AlignmentResult
    ) -> float:
        """
        Calculate percentage of frames within pitch threshold.

        Args:
            user: User contour
            reference: Reference contour
            alignment: Alignment result

        Returns:
            On-note percentage [0, 100]
        """
        if not alignment.path:
            return 0.0

        errors = []

        for user_idx, ref_idx in alignment.path:
            # Skip unvoiced frames
            if user.confidence[user_idx] < 0.5 or reference.confidence[ref_idx] < 0.5:
                continue

            user_cents = user.f0_cents[user_idx]
            ref_cents = reference.f0_cents[ref_idx]

            # Pitch error in cents
            error = abs(user_cents - ref_cents)
            errors.append(error)

        if not errors:
            return 0.0

        # Percentage within threshold
        on_note_count = sum(1 for e in errors if e <= self.on_note_threshold_cents)
        on_note_pct = (on_note_count / len(errors)) * 100

        return on_note_pct

    def _calculate_pitch_stability(self, contour: PitchContour) -> float:
        """
        Calculate pitch stability score.

        Measures how stable the pitch is within voiced regions.
        Lower jitter = higher score.

        Args:
            contour: Pitch contour

        Returns:
            Stability score [0, 100]
        """
        voiced_segments = contour.get_voiced_segments(confidence_threshold=0.5)

        if not voiced_segments:
            return 0.0

        stabilities = []

        for start_time, end_time in voiced_segments:
            # Get frames in this segment
            mask = (contour.timestamps >= start_time) & (contour.timestamps <= end_time)
            segment_f0 = contour.f0_cents[mask]

            if len(segment_f0) < self.stability_window:
                continue

            # Calculate local variance (rolling window)
            variances = []
            for i in range(len(segment_f0) - self.stability_window):
                window = segment_f0[i:i + self.stability_window]
                variances.append(np.std(window))

            if variances:
                avg_std = np.mean(variances)
                # Convert std in cents to stability score
                # Low std (~0-10 cents) = high score
                # High std (>50 cents) = low score
                stability = max(0, 100 - 2 * avg_std)
                stabilities.append(stability)

        if not stabilities:
            return 0.0

        return np.mean(stabilities)

    def _calculate_tempo_score(
        self,
        user: PitchContour,
        reference: PitchContour,
        alignment: AlignmentResult
    ) -> float:
        """
        Calculate tempo matching score.

        Measures consistency of tempo ratio.
        Constant tempo ratio = high score.

        Args:
            user: User contour
            reference: Reference contour
            alignment: Alignment result

        Returns:
            Tempo score [0, 100]
        """
        tempo_ratio = user.duration / reference.duration if reference.duration > 0 else 1.0

        # Score based on how close to 1.0 (same tempo)
        # Allow ±20% variation
        if 0.8 <= tempo_ratio <= 1.2:
            # Linear score: perfect at 1.0, decreases to 80 at edges
            score = 100 - 100 * abs(tempo_ratio - 1.0)
        else:
            # Penalty for outside acceptable range
            score = max(0, 80 - 50 * abs(tempo_ratio - 1.0))

        return score

    def _calculate_voiced_ratio(self, contour: PitchContour) -> float:
        """
        Calculate ratio of voiced frames.

        Args:
            contour: Pitch contour

        Returns:
            Voiced ratio [0, 1]
        """
        voiced = contour.confidence >= 0.5
        return np.mean(voiced)

    def _mean_pitch_error(
        self,
        user: PitchContour,
        reference: PitchContour,
        alignment: AlignmentResult
    ) -> float:
        """
        Calculate mean pitch error in cents.

        Args:
            user: User contour
            reference: Reference contour
            alignment: Alignment result

        Returns:
            Mean absolute error in cents
        """
        if not alignment.path:
            return float('inf')

        errors = []

        for user_idx, ref_idx in alignment.path:
            # Skip unvoiced
            if user.confidence[user_idx] < 0.5 or reference.confidence[ref_idx] < 0.5:
                continue

            error = abs(user.f0_cents[user_idx] - reference.f0_cents[ref_idx])
            errors.append(error)

        return np.mean(errors) if errors else float('inf')
