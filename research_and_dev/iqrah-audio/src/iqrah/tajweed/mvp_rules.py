"""
Tajweed MVP Rules: Madd, Shadda, Waqf

High-precision rules using duration and energy analysis only.
Phonetically complex rules (Ghunnah, Qalqalah, etc.) are deferred.
"""

from typing import List, Dict, Optional
import numpy as np
import librosa


# Madd (long vowels) detection
MADD_VOWELS = {'ا', 'و', 'ي'}


def validate_madd(
    aligned_tokens: List[Dict],
    min_duration_ms: float = 200.0
) -> List[Dict]:
    """
    Validate Madd (elongation) rules.

    Detection: Orthographic (long vowels: ا و ي)
    Validation: Duration must meet minimum threshold

    Args:
        aligned_tokens: Aligned tokens from CTC aligner
        min_duration_ms: Minimum duration for Madd in milliseconds

    Returns:
        List of violations:
        [
            {
                "token_index": int,
                "token": str,
                "expected_duration_ms": float,
                "actual_duration_ms": float,
                "severity": "minor" | "moderate" | "critical"
            }
        ]

    Examples:
        >>> tokens = [{"token": "ا", "start": 0.0, "end": 0.15, "confidence": 0.8}]
        >>> violations = validate_madd(tokens, min_duration_ms=200.0)
        >>> len(violations)
        1
    """
    violations = []

    for i, token in enumerate(aligned_tokens):
        char = token["token"]

        # Check if this is a Madd vowel
        if char not in MADD_VOWELS:
            continue

        # Calculate duration
        duration_ms = (token["end"] - token["start"]) * 1000

        if duration_ms < min_duration_ms:
            # Determine severity based on how short it is
            ratio = duration_ms / min_duration_ms
            if ratio < 0.5:
                severity = "critical"
            elif ratio < 0.75:
                severity = "moderate"
            else:
                severity = "minor"

            violations.append({
                "token_index": i,
                "token": char,
                "expected_duration_ms": min_duration_ms,
                "actual_duration_ms": duration_ms,
                "severity": severity,
                "message": f"Madd '{char}' too short: {duration_ms:.0f}ms < {min_duration_ms:.0f}ms"
            })

    return violations


def validate_shadda(
    aligned_tokens: List[Dict],
    duration_multiplier: float = 1.6
) -> List[Dict]:
    """
    Validate Shadda (gemination) rules.

    Detection: Doubled consonants in sequence
    Validation: Duration must be ≥ multiplier × median duration for that consonant

    Args:
        aligned_tokens: Aligned tokens from CTC aligner
        duration_multiplier: Required duration multiplier (default 1.6×)

    Returns:
        List of violations

    Examples:
        >>> tokens = [
        ...     {"token": "ل", "start": 0.0, "end": 0.05, "confidence": 0.8},
        ...     {"token": "ل", "start": 0.05, "end": 0.08, "confidence": 0.8}
        ... ]
        >>> violations = validate_shadda(tokens)
        >>> isinstance(violations, list)
        True
    """
    violations = []

    # Build consonant duration statistics
    consonant_durations = {}
    for token in aligned_tokens:
        char = token["token"]
        duration = token["end"] - token["start"]

        if char not in consonant_durations:
            consonant_durations[char] = []
        consonant_durations[char].append(duration)

    # Compute median durations
    consonant_medians = {
        char: np.median(durations)
        for char, durations in consonant_durations.items()
    }

    # Detect doubled consonants (simple heuristic: consecutive same char)
    for i in range(len(aligned_tokens) - 1):
        curr_token = aligned_tokens[i]
        next_token = aligned_tokens[i + 1]

        if curr_token["token"] == next_token["token"]:
            # This is a potential Shadda (doubled consonant)
            char = curr_token["token"]
            combined_duration = (next_token["end"] - curr_token["start"])

            # Get expected duration
            if char in consonant_medians:
                expected_duration = consonant_medians[char] * duration_multiplier

                if combined_duration < expected_duration:
                    ratio = combined_duration / expected_duration
                    if ratio < 0.7:
                        severity = "critical"
                    elif ratio < 0.85:
                        severity = "moderate"
                    else:
                        severity = "minor"

                    violations.append({
                        "token_index": i,
                        "token": char,
                        "expected_duration_ms": expected_duration * 1000,
                        "actual_duration_ms": combined_duration * 1000,
                        "severity": severity,
                        "message": f"Shadda '{char}' too short: {combined_duration*1000:.0f}ms < {expected_duration*1000:.0f}ms"
                    })

    return violations


def validate_waqf(
    aligned_tokens: List[Dict],
    audio: np.ndarray,
    sample_rate: int = 16000,
    energy_threshold: float = 0.3,
    window_ms: float = 300.0
) -> List[Dict]:
    """
    Validate Waqf (pause at end of Ayah).

    Detection: Final aligned token
    Validation: Energy drop (RMS) after final token

    Args:
        aligned_tokens: Aligned tokens from CTC aligner
        audio: Audio array
        sample_rate: Sample rate in Hz
        energy_threshold: RMS must drop to ≤ this fraction of mean RMS
        window_ms: Time window after final token to check (ms)

    Returns:
        List of violations

    Examples:
        >>> tokens = [{"token": "م", "start": 0.0, "end": 0.1, "confidence": 0.8}]
        >>> audio = np.random.randn(16000)
        >>> violations = validate_waqf(tokens, audio)
        >>> isinstance(violations, list)
        True
    """
    violations = []

    if not aligned_tokens:
        return violations

    final_token = aligned_tokens[-1]
    final_end_time = final_token["end"]

    # Extract audio after final token
    final_end_sample = int(final_end_time * sample_rate)
    window_samples = int(window_ms / 1000 * sample_rate)

    after_segment_end = min(final_end_sample + window_samples, len(audio))
    after_segment = audio[final_end_sample:after_segment_end]

    if len(after_segment) < sample_rate // 10:  # Less than 100ms
        # Not enough audio to check
        return violations

    # Compute RMS of full audio
    full_rms = librosa.feature.rms(y=audio, frame_length=2048, hop_length=512)[0]
    mean_rms = np.mean(full_rms)

    # Compute RMS of after-segment
    after_rms = librosa.feature.rms(y=after_segment, frame_length=2048, hop_length=512)[0]
    after_mean_rms = np.mean(after_rms)

    # Check if energy dropped sufficiently
    if after_mean_rms > energy_threshold * mean_rms:
        violations.append({
            "token_index": len(aligned_tokens) - 1,
            "token": final_token["token"],
            "expected_energy_drop": f"≤ {energy_threshold*100:.0f}% of mean RMS",
            "actual_energy_ratio": after_mean_rms / mean_rms if mean_rms > 0 else 1.0,
            "severity": "moderate",
            "message": f"Missing Waqf: no energy drop after final token ({after_mean_rms/mean_rms*100:.0f}% of mean RMS)"
        })

    return violations


class TajweedValidatorMVP:
    """
    MVP Tajweed validator: Duration + Energy rules only.

    Rules implemented:
    - Madd (elongation)
    - Shadda (gemination)
    - Waqf (pause at end)

    Rules deferred:
    - Ghunnah, Qalqalah, Ikhfa', Idgham, Iqlab (require phonetic analysis)
    """

    def __init__(
        self,
        madd_min_duration_ms: float = 200.0,
        shadda_duration_multiplier: float = 1.6,
        waqf_energy_threshold: float = 0.3
    ):
        """
        Initialize Tajweed validator.

        Args:
            madd_min_duration_ms: Minimum duration for Madd
            shadda_duration_multiplier: Duration multiplier for Shadda
            waqf_energy_threshold: Energy threshold for Waqf
        """
        self.madd_min_duration_ms = madd_min_duration_ms
        self.shadda_multiplier = shadda_duration_multiplier
        self.waqf_threshold = waqf_energy_threshold

    def validate(
        self,
        aligned_tokens: List[Dict],
        audio: Optional[np.ndarray] = None,
        sample_rate: int = 16000
    ) -> Dict:
        """
        Validate Tajweed rules for aligned tokens.

        Args:
            aligned_tokens: Output from CTCAligner.align()
            audio: Audio array (required for Waqf validation)
            sample_rate: Sample rate

        Returns:
            {
                "madd_violations": List[Dict],
                "shadda_violations": List[Dict],
                "waqf_violations": List[Dict],
                "overall_score": float  # 0-100
            }
        """
        # Validate each rule
        madd_viol = validate_madd(aligned_tokens, self.madd_min_duration_ms)
        shadda_viol = validate_shadda(aligned_tokens, self.shadda_multiplier)

        waqf_viol = []
        if audio is not None:
            waqf_viol = validate_waqf(
                aligned_tokens,
                audio,
                sample_rate,
                self.waqf_threshold
            )

        # Compute overall score
        total_checks = len(madd_viol) + len(shadda_viol) + len(waqf_viol)

        if total_checks == 0:
            overall_score = 100.0
        else:
            # Count violations
            num_violations = len(madd_viol) + len(shadda_viol) + len(waqf_viol)
            overall_score = max(0.0, 100.0 * (1 - num_violations / max(total_checks, 1)))

        return {
            "madd_violations": madd_viol,
            "shadda_violations": shadda_viol,
            "waqf_violations": waqf_viol,
            "overall_score": float(overall_score),
            "total_violations": len(madd_viol) + len(shadda_viol) + len(waqf_viol)
        }
