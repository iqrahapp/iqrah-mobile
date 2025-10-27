"""Tests for LLR confidence scoring."""

import pytest
import numpy as np
from src.iqrah.align import compute_llr, add_llr_scores, interpret_llr


def test_compute_llr_basic():
    """Test: LLR computation returns float"""
    logits = np.random.randn(100, 50)
    llr = compute_llr(logits, token_id=5, start_frame=10, end_frame=20)

    assert isinstance(llr, float)


def test_compute_llr_high_confidence():
    """Test: High posterior yields high LLR"""
    # Create logits where token_id=5 has much higher probability
    logits = np.random.randn(100, 50) - 10  # Low baseline
    logits[10:20, 5] = 5.0  # Token 5 very high

    llr = compute_llr(logits, token_id=5, start_frame=10, end_frame=20)

    assert llr > 0.0  # Should be positive (confident)


def test_compute_llr_low_confidence():
    """Test: Uniform posteriors yield low LLR"""
    # All tokens equally likely
    logits = np.zeros((100, 50))

    llr = compute_llr(logits, token_id=5, start_frame=10, end_frame=20)

    assert llr < 1.0  # Should be low (not confident)


def test_compute_llr_empty_segment():
    """Test: Empty segment returns low score"""
    logits = np.random.randn(100, 50)

    llr = compute_llr(logits, token_id=5, start_frame=50, end_frame=50)

    assert llr == -10.0


def test_compute_llr_numerical_stability():
    """Test: No NaN or Inf"""
    logits = np.random.randn(100, 50) * 100  # Large values

    llr = compute_llr(logits, token_id=5, start_frame=10, end_frame=20)

    assert np.isfinite(llr)


def test_add_llr_scores():
    """Test: LLR scores added to tokens"""
    tokens = [
        {"token": "ا", "start": 0.0, "end": 0.1, "confidence": 0.8},
        {"token": "ل", "start": 0.1, "end": 0.2, "confidence": 0.7}
    ]

    logits = np.random.randn(100, 50)
    vocab = {'ا': 0, 'ل': 1}

    result = add_llr_scores(tokens, logits, vocab)

    assert 'gop_score' in result[0]
    assert 'gop_score' in result[1]
    assert isinstance(result[0]['gop_score'], float)


def test_add_llr_scores_unknown_token():
    """Test: Unknown token gets low LLR"""
    tokens = [{"token": "x", "start": 0.0, "end": 0.1, "confidence": 0.8}]

    logits = np.random.randn(100, 50)
    vocab = {'ا': 0}  # 'x' not in vocab

    result = add_llr_scores(tokens, logits, vocab)

    assert result[0]['gop_score'] == -10.0


def test_interpret_llr_very_confident():
    """Test: LLR > 2.0 → very_confident"""
    assert interpret_llr(2.5) == "very_confident"
    assert interpret_llr(5.0) == "very_confident"


def test_interpret_llr_confident():
    """Test: 1.0 < LLR ≤ 2.0 → confident"""
    assert interpret_llr(1.5) == "confident"
    assert interpret_llr(2.0) == "confident"


def test_interpret_llr_moderate():
    """Test: 0.5 < LLR ≤ 1.0 → moderate"""
    assert interpret_llr(0.7) == "moderate"
    assert interpret_llr(1.0) == "moderate"


def test_interpret_llr_low():
    """Test: LLR ≤ 0.5 → low_confidence"""
    assert interpret_llr(0.3) == "low_confidence"
    assert interpret_llr(-1.0) == "low_confidence"


def test_llr_monotonicity():
    """Test: Higher posterior → higher LLR (approximately)"""
    # This is a statistical test, may have some variance
    logits_low = np.random.randn(100, 50) - 5
    logits_low[10:20, 5] = 0.0  # Token 5 slightly elevated

    logits_high = np.random.randn(100, 50) - 5
    logits_high[10:20, 5] = 10.0  # Token 5 much more elevated

    llr_low = compute_llr(logits_low, token_id=5, start_frame=10, end_frame=20)
    llr_high = compute_llr(logits_high, token_id=5, start_frame=10, end_frame=20)

    # High confidence should have higher LLR
    assert llr_high > llr_low
