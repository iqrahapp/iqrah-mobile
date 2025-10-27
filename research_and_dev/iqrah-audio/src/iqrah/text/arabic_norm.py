"""
Arabic Text Normalization (Library-Based with Fallback)

Uses PyArabic library for robust normalization when available,
with regex-only fallback for environments without it.

Normalization Operations:
1. Library path (PyArabic):
   - Normalize ligatures (araby.normalize_ligature)
   - Strip tashkeel/harakat (araby.strip_tashkeel)
   - Strip tatweel (araby.strip_tatweel)
   - Normalize hamza (araby.normalize_hamza)
2. Canonical mappings (both paths):
   - Map alif forms: أ/إ/آ/ٱ → ا
   - Map hamza carriers: ؤ → و, ئ → ي, ء → (removed)
3. Strip punctuation and collapse whitespace

This provides robust matching for ASR gatekeeper (WER/CER).
"""

import re
from typing import List

# Try to import PyArabic for robust normalization
try:
    from pyarabic import araby
    _HAVE_PYARABIC = True
except ImportError:
    _HAVE_PYARABIC = False

# Punctuation filter (used in both paths)
# Remove: ASCII punctuation + Arabic punctuation (،؛؟٪٫٬)
_PUNCT = re.compile(r"[^\w\u0621-\u064A\u0660-\u0669\s]", re.UNICODE)
# This keeps: Arabic letters (U+0621-U+064A), Arabic digits (U+0660-U+0669), and whitespace
# Removes: Punctuation, including Arabic comma ،(U+060C), semicolon ؛(U+061B), question ؟(U+061F)


def _lib_normalize(text: str) -> str:
    """
    Normalize Arabic using PyArabic library (preferred path).

    Steps:
    1. Canonicalize alif variants & hamza carriers (BEFORE stripping)
    2. Normalize ligatures (لا → ل + ا, etc.)
    3. Strip tashkeel (diacritics/harakat)
    4. Strip tatweel (kashida/elongation)
    5. Strip superscript alef and other remaining marks
    6. Strip punctuation and collapse whitespace

    Args:
        text: Raw Arabic text

    Returns:
        Normalized text

    Examples:
        >>> _lib_normalize("بِسْمِ اللَّهِ")
        'بسم الله'
    """
    # FIRST: Canonicalize alif variants & hamza carriers (before PyArabic strips things)
    # This ensures أ → ا conversion happens before any other processing
    text = (text
            .replace("أ", "ا").replace("إ", "ا").replace("آ", "ا").replace("ٱ", "ا")
            .replace("ؤ", "و").replace("ئ", "ي").replace("ء", ""))

    # PyArabic normalization
    text = araby.normalize_ligature(text)
    text = araby.strip_tashkeel(text)   # Remove harakat/diacritics
    text = araby.strip_tatweel(text)    # Remove tatweel
    # NOTE: We DON'T use araby.normalize_hamza() - it's too aggressive

    # Remove superscript alef (U+0670) which might not be caught by strip_tashkeel
    text = text.replace("\u0670", "")

    # Punctuation and spacing (including Arabic punctuation)
    text = _PUNCT.sub(" ", text)
    text = re.sub(r"\s+", " ", text).strip()
    return text


def _fallback_normalize(text: str) -> str:
    """
    Regex-only fallback normalization (when PyArabic not available).

    Steps:
    1. Remove combining marks (harakat, sukun, madd, small signs)
    2. Remove tatweel
    3. Canonicalize alif variants & hamza carriers
    4. Strip punctuation and collapse whitespace

    Args:
        text: Raw Arabic text

    Returns:
        Normalized text

    Examples:
        >>> _fallback_normalize("بِسْمِ اللَّهِ")
        'بسم الله'
    """
    # Remove combining marks (harakat, sukun, madd, small signs)
    # Range: U+064B (fathatan) to U+0652 (sukun), U+0670 (alif khanjariyah),
    #        U+06DC-U+06ED (small signs)
    text = re.sub(r"[\u064B-\u0652\u0670\u06DC-\u06ED]", "", text)

    # Remove tatweel (kashida)
    text = text.replace("\u0640", "")

    # Canonicalize alif variants & hamza carriers
    text = (text
            .replace("أ", "ا").replace("إ", "ا").replace("آ", "ا").replace("ٱ", "ا")
            .replace("ؤ", "و").replace("ئ", "ي").replace("ء", ""))

    # Punctuation and spacing
    text = _PUNCT.sub(" ", text)
    text = re.sub(r"\s+", " ", text).strip()
    return text


def normalize_arabic_text(text: str) -> str:
    """
    Normalize Arabic text using library (preferred) or fallback path.

    Automatically selects PyArabic if available, otherwise uses regex-only approach.

    Args:
        text: Raw Arabic text (with diacritics, punctuation, etc.)

    Returns:
        Normalized text (single string with normalized whitespace)

    Examples:
        >>> normalize_arabic_text("بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ")
        'بسم الله الرحمن الرحيم'
        >>> normalize_arabic_text("أَحْمَدُ")
        'احمد'
    """
    return _lib_normalize(text) if _HAVE_PYARABIC else _fallback_normalize(text)


def normalize_arabic_words(text: str) -> List[str]:
    """
    Normalize Arabic text and split into word list.

    This is the standard function for WER (Word Error Rate) calculation.

    Args:
        text: Raw Arabic text

    Returns:
        List of normalized words

    Examples:
        >>> normalize_arabic_words("بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ")
        ['بسم', 'الله', 'الرحمن', 'الرحيم']
        >>> len(normalize_arabic_words("بِسْمِ اللَّهِ"))
        2
    """
    normalized = normalize_arabic_text(text)
    return normalized.split()


def normalize_arabic_chars(text: str) -> str:
    """
    Normalize Arabic text and return as character string (no spaces).

    This is the standard function for CER (Character Error Rate) calculation.

    Args:
        text: Raw Arabic text

    Returns:
        Normalized text with spaces removed

    Examples:
        >>> normalize_arabic_chars("بِسْمِ اللَّهِ")
        'بسمالله'
        >>> len(normalize_arabic_chars("قُلْ"))
        2
    """
    normalized = normalize_arabic_text(text)
    return normalized.replace(' ', '')


# Diagnostic function to check which normalization path is being used
def get_normalization_backend() -> str:
    """
    Get the active normalization backend.

    Returns:
        "pyarabic" if PyArabic library is available, "fallback" otherwise

    Examples:
        >>> backend = get_normalization_backend()
        >>> backend in ["pyarabic", "fallback"]
        True
    """
    return "pyarabic" if _HAVE_PYARABIC else "fallback"
