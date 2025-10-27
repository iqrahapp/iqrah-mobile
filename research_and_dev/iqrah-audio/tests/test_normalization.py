"""Tests for Arabic text normalization."""

import pytest
from src.iqrah.text import normalize_arabic_text, normalize_arabic_words, normalize_arabic_chars


def test_diacritic_removal():
    """Test: Diacritics are removed consistently"""
    with_diacritics = "بِسْمِ اللَّهِ"
    without_diacritics = "بسم الله"
    assert normalize_arabic_text(with_diacritics) == normalize_arabic_text(without_diacritics)


def test_alif_normalization():
    """Test: Normalization handles alef variants correctly"""
    text1 = "أَحْمَد"
    text2 = "احمد"
    assert normalize_arabic_text(text1) == normalize_arabic_text(text2)


def test_alif_all_forms():
    """Test: All alif forms normalize to plain alif"""
    forms = ["أحمد", "إحمد", "آحمد", "ٱحمد"]
    normalized = [normalize_arabic_text(f) for f in forms]
    assert len(set(normalized)) == 1  # All should be the same


def test_hamza_carriers():
    """Test: Hamza carriers are normalized"""
    text_with_hamza = "مؤمن"  # ؤ → و
    expected = "مومن"
    assert normalize_arabic_text(text_with_hamza) == expected

    text_with_hamza2 = "رئيس"  # ئ → ي
    expected2 = "رييس"
    assert normalize_arabic_text(text_with_hamza2) == expected2


def test_standalone_hamza_removed():
    """Test: Standalone hamza is removed"""
    text = "شيء"
    result = normalize_arabic_text(text)
    assert 'ء' not in result


def test_tatweel_removal():
    """Test: Tatweel (kashida) is removed"""
    with_tatweel = "اللـــه"
    without_tatweel = "الله"
    assert normalize_arabic_text(with_tatweel) == normalize_arabic_text(without_tatweel)


def test_punctuation_stripping():
    """Test: Punctuation is removed"""
    text_with_punctuation = "بسم الله، الرحمن الرحيم؟"
    result = normalize_arabic_text(text_with_punctuation)
    assert '،' not in result
    assert '؟' not in result


def test_whitespace_collapse():
    """Test: Multiple spaces collapsed to single space"""
    text = "بسم     الله  \n\t  الرحمن"
    result = normalize_arabic_text(text)
    assert "  " not in result
    assert "\n" not in result
    assert "\t" not in result


def test_normalize_arabic_words():
    """Test: normalize_arabic_words returns word list"""
    text = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"
    words = normalize_arabic_words(text)
    assert isinstance(words, list)
    assert len(words) == 4
    assert words == ['بسم', 'الله', 'الرحمن', 'الرحيم']


def test_normalize_arabic_chars():
    """Test: normalize_arabic_chars returns string without spaces"""
    text = "بِسْمِ اللَّهِ"
    chars = normalize_arabic_chars(text)
    assert isinstance(chars, str)
    assert ' ' not in chars
    assert chars == 'بسمالله'


def test_empty_string():
    """Test: Empty string handling"""
    assert normalize_arabic_text("") == ""
    assert normalize_arabic_words("") == []
    assert normalize_arabic_chars("") == ""


def test_whitespace_only():
    """Test: Whitespace-only string"""
    assert normalize_arabic_text("   \n\t  ") == ""
    assert normalize_arabic_words("   ") == []


def test_fatiha_first_verse():
    """Test: Full Al-Fatiha first verse normalization"""
    original = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"
    normalized = normalize_arabic_words(original)
    assert normalized == ['بسم', 'الله', 'الرحمن', 'الرحيم']
