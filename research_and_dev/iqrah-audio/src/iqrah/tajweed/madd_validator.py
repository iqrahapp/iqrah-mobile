"""
Madd (Vowel Elongation) Validator - M4 Tier 2 Priority 1

Implements probabilistic duration modeling for Madd rules validation.

Approach:
- Estimate harakat duration from recent audio (local distribution)
- Validate Madd elongations using Gaussian model (2-sigma rule)
- Support multiple Madd types: Tabi'i (1), Jaiz (2-4), Lazim (6)

Accuracy Target: 95%+ (Phase 1)

References:
- doc/01-architecture/m4-tajweed.md Section 3.1
- Classical Tajweed rules for Madd types
"""

from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
import numpy as np
from scipy.stats import norm


@dataclass
class MaddViolation:
    """
    Single Madd violation with detailed information.

    Attributes:
        rule: Always "Madd"
        subtype: Specific Madd type (e.g., "6_harakats" for Madd Lazim)
        phoneme_idx: Index in phoneme sequence
        phoneme: Phoneme character
        timestamp: Time in audio (seconds)
        expected_duration: Expected duration in ms
        actual_duration: Actual duration in ms
        z_score: Standard deviations from expected
        confidence: Confidence that this is a violation (0-1)
        severity: "critical", "moderate", or "minor"
        tier: Always 2 (Tier 2 specialized module)
        feedback: User-facing message
    """
    rule: str
    subtype: str
    phoneme_idx: int
    phoneme: str
    timestamp: float
    expected_duration: float
    actual_duration: float
    z_score: float
    confidence: float
    severity: str
    tier: int
    feedback: str


class MaddValidator:
    """
    Madd (vowel elongation) validator using probabilistic duration modeling.

    Uses Gaussian distributions to model harakat duration variability:
    - Local distribution: Recent audio window (adaptive to current pace)
    - Global distribution: User history (optional, for Phase 2)

    Validation:
    1. Estimate local harakat duration (mean, std) from short vowels
    2. For each Madd: expected = harakats × local_mean
    3. Tolerance = 2 × local_std (2-sigma rule, ~95% confidence)
    4. Violation if |actual - expected| > tolerance
    """

    # Default harakat duration (fallback if estimation fails)
    DEFAULT_HARAKAT_MS = 100.0
    DEFAULT_HARAKAT_STD = 20.0

    # Madd type definitions (from Tajweed rules)
    MADD_TYPES = {
        "tabi'i": 1,      # Natural (مد طبيعي)
        "muttasil": 4,    # Connected necessary (مد متصل)
        "munfasil": 2,    # Separated permissible (مد منفصل)
        "lazim": 6,       # Necessary (مد لازم)
        "aared": 2,       # Incidental (مد عارض)
        "leen": 2,        # Softness (مد لين)
        "badal": 1,       # Substitute (مد بدل)
        "sila": 2,        # Connection (مد صلة)
    }

    # Short vowels used for harakat estimation
    SHORT_VOWELS = {'a', 'i', 'u', 'َ', 'ِ', 'ُ'}

    # Long vowels (Madd letters)
    LONG_VOWELS = {'aa', 'ii', 'uu', 'ā', 'ī', 'ū', 'ا', 'ي', 'و'}

    def __init__(
        self,
        local_window_seconds: float = 10.0,
        z_score_threshold: float = 2.0,
        min_samples_for_estimation: int = 5
    ):
        """
        Initialize Madd validator.

        Args:
            local_window_seconds: Time window for local distribution (default: 10s)
            z_score_threshold: Sigma tolerance for violations (default: 2.0)
            min_samples_for_estimation: Minimum short vowels needed (default: 5)
        """
        self.local_window_seconds = local_window_seconds
        self.z_score_threshold = z_score_threshold
        self.min_samples_for_estimation = min_samples_for_estimation

        # Distribution parameters (updated by update_distributions)
        self.local_mean_ms: float = self.DEFAULT_HARAKAT_MS
        self.local_std_ms: float = self.DEFAULT_HARAKAT_STD
        self.n_local_samples: int = 0

        # Global distribution (optional, for Phase 2)
        self.global_mean_ms: Optional[float] = None
        self.global_std_ms: Optional[float] = None
        self.global_weight: float = 0.0  # 0 = local only, 1 = global only

    def update_distributions(
        self,
        aligned_phonemes: List,
        global_stats: Optional[Dict] = None
    ) -> None:
        """
        Update local and global harakat duration distributions.

        Args:
            aligned_phonemes: All aligned phonemes from M3
            global_stats: Optional dict with {"mean": float, "std": float}
        """
        # Estimate local distribution
        self.local_mean_ms, self.local_std_ms, self.n_local_samples = \
            self._estimate_local_distribution(aligned_phonemes)

        # Update global distribution if provided
        if global_stats:
            self.global_mean_ms = global_stats.get("mean")
            self.global_std_ms = global_stats.get("std")
            self.global_weight = global_stats.get("weight", 0.0)

    def _estimate_local_distribution(
        self,
        aligned_phonemes: List
    ) -> Tuple[float, float, int]:
        """
        Estimate harakat duration from recent short vowels.

        Args:
            aligned_phonemes: All aligned phonemes from M3

        Returns:
            (mean_ms, std_ms, n_samples)
        """
        if not aligned_phonemes:
            return self.DEFAULT_HARAKAT_MS, self.DEFAULT_HARAKAT_STD, 0

        # Get max timestamp
        max_time = max(p.end for p in aligned_phonemes)

        # Find short vowels in recent window
        recent_short_vowels = [
            p for p in aligned_phonemes
            if p.phoneme in self.SHORT_VOWELS and
               p.end > (max_time - self.local_window_seconds)
        ]

        if len(recent_short_vowels) < self.min_samples_for_estimation:
            # Not enough samples, use default
            return self.DEFAULT_HARAKAT_MS, self.DEFAULT_HARAKAT_STD, 0

        # Compute durations in milliseconds
        durations_ms = [(p.end - p.start) * 1000.0 for p in recent_short_vowels]

        mean_ms = float(np.mean(durations_ms))
        std_ms = float(np.std(durations_ms))

        # Sanity checks
        if mean_ms < 20.0 or mean_ms > 500.0:
            # Unrealistic harakat duration, use default
            return self.DEFAULT_HARAKAT_MS, self.DEFAULT_HARAKAT_STD, 0

        if std_ms < 5.0:
            std_ms = 5.0  # Minimum variance

        return mean_ms, std_ms, len(recent_short_vowels)

    def validate(
        self,
        aligned_phonemes: List,
        phonetic_ref=None
    ) -> List[MaddViolation]:
        """
        Validate Madd elongations using probabilistic model.

        Args:
            aligned_phonemes: Aligned phonemes from M3 with timing
            phonetic_ref: Optional phonetic reference with Madd metadata

        Returns:
            List of MaddViolation objects (empty if no violations)
        """
        violations = []

        # Ensure distributions are up to date
        if self.n_local_samples == 0:
            self.update_distributions(aligned_phonemes)

        # Get effective harakat duration (blend local + global)
        effective_mean = self._get_effective_mean()
        effective_std = self._get_effective_std()

        for idx, phoneme in enumerate(aligned_phonemes):
            # Check if this is a Madd phoneme
            madd_type, expected_harakats = self._get_madd_info(phoneme, phonetic_ref)

            if madd_type is None:
                continue  # Not a Madd phoneme

            # Get actual duration
            actual_duration_ms = (phoneme.end - phoneme.start) * 1000.0

            # Compute expected duration
            expected_duration_ms = expected_harakats * effective_mean

            # Tolerance (2-sigma rule)
            tolerance_ms = self.z_score_threshold * effective_std

            # Check violation
            deviation_ms = abs(actual_duration_ms - expected_duration_ms)

            if deviation_ms > tolerance_ms:
                # Compute z-score
                z_score = (actual_duration_ms - expected_duration_ms) / effective_std

                # Confidence (probability of observing this deviation)
                # Use survival function for one-tailed test
                confidence = 1.0 - norm.cdf(abs(z_score))

                # Severity based on z-score magnitude
                if abs(z_score) > 3.0:
                    severity = "critical"
                elif abs(z_score) > 2.5:
                    severity = "moderate"
                else:
                    severity = "minor"

                # Generate feedback
                if actual_duration_ms < expected_duration_ms:
                    direction = "too short"
                    suggestion = f"Hold for {expected_harakats} harakats (~{expected_duration_ms:.0f}ms)"
                else:
                    direction = "too long"
                    suggestion = f"Shorten to {expected_harakats} harakats (~{expected_duration_ms:.0f}ms)"

                feedback = (
                    f"Madd {madd_type} at {phoneme.start:.2f}s is {direction}. "
                    f"Expected: {expected_duration_ms:.0f}ms, "
                    f"Actual: {actual_duration_ms:.0f}ms. "
                    f"{suggestion}"
                )

                violations.append(MaddViolation(
                    rule="Madd",
                    subtype=f"{madd_type}_{expected_harakats}_harakats",
                    phoneme_idx=idx,
                    phoneme=phoneme.phoneme,
                    timestamp=phoneme.start,
                    expected_duration=expected_duration_ms,
                    actual_duration=actual_duration_ms,
                    z_score=z_score,
                    confidence=min(confidence, 1.0),
                    severity=severity,
                    tier=2,
                    feedback=feedback
                ))

        return violations

    def _get_madd_info(
        self,
        phoneme,
        phonetic_ref
    ) -> Tuple[Optional[str], Optional[int]]:
        """
        Determine if phoneme is a Madd and get expected harakats.

        Args:
            phoneme: AlignedPhoneme object
            phonetic_ref: Optional phonetic reference with metadata

        Returns:
            (madd_type, expected_harakats) or (None, None) if not a Madd
        """
        # Check if phoneme is a long vowel
        if phoneme.phoneme not in self.LONG_VOWELS:
            return None, None

        # Try to get Madd info from phonetic reference metadata
        if phonetic_ref and hasattr(phonetic_ref, 'units'):
            # Look for metadata in corresponding unit
            for unit in phonetic_ref.units:
                if unit.phoneme_index == getattr(phoneme, 'phoneme_index', -1):
                    if hasattr(unit, 'madd_type'):
                        madd_type = unit.madd_type
                        expected_harakats = self.MADD_TYPES.get(madd_type, 2)
                        return madd_type, expected_harakats

        # Fallback: assume Madd Tabi'i (natural, 1 harakat) for long vowels
        # In production, this should be enhanced with rule-based detection
        return "tabi'i", 1

    def _get_effective_mean(self) -> float:
        """Get effective harakat mean (blend local + global)."""
        if self.global_mean_ms is not None and self.global_weight > 0:
            return (
                (1 - self.global_weight) * self.local_mean_ms +
                self.global_weight * self.global_mean_ms
            )
        return self.local_mean_ms

    def _get_effective_std(self) -> float:
        """Get effective harakat std (blend local + global)."""
        if self.global_std_ms is not None and self.global_weight > 0:
            # Combine variances (weighted average of squared stds)
            var_local = self.local_std_ms ** 2
            var_global = self.global_std_ms ** 2
            var_effective = (
                (1 - self.global_weight) * var_local +
                self.global_weight * var_global
            )
            return float(np.sqrt(var_effective))
        return self.local_std_ms

    def get_statistics(self) -> Dict:
        """Get current distribution statistics for debugging."""
        return {
            "local_mean_ms": self.local_mean_ms,
            "local_std_ms": self.local_std_ms,
            "n_local_samples": self.n_local_samples,
            "global_mean_ms": self.global_mean_ms,
            "global_std_ms": self.global_std_ms,
            "global_weight": self.global_weight,
            "effective_mean_ms": self._get_effective_mean(),
            "effective_std_ms": self._get_effective_std()
        }
