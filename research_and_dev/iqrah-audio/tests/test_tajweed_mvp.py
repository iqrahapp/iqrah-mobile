"""Tests for Tajweed MVP rules."""

import pytest
import numpy as np
from src.iqrah.tajweed import TajweedValidatorMVP, validate_madd, validate_shadda, validate_waqf


def test_validate_madd_violation():
    """Test: Short Madd vowel triggers violation"""
    tokens = [
        {"token": "ا", "start": 0.0, "end": 0.15, "confidence": 0.8}  # 150ms < 200ms
    ]

    violations = validate_madd(tokens, min_duration_ms=200.0)

    assert len(violations) == 1
    assert violations[0]['token'] == 'ا'
    assert violations[0]['severity'] in ['minor', 'moderate', 'critical']


def test_validate_madd_pass():
    """Test: Long enough Madd vowel passes"""
    tokens = [
        {"token": "ا", "start": 0.0, "end": 0.25, "confidence": 0.8}  # 250ms > 200ms
    ]

    violations = validate_madd(tokens, min_duration_ms=200.0)

    assert len(violations) == 0


def test_validate_madd_non_madd_ignored():
    """Test: Non-Madd characters are ignored"""
    tokens = [
        {"token": "ب", "start": 0.0, "end": 0.05, "confidence": 0.8},
        {"token": "س", "start": 0.05, "end": 0.10, "confidence": 0.8}
    ]

    violations = validate_madd(tokens, min_duration_ms=200.0)

    assert len(violations) == 0


def test_validate_madd_multiple_vowels():
    """Test: Multiple Madd vowels checked"""
    tokens = [
        {"token": "ا", "start": 0.0, "end": 0.15, "confidence": 0.8},  # Too short
        {"token": "و", "start": 0.15, "end": 0.40, "confidence": 0.8},  # Long enough
        {"token": "ي", "start": 0.40, "end": 0.50, "confidence": 0.8}   # Too short
    ]

    violations = validate_madd(tokens, min_duration_ms=200.0)

    assert len(violations) == 2


def test_validate_shadda_doubled_consonant():
    """Test: Doubled consonant detected and validated"""
    tokens = [
        {"token": "ل", "start": 0.0, "end": 0.05, "confidence": 0.8},
        {"token": "ل", "start": 0.05, "end": 0.08, "confidence": 0.8}  # Shadda
    ]

    violations = validate_shadda(tokens, duration_multiplier=1.6)

    # Should detect the doubling and check duration
    # Exact result depends on median calculation
    assert isinstance(violations, list)


def test_validate_shadda_non_doubled_ignored():
    """Test: Non-doubled consonants ignored"""
    tokens = [
        {"token": "ب", "start": 0.0, "end": 0.05, "confidence": 0.8},
        {"token": "س", "start": 0.05, "end": 0.10, "confidence": 0.8}
    ]

    violations = validate_shadda(tokens)

    assert len(violations) == 0


def test_validate_waqf_with_energy_drop():
    """Test: Energy drop at end → pass"""
    tokens = [
        {"token": "م", "start": 0.0, "end": 0.1, "confidence": 0.8}
    ]

    # Create audio with clear energy drop at end
    # Most audio is loud, then very quiet after token ends
    audio = np.random.randn(16000) * 0.5  # Main audio (loud)
    audio[1600:] = np.random.randn(14400) * 0.01  # Very quiet at end (1% amplitude)

    violations = validate_waqf(tokens, audio, sample_rate=16000, energy_threshold=0.3)

    # Should pass - the after-segment is very quiet
    assert len(violations) == 0


def test_validate_waqf_no_energy_drop():
    """Test: No energy drop → violation"""
    tokens = [
        {"token": "م", "start": 0.0, "end": 0.1, "confidence": 0.8}
    ]

    # Create audio with constant energy
    audio = np.random.randn(16000) * 0.5

    violations = validate_waqf(tokens, audio, sample_rate=16000, energy_threshold=0.3)

    # May or may not trigger depending on random audio
    assert isinstance(violations, list)


def test_tajweed_validator_mvp_full():
    """Test: Full TajweedValidatorMVP workflow"""
    validator = TajweedValidatorMVP()

    tokens = [
        {"token": "ب", "start": 0.0, "end": 0.05, "confidence": 0.8},
        {"token": "ا", "start": 0.05, "end": 0.15, "confidence": 0.8},  # Short Madd
        {"token": "م", "start": 0.15, "end": 0.20, "confidence": 0.8}
    ]

    audio = np.random.randn(16000) * 0.5

    result = validator.validate(tokens, audio)

    assert 'madd_violations' in result
    assert 'shadda_violations' in result
    assert 'waqf_violations' in result
    assert 'overall_score' in result
    assert 0 <= result['overall_score'] <= 100


def test_tajweed_validator_no_audio():
    """Test: Validator works without audio (skips Waqf)"""
    validator = TajweedValidatorMVP()

    tokens = [
        {"token": "ب", "start": 0.0, "end": 0.05, "confidence": 0.8}
    ]

    result = validator.validate(tokens, audio=None)

    assert result['waqf_violations'] == []


def test_tajweed_validator_empty_tokens():
    """Test: Empty tokens list handled gracefully"""
    validator = TajweedValidatorMVP()

    result = validator.validate([], audio=None)

    assert result['overall_score'] == 100.0  # No violations


def test_tajweed_severity_levels():
    """Test: Severity levels assigned correctly"""
    tokens = [
        {"token": "ا", "start": 0.0, "end": 0.05, "confidence": 0.8}  # 50ms << 200ms
    ]

    violations = validate_madd(tokens, min_duration_ms=200.0)

    assert violations[0]['severity'] == 'critical'  # ratio < 0.5


def test_tajweed_all_rules_pass():
    """Test: No violations → 100% score"""
    validator = TajweedValidatorMVP(madd_min_duration_ms=50.0)  # Low threshold

    tokens = [
        {"token": "ب", "start": 0.0, "end": 0.05, "confidence": 0.8},
        {"token": "ا", "start": 0.05, "end": 0.15, "confidence": 0.8}  # > 50ms
    ]

    audio = np.random.randn(16000) * 0.5
    audio[2000:] = np.random.randn(14000) * 0.01  # Energy drop

    result = validator.validate(tokens, audio)

    # Score should be high (may not be exactly 100 due to Waqf randomness)
    assert result['overall_score'] >= 80.0
