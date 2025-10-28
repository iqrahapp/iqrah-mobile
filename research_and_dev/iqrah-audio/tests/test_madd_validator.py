"""
Tests for Madd Validator (M4 Tier 2 Priority 1)

Tests cover:
- Local distribution estimation
- Madd duration validation
- Violation generation and severity
- Edge cases and error handling
"""

import pytest
import numpy as np
from dataclasses import dataclass

from iqrah.tajweed import MaddValidator, MaddViolation


# Mock AlignedPhoneme for testing
@dataclass
class MockPhoneme:
    phoneme: str
    start: float
    end: float
    phoneme_index: int = 0


class TestDistributionEstimation:
    """Test harakat duration distribution estimation."""

    def test_local_distribution_basic(self):
        """Test basic local distribution estimation."""
        validator = MaddValidator()

        # Create mock short vowels with consistent duration (~100ms)
        short_vowels = [
            MockPhoneme(phoneme='a', start=0.0, end=0.1),
            MockPhoneme(phoneme='i', start=0.2, end=0.3),
            MockPhoneme(phoneme='u', start=0.4, end=0.5),
            MockPhoneme(phoneme='a', start=0.6, end=0.7),
            MockPhoneme(phoneme='i', start=0.8, end=0.9),
        ]

        mean, std, n = validator._estimate_local_distribution(short_vowels)

        assert n == 5
        assert 95.0 < mean < 105.0  # ~100ms
        assert 0 <= std < 20.0  # Low variance (consistent)

    def test_local_distribution_variable_pace(self):
        """Test distribution with variable pace."""
        validator = MaddValidator()

        # Create short vowels with variable durations
        short_vowels = [
            MockPhoneme(phoneme='a', start=0.0, end=0.08),   # 80ms
            MockPhoneme(phoneme='i', start=0.2, end=0.32),   # 120ms
            MockPhoneme(phoneme='u', start=0.4, end=0.49),   # 90ms
            MockPhoneme(phoneme='a', start=0.6, end=0.75),   # 150ms
            MockPhoneme(phoneme='i', start=0.8, end=0.88),   # 80ms
        ]

        mean, std, n = validator._estimate_local_distribution(short_vowels)

        assert n == 5
        assert 100.0 < mean < 110.0  # Mean ~104ms
        assert std > 20.0  # High variance (inconsistent pace)

    def test_local_distribution_insufficient_samples(self):
        """Test with insufficient samples (< 5)."""
        validator = MaddValidator(min_samples_for_estimation=5)

        # Only 3 short vowels
        short_vowels = [
            MockPhoneme(phoneme='a', start=0.0, end=0.1),
            MockPhoneme(phoneme='i', start=0.2, end=0.3),
            MockPhoneme(phoneme='u', start=0.4, end=0.5),
        ]

        mean, std, n = validator._estimate_local_distribution(short_vowels)

        # Should return defaults
        assert n == 0
        assert mean == validator.DEFAULT_HARAKAT_MS
        assert std == validator.DEFAULT_HARAKAT_STD

    def test_local_distribution_window_filtering(self):
        """Test that only recent phonemes are used."""
        validator = MaddValidator(local_window_seconds=5.0)

        # Mix of old and recent short vowels
        phonemes = [
            # Old phonemes (outside window)
            MockPhoneme(phoneme='a', start=0.0, end=0.05),   # 50ms
            MockPhoneme(phoneme='i', start=0.2, end=0.25),   # 50ms

            # Recent phonemes (inside window, ending at 10s)
            MockPhoneme(phoneme='a', start=6.0, end=6.1),    # 100ms
            MockPhoneme(phoneme='i', start=7.0, end=7.1),    # 100ms
            MockPhoneme(phoneme='u', start=8.0, end=8.1),    # 100ms
            MockPhoneme(phoneme='a', start=9.0, end=9.1),    # 100ms
            MockPhoneme(phoneme='i', start=10.0, end=10.1),  # 100ms
        ]

        mean, std, n = validator._estimate_local_distribution(phonemes)

        assert n == 5  # Only recent vowels counted
        assert 95.0 < mean < 105.0  # ~100ms (not affected by old 50ms vowels)

    def test_update_distributions(self):
        """Test update_distributions method."""
        validator = MaddValidator()

        phonemes = [
            MockPhoneme(phoneme='a', start=0.0, end=0.1),
            MockPhoneme(phoneme='i', start=0.2, end=0.3),
            MockPhoneme(phoneme='u', start=0.4, end=0.5),
            MockPhoneme(phoneme='a', start=0.6, end=0.7),
            MockPhoneme(phoneme='i', start=0.8, end=0.9),
        ]

        validator.update_distributions(phonemes)

        assert validator.n_local_samples == 5
        assert 95.0 < validator.local_mean_ms < 105.0


class TestMaddValidation:
    """Test Madd duration validation logic."""

    def test_valid_madd_no_violations(self):
        """Test valid Madd (within tolerance) produces no violations."""
        validator = MaddValidator()

        # Create consistent pace (100ms harakat)
        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(5)
        ]

        # Create valid Madd (1 harakat ≈ 100ms)
        long_vowel = MockPhoneme(phoneme='ā', start=1.0, end=1.1)  # 100ms

        all_phonemes = short_vowels + [long_vowel]

        validator.update_distributions(all_phonemes)
        violations = validator.validate(all_phonemes)

        # Should have no violations (100ms is expected for 1 harakat)
        assert len(violations) == 0

    def test_madd_too_short_violation(self):
        """Test Madd that's too short generates violation."""
        validator = MaddValidator(z_score_threshold=2.0)

        # Establish pace: 100ms harakat
        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(5)
        ]

        # Create Madd that's too short (50ms, but should be ~100ms)
        long_vowel = MockPhoneme(phoneme='ā', start=1.0, end=1.05)  # 50ms

        all_phonemes = short_vowels + [long_vowel]

        validator.update_distributions(all_phonemes)
        violations = validator.validate(all_phonemes)

        # Should have 1 violation
        assert len(violations) == 1

        violation = violations[0]
        assert violation.rule == "Madd"
        assert violation.tier == 2
        assert violation.actual_duration < violation.expected_duration
        assert "too short" in violation.feedback.lower()

    def test_madd_too_long_violation(self):
        """Test Madd that's too long generates violation."""
        validator = MaddValidator(z_score_threshold=2.0)

        # Establish pace: 100ms harakat
        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(5)
        ]

        # Create Madd that's too long (300ms, but should be ~100ms for 1 harakat)
        long_vowel = MockPhoneme(phoneme='ā', start=1.0, end=1.3)  # 300ms

        all_phonemes = short_vowels + [long_vowel]

        validator.update_distributions(all_phonemes)
        violations = validator.validate(all_phonemes)

        # Should have 1 violation
        assert len(violations) == 1

        violation = violations[0]
        assert violation.rule == "Madd"
        assert violation.tier == 2
        assert violation.actual_duration > violation.expected_duration
        assert "too long" in violation.feedback.lower()

    def test_z_score_computation(self):
        """Test z-score and confidence computation."""
        validator = MaddValidator()

        # Establish pace: mean=100ms, std~5ms (very consistent)
        # Create 10 samples for stable estimation (all exactly 100ms)
        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(10)
        ]

        # Create Madd with deviation = 30ms
        # Expected: 1 × 100 = 100ms
        # Actual: 130ms
        # std ≈ 5ms (min enforced), so z = (130 - 100) / 5 = 6.0
        long_vowel = MockPhoneme(phoneme='ā', start=2.0, end=2.13)

        all_phonemes = short_vowels + [long_vowel]

        validator.update_distributions(all_phonemes)
        violations = validator.validate(all_phonemes)

        assert len(violations) == 1
        violation = violations[0]

        # Z-score should be ~6.0 (30ms deviation / 5ms std)
        assert 5.5 < abs(violation.z_score) < 6.5
        assert violation.severity == "critical"  # |z| > 3

    def test_severity_levels(self):
        """Test severity classification."""
        validator = MaddValidator()

        # Setup pace
        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(10)
        ]

        # Create Madds with different deviations
        madds = [
            MockPhoneme(phoneme='ā', start=2.0, end=2.13),   # 130ms, z~3.0 → critical
            MockPhoneme(phoneme='ī', start=3.0, end=3.127),  # 127ms, z~2.7 → moderate
            MockPhoneme(phoneme='ū', start=4.0, end=4.123),  # 123ms, z~2.3 → minor
        ]

        all_phonemes = short_vowels + madds

        validator.update_distributions(all_phonemes)
        violations = validator.validate(all_phonemes)

        # All should violate (z > 2.0 threshold)
        assert len(violations) == 3

        # Check severity gradation
        severities = [v.severity for v in violations]
        assert "critical" in severities or "moderate" in severities


class TestMaddTypes:
    """Test different Madd types."""

    def test_madd_tabi_i_default(self):
        """Test Madd Tabi'i (natural, 1 harakat)."""
        validator = MaddValidator()

        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(5)
        ]

        # Long vowel without metadata defaults to Tabi'i (1 harakat)
        long_vowel = MockPhoneme(phoneme='ā', start=1.0, end=1.1)

        all_phonemes = short_vowels + [long_vowel]

        validator.update_distributions(all_phonemes)
        violations = validator.validate(all_phonemes)

        # Should be valid (100ms = 1 × 100ms)
        assert len(violations) == 0

    def test_non_long_vowel_ignored(self):
        """Test that non-long vowels are ignored."""
        validator = MaddValidator()

        # Only short vowels, no Madds
        phonemes = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(5)
        ]

        # Add consonants
        phonemes.append(MockPhoneme(phoneme='b', start=1.0, end=1.05))
        phonemes.append(MockPhoneme(phoneme='s', start=1.1, end=1.15))

        validator.update_distributions(phonemes)
        violations = validator.validate(phonemes)

        # No violations (no Madds)
        assert len(violations) == 0


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_empty_phoneme_list(self):
        """Test with empty phoneme list."""
        validator = MaddValidator()

        violations = validator.validate([])

        assert len(violations) == 0

    def test_no_short_vowels_uses_defaults(self):
        """Test fallback to defaults when no short vowels."""
        validator = MaddValidator()

        # Only long vowels and consonants
        phonemes = [
            MockPhoneme(phoneme='b', start=0.0, end=0.05),
            MockPhoneme(phoneme='ā', start=0.1, end=0.2),  # 100ms long vowel
        ]

        validator.update_distributions(phonemes)

        # Should use defaults
        assert validator.local_mean_ms == validator.DEFAULT_HARAKAT_MS
        assert validator.local_std_ms == validator.DEFAULT_HARAKAT_STD

        violations = validator.validate(phonemes)

        # Should still validate (using defaults)
        assert isinstance(violations, list)

    def test_get_statistics(self):
        """Test statistics getter."""
        validator = MaddValidator()

        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(5)
        ]

        validator.update_distributions(short_vowels)

        stats = validator.get_statistics()

        assert "local_mean_ms" in stats
        assert "local_std_ms" in stats
        assert "n_local_samples" in stats
        assert "effective_mean_ms" in stats
        assert "effective_std_ms" in stats

        assert stats["n_local_samples"] == 5
        assert stats["local_mean_ms"] == stats["effective_mean_ms"]


class TestGlobalDistribution:
    """Test global distribution blending (Phase 2 feature)."""

    def test_global_distribution_blending(self):
        """Test blending local + global distributions."""
        validator = MaddValidator()

        # Local: mean=100ms, std=10ms
        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(10)
        ]

        # Global stats: mean=120ms, std=15ms, weight=0.5
        global_stats = {
            "mean": 120.0,
            "std": 15.0,
            "weight": 0.5
        }

        validator.update_distributions(short_vowels, global_stats)

        # Effective mean should be blend
        effective_mean = validator._get_effective_mean()
        expected_mean = 0.5 * 100.0 + 0.5 * 120.0  # 110ms

        assert abs(effective_mean - expected_mean) < 1.0

    def test_global_weight_zero_uses_local_only(self):
        """Test that global_weight=0 uses local only."""
        validator = MaddValidator()

        short_vowels = [
            MockPhoneme(phoneme='a', start=i*0.2, end=i*0.2+0.1)
            for i in range(10)
        ]

        global_stats = {
            "mean": 200.0,  # Very different
            "std": 50.0,
            "weight": 0.0   # Don't use global
        }

        validator.update_distributions(short_vowels, global_stats)

        effective_mean = validator._get_effective_mean()

        # Should match local only
        assert abs(effective_mean - validator.local_mean_ms) < 0.1


class TestViolationDataclass:
    """Test MaddViolation dataclass."""

    def test_madd_violation_structure(self):
        """Test MaddViolation has all required fields."""
        violation = MaddViolation(
            rule="Madd",
            subtype="tabi'i_1_harakats",
            phoneme_idx=5,
            phoneme='ā',
            timestamp=1.5,
            expected_duration=100.0,
            actual_duration=50.0,
            z_score=-2.5,
            confidence=0.95,
            severity="moderate",
            tier=2,
            feedback="Madd tabi'i at 1.50s is too short."
        )

        assert violation.rule == "Madd"
        assert violation.tier == 2
        assert violation.z_score == -2.5
        assert violation.phoneme == 'ā'
        assert "too short" in violation.feedback


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
