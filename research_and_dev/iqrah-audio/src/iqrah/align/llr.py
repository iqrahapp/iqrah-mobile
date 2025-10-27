"""
Log-Likelihood Ratio (LLR) Confidence Scoring

LLR provides a discriminative confidence measure without requiring phoneme labels.

Formula:
    LLR = mean_over_time[log P(token) - max_other[log P(other_token)]]

Interpretation:
- LLR > 2.0: Very confident
- LLR 1.0-2.0: Confident
- LLR 0.5-1.0: Moderate
- LLR < 0.5: Low confidence (potential mispronunciation)
"""

from typing import List, Dict
import numpy as np


def compute_llr(
    logits: np.ndarray,
    token_id: int,
    start_frame: int,
    end_frame: int
) -> float:
    """
    Compute Log-Likelihood Ratio for a token segment.

    Args:
        logits: CTC logits (time_steps, vocab_size)
        token_id: Target token ID
        start_frame: Start frame index
        end_frame: End frame index

    Returns:
        LLR score (higher = more confident)

    Examples:
        >>> logits = np.random.randn(100, 50)
        >>> llr = compute_llr(logits, token_id=5, start_frame=10, end_frame=20)
        >>> isinstance(llr, float)
        True
    """
    # Extract segment
    segment_logits = logits[start_frame:end_frame, :]

    if len(segment_logits) == 0:
        return -10.0  # Invalid segment

    # Convert to posteriors
    posteriors = np.exp(segment_logits - np.max(segment_logits, axis=-1, keepdims=True))
    posteriors = posteriors / np.sum(posteriors, axis=-1, keepdims=True)

    # Get target token log probabilities
    target_logprob = np.log(posteriors[:, token_id] + 1e-10)

    # For each time step, get max prob of non-target tokens
    mask = np.ones(posteriors.shape[-1], dtype=bool)
    mask[token_id] = False
    other_max_logprob = np.log(posteriors[:, mask].max(axis=-1) + 1e-10)

    # LLR is the mean difference
    llr = (target_logprob - other_max_logprob).mean()

    return float(llr)


def add_llr_scores(
    aligned_tokens: List[Dict],
    logits: np.ndarray,
    vocab: Dict[str, int],
    frame_rate: float = 50.0
) -> List[Dict]:
    """
    Add LLR scores to aligned tokens.

    Args:
        aligned_tokens: Output from CTCAligner.align()
        logits: CTC logits from ASR model
        vocab: Token to ID mapping
        frame_rate: Frame rate in Hz

    Returns:
        aligned_tokens with gop_score field added

    Examples:
        >>> tokens = [{"token": "ا", "start": 0.0, "end": 0.1, "confidence": 0.8}]
        >>> logits = np.random.randn(100, 50)
        >>> vocab = {'ا': 0}
        >>> result = add_llr_scores(tokens, logits, vocab)
        >>> 'gop_score' in result[0]
        True
    """
    for token in aligned_tokens:
        # Get token ID
        char = token["token"]
        if char not in vocab:
            token["gop_score"] = -10.0  # Unknown token
            continue

        token_id = vocab[char]

        # Convert time to frames
        start_frame = int(token["start"] * frame_rate)
        end_frame = int(token["end"] * frame_rate)

        # Compute LLR
        llr = compute_llr(logits, token_id, start_frame, end_frame)
        token["gop_score"] = llr

    return aligned_tokens


def interpret_llr(llr: float) -> str:
    """
    Interpret LLR score as confidence level.

    Args:
        llr: LLR score

    Returns:
        Confidence level string

    Examples:
        >>> interpret_llr(2.5)
        'very_confident'
        >>> interpret_llr(0.3)
        'low_confidence'
    """
    if llr > 2.0:
        return "very_confident"
    elif llr > 1.0:
        return "confident"
    elif llr > 0.5:
        return "moderate"
    else:
        return "low_confidence"
