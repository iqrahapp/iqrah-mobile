"""ASR module using obadx/muaalem-model-v3_2."""

from .asr_model import ASRModel, chunk_audio
from .muaalem_wrapper import (
    MuaalemASR,
    MuaalemInferenceOutput,
    chunk_audio_for_muaalem,
    merge_chunked_logits_precise,
)

__all__ = [
    "ASRModel",
    "chunk_audio",
    "MuaalemASR",
    "MuaalemInferenceOutput",
    "chunk_audio_for_muaalem",
    "merge_chunked_logits_precise",
]
