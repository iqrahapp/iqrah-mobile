"""
Muaalem ASR Wrapper for Phoneme Recognition + Tajweed Sifat (M3.2)

This module wraps **obadx/muaalem-model-v3_2** to provide a robust, production-ready
interface for:

- **Phoneme sequence prediction** (CTC-based)
- **Tajweed sifat** extraction (multi-head classification; 10+ properties)
- **CTC logits** export for downstream **forced alignment**
- **Automatic chunking** with precise logits stitching for long audio (> ~20s)

Key improvements over the older wrapper:
- CPU-safe dtype defaulting (float32 on CPU, bf16/fp16 on CUDA as requested)
- Exact frame-level trimming during logits merge using real per-chunk sample counts
- Single **multi-level** decode on the merged sequence (keeps levels aligned)
- Optional logits collection without leaking large tensors when not requested
- Clean separation of single-segment vs. chunked inference paths

Expected Inputs
---------------
- Audio waveform: `np.ndarray` 1-D float32 at **16 kHz**
- Phonetic reference: `IqrahPhoneticOutput` (produced by the Iqrah phonetizer),
  which internally carries the upstream `QuranPhoneticScriptOutput`.

Outputs
-------
- `MuaalemInferenceOutput`:
  - `phonemes`: upstream `Unit` with `.text`, `.ids`, `.probs`
  - `sifat`: list of upstream `Sifa` objects
  - `ctc_logits` (optional): `torch.Tensor` of shape `(T, V)` on CPU float32
  - `duration`, `sample_rate`, and a lightweight `raw_output`

Usage Example
-------------
>>> from iqrah.asr.muaalem_wrapper import MuaalemASR
>>> from iqrah.text.phonetizer import phonetize_ayah
>>> import numpy as np
>>>
>>> model = MuaalemASR(device="cuda", dtype=torch.bfloat16)
>>> phonetic_ref = phonetize_ayah("بِسْمِ اللّٰهِ الرَّحْمٰنِ الرَّحِيمِ")
>>> audio = np.random.randn(16000 * 5).astype(np.float32)  # 5 seconds @16k
>>>
>>> out = model.infer(audio=audio, phonetic_ref=phonetic_ref, return_ctc_logits=True)
>>> out.phonemes.text[:32]
'bismillahirrahmanirrahim'
>>> out.ctc_logits.shape  # (T, V)
torch.Size([..., ...])

Notes
-----
- Sample rate **must** be 16,000 Hz (required by the upstream model).
- For long utterances, chunking is automatic and overlap is handled at the **logits**
  level to avoid boundary artifacts.
"""

from typing import List, Optional, Dict, Tuple
from dataclasses import dataclass
from collections import defaultdict
import logging
import numpy as np
import torch

# Upstream model + types
from quran_muaalem import Muaalem, MuaalemOutput, Unit, Sifa
from quran_muaalem.decode import (
    multilevel_greedy_decode,
    phonemes_level_greedy_decode,
)

# format_sifat may be exported from package or defined in inference; support both
try:
    from quran_muaalem import format_sifat
except Exception:
    from quran_muaalem.inference import format_sifat  # fallback if not exported

from quran_transcript import chunck_phonemes
from ..text.phonetizer import IqrahPhoneticOutput


@dataclass
class MuaalemInferenceOutput:
    """
    Output from Muaalem ASR inference with Iqrah extensions.

    Attributes
    ----------
    phonemes : Unit
        Predicted phoneme sequence (text, ids, probs).
    sifat : List[Sifa]
        Tajweed properties aligned to the decoded sequence.
    ctc_logits : Optional[torch.Tensor]
        (T, V) CTC logits on CPU float32 if requested, otherwise None.
    duration : float
        Audio duration in seconds.
    sample_rate : int
        Audio sample rate (must be 16000).
    raw_output : MuaalemOutput
        Lightweight upstream output for reference/debug (large tensors stripped
        if logits were not requested).
    """

    phonemes: Unit
    sifat: List[Sifa]
    ctc_logits: Optional[torch.Tensor]
    duration: float
    sample_rate: int
    raw_output: MuaalemOutput


def chunk_audio_for_muaalem(
    audio: np.ndarray,
    sample_rate: int = 16000,
    chunk_duration: float = 20.0,
    stride: float = 0.4,
) -> List[Tuple[np.ndarray, float, float]]:
    """
    Split long audio into overlapping chunks for Muaalem.

    Parameters
    ----------
    audio : np.ndarray
        1-D float32 waveform at `sample_rate`.
    sample_rate : int
        Must be 16000 for the upstream model.
    chunk_duration : float
        Target chunk length in seconds (default 20.0).
    stride : float
        Overlap in seconds between consecutive chunks (default 0.4).

    Returns
    -------
    List[Tuple[np.ndarray, float, float]]
        List of `(audio_chunk, start_time_sec, end_time_sec)` tuples.

    Example
    -------
    >>> x = np.random.randn(16000 * 30).astype(np.float32)  # 30s
    >>> chunks = chunk_audio_for_muaalem(x, 16000, 20.0, 0.4)
    >>> len(chunks)
    2
    """
    total_sec = len(audio) / sample_rate
    if total_sec <= chunk_duration:
        return [(audio, 0.0, total_sec)]

    chunk_samples = int(chunk_duration * sample_rate)
    stride_samples = int(stride * sample_rate)

    chunks: List[Tuple[np.ndarray, float, float]] = []
    step = max(1, chunk_samples - stride_samples)

    for start in range(0, len(audio), step):
        end = min(start + chunk_samples, len(audio))
        seg = audio[start:end]
        # keep chunks >= 1 second for stability
        if len(seg) >= sample_rate:
            chunks.append((seg, start / sample_rate, end / sample_rate))
        if end >= len(audio):
            break

    return chunks


def trim_frames_exact(
    T_i: int,
    samples_in_chunk: int,
    stride_samples: int,
) -> int:
    """
    Compute the exact number of leading frames to trim from a chunk's logits,
    proportional to the time overlap in **samples**.

    This uses the per-chunk frames-per-sample ratio `T_i / samples_in_chunk`, so
    the shorter last chunk (or any variable-length chunk) is handled correctly.

    Parameters
    ----------
    T_i : int
        Number of time frames in the i-th logits tensor.
    samples_in_chunk : int
        Number of raw audio samples in the i-th audio chunk.
    stride_samples : int
        Overlap length in samples.

    Returns
    -------
    int
        Number of frames to trim from the beginning of logits[i].
    """
    if samples_in_chunk == 0:
        return 0
    fps = T_i / samples_in_chunk
    return int(round(stride_samples * fps))


def merge_chunked_logits_precise(
    logits_list: List[torch.Tensor],
    chunk_sample_counts: List[int],
    stride_samples: int,
) -> torch.Tensor:
    """
    Merge CTC logits from overlapping chunks using **per-chunk** sample counts.

    Strategy
    --------
    For each chunk after the first, trim `trim_frames_exact(...)` from the start
    to remove the overlapped time region, then concatenate along time.

    Parameters
    ----------
    logits_list : List[torch.Tensor]
        List of per-chunk logits with shapes `(T_i, V)`.
    chunk_sample_counts : List[int]
        Exact sample counts for each chunk (used for robust trimming).
    stride_samples : int
        Overlap length in samples.

    Returns
    -------
    torch.Tensor
        Merged logits of shape `(sum(T_i) - trims, V)` on the current device.

    Example
    -------
    >>> L1, L2 = torch.randn(1000, 50), torch.randn(1000, 50)
    >>> merged = merge_chunked_logits_precise([L1, L2], [320000, 320000], 6400)
    >>> merged.shape  # 0.4s overlap @16k ≈ 6400 samples; trims ≈ 20 frames if 50 fps
    torch.Size([1980, 50])
    """
    if not logits_list:
        return torch.empty(0)
    if len(logits_list) == 1:
        return logits_list[0]

    merged = [logits_list[0]]
    for i in range(1, len(logits_list)):
        T_i = logits_list[i].shape[0]
        trim = trim_frames_exact(T_i, chunk_sample_counts[i], stride_samples)
        merged.append(logits_list[i][trim:] if trim < T_i else logits_list[i])
    return torch.cat(merged, dim=0)


class MuaalemASR:
    """
    Iqrah wrapper for **obadx/muaalem-model-v3_2**.

    Provides:
    - Phoneme sequence prediction (CTC)
    - Tajweed sifat extraction (multi-head)
    - Optional CTC logits for forced alignment
    - Automatic chunking + precise logits stitching

    Parameters
    ----------
    model_name : str
        Hugging Face model name or path (default: "obadx/muaalem-model-v3_2").
    device : Optional[str]
        "cuda", "cpu", or None to auto-select (CUDA if available).
    dtype : torch.dtype
        Compute dtype. On **CPU** we enforce `torch.float32` for numerical safety.
        On CUDA, `torch.bfloat16` or `torch.float16` are supported by the upstream model.
    chunk_duration : float
        Target chunk size in seconds for long audio (default 20.0).
    stride : float
        Overlap in seconds between chunks (default 0.4).

    Example
    -------
    >>> asr = MuaalemASR(device="cuda", dtype=torch.bfloat16)
    """

    def __init__(
        self,
        model_name: str = "obadx/muaalem-model-v3_2",
        device: Optional[str] = None,
        dtype=torch.bfloat16,
        chunk_duration: float = 20.0,
        stride: float = 0.4,
    ):
        self.model_name = model_name
        self.device = device or ("cuda" if torch.cuda.is_available() else "cpu")
        # Safer / more stable default on CPU
        self.dtype = torch.float32 if self.device == "cpu" else dtype
        self.chunk_duration = float(chunk_duration)
        self.stride = float(stride)

        self.model = Muaalem(
            model_name_or_path=model_name,
            device=self.device,
            dtype=self.dtype,
        )
        # Upstream blank/pad id for CTC (kept for potential downstream use)
        self.blank_id = self.model.model.config.pad_token_id

    def infer(
        self,
        audio: np.ndarray,
        phonetic_ref: IqrahPhoneticOutput,
        sample_rate: int = 16000,
        return_ctc_logits: bool = False,
    ) -> MuaalemInferenceOutput:
        """
        Run ASR inference with optional CTC logits for alignment.

        Pipeline
        --------
        1) Validate & normalize inputs (dtype/contiguity/sr)
        2) If `duration > chunk_duration`, use chunked path; otherwise single
        3) Decode phonemes + sifat; optionally collect logits

        Parameters
        ----------
        audio : np.ndarray
            1-D float32 waveform at 16 kHz.
        phonetic_ref : IqrahPhoneticOutput
            Phonetic reference produced by the Iqrah phonetizer.
        sample_rate : int
            Must be 16000 Hz.
        return_ctc_logits : bool
            If True, returns `(T, V)` CTC logits on CPU float32.

        Returns
        -------
        MuaalemInferenceOutput
            Unified output containing decoded units, sifat, optional logits, etc.

        Raises
        ------
        ValueError
            If sample rate is not 16000 or audio is empty.
        """
        if sample_rate != 16000:
            raise ValueError(f"Sample rate must be 16kHz, got {sample_rate}Hz")
        if audio.size == 0:
            raise ValueError("Audio cannot be empty")

        if audio.dtype != np.float32:
            audio = audio.astype(np.float32, copy=False)
        audio = np.ascontiguousarray(audio)

        duration = len(audio) / sample_rate
        if duration > self.chunk_duration:
            return self._infer_chunked(audio, phonetic_ref, sample_rate, return_ctc_logits)
        else:
            return self._infer_single(audio, phonetic_ref, sample_rate, return_ctc_logits)

    def _infer_single(
        self,
        audio: np.ndarray,
        phonetic_ref: IqrahPhoneticOutput,
        sample_rate: int,
        return_ctc_logits: bool,
    ) -> MuaalemInferenceOutput:
        """
        Inference on a single segment (≤ `chunk_duration`).

        Notes
        -----
        - When `return_ctc_logits=False`, large tensors (e.g., level logits)
          are dropped from `raw_output` to minimize memory footprint.
        """
        results = self.model(
            waves=[audio.tolist()],
            ref_quran_phonetic_script_list=[phonetic_ref.raw_output],
            sampling_rate=sample_rate,
            return_logits=return_ctc_logits,
        )
        muaalem_output = results[0]

        # Avoid leaking big tensors if not requested
        if not return_ctc_logits and getattr(muaalem_output, "level_to_logits", None) is not None:
            muaalem_output.level_to_logits = None

        ctc_logits = None
        if return_ctc_logits and muaalem_output.level_to_logits is not None:
            ctc_logits = muaalem_output.level_to_logits.get("phonemes")

        return MuaalemInferenceOutput(
            phonemes=muaalem_output.phonemes,
            sifat=muaalem_output.sifat,
            ctc_logits=(
                ctc_logits.detach().to("cpu", torch.float32) if ctc_logits is not None else None
            ),
            duration=len(audio) / sample_rate,
            sample_rate=sample_rate,
            raw_output=muaalem_output,
        )

    def _infer_chunked(
        self,
        audio: np.ndarray,
        phonetic_ref: IqrahPhoneticOutput,
        sample_rate: int,
        return_ctc_logits: bool,
    ) -> MuaalemInferenceOutput:
        """
        Inference on long audio with **automatic chunking** and **precise** logit merging.

        Strategy
        --------
        1) Split audio into overlapping chunks
        2) For each chunk, run `_infer_single(..., return_ctc_logits=True)` to gather all
           available **per-level** logits on CPU
        3) Merge logits **per level** using `merge_chunked_logits_precise(...)`
        4) Run a single **multi-level** decode on the merged sequence to keep
           phonemes & sifat perfectly aligned across former boundaries

        Returns
        -------
        MuaalemInferenceOutput
            Merged phoneme `Unit`, merged sifat list, optional merged phoneme
            logits `(T, V)` on CPU float32, duration, etc.

        Notes
        -----
        - Levels other than "phonemes" with inconsistent chunk counts are dropped
          gracefully with a warning (defensive coding).
        """
        stride_samples = int(self.stride * sample_rate)
        chunks = chunk_audio_for_muaalem(audio, sample_rate, self.chunk_duration, self.stride)
        chunk_sample_counts = [len(seg) for seg, _, _ in chunks]

        per_level_logits: Dict[str, List[torch.Tensor]] = defaultdict(list)
        for seg, _, _ in chunks:
            result = self._infer_single(seg, phonetic_ref, sample_rate, return_ctc_logits=True)
            if result.raw_output.level_to_logits:
                for level, tensor in result.raw_output.level_to_logits.items():
                    # Force CPU early to minimize GPU memory growth
                    per_level_logits[level].append(tensor.detach().to("cpu"))

        # Safety checks — phoneme level is required
        phoneme_logits_list = per_level_logits.get("phonemes", [])
        assert phoneme_logits_list, "No phoneme logits were gathered from chunks."
        assert len(phoneme_logits_list) == len(chunk_sample_counts), (
            f"Logit list size ({len(phoneme_logits_list)}) "
            f"mismatches chunk count ({len(chunk_sample_counts)})."
        )

        # Drop inconsistent non-phoneme levels gracefully
        for level, tensor_list in list(per_level_logits.items()):
            if not tensor_list:
                per_level_logits.pop(level, None)
                continue
            if len(tensor_list) != len(chunk_sample_counts) and level != "phonemes":
                logging.warning(
                    "Inconsistent chunk count for level '%s' (%d vs %d). Dropping this level.",
                    level,
                    len(tensor_list),
                    len(chunk_sample_counts),
                )
                per_level_logits.pop(level, None)

        # Merge logits per level
        merged_level_logits: Dict[str, torch.Tensor] = {
            level: merge_chunked_logits_precise(t_list, chunk_sample_counts, stride_samples)
            for level, t_list in per_level_logits.items()
        }

        # Single multi-level decode on merged sequence
        tok = self.model.multi_level_tokenizer

        # Convert logits -> probabilities for decoding; add batch dim
        level_to_probs = {
            level: torch.softmax(t.to(torch.float32), dim=-1).cpu().unsqueeze(0)  # (1, T, V)
            for level, t in merged_level_logits.items()
        }

        # Tokenize reference for constrained multilevel decoding
        level_to_ref_ids = tok.tokenize(
            [phonetic_ref.raw_output.phonemes],
            [phonetic_ref.raw_output.sifat],
            to_dict=True,
            return_tensors="pt",
            padding="longest",
        )["input_ids"]

        # Decode phoneme Unit first (kept for downstream + chunking helpers)
        merged_phonemes_unit = phonemes_level_greedy_decode(
            level_to_probs["phonemes"], tok.id_to_vocab["phonemes"]
        )[0]

        # Prepare chunked helpers (library provides these utilities)
        chunked_ph_batch = [chunck_phonemes(merged_phonemes_unit.text)]

        # Multi-level greedy decode (uses reference ids & chunked helpers)
        level_to_units = multilevel_greedy_decode(
            level_to_probs=level_to_probs,
            level_to_id_to_vocab=tok.id_to_vocab,
            level_to_ref_ids=level_to_ref_ids,
            chunked_phonemes_batch=chunked_ph_batch,
            ref_chuncked_phonemes_batch=[[s.phonemes for s in phonetic_ref.raw_output.sifat]],
            phonemes_units=[merged_phonemes_unit],
        )
        merged_sifat = format_sifat(level_to_units, chunked_ph_batch, tok)[0]

        duration = len(audio) / sample_rate
        merged_raw = MuaalemOutput(phonemes=merged_phonemes_unit, sifat=merged_sifat)

        return MuaalemInferenceOutput(
            phonemes=merged_phonemes_unit,
            sifat=merged_sifat,
            ctc_logits=(
                merged_level_logits["phonemes"].detach().to("cpu", torch.float32)
                if return_ctc_logits and "phonemes" in merged_level_logits
                else None
            ),
            duration=duration,
            sample_rate=sample_rate,
            raw_output=merged_raw,
        )

    def get_ctc_posteriors(
        self,
        audio: np.ndarray,
        phonetic_ref: IqrahPhoneticOutput,
        sample_rate: int = 16000,
        return_log_posteriors: bool = False,
    ) -> np.ndarray:
        """
        Convenience method: return (log-)posteriors `(T, V)` for alignment.

        Parameters
        ----------
        audio : np.ndarray
            1-D float32 waveform at 16 kHz.
        phonetic_ref : IqrahPhoneticOutput
            Phonetic reference produced by the Iqrah phonetizer.
        sample_rate : int
            Must be 16000 Hz.
        return_log_posteriors : bool
            If True, return `log_softmax(logits)`; else `softmax(logits)`.

        Returns
        -------
        np.ndarray
            (T, V) array on CPU float32.

        Example
        -------
        >>> post = model.get_ctc_posteriors(audio, phonetic_ref)
        >>> post.shape
        (T, V)
        """
        result = self.infer(audio, phonetic_ref, sample_rate, return_ctc_logits=True)
        if result.ctc_logits is None:
            raise RuntimeError("CTC logits not available. Ensure return_ctc_logits=True.")
        logits = result.ctc_logits  # already CPU float32
        post = (
            torch.log_softmax(logits, dim=-1)
            if return_log_posteriors
            else torch.softmax(logits, dim=-1)
        )
        return post.cpu().numpy()
