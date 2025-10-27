"""
Phonetic Content Verification Gatekeeper (M3.5)

Implements Phoneme Error Rate (PER) based content verification.
This replaces grapheme-based WER/CER with phoneme-level comparison.

Why PER instead of WER/CER:
- Muaalem outputs phonemes, not graphemes
- Phonetic comparison is more accurate for recitation
- Aligns with phonetic-first architecture

Thresholds:
- PER ≤ 0.02 → High confidence (proceed)
- PER ≤ 0.05 → Medium confidence (proceed with warning)
- PER > 0.05 → Fail (stop analysis)
"""

from typing import Dict, List
from dataclasses import dataclass
from rapidfuzz.distance import Levenshtein as RLev


@dataclass
class PhoneticError:
    """
    Single phonetic error with detailed information.

    Attributes:
        type: Error type ("substitution", "deletion", "insertion")
        position: Position in reference sequence
        reference_phoneme: Expected phoneme (None for insertion)
        predicted_phoneme: Predicted phoneme (None for deletion)
    """
    type: str
    position: int
    reference_phoneme: str | None
    predicted_phoneme: str | None


def compute_per(
    reference_phonemes: List[str],
    predicted_phonemes: List[str]
) -> tuple[float, List[PhoneticError]]:
    """
    Compute Phoneme Error Rate using Levenshtein distance.

    PER = (Substitutions + Deletions + Insertions) / Total Reference Phonemes

    Args:
        reference_phonemes: Expected phoneme sequence
        predicted_phonemes: ASR predicted phoneme sequence

    Returns:
        Tuple of (PER, errors_list)
        - PER: float (0.0 to 1.0+, can exceed 1.0 if many insertions)
        - errors_list: List of PhoneticError objects with details

    Examples:
        >>> per, errors = compute_per(['b', 'i', 's', 'm'], ['b', 'i', 's', 'm'])
        >>> per
        0.0
        >>> len(errors)
        0
        >>> per, errors = compute_per(['b', 'i', 's', 'm'], ['b', 'a', 's', 'm'])
        >>> per
        0.25
        >>> errors[0].type
        'substitution'
    """
    # Handle empty sequences
    if len(reference_phonemes) == 0 and len(predicted_phonemes) == 0:
        return 0.0, []

    if len(reference_phonemes) == 0:
        # All insertions
        errors = [
            PhoneticError(
                type="insertion",
                position=i,
                reference_phoneme=None,
                predicted_phoneme=predicted_phonemes[i]
            )
            for i in range(len(predicted_phonemes))
        ]
        return float('inf'), errors

    # Get edit operations
    ops = RLev.editops(reference_phonemes, predicted_phonemes)

    # Count operations
    subs = sum(1 for o in ops if o.tag == "replace")
    dels = sum(1 for o in ops if o.tag == "delete")
    ins = sum(1 for o in ops if o.tag == "insert")

    # Compute PER
    denom = max(len(reference_phonemes), 1)
    per = (subs + dels + ins) / denom

    # Extract detailed error information
    errors: List[PhoneticError] = []

    for o in ops:
        ref_phoneme = (
            reference_phonemes[o.src_pos]
            if o.tag != "insert" and o.src_pos < len(reference_phonemes)
            else None
        )
        pred_phoneme = (
            predicted_phonemes[o.dest_pos]
            if o.tag != "delete" and o.dest_pos < len(predicted_phonemes)
            else None
        )

        error_type = {
            "replace": "substitution",
            "delete": "deletion",
            "insert": "insertion"
        }[o.tag]

        errors.append(PhoneticError(
            type=error_type,
            position=o.src_pos,
            reference_phoneme=ref_phoneme,
            predicted_phoneme=pred_phoneme
        ))

    return float(per), errors


def verify_phonetic_content(
    reference_phonemes: List[str],
    predicted_phonemes: List[str]
) -> Dict:
    """
    Verify content match using Phoneme Error Rate (PER).

    Pipeline:
    1. Compute PER using Levenshtein distance on phoneme sequences
    2. Extract detailed error information
    3. Determine confidence level and proceed decision

    Args:
        reference_phonemes: Expected phoneme sequence from phonetizer
        predicted_phonemes: ASR predicted phoneme sequence from Muaalem

    Returns:
        Dictionary with:
        - per: Phoneme Error Rate (0.0 to 1.0+)
        - confidence: "high", "medium", or "fail"
        - should_proceed: bool (whether to continue analysis)
        - errors: List[PhoneticError] (detailed error information)
        - num_substitutions: int
        - num_deletions: int
        - num_insertions: int
        - total_errors: int

    Examples:
        >>> result = verify_phonetic_content(
        ...     ['b', 'i', 's', 'm'],
        ...     ['b', 'i', 's', 'm']
        ... )
        >>> result['per']
        0.0
        >>> result['confidence']
        'high'
        >>> result['should_proceed']
        True
    """
    # Compute PER
    per, errors = compute_per(reference_phonemes, predicted_phonemes)

    # Count error types
    num_subs = sum(1 for e in errors if e.type == "substitution")
    num_dels = sum(1 for e in errors if e.type == "deletion")
    num_ins = sum(1 for e in errors if e.type == "insertion")

    # Determine confidence level
    if per <= 0.02:
        confidence = "high"
        should_proceed = True
    elif per <= 0.05:
        confidence = "medium"
        should_proceed = True
    else:
        confidence = "fail"
        should_proceed = False

    return {
        "per": float(per),
        "confidence": confidence,
        "should_proceed": should_proceed,
        "errors": errors,
        "num_substitutions": num_subs,
        "num_deletions": num_dels,
        "num_insertions": num_ins,
        "total_errors": len(errors)
    }


class PhoneticGatekeeper:
    """
    Phonetic content verification gatekeeper using PER.

    This replaces the grapheme-based ContentGate with phoneme-level verification.
    Designed for use with Muaalem ASR output.

    Features:
    - Phoneme Error Rate (PER) instead of WER/CER
    - Detailed phoneme-level error tracking
    - Confidence-based proceed decision
    - Compatible with Muaalem phoneme sequences

    Thresholds:
    - PER ≤ 0.02 (2%) → High confidence
    - PER ≤ 0.05 (5%) → Medium confidence
    - PER > 0.05 (5%) → Fail

    Examples:
        >>> gate = PhoneticGatekeeper()
        >>> result = gate.verify(
        ...     reference_phonemes=['b', 'i', 's', 'm'],
        ...     predicted_phonemes=['b', 'i', 's', 'm']
        ... )
        >>> result['confidence']
        'high'
        >>> result['should_proceed']
        True
    """

    # Thresholds (class constants)
    THRESHOLD_HIGH = 0.02  # 2% PER
    THRESHOLD_MEDIUM = 0.05  # 5% PER

    def __init__(
        self,
        threshold_high: float = 0.02,
        threshold_medium: float = 0.05
    ):
        """
        Initialize phonetic gatekeeper with custom thresholds.

        Args:
            threshold_high: PER threshold for high confidence (default: 0.02)
            threshold_medium: PER threshold for medium confidence (default: 0.05)

        Examples:
            >>> gate = PhoneticGatekeeper()  # Default thresholds
            >>> gate = PhoneticGatekeeper(threshold_high=0.01, threshold_medium=0.03)
        """
        self.threshold_high = threshold_high
        self.threshold_medium = threshold_medium

    def verify(
        self,
        reference_phonemes: List[str],
        predicted_phonemes: List[str]
    ) -> Dict:
        """
        Verify content and determine if analysis should proceed.

        Args:
            reference_phonemes: Expected phoneme sequence
            predicted_phonemes: ASR predicted phoneme sequence

        Returns:
            Verification result dictionary with detailed errors

        Examples:
            >>> gate = PhoneticGatekeeper()
            >>> result = gate.verify(['b', 'i'], ['b', 'i'])
            >>> result['should_proceed']
            True
            >>> result['per']
            0.0
        """
        result = verify_phonetic_content(reference_phonemes, predicted_phonemes)

        # Override confidence thresholds if custom ones provided
        per = result['per']
        if per <= self.threshold_high:
            result['confidence'] = "high"
            result['should_proceed'] = True
        elif per <= self.threshold_medium:
            result['confidence'] = "medium"
            result['should_proceed'] = True
        else:
            result['confidence'] = "fail"
            result['should_proceed'] = False

        return result

    def verify_from_text(
        self,
        reference_text: str,
        predicted_text: str
    ) -> Dict:
        """
        Verify content from phonetic text strings.

        Convenience method that splits text into phoneme lists.

        Args:
            reference_text: Expected phonetic string (e.g., "bismillaah")
            predicted_text: Predicted phonetic string

        Returns:
            Verification result dictionary

        Examples:
            >>> gate = PhoneticGatekeeper()
            >>> result = gate.verify_from_text("bism", "bism")
            >>> result['per']
            0.0
        """
        # Split into character-level phonemes
        # Note: Real phonetic strings may have multi-char phonemes
        # For now, we treat each character as a phoneme
        ref_phonemes = list(reference_text)
        pred_phonemes = list(predicted_text)

        return self.verify(ref_phonemes, pred_phonemes)

    @staticmethod
    def phoneme_sequence_from_muaalem_output(muaalem_output) -> List[str]:
        """
        Extract phoneme sequence from Muaalem output.

        Utility method to convert Muaalem Unit to phoneme list.

        Args:
            muaalem_output: MuaalemOutput or MuaalemInferenceOutput

        Returns:
            List of phonemes as strings

        Examples:
            >>> phonemes = PhoneticGatekeeper.phoneme_sequence_from_muaalem_output(result)
            >>> phonemes
            ['b', 'i', 's', 'm', 'i', 'l', 'l', 'a', 'a', 'h']
        """
        # Extract phoneme text from Unit
        phoneme_text = muaalem_output.phonemes.text

        # Split into individual phonemes
        # Note: Muaalem uses concatenated phonetic string
        # We split character by character (adjust if multi-char phonemes used)
        return list(phoneme_text)

    @staticmethod
    def phoneme_sequence_from_phonetic_ref(phonetic_ref) -> List[str]:
        """
        Extract phoneme sequence from phonetic reference.

        Utility method to convert IqrahPhoneticOutput to phoneme list.

        Args:
            phonetic_ref: IqrahPhoneticOutput from phonetizer

        Returns:
            List of phonemes as strings

        Examples:
            >>> phonemes = PhoneticGatekeeper.phoneme_sequence_from_phonetic_ref(ref)
            >>> phonemes
            ['b', 'i', 's', 'm']
        """
        # Extract phoneme text
        phoneme_text = phonetic_ref.text

        # Split into individual phonemes
        return list(phoneme_text)
