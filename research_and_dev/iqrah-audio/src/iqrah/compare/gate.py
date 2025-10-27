"""
Content Verification Gatekeeper (Library-Based with Error Details)

Implements hybrid WER/CER logic:
- Reference ≤ 3 words → Use CER
- Reference > 3 words → Use WER

Uses rapidfuzz for robust Levenshtein distance with detailed edit operations.

Thresholds:
- ≤ 0.05 → High confidence
- ≤ 0.08 → Medium confidence
- > 0.08 → Fail (stop analysis)
"""

from typing import Dict, List
from rapidfuzz.distance import Levenshtein as RLev

from ..text import normalize_arabic_words, normalize_arabic_chars


def compute_wer(reference_words: List[str], hypothesis_words: List[str]) -> tuple[float, List[Dict]]:
    """
    Compute Word Error Rate using Levenshtein distance with detailed errors.

    Uses rapidfuzz editops to extract detailed error information:
    - Substitutions (word replaced)
    - Deletions (word missing)
    - Insertions (extra word)

    Args:
        reference_words: List of reference words (normalized)
        hypothesis_words: List of hypothesis words (normalized)

    Returns:
        Tuple of (WER, errors_list)
        - WER: float (0.0 to 1.0+)
        - errors_list: List of error dictionaries with details

    Examples:
        >>> wer, errors = compute_wer(['بسم', 'الله'], ['بسم', 'الله'])
        >>> wer
        0.0
        >>> len(errors)
        0
        >>> wer, errors = compute_wer(['بسم', 'الله'], ['بسم'])
        >>> wer
        0.5
        >>> len(errors)
        1
    """
    # Get edit operations (supports sequences/lists)
    ops = RLev.editops(reference_words, hypothesis_words)

    # Count operations
    subs = sum(1 for o in ops if o.tag == "replace")
    dels = sum(1 for o in ops if o.tag == "delete")
    ins = sum(1 for o in ops if o.tag == "insert")

    # Compute WER
    denom = max(len(reference_words), 1)
    wer = (subs + dels + ins) / denom

    # Extract detailed error information
    errors = []
    for o in ops:
        ref_word = reference_words[o.src_pos] if o.tag != "insert" and o.src_pos < len(reference_words) else None
        hyp_word = hypothesis_words[o.dest_pos] if o.tag != "delete" and o.dest_pos < len(hypothesis_words) else None

        error_type = {
            "replace": "substitution",
            "delete": "deletion",
            "insert": "insertion"
        }[o.tag]

        errors.append({
            "type": error_type,
            "position": o.src_pos,
            "reference_word": ref_word,
            "recited_word": hyp_word
        })

    return float(wer), errors


def compute_cer(reference_text: str, hypothesis_text: str) -> float:
    """
    Compute Character Error Rate using Levenshtein distance.

    For short texts (≤3 words), CER is more robust than WER.
    Note: We don't extract detailed errors for CER (too granular for user feedback).

    Args:
        reference_text: Reference text (normalized, no spaces)
        hypothesis_text: Hypothesis text (normalized, no spaces)

    Returns:
        CER (0.0 to 1.0+)

    Examples:
        >>> compute_cer('بسمالله', 'بسمالله')
        0.0
        >>> compute_cer('بسمالله', 'بسم')
        0.5
    """
    ops = RLev.editops(list(reference_text), list(hypothesis_text))
    subs = sum(1 for o in ops if o.tag == "replace")
    dels = sum(1 for o in ops if o.tag == "delete")
    ins = sum(1 for o in ops if o.tag == "insert")

    denom = max(len(reference_text), 1)
    cer = (subs + dels + ins) / denom

    return float(cer)


def select_error_metric(reference_words: List[str]) -> str:
    """
    Select appropriate error metric based on reference length.

    Short texts (≤3 words) use CER for robustness.
    Longer texts (>3 words) use WER for interpretability.

    Args:
        reference_words: Normalized reference words

    Returns:
        "cer" if ≤ 3 words, else "wer"

    Examples:
        >>> select_error_metric(['بسم', 'الله'])
        'cer'
        >>> select_error_metric(['بسم', 'الله', 'الرحمن', 'الرحيم'])
        'wer'
    """
    word_count = len(reference_words)
    return "cer" if word_count <= 3 else "wer"


def verify_content(
    reference_text: str,
    hypothesis_text: str
) -> Dict:
    """
    Verify content match using hybrid WER/CER with detailed errors.

    Pipeline:
    1. Normalize both texts (library-based normalization)
    2. Select metric (CER for ≤3 words, WER otherwise)
    3. Compute error rate with detailed operations
    4. Determine confidence level and proceed decision

    Args:
        reference_text: Ground-truth text (raw, with diacritics)
        hypothesis_text: ASR output text (raw)

    Returns:
        Dictionary with:
        - error_rate: WER or CER value
        - metric_type: "wer" or "cer"
        - confidence: "high", "medium", or "fail"
        - should_proceed: bool
        - normalized_reference: str (normalized)
        - normalized_transcript: str (normalized)
        - errors: List[Dict] (only for WER path, empty for CER)

    Examples:
        >>> result = verify_content("بِسْمِ اللَّهِ", "بسم الله")
        >>> result['error_rate']
        0.0
        >>> result['confidence']
        'high'
        >>> result['should_proceed']
        True
    """
    # 1. Normalize both texts
    ref_words = normalize_arabic_words(reference_text)
    hyp_words = normalize_arabic_words(hypothesis_text)

    # 2. Select metric
    metric_type = select_error_metric(ref_words)

    # 3. Compute error rate
    if metric_type == "cer":
        ref_chars = normalize_arabic_chars(reference_text)
        hyp_chars = normalize_arabic_chars(hypothesis_text)
        error_rate = compute_cer(ref_chars, hyp_chars)
        errors = []  # No detailed errors for CER (too granular)
        norm_reference = ref_chars
        norm_transcript = hyp_chars
    else:  # wer
        error_rate, errors = compute_wer(ref_words, hyp_words)
        norm_reference = " ".join(ref_words)
        norm_transcript = " ".join(hyp_words)

    # 4. Determine confidence level
    if error_rate <= 0.05:
        confidence = "high"
        should_proceed = True
    elif error_rate <= 0.08:
        confidence = "medium"
        should_proceed = True
    else:
        confidence = "fail"
        should_proceed = False

    return {
        "error_rate": float(error_rate),
        "metric_type": metric_type,
        "confidence": confidence,
        "should_proceed": should_proceed,
        "normalized_reference": norm_reference,
        "normalized_transcript": norm_transcript,
        "errors": errors
    }


class ContentGate:
    """
    Content verification gatekeeper with hybrid WER/CER.

    Features:
    - Hybrid metric selection (CER for short, WER for long)
    - Library-based Arabic normalization (PyArabic + fallback)
    - Detailed error information (word-level for WER)
    - Confidence-based proceed decision

    Examples:
        >>> gate = ContentGate()
        >>> result = gate.verify("بِسْمِ اللَّهِ", "بسم الله")
        >>> result['confidence']
        'high'
    """

    # Thresholds (class constants)
    THRESHOLD_HIGH = 0.05
    THRESHOLD_MEDIUM = 0.08

    def __init__(self):
        """Initialize content gate."""
        pass

    def verify(
        self,
        reference_text: str,
        hypothesis_text: str
    ) -> Dict:
        """
        Verify content and determine if analysis should proceed.

        Args:
            reference_text: Ground-truth text
            hypothesis_text: ASR output

        Returns:
            Verification result dictionary with detailed errors

        Examples:
            >>> gate = ContentGate()
            >>> result = gate.verify("بِسْمِ اللَّهِ", "بسم الله")
            >>> result['should_proceed']
            True
        """
        return verify_content(reference_text, hypothesis_text)

    @staticmethod
    def select_metric(reference_text: str) -> str:
        """
        Determine which metric to use for given reference text.

        Args:
            reference_text: Reference text (raw or normalized)

        Returns:
            "cer" or "wer"

        Examples:
            >>> ContentGate.select_metric("بسم الله")
            'cer'
            >>> ContentGate.select_metric("بسم الله الرحمن الرحيم")
            'wer'
        """
        ref_words = normalize_arabic_words(reference_text)
        return select_error_metric(ref_words)
