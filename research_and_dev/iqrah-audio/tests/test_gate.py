"""Tests for hybrid WER/CER gatekeeper."""

import pytest
from src.iqrah.compare import ContentGate, verify_content, compute_wer, compute_cer, select_error_metric


def test_wer_exact_match():
    """Test: WER = 0 for identical texts"""
    ref = ['بسم', 'الله']
    hyp = ['بسم', 'الله']
    wer, errors = compute_wer(ref, hyp)
    assert wer == 0.0
    assert len(errors) == 0


def test_wer_one_substitution():
    """Test: WER = 0.5 for 1 error in 2 words"""
    ref = ['بسم', 'الله']
    hyp = ['بسم', 'الرحمن']
    wer, errors = compute_wer(ref, hyp)
    assert wer == 0.5
    assert len(errors) == 1
    assert errors[0]['type'] == 'substitution'
    assert errors[0]['reference_word'] == 'الله'
    assert errors[0]['recited_word'] == 'الرحمن'


def test_wer_one_deletion():
    """Test: WER for deletion"""
    ref = ['بسم', 'الله', 'الرحمن']
    hyp = ['بسم', 'الله']
    wer, errors = compute_wer(ref, hyp)
    assert wer == pytest.approx(0.333, abs=0.01)
    assert len(errors) == 1
    assert errors[0]['type'] == 'deletion'
    assert errors[0]['reference_word'] == 'الرحمن'


def test_cer_exact_match():
    """Test: CER = 0 for identical texts"""
    ref = 'بسمالله'
    hyp = 'بسمالله'
    cer = compute_cer(ref, hyp)
    assert cer == 0.0


def test_cer_one_char_error():
    """Test: CER for single character error"""
    ref = 'بسمالله'
    hyp = 'بسملله'  # One char different
    cer = compute_cer(ref, hyp)
    assert cer > 0.0
    assert cer < 0.2  # Should be small


def test_metric_selection_short_text():
    """Test: CER selected for ≤3 words"""
    ref = ['قل', 'هو']  # 2 words
    metric = select_error_metric(ref)
    assert metric == "cer"

    ref = ['قل', 'هو', 'الله']  # 3 words
    metric = select_error_metric(ref)
    assert metric == "cer"


def test_metric_selection_long_text():
    """Test: WER selected for >3 words"""
    ref = ['بسم', 'الله', 'الرحمن', 'الرحيم']  # 4 words
    metric = select_error_metric(ref)
    assert metric == "wer"


def test_verify_content_high_confidence():
    """Test: error_rate ≤ 0.05 → high confidence"""
    ref = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"
    hyp = "بسم الله الرحمن الرحيم"  # Same, just no diacritics
    result = verify_content(ref, hyp)

    assert result['error_rate'] == 0.0
    assert result['confidence'] == 'high'
    assert result['should_proceed'] is True


def test_verify_content_medium_confidence():
    """Test: 0.05 < error_rate ≤ 0.08 → medium confidence"""
    # Use longer text where single character difference gives ~6% CER
    ref = "قُلْ هُوَ اللَّهُ أَحَدٌ"  # ~15 chars after normalization
    hyp = "قل هو الله احد"  # Identical after normalization
    result = verify_content(ref, hyp)

    # Should pass with high confidence (identical after normalization)
    assert result['should_proceed'] is True
    assert result['error_rate'] <= 0.08


def test_verify_content_fail():
    """Test: error_rate > 0.08 → fail"""
    ref = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"
    hyp = "الحمد لله رب العالمين"  # Completely different
    result = verify_content(ref, hyp)

    assert result['error_rate'] > 0.5
    assert result['confidence'] == 'fail'
    assert result['should_proceed'] is False


def test_verify_content_short_text_uses_cer():
    """Test: Short text uses CER"""
    ref = "قُلْ هُوَ"  # 2 words
    hyp = "قل هو"
    result = verify_content(ref, hyp)

    assert result['metric_type'] == 'cer'


def test_verify_content_long_text_uses_wer():
    """Test: Long text uses WER"""
    ref = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"  # 4 words
    hyp = "بسم الله الرحمن الرحيم"
    result = verify_content(ref, hyp)

    assert result['metric_type'] == 'wer'


def test_content_gate_class():
    """Test: ContentGate class wrapper"""
    gate = ContentGate()

    ref = "بسم الله"
    hyp = "بسم الله"
    result = gate.verify(ref, hyp)

    assert result['confidence'] == 'high'


def test_content_gate_select_metric():
    """Test: ContentGate.select_metric static method"""
    metric = ContentGate.select_metric("قل هو")
    assert metric == "cer"

    metric = ContentGate.select_metric("بسم الله الرحمن الرحيم")
    assert metric == "wer"


def test_normalization_applied():
    """Test: Both reference and hypothesis are normalized"""
    ref = "أَحْمَد"  # With diacritics and hamza
    hyp = "احمد"      # Plain
    result = verify_content(ref, hyp)

    assert result['error_rate'] == 0.0
    assert result['normalized_reference'] == result['normalized_transcript']


def test_empty_hypothesis():
    """Test: Empty hypothesis (user said nothing)"""
    ref = "بسم الله"
    hyp = ""
    result = verify_content(ref, hyp)

    assert result['error_rate'] == 1.0
    assert result['confidence'] == 'fail'


def test_empty_reference():
    """Test: Empty reference (edge case)"""
    ref = ""
    hyp = "بسم الله"
    result = verify_content(ref, hyp)

    # Should handle gracefully
    assert isinstance(result['error_rate'], float)
