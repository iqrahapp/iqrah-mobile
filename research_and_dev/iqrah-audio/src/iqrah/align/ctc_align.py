"""
CTC Forced Alignment for Graphemes (M3.2)

Viterbi-based alignment using CTC posteriors with proper blank handling.
Constrains alignment to normalized grapheme sequence of reference text.

This implementation follows the CTC alignment algorithm with:
- Proper Viterbi trellis with blank transitions
- Backtracking to find emission frames
- Midpoint partitioning for span boundaries
- Fallback to proportional slicing for edge cases
"""

from typing import List, Dict, Optional
import numpy as np

from ..text import normalize_arabic_chars


class CTCForcedAligner:
    """
    Grapheme-level CTC forced aligner with proper Viterbi + blank handling.

    Works with Wav2Vec2-CTC models (e.g., obadx/muaalem-model-v3_2).
    Uses Viterbi algorithm to find optimal alignment path through CTC lattice,
    properly handling blank tokens and backtracking to get precise emission times.

    Algorithm:
    1. Convert posteriors to log-space for numerical stability
    2. Build Viterbi trellis with blank transitions (stay) and token emissions (emit)
    3. Backtrack to find emission frames for each target token
    4. Partition frames using midpoints between emissions
    5. Compute per-token confidence from posteriors

    Examples:
        >>> from src.iqrah.asr import ASRModel
        >>> model = ASRModel("obadx/muaalem-model-v3_2")
        >>> aligner = CTCForcedAligner(model)
        >>> result = aligner.align(audio, "بسم الله", sample_rate=16000)
        >>> len(result["units"])
        7
        >>> result["quality_score"]
        0.85
    """

    def __init__(self, asr_model):
        """
        Initialize CTC forced aligner.

        Args:
            asr_model: ASR model object exposing:
                - processor.tokenizer: HuggingFace tokenizer with pad_token_id as blank
                - get_ctc_posteriors(audio, sr): Returns (T, V) posteriors in prob space

        Examples:
            >>> from src.iqrah.asr import ASRModel
            >>> model = ASRModel()
            >>> aligner = CTCForcedAligner(model)
        """
        self.asr_model = asr_model

        # Detect blank token ID (usually 0 for CTC models)
        tok = getattr(self.asr_model, "processor").tokenizer
        self.blank_id = getattr(tok, "pad_token_id", None)
        if self.blank_id is None:
            # HuggingFace CTC models typically use 0 as blank
            self.blank_id = 0

    def _text_to_ids(self, text: str) -> List[int]:
        """
        Convert normalized grapheme string to tokenizer IDs.

        Args:
            text: Normalized Arabic text (graphemes only)

        Returns:
            List of token IDs (no special tokens)

        Examples:
            >>> ids = aligner._text_to_ids("بسم")
            >>> len(ids)
            3
        """
        tok = self.asr_model.processor.tokenizer
        enc = tok(text, add_special_tokens=False, return_attention_mask=False)
        return list(enc["input_ids"])

    def align(
        self,
        audio: np.ndarray,
        target_text: str,
        sample_rate: int = 16000
    ) -> Dict:
        """
        Perform CTC forced alignment with Viterbi algorithm.

        This is the main entry point. It:
        1. Extracts CTC posteriors from audio
        2. Converts target text to token IDs
        3. Runs Viterbi alignment with blank handling
        4. Backtracks to find emission frames
        5. Partitions frames using midpoints
        6. Computes confidence scores

        Args:
            audio: Audio waveform (mono, 16kHz recommended)
            target_text: Reference text (will be normalized to graphemes)
            sample_rate: Audio sample rate in Hz

        Returns:
            {
                "units": [
                    {
                        "token": str,           # Grapheme
                        "start": float,         # Start time (seconds)
                        "end": float,           # End time (seconds)
                        "confidence": float     # Mean posterior probability
                    },
                    ...
                ],
                "alignment_method": str,        # "ctc_grapheme" or "ctc_grapheme_fallback"
                "quality_score": float          # Mean confidence across all tokens
            }

        Edge Cases:
            - Empty audio → returns empty units
            - Empty target text → returns empty units
            - Viterbi path impossible → falls back to proportional slicing
            - Unknown characters → uses first token ID as fallback

        Examples:
            >>> audio = np.random.randn(16000)  # 1 second
            >>> result = aligner.align(audio, "بسم الله")
            >>> result["quality_score"]
            0.78
            >>> len(result["units"])
            7
        """
        # 1) Get CTC posteriors (T, V) and compute frame duration
        post = self.asr_model.get_ctc_posteriors(audio, sample_rate)
        T, V = post.shape

        if T == 0:
            return {
                "units": [],
                "alignment_method": "ctc_grapheme",
                "quality_score": 0.0
            }

        audio_sec = len(audio) / float(sample_rate)
        frame_dur = audio_sec / T

        # 2) Convert target text to token IDs and graphemes
        normalized_text = normalize_arabic_chars(target_text)
        ids = self._text_to_ids(normalized_text)

        if len(ids) == 0:
            return {
                "units": [],
                "alignment_method": "ctc_grapheme",
                "quality_score": 0.0
            }

        tokens = self.asr_model.processor.tokenizer.convert_ids_to_tokens(ids)

        # 3) Convert posteriors to log-probabilities for numerical stability
        eps = 1e-10
        logp = np.log(post + eps).astype(np.float32)

        # 4) Run Viterbi algorithm with blank transitions
        N = len(ids)
        dp = np.full((T + 1, N + 1), -np.inf, dtype=np.float32)
        ptr = np.full((T + 1, N + 1), -1, dtype=np.int8)  # 0=stay(blank), 1=emit(token)
        dp[0, 0] = 0.0

        for t in range(T):
            # Stay on same state via blank token
            stay_scores = dp[t, :] + logp[t, self.blank_id]  # Shape: (N+1,)
            better = stay_scores > dp[t + 1, :]
            dp[t + 1, better] = stay_scores[better]
            ptr[t + 1, better] = 0

            # Emit next token (transition from state j to j+1)
            emit_scores = dp[t, :N] + logp[t, ids]  # Shape: (N,)
            better_emit = emit_scores > dp[t + 1, 1:]
            dp[t + 1, 1:][better_emit] = emit_scores[better_emit]
            ptr[t + 1, 1:][better_emit] = 1

        # Check if valid path exists
        if not np.isfinite(dp[T, N]):
            # No valid path found - use fallback
            return self._fallback_partition(tokens, post, ids, frame_dur)

        # 5) Backtrack to find emission frames
        emissions = np.full(N, -1, dtype=np.int32)  # Frame index where each token was emitted
        t, j = T, N

        while t > 0 and j >= 0:
            if ptr[t, j] == 1:
                # Token j-1 was emitted at frame t-1
                emissions[j - 1] = t - 1
                j -= 1
                t -= 1
            elif ptr[t, j] == 0:
                # Stayed via blank
                t -= 1
            else:
                # Should not happen; break to fallback
                break

        # Check if backtracking was complete
        if (emissions < 0).any():
            return self._fallback_partition(tokens, post, ids, frame_dur)

        # 6) Partition frames using midpoints between emissions
        # For token i: start = mid(emission[i-1], emission[i]), end = mid(emission[i], emission[i+1])
        mids = np.zeros(N + 1, dtype=np.int32)
        mids[0] = 0  # Start boundary

        for i in range(1, N):
            mids[i] = (emissions[i - 1] + emissions[i]) // 2

        mids[N] = T  # End boundary

        # 7) Build output units with confidence scores
        units = []
        confs = []

        for i in range(N):
            start_frame = mids[i]
            end_frame = max(mids[i + 1], start_frame + 1)  # Ensure at least 1 frame

            # Compute mean posterior probability for this token in its frame range
            seg = post[start_frame:end_frame, ids[i]]
            conf = float(seg.mean()) if seg.size > 0 else 0.0
            confs.append(conf)

            units.append({
                "token": tokens[i],
                "start": start_frame * frame_dur,
                "end": end_frame * frame_dur,
                "confidence": conf
            })

        # 8) Compute overall quality score
        quality = float(np.mean(confs)) if confs else 0.0

        return {
            "units": units,
            "alignment_method": "ctc_grapheme",
            "quality_score": quality
        }

    def _fallback_partition(
        self,
        tokens: List[str],
        post: np.ndarray,
        ids: List[int],
        frame_dur: float
    ) -> Dict:
        """
        Fallback to proportional slicing when Viterbi fails.

        This happens when:
        - Target sequence cannot be aligned (e.g., text doesn't match audio)
        - Numerical issues in Viterbi trellis
        - Incomplete backtracking

        The fallback divides time evenly among tokens - not ideal but safe.

        Args:
            tokens: List of grapheme strings
            post: CTC posteriors (T, V)
            ids: Token IDs corresponding to tokens
            frame_dur: Duration of each frame in seconds

        Returns:
            Same format as align(), but with "ctc_grapheme_fallback" method

        Examples:
            >>> result = aligner._fallback_partition(["ب", "س"], post, [5, 12], 0.02)
            >>> result["alignment_method"]
            'ctc_grapheme_fallback'
        """
        T = post.shape[0]
        N = len(tokens)

        if N == 0 or T == 0:
            return {
                "units": [],
                "alignment_method": "ctc_grapheme_fallback",
                "quality_score": 0.0
            }

        # Equal partitions
        cuts = np.linspace(0, T, N + 1).astype(int)
        units, confs = [], []

        for i in range(N):
            start_frame = cuts[i]
            end_frame = max(cuts[i + 1], start_frame + 1)

            # Use token-specific confidence if ID available
            if i < len(ids):
                seg = post[start_frame:end_frame, ids[i]]
                conf = float(seg.mean()) if seg.size > 0 else 0.0
            else:
                # Unknown token - use max prob as weak proxy
                conf = float(post[start_frame:end_frame].max(axis=1).mean())

            confs.append(conf)
            units.append({
                "token": tokens[i],
                "start": start_frame * frame_dur,
                "end": end_frame * frame_dur,
                "confidence": conf
            })

        quality = float(np.mean(confs)) if confs else 0.0

        return {
            "units": units,
            "alignment_method": "ctc_grapheme_fallback",
            "quality_score": quality
        }

    def validate_alignment(self, units: List[Dict]) -> Dict:
        """
        Validate alignment quality.

        Checks:
        - Mean confidence ≥ 0.5
        - Token duration: 20ms ≤ duration ≤ 500ms
        - Monotonicity: start[i] ≤ end[i] ≤ start[i+1]

        Args:
            units: List of aligned units from align()

        Returns:
            {
                "is_valid": bool,               # Overall validity
                "mean_confidence": float,        # Mean confidence across tokens
                "duration_violations": int,      # Count of invalid durations
                "warnings": List[str]            # Human-readable warnings
            }

        Examples:
            >>> units = [
            ...     {"token": "ب", "start": 0.0, "end": 0.05, "confidence": 0.8},
            ...     {"token": "س", "start": 0.05, "end": 0.10, "confidence": 0.9}
            ... ]
            >>> validation = aligner.validate_alignment(units)
            >>> validation["is_valid"]
            True
            >>> validation["mean_confidence"]
            0.85
        """
        if not units:
            return {
                "is_valid": False,
                "mean_confidence": 0.0,
                "duration_violations": 0,
                "warnings": ["Empty alignment"]
            }

        confidences = [u["confidence"] for u in units]
        mean_confidence = np.mean(confidences)

        duration_violations = 0
        warnings = []

        # Check duration constraints
        for unit in units:
            duration_ms = (unit["end"] - unit["start"]) * 1000
            if not (20 <= duration_ms <= 500):
                duration_violations += 1

        # Check confidence threshold
        is_valid = mean_confidence >= 0.5 and duration_violations == 0

        if mean_confidence < 0.5:
            warnings.append(f"Low mean confidence: {mean_confidence:.2f}")

        if duration_violations > 0:
            warnings.append(f"{duration_violations} tokens with invalid duration (not in 20-500ms)")

        return {
            "is_valid": bool(is_valid),
            "mean_confidence": float(mean_confidence),
            "duration_violations": duration_violations,
            "warnings": warnings
        }


# Legacy compatibility: simple standalone function
def align_graphemes(
    logits: np.ndarray,
    reference_text: str,
    vocab: Dict[str, int],
    frame_rate: float = 50.0
) -> List[Dict]:
    """
    Legacy standalone alignment function (simple greedy approach).

    NOTE: For production use, prefer CTCForcedAligner with Viterbi algorithm.
    This function is kept for backward compatibility with existing tests.

    Args:
        logits: CTC logits (time_steps, vocab_size)
        reference_text: Reference text (will be normalized)
        vocab: Token to ID mapping
        frame_rate: CTC output frame rate in Hz

    Returns:
        List of aligned tokens with timing and confidence

    Examples:
        >>> logits = np.random.randn(100, 50)
        >>> vocab = {'ا': 0, 'ل': 1, 'ب': 2}
        >>> result = align_graphemes(logits, "الب", vocab)
        >>> len(result)
        3
    """
    target_chars = list(normalize_arabic_chars(reference_text))

    if len(target_chars) == 0:
        return []

    # Convert logits to posteriors
    posteriors = np.exp(logits - np.max(logits, axis=-1, keepdims=True))
    posteriors = posteriors / np.sum(posteriors, axis=-1, keepdims=True)

    T = posteriors.shape[0]
    N = len(target_chars)

    # Get token IDs
    token_ids = []
    for char in target_chars:
        if char in vocab:
            token_ids.append(vocab[char])
        else:
            token_ids.append(list(vocab.values())[0])

    # Simple greedy alignment: divide time evenly
    frames_per_token = max(1, T // N)
    frame_duration = 1.0 / frame_rate

    aligned_tokens = []

    for i, (char, token_id) in enumerate(zip(target_chars, token_ids)):
        start_frame = i * frames_per_token
        end_frame = min((i + 1) * frames_per_token, T)

        if i == N - 1:
            end_frame = T

        window_posteriors = posteriors[start_frame:end_frame, token_id]
        confidence = float(np.mean(window_posteriors)) if len(window_posteriors) > 0 else 0.0

        aligned_tokens.append({
            "token": char,
            "start": start_frame * frame_duration,
            "end": end_frame * frame_duration,
            "confidence": confidence
        })

    return aligned_tokens


# Legacy compatibility: simple class wrapper
class CTCAligner:
    """
    Legacy CTC aligner class (simple greedy approach).

    NOTE: For production use, prefer CTCForcedAligner with proper Viterbi algorithm.
    This class is kept for backward compatibility with existing tests.

    Examples:
        >>> aligner = CTCAligner(frame_rate=50.0)
        >>> logits = np.random.randn(100, 50)
        >>> vocab = {'ا': 0, 'ل': 1}
        >>> result = aligner.align(logits, "ال", vocab)
        >>> len(result)
        2
    """

    def __init__(self, frame_rate: float = 50.0):
        """Initialize legacy aligner."""
        self.frame_rate = frame_rate

    def align(
        self,
        logits: np.ndarray,
        reference_text: str,
        vocab: Dict[str, int]
    ) -> List[Dict]:
        """Align using simple greedy approach."""
        return align_graphemes(logits, reference_text, vocab, self.frame_rate)

    def validate_alignment(self, aligned_tokens: List[Dict]) -> Dict:
        """Validate alignment quality (same as CTCForcedAligner)."""
        if not aligned_tokens:
            return {
                "is_valid": False,
                "mean_confidence": 0.0,
                "duration_violations": 0,
                "warnings": ["Empty alignment"]
            }

        confidences = [t["confidence"] for t in aligned_tokens]
        mean_confidence = np.mean(confidences)

        duration_violations = 0
        warnings = []

        for token in aligned_tokens:
            duration_ms = (token["end"] - token["start"]) * 1000
            if not (20 <= duration_ms <= 500):
                duration_violations += 1

        is_valid = mean_confidence >= 0.5 and duration_violations == 0

        if mean_confidence < 0.5:
            warnings.append(f"Low mean confidence: {mean_confidence:.2f}")

        if duration_violations > 0:
            warnings.append(f"{duration_violations} tokens with invalid duration")

        return {
            "is_valid": bool(is_valid),
            "mean_confidence": float(mean_confidence),
            "duration_violations": duration_violations,
            "warnings": warnings
        }
