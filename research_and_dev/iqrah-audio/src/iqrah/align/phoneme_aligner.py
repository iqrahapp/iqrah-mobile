"""
Phoneme-Level CTC Forced Alignment (M3.3)

Viterbi-based alignment using CTC posteriors from Muaalem.
Provides phoneme-level timestamps with proper blank handling.

This implementation:
- Uses CTC Viterbi algorithm with blank transitions
- Extracts precise timestamps for each phoneme
- Aggregates phonemes into word-level segments
- Computes confidence scores from CTC posteriors
"""

from typing import List, Dict, Optional
from dataclasses import dataclass
import numpy as np
import torch

from ..asr.muaalem_wrapper import MuaalemASR
from ..text.phonetizer import IqrahPhoneticOutput


@dataclass
class PhonemeAlignment:
    """
    Single phoneme with timing and confidence.

    Attributes:
        phoneme: Phoneme string
        start: Start time in seconds
        end: End time in seconds
        confidence: Mean CTC posterior probability
        sifa: Optional Tajweed properties from Muaalem
    """
    phoneme: str
    start: float
    end: float
    confidence: float
    sifa: Optional[dict] = None


@dataclass
class WordAlignment:
    """
    Word-level segment aggregated from phonemes.

    Attributes:
        word: Arabic word (grapheme form)
        start: Start time in seconds
        end: End time in seconds
        phoneme_indices: Indices into phonemes array
        confidence: Mean confidence across phonemes
    """
    word: str
    start: float
    end: float
    phoneme_indices: List[int]
    confidence: float


class PhonemeCTCAligner:
    """
    CTC forced aligner for phoneme-level alignment with Muaalem.

    This aligner:
    1. Takes CTC posteriors from Muaalem
    2. Runs Viterbi alignment with phonetic reference
    3. Extracts phoneme timestamps
    4. Aggregates into word-level segments
    5. Attaches sifat (Tajweed properties) to phonemes

    Examples:
        >>> from iqrah.asr import MuaalemASR
        >>> from iqrah.text import phonetize_ayah
        >>> from iqrah.align import PhonemeCTCAligner
        >>>
        >>> model = MuaalemASR()
        >>> aligner = PhonemeCTCAligner(model)
        >>>
        >>> phonetic_ref = phonetize_ayah("بِسْمِ اللَّهِ")
        >>> result = aligner.align(audio, phonetic_ref, sample_rate=16000)
        >>> len(result["phonemes"])
        10
        >>> len(result["words"])
        2
    """

    def __init__(self, asr_model: MuaalemASR):
        """
        Initialize phoneme CTC aligner.

        Args:
            asr_model: MuaalemASR model instance

        Examples:
            >>> model = MuaalemASR(device="cuda")
            >>> aligner = PhonemeCTCAligner(model)
        """
        self.asr_model = asr_model
        # Muaalem uses blank token ID (typically pad_token_id)
        # We'll detect this from model internals
        self.blank_id = 0  # CTC models typically use 0 as blank

    def align(
        self,
        audio: np.ndarray,
        phonetic_ref: IqrahPhoneticOutput,
        sample_rate: int = 16000
    ) -> Dict:
        """
        Perform phoneme-level CTC forced alignment.

        Pipeline:
        1. Run Muaalem inference to get phonemes + sifat
        2. Get CTC posteriors from model
        3. Run Viterbi alignment with phonetic reference
        4. Extract phoneme timestamps
        5. Aggregate into word-level segments
        6. Attach sifat to phonemes

        Args:
            audio: Audio waveform (mono, 16kHz recommended)
            phonetic_ref: Phonetic reference from phonetizer
            sample_rate: Audio sample rate in Hz

        Returns:
            {
                "phonemes": List[PhonemeAlignment],  # Phoneme-level alignments
                "words": List[WordAlignment],         # Word-level aggregations
                "alignment_method": str,              # "ctc_phoneme_forced"
                "quality_score": float                # Mean confidence
            }

        Examples:
            >>> result = aligner.align(audio, phonetic_ref)
            >>> result["phonemes"][0].phoneme
            'b'
            >>> result["phonemes"][0].start
            0.0
            >>> result["phonemes"][0].end
            0.05
        """
        # 1. Run Muaalem inference
        muaalem_result = self.asr_model.infer(
            audio,
            phonetic_ref,
            sample_rate,
            return_ctc_logits=True
        )

        # 2. Get CTC posteriors
        posteriors = self.asr_model.get_ctc_posteriors(
            audio,
            phonetic_ref,
            sample_rate
        )

        T, V = posteriors.shape
        if T == 0:
            return {
                "phonemes": [],
                "words": [],
                "alignment_method": "ctc_phoneme_forced",
                "quality_score": 0.0
            }

        audio_sec = len(audio) / float(sample_rate)
        frame_dur = audio_sec / T

        # 3. Get phoneme sequence from reference
        phoneme_sequence = list(phonetic_ref.text)  # Split into characters
        N = len(phoneme_sequence)

        if N == 0:
            return {
                "phonemes": [],
                "words": [],
                "alignment_method": "ctc_phoneme_forced",
                "quality_score": 0.0
            }

        # 4. Convert to log-space for Viterbi
        eps = 1e-10
        logp = np.log(posteriors + eps).astype(np.float32)

        # 5. Run Viterbi alignment
        # NOTE: This requires phoneme IDs from Muaalem tokenizer
        # For now, we use a simplified approach with character-level mapping
        phoneme_ids = self._get_phoneme_ids(phoneme_sequence)

        emissions = self._viterbi_align(logp, phoneme_ids, self.blank_id)

        if emissions is None:
            # Fallback to proportional partition
            return self._fallback_partition(
                phoneme_sequence,
                posteriors,
                phoneme_ids,
                frame_dur,
                muaalem_result.sifat
            )

        # 6. Partition frames using midpoints
        mids = self._compute_midpoints(emissions, T, N)

        # 7. Build phoneme alignments
        phonemes: List[PhonemeAlignment] = []
        confidences = []

        for i in range(N):
            start_frame = mids[i]
            end_frame = max(mids[i + 1], start_frame + 1)

            # Compute confidence from posteriors
            seg = posteriors[start_frame:end_frame, phoneme_ids[i]]
            conf = float(seg.mean()) if seg.size > 0 else 0.0
            confidences.append(conf)

            # Get sifa for this phoneme (if available)
            sifa_dict = None
            if i < len(muaalem_result.sifat):
                sifa_dict = self._sifa_to_dict(muaalem_result.sifat[i])

            phonemes.append(PhonemeAlignment(
                phoneme=phoneme_sequence[i],
                start=start_frame * frame_dur,
                end=end_frame * frame_dur,
                confidence=conf,
                sifa=sifa_dict
            ))

        # 8. Aggregate into word-level segments
        words = self._aggregate_words(phonemes, phonetic_ref)

        # 9. Compute quality score
        quality = float(np.mean(confidences)) if confidences else 0.0

        return {
            "phonemes": phonemes,
            "words": words,
            "alignment_method": "ctc_phoneme_forced",
            "quality_score": quality
        }

    def _get_phoneme_ids(self, phoneme_sequence: List[str]) -> List[int]:
        """
        Map phoneme strings to token IDs.

        NOTE: This is a placeholder. Real implementation should use
        Muaalem's tokenizer to get proper phoneme IDs.

        Args:
            phoneme_sequence: List of phoneme strings

        Returns:
            List of token IDs
        """
        # TODO: Use Muaalem tokenizer to get real phoneme IDs
        # For now, use character ordinals as placeholder
        return [ord(p) % 128 for p in phoneme_sequence]

    def _viterbi_align(
        self,
        logp: np.ndarray,
        phoneme_ids: List[int],
        blank_id: int
    ) -> Optional[np.ndarray]:
        """
        Run Viterbi algorithm to find emission frames.

        Args:
            logp: Log posteriors (T, V)
            phoneme_ids: Target phoneme IDs
            blank_id: Blank token ID

        Returns:
            Emission frames for each phoneme, or None if alignment fails
        """
        T = logp.shape[0]
        N = len(phoneme_ids)

        # Initialize DP table
        dp = np.full((T + 1, N + 1), -np.inf, dtype=np.float32)
        ptr = np.full((T + 1, N + 1), -1, dtype=np.int8)
        dp[0, 0] = 0.0

        # Forward pass
        for t in range(T):
            # Stay via blank
            stay_scores = dp[t, :] + logp[t, blank_id]
            better = stay_scores > dp[t + 1, :]
            dp[t + 1, better] = stay_scores[better]
            ptr[t + 1, better] = 0

            # Emit token
            if N > 0:
                # Ensure phoneme_ids are within bounds
                valid_ids = [pid for pid in phoneme_ids if pid < logp.shape[1]]
                if len(valid_ids) == N:
                    emit_scores = dp[t, :N] + logp[t, valid_ids]
                    better_emit = emit_scores > dp[t + 1, 1:]
                    dp[t + 1, 1:][better_emit] = emit_scores[better_emit]
                    ptr[t + 1, 1:][better_emit] = 1

        # Check if valid path exists
        if not np.isfinite(dp[T, N]):
            return None

        # Backtrack
        emissions = np.full(N, -1, dtype=np.int32)
        t, j = T, N

        while t > 0 and j >= 0:
            if ptr[t, j] == 1:
                emissions[j - 1] = t - 1
                j -= 1
                t -= 1
            elif ptr[t, j] == 0:
                t -= 1
            else:
                break

        if (emissions < 0).any():
            return None

        return emissions

    def _compute_midpoints(
        self,
        emissions: np.ndarray,
        T: int,
        N: int
    ) -> np.ndarray:
        """
        Compute frame boundaries using midpoints between emissions.

        Args:
            emissions: Emission frames for each phoneme
            T: Total frames
            N: Number of phonemes

        Returns:
            Boundary frames (N+1,) with start and end
        """
        mids = np.zeros(N + 1, dtype=np.int32)
        mids[0] = 0

        for i in range(1, N):
            mids[i] = (emissions[i - 1] + emissions[i]) // 2

        mids[N] = T

        return mids

    def _sifa_to_dict(self, sifa) -> dict:
        """
        Convert Sifa dataclass to dictionary.

        Args:
            sifa: Sifa object from Muaalem

        Returns:
            Dictionary with sifa properties
        """
        sifa_dict = {}

        # Extract all sifa properties
        properties = [
            "hams_or_jahr",
            "shidda_or_rakhawa",
            "tafkheem_or_taqeeq",
            "itbaq",
            "safeer",
            "qalqla",
            "tikraar",
            "tafashie",
            "istitala",
            "ghonna"
        ]

        for prop in properties:
            value = getattr(sifa, prop, None)
            if value is not None:
                sifa_dict[prop] = {
                    "text": value.text,
                    "prob": float(value.prob),
                    "idx": int(value.idx)
                }

        return sifa_dict

    def _aggregate_words(
        self,
        phonemes: List[PhonemeAlignment],
        phonetic_ref: IqrahPhoneticOutput
    ) -> List[WordAlignment]:
        """
        Aggregate phonemes into word-level segments.

        Uses word_index from PhoneticUnit metadata to group phonemes.

        Args:
            phonemes: List of aligned phonemes
            phonetic_ref: Phonetic reference with word boundaries

        Returns:
            List of WordAlignment objects
        """
        words: List[WordAlignment] = []

        # Group phonemes by word_index
        word_groups: Dict[int, List[int]] = {}

        for i, phoneme in enumerate(phonemes):
            # Get word index from phonetic_ref.units
            if i < len(phonetic_ref.units):
                word_idx = phonetic_ref.units[i].word_index
            else:
                word_idx = -1

            if word_idx not in word_groups:
                word_groups[word_idx] = []

            word_groups[word_idx].append(i)

        # Create word alignments
        for word_idx in sorted(word_groups.keys()):
            if word_idx < 0:
                continue

            indices = word_groups[word_idx]
            if not indices:
                continue

            start = phonemes[indices[0]].start
            end = phonemes[indices[-1]].end

            # Compute mean confidence
            confs = [phonemes[i].confidence for i in indices]
            mean_conf = float(np.mean(confs))

            # Get word text (placeholder - should come from reference)
            word_text = f"word_{word_idx}"

            words.append(WordAlignment(
                word=word_text,
                start=start,
                end=end,
                phoneme_indices=indices,
                confidence=mean_conf
            ))

        return words

    def _fallback_partition(
        self,
        phoneme_sequence: List[str],
        posteriors: np.ndarray,
        phoneme_ids: List[int],
        frame_dur: float,
        sifat_list: List
    ) -> Dict:
        """
        Fallback to proportional slicing when Viterbi fails.

        Args:
            phoneme_sequence: List of phonemes
            posteriors: CTC posteriors
            phoneme_ids: Phoneme token IDs
            frame_dur: Frame duration
            sifat_list: Sifat from Muaalem

        Returns:
            Alignment result dictionary
        """
        T = posteriors.shape[0]
        N = len(phoneme_sequence)

        if N == 0 or T == 0:
            return {
                "phonemes": [],
                "words": [],
                "alignment_method": "ctc_phoneme_fallback",
                "quality_score": 0.0
            }

        # Equal partitions
        cuts = np.linspace(0, T, N + 1).astype(int)
        phonemes: List[PhonemeAlignment] = []
        confidences = []

        for i in range(N):
            start_frame = cuts[i]
            end_frame = max(cuts[i + 1], start_frame + 1)

            # Compute confidence
            if i < len(phoneme_ids) and phoneme_ids[i] < posteriors.shape[1]:
                seg = posteriors[start_frame:end_frame, phoneme_ids[i]]
                conf = float(seg.mean()) if seg.size > 0 else 0.0
            else:
                conf = 0.5  # Default confidence

            confidences.append(conf)

            # Get sifa
            sifa_dict = None
            if i < len(sifat_list):
                sifa_dict = self._sifa_to_dict(sifat_list[i])

            phonemes.append(PhonemeAlignment(
                phoneme=phoneme_sequence[i],
                start=start_frame * frame_dur,
                end=end_frame * frame_dur,
                confidence=conf,
                sifa=sifa_dict
            ))

        quality = float(np.mean(confidences)) if confidences else 0.0

        return {
            "phonemes": phonemes,
            "words": [],  # No word aggregation in fallback
            "alignment_method": "ctc_phoneme_fallback",
            "quality_score": quality
        }
