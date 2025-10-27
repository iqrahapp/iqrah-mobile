"""Tests for CTC forced alignment."""

import pytest
import numpy as np
from src.iqrah.align import CTCAligner, align_graphemes


def test_align_graphemes_basic():
    """Test: Basic alignment produces expected output"""
    logits = np.random.randn(100, 50)
    reference = "بسم"
    vocab = {'ب': 0, 'س': 1, 'م': 2}

    result = align_graphemes(logits, reference, vocab)

    assert len(result) == 3
    assert all('token' in t for t in result)
    assert all('start' in t for t in result)
    assert all('end' in t for t in result)
    assert all('confidence' in t for t in result)


def test_align_graphemes_empty_reference():
    """Test: Empty reference returns empty list"""
    logits = np.random.randn(100, 50)
    vocab = {'ا': 0}

    result = align_graphemes(logits, "", vocab)

    assert result == []


def test_align_graphemes_timing_monotonic():
    """Test: Token timings are monotonically increasing"""
    logits = np.random.randn(100, 50)
    vocab = {'ب': 0, 'س': 1, 'م': 2, 'ا': 3, 'ل': 4}
    reference = "بسمال"

    result = align_graphemes(logits, reference, vocab)

    for i in range(len(result) - 1):
        assert result[i]['end'] <= result[i+1]['start'] or \
               result[i]['end'] <= result[i+1]['end']


def test_align_graphemes_duration_sanity():
    """Test: Token durations are reasonable"""
    logits = np.random.randn(100, 50)
    vocab = {'ب': 0, 'س': 1, 'م': 2}
    reference = "بسم"

    result = align_graphemes(logits, reference, vocab, frame_rate=50.0)

    for token in result:
        duration = token['end'] - token['start']
        assert duration > 0.0
        assert duration < 2.0  # No token should be > 2s


def test_align_graphemes_confidence_range():
    """Test: Confidence values are in [0, 1]"""
    logits = np.random.randn(100, 50)
    vocab = {'ب': 0, 'س': 1, 'م': 2}
    reference = "بسم"

    result = align_graphemes(logits, reference, vocab)

    for token in result:
        assert 0.0 <= token['confidence'] <= 1.0


def test_ctc_aligner_class():
    """Test: CTCAligner class wrapper"""
    aligner = CTCAligner(frame_rate=50.0)

    logits = np.random.randn(100, 50)
    vocab = {'ب': 0, 'س': 1, 'م': 2}
    reference = "بسم"

    result = aligner.align(logits, reference, vocab)

    assert len(result) == 3


def test_validate_alignment_valid():
    """Test: Valid alignment passes validation"""
    aligner = CTCAligner()

    tokens = [
        {"token": "ب", "start": 0.0, "end": 0.05, "confidence": 0.8},
        {"token": "س", "start": 0.05, "end": 0.10, "confidence": 0.7},
        {"token": "م", "start": 0.10, "end": 0.15, "confidence": 0.9}
    ]

    validation = aligner.validate_alignment(tokens)

    assert validation['is_valid'] is True
    assert validation['mean_confidence'] > 0.7
    assert validation['duration_violations'] == 0


def test_validate_alignment_low_confidence():
    """Test: Low confidence fails validation"""
    aligner = CTCAligner()

    tokens = [
        {"token": "ب", "start": 0.0, "end": 0.05, "confidence": 0.3},
        {"token": "س", "start": 0.05, "end": 0.10, "confidence": 0.2}
    ]

    validation = aligner.validate_alignment(tokens)

    assert validation['is_valid'] == False
    assert validation['mean_confidence'] < 0.5
    assert len(validation['warnings']) > 0


def test_validate_alignment_invalid_durations():
    """Test: Invalid durations fail validation"""
    aligner = CTCAligner()

    tokens = [
        {"token": "ب", "start": 0.0, "end": 0.001, "confidence": 0.8},  # Too short
        {"token": "س", "start": 0.001, "end": 1.0, "confidence": 0.8}   # Too long
    ]

    validation = aligner.validate_alignment(tokens)

    assert validation['is_valid'] is False
    assert validation['duration_violations'] > 0


def test_validate_alignment_empty():
    """Test: Empty alignment fails validation"""
    aligner = CTCAligner()

    validation = aligner.validate_alignment([])

    assert validation['is_valid'] is False
    assert "Empty alignment" in validation['warnings']


def test_normalization_applied_in_alignment():
    """Test: Reference text is normalized before alignment"""
    logits = np.random.randn(100, 50)
    vocab = {'ب': 0, 'س': 1, 'م': 2}

    # With diacritics
    reference_with_diacritics = "بِسْمِ"
    result1 = align_graphemes(logits, reference_with_diacritics, vocab)

    # Without diacritics
    reference_plain = "بسم"
    result2 = align_graphemes(logits, reference_plain, vocab)

    # Should produce same number of tokens
    assert len(result1) == len(result2)
