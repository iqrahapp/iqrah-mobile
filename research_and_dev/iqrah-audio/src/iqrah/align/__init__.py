"""CTC forced alignment and confidence scoring."""

from .ctc_align import CTCAligner, CTCForcedAligner, align_graphemes
from .llr import compute_llr, add_llr_scores, interpret_llr
from .phoneme_aligner import (
    PhonemeCTCAligner,
    PhonemeAlignment,
    WordAlignment
)

__all__ = [
    "CTCAligner",           # Legacy greedy aligner
    "CTCForcedAligner",     # Production Viterbi aligner (grapheme)
    "align_graphemes",      # Legacy standalone function
    "PhonemeCTCAligner",    # Phoneme-level aligner (Muaalem)
    "PhonemeAlignment",
    "WordAlignment",
    "compute_llr",
    "add_llr_scores",
    "interpret_llr"
]
