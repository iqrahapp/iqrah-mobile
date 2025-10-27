"""
Enhanced Recitation Scoring
============================

Confidence-weighted scoring with advanced metrics.
"""

import numpy as np
from dataclasses import dataclass
from typing import Optional

from .pitch import PitchContour
from .dtw import DTWAligner, AlignmentResult
from .features import AudioFeatures
from .octave import detect_octave_errors


@dataclass
class EnhancedRecitationScore:
    """
    Enhanced recitation scoring result.

    Adds confidence weighting and detailed sub-metrics.
    """
    # Overall scores
    overall_score: float
    alignment_score: float
    on_note_percent: float
    pitch_stability: float
    tempo_score: float
    voiced_ratio: float

    # New: Confidence-weighted scores
    weighted_on_note_percent: float
    weighted_pitch_accuracy: float

    # New: Error analysis
    octave_error_rate: float
    gross_error_rate: float  # Errors >100 cents
    median_error_cents: float
    p95_error_cents: float  # 95th percentile

    # New: Timing metrics
    pause_accuracy: float  # How well pauses match
    timing_consistency: float  # Tempo variation

    # Features
    timbre_similarity: Optional[float] = None
    energy_correlation: Optional[float] = None

    # Detailed breakdown
    metrics: dict = None

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            "overall_score": self.overall_score,
            "alignment_score": self.alignment_score,
            "on_note_percent": self.on_note_percent,
            "pitch_stability": self.pitch_stability,
            "tempo_score": self.tempo_score,
            "voiced_ratio": self.voiced_ratio,
            "weighted_on_note_percent": self.weighted_on_note_percent,
            "weighted_pitch_accuracy": self.weighted_pitch_accuracy,
            "octave_error_rate": self.octave_error_rate,
            "gross_error_rate": self.gross_error_rate,
            "median_error_cents": self.median_error_cents,
            "p95_error_cents": self.p95_error_cents,
            "pause_accuracy": self.pause_accuracy,
            "timing_consistency": self.timing_consistency,
            "timbre_similarity": self.timbre_similarity,
            "energy_correlation": self.energy_correlation,
            "metrics": self.metrics or {},
        }


class EnhancedRecitationScorer:
    """
    Enhanced scorer with confidence weighting and advanced metrics.
    """

    def __init__(
        self,
        on_note_threshold_cents: float = 50.0,
        gross_error_threshold_cents: float = 100.0,
        stability_window: int = 5,
        confidence_weight_power: float = 2.0,  # Emphasis on high confidence
    ):
        """
        Initialize enhanced scorer.

        Args:
            on_note_threshold_cents: Threshold for "on-note" (cents)
            gross_error_threshold_cents: Threshold for gross errors
            stability_window: Window for stability calculation
            confidence_weight_power: Power for confidence weighting (higher = more emphasis)
        """
        self.on_note_threshold_cents = on_note_threshold_cents
        self.gross_error_threshold_cents = gross_error_threshold_cents
        self.stability_window = stability_window
        self.confidence_weight_power = confidence_weight_power

        self.aligner = DTWAligner(window=None)

    def score(
        self,
        user_contour: PitchContour,
        reference_contour: PitchContour,
        user_features: Optional[AudioFeatures] = None,
        ref_features: Optional[AudioFeatures] = None,
        alignment: Optional[AlignmentResult] = None,
    ) -> EnhancedRecitationScore:
        """
        Score user recitation with enhanced metrics.

        Args:
            user_contour: User pitch contour
            reference_contour: Reference pitch contour
            user_features: User audio features (optional)
            ref_features: Reference audio features (optional)
            alignment: Pre-computed alignment (optional)

        Returns:
            EnhancedRecitationScore
        """
        # Align if not provided
        if alignment is None:
            alignment = self.aligner.align(
                user_contour.f0_cents,
                reference_contour.f0_cents,
            )

        # 1. Basic metrics (from original scorer)
        alignment_score = alignment.alignment_score * 100

        on_note_pct = self._calculate_on_note_percent(
            user_contour, reference_contour, alignment
        )

        pitch_stability = self._calculate_pitch_stability(user_contour)

        tempo_score = self._calculate_tempo_score(
            user_contour, reference_contour, alignment
        )

        voiced_ratio = np.mean(user_contour.confidence >= 0.5)

        # 2. Confidence-weighted metrics (NEW)
        weighted_on_note = self._calculate_weighted_on_note(
            user_contour, reference_contour, alignment
        )

        weighted_accuracy = self._calculate_weighted_pitch_accuracy(
            user_contour, reference_contour, alignment
        )

        # 3. Error analysis (NEW)
        octave_errors = detect_octave_errors(
            user_contour.f0_cents,
            reference_contour.f0_cents,
        )
        octave_error_rate = float(np.mean(octave_errors))

        errors_cents = self._get_pitch_errors(
            user_contour, reference_contour, alignment
        )

        if len(errors_cents) > 0:
            gross_error_rate = float(
                np.mean(errors_cents > self.gross_error_threshold_cents)
            )
            median_error = float(np.median(errors_cents))
            p95_error = float(np.percentile(errors_cents, 95))
        else:
            gross_error_rate = 1.0
            median_error = float('inf')
            p95_error = float('inf')

        # 4. Timing metrics (NEW)
        pause_accuracy = self._calculate_pause_accuracy(
            user_contour, reference_contour
        )

        timing_consistency = self._calculate_timing_consistency(alignment)

        # 5. Feature similarity (NEW, if features provided)
        timbre_sim = None
        energy_corr = None

        if user_features is not None and ref_features is not None:
            timbre_sim = self._calculate_timbre_similarity(
                user_features, ref_features, alignment
            )
            energy_corr = self._calculate_energy_correlation(
                user_features, ref_features, alignment
            )

        # 6. Calculate overall score (weighted combination)
        overall_score = self._calculate_overall_score(
            alignment_score=alignment_score,
            weighted_on_note=weighted_on_note,
            pitch_stability=pitch_stability,
            tempo_score=tempo_score,
            octave_error_rate=octave_error_rate,
            gross_error_rate=gross_error_rate,
            pause_accuracy=pause_accuracy,
            timbre_similarity=timbre_sim,
        )

        # Detailed metrics
        metrics = {
            "mean_error_cents": float(np.mean(errors_cents)) if len(errors_cents) > 0 else 0,
            "std_error_cents": float(np.std(errors_cents)) if len(errors_cents) > 0 else 0,
            "user_duration": user_contour.duration,
            "reference_duration": reference_contour.duration,
            "tempo_ratio": user_contour.duration / reference_contour.duration
                if reference_contour.duration > 0 else 1.0,
            "n_voiced_frames": int(np.sum(user_contour.confidence >= 0.5)),
            "n_aligned_frames": len(alignment.path) if alignment.path else 0,
        }

        return EnhancedRecitationScore(
            overall_score=overall_score,
            alignment_score=alignment_score,
            on_note_percent=on_note_pct,
            pitch_stability=pitch_stability,
            tempo_score=tempo_score,
            voiced_ratio=voiced_ratio,
            weighted_on_note_percent=weighted_on_note,
            weighted_pitch_accuracy=weighted_accuracy,
            octave_error_rate=octave_error_rate,
            gross_error_rate=gross_error_rate,
            median_error_cents=median_error,
            p95_error_cents=p95_error,
            pause_accuracy=pause_accuracy,
            timing_consistency=timing_consistency,
            timbre_similarity=timbre_sim,
            energy_correlation=energy_corr,
            metrics=metrics,
        )

    def _calculate_on_note_percent(
        self, user, reference, alignment
    ) -> float:
        """Standard on-note percentage."""
        if not alignment.path:
            return 0.0

        on_note = 0
        total = 0

        for u_idx, r_idx in alignment.path:
            if user.confidence[u_idx] >= 0.5 and reference.confidence[r_idx] >= 0.5:
                error = abs(user.f0_cents[u_idx] - reference.f0_cents[r_idx])
                if error <= self.on_note_threshold_cents:
                    on_note += 1
                total += 1

        return (on_note / total * 100) if total > 0 else 0.0

    def _calculate_weighted_on_note(
        self, user, reference, alignment
    ) -> float:
        """Confidence-weighted on-note percentage (NEW)."""
        if not alignment.path:
            return 0.0

        weighted_on_note = 0.0
        total_weight = 0.0

        for u_idx, r_idx in alignment.path:
            u_conf = user.confidence[u_idx]
            r_conf = reference.confidence[r_idx]

            if u_conf >= 0.5 and r_conf >= 0.5:
                # Weight by confidence (emphasize high confidence frames)
                weight = (u_conf ** self.confidence_weight_power) * r_conf

                error = abs(user.f0_cents[u_idx] - reference.f0_cents[r_idx])
                if error <= self.on_note_threshold_cents:
                    weighted_on_note += weight

                total_weight += weight

        return (weighted_on_note / total_weight * 100) if total_weight > 0 else 0.0

    def _calculate_weighted_pitch_accuracy(
        self, user, reference, alignment
    ) -> float:
        """Confidence-weighted pitch accuracy score (NEW)."""
        errors = []
        weights = []

        if not alignment.path:
            return 0.0

        for u_idx, r_idx in alignment.path:
            u_conf = user.confidence[u_idx]
            r_conf = reference.confidence[r_idx]

            if u_conf >= 0.5 and r_conf >= 0.5:
                error = abs(user.f0_cents[u_idx] - reference.f0_cents[r_idx])
                weight = (u_conf ** self.confidence_weight_power) * r_conf

                errors.append(error)
                weights.append(weight)

        if not errors:
            return 0.0

        # Weighted mean error
        weighted_mean_error = np.average(errors, weights=weights)

        # Convert to score: 0 cents = 100, 300 cents = 0
        # Using 300 cents (3 semitones) as maximum acceptable error
        max_error_cents = 300.0
        score = max(0, 100 * (1 - min(1, weighted_mean_error / max_error_cents)))

        return float(score)

    def _get_pitch_errors(
        self, user, reference, alignment
    ) -> np.ndarray:
        """Get all pitch errors in cents."""
        errors = []

        if not alignment.path:
            return np.array([])

        for u_idx, r_idx in alignment.path:
            if user.confidence[u_idx] >= 0.5 and reference.confidence[r_idx] >= 0.5:
                error = abs(user.f0_cents[u_idx] - reference.f0_cents[r_idx])
                errors.append(error)

        return np.array(errors)

    def _calculate_pitch_stability(self, contour) -> float:
        """Pitch stability score."""
        from .scorer import RecitationScorer

        # Use original implementation
        scorer = RecitationScorer(stability_window=self.stability_window)
        return scorer._calculate_pitch_stability(contour)

    def _calculate_tempo_score(self, user, reference, alignment) -> float:
        """Tempo matching score."""
        from .scorer import RecitationScorer

        scorer = RecitationScorer()
        return scorer._calculate_tempo_score(user, reference, alignment)

    def _calculate_pause_accuracy(
        self, user, reference
    ) -> float:
        """
        Pause accuracy score (NEW).

        Measures how well user pauses match reference pauses.
        """
        # Detect pauses (unvoiced segments)
        user_pauses = self._detect_pauses(user)
        ref_pauses = self._detect_pauses(reference)

        if not ref_pauses:
            return 100.0 if not user_pauses else 50.0

        # Match pauses
        matched = 0
        for ref_start, ref_end in ref_pauses:
            # Check if user has pause around same time (relative)
            ref_relative = ref_start / reference.duration
            user_time = ref_relative * user.duration

            # Look for user pause within ±200ms
            for user_start, user_end in user_pauses:
                if abs(user_start - user_time) < 0.2:
                    matched += 1
                    break

        accuracy = (matched / len(ref_pauses) * 100) if ref_pauses else 100.0
        return float(accuracy)

    def _detect_pauses(self, contour) -> list:
        """Detect pause segments (unvoiced >100ms)."""
        pauses = []
        in_pause = False
        pause_start = 0

        for i, conf in enumerate(contour.confidence):
            if conf < 0.5 and not in_pause:
                pause_start = contour.timestamps[i]
                in_pause = True
            elif conf >= 0.5 and in_pause:
                pause_end = contour.timestamps[i - 1]
                duration = pause_end - pause_start
                if duration >= 0.1:  # 100ms minimum
                    pauses.append((pause_start, pause_end))
                in_pause = False

        return pauses

    def _calculate_timing_consistency(self, alignment) -> float:
        """
        Timing consistency score (NEW).

        Measures how consistent the tempo is throughout.
        """
        if not alignment.path or len(alignment.path) < 10:
            return 50.0

        # Calculate local tempo ratios along the path
        tempo_ratios = []

        for i in range(1, len(alignment.path)):
            u_delta = alignment.path[i][0] - alignment.path[i-1][0]
            r_delta = alignment.path[i][1] - alignment.path[i-1][1]

            if r_delta > 0:
                ratio = u_delta / r_delta
                tempo_ratios.append(ratio)

        if not tempo_ratios:
            return 50.0

        # Low variance = high consistency
        std = np.std(tempo_ratios)
        score = max(0, 100 - 100 * std)  # std=0 → 100, std≥1 → 0

        return float(score)

    def _calculate_timbre_similarity(
        self, user_features, ref_features, alignment
    ) -> float:
        """Timbre similarity from mel-spectrograms (NEW)."""
        if not alignment.path:
            return 0.0

        similarities = []

        for u_idx, r_idx in alignment.path:
            user_mel = user_features.mel_spec[:, u_idx]
            ref_mel = ref_features.mel_spec[:, r_idx]

            # Cosine similarity
            cos_sim = np.dot(user_mel, ref_mel) / (
                np.linalg.norm(user_mel) * np.linalg.norm(ref_mel) + 1e-8
            )
            similarities.append(np.clip(cos_sim, 0, 1))

        score = np.mean(similarities) * 100 if similarities else 0.0
        return float(score)

    def _calculate_energy_correlation(
        self, user_features, ref_features, alignment
    ) -> float:
        """Energy correlation (NEW)."""
        if not alignment.path or user_features.rms is None or ref_features.rms is None:
            return None

        user_energy = []
        ref_energy = []

        for u_idx, r_idx in alignment.path:
            user_energy.append(user_features.rms[u_idx])
            ref_energy.append(ref_features.rms[r_idx])

        if len(user_energy) < 2:
            return None

        corr = np.corrcoef(user_energy, ref_energy)[0, 1]
        return float(np.clip(corr, 0, 1) * 100)

    def _calculate_overall_score(
        self,
        alignment_score,
        weighted_on_note,
        pitch_stability,
        tempo_score,
        octave_error_rate,
        gross_error_rate,
        pause_accuracy,
        timbre_similarity,
    ) -> float:
        """
        Calculate overall score with enhanced weighting.

        Weights:
        - 30% Weighted on-note % (confidence-weighted pitch accuracy)
        - 25% Alignment score (DTW)
        - 15% Pitch stability
        - 10% Tempo score
        - 10% Pause accuracy
        - 5% Timbre similarity (if available)
        - Penalties: octave errors, gross errors
        """
        score = (
            0.30 * weighted_on_note +
            0.25 * alignment_score +
            0.15 * pitch_stability +
            0.10 * tempo_score +
            0.10 * pause_accuracy
        )

        # Add timbre if available
        if timbre_similarity is not None:
            score += 0.05 * timbre_similarity
        else:
            # Redistribute weight
            score *= 1.05

        # Penalties
        octave_penalty = min(20, octave_error_rate * 100)
        gross_error_penalty = min(15, gross_error_rate * 50)

        score = max(0, score - octave_penalty - gross_error_penalty)

        return float(score)
