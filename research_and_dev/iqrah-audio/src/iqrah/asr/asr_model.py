"""
ASR Model Wrapper for obadx/muaalem-model-v3_2

Features:
- FP16 inference on CUDA
- Automatic chunking for audio >20s
- Returns both transcript and CTC logits for alignment
"""

from typing import Dict, List, Tuple, Optional
from contextlib import nullcontext
import numpy as np
import torch
from transformers import AutoModelForCTC, AutoProcessor

# (optional) from transformers import AutoModelForCTC, AutoProcessor


def chunk_audio(
    audio: np.ndarray, sample_rate: int = 16000, chunk_duration: float = 20.0, stride: float = 0.4
) -> List[Tuple[np.ndarray, float, float]]:
    """Split long audio into overlapping chunks for ASR."""
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
        if len(seg) > sample_rate:  # keep >= 1 s chunks
            chunks.append((seg, start / sample_rate, end / sample_rate))
        if end >= len(audio):
            break
    return chunks


class ASRModel:
    """
    ASR model wrapper for obadx/muaalem-model-v3_2.

    Features:
    - FP16 inference on CUDA
    - Automatic chunking for long audio
    - Returns both transcript and CTC logits
    """

    def __init__(
        self,
        model_name: str = "obadx/muaalem-model-v3_2",
        device: Optional[str] = None,
        use_fp16: bool = True,
        chunk_duration: float = 20.0,
        stride: float = 0.4,
        trim_overlap_for_logits: bool = True,
    ):
        self.model_name = model_name
        self.device = device or ("cuda" if torch.cuda.is_available() else "cpu")
        self.use_fp16 = bool(use_fp16 and self.device == "cuda")
        self.chunk_duration = float(chunk_duration)
        self.stride = float(stride)
        self.trim_overlap_for_logits = bool(trim_overlap_for_logits)

        # Processor/model
        self.processor = AutoProcessor.from_pretrained(model_name)
        self.model = AutoModelForCTC.from_pretrained(
            model_name, torch_dtype=torch.float16 if self.use_fp16 else None, trust_remote_code=True
        ).to(self.device)
        # (optional) Auto*:
        # self.processor = AutoProcessor.from_pretrained(model_name)
        # self.model = AutoModelForCTC.from_pretrained(model_name).to(self.device)

        self.model.eval()

    def _autocast_ctx(self):
        if self.use_fp16 and self.device == "cuda":
            return torch.autocast("cuda", dtype=torch.float16)
        return nullcontext()

    def transcribe(
        self, audio: np.ndarray, sample_rate: int = 16000, return_logits: bool = True
    ) -> Dict:
        if sample_rate != 16000:
            raise ValueError(f"Sample rate must be 16kHz, got {sample_rate}Hz")

        # ensure dtype/contiguity
        if audio.dtype != np.float32:
            audio = audio.astype(np.float32, copy=False)
        audio = np.ascontiguousarray(audio)

        duration = len(audio) / sample_rate
        if duration > self.chunk_duration:
            return self._transcribe_chunked(audio, sample_rate, return_logits)
        else:
            return self._transcribe_single(audio, sample_rate, return_logits)

    @torch.inference_mode()
    def _transcribe_single(self, audio: np.ndarray, sample_rate: int, return_logits: bool) -> Dict:
        inputs = self.processor(
            audio, sampling_rate=sample_rate, return_tensors="pt", padding=False
        )
        inputs = {k: v.to(self.device) for k, v in inputs.items()}

        with self._autocast_ctx():
            outputs = self.model(**inputs)
        logits = outputs.logits[0].detach().cpu()  # (T, V)

        pred_ids = torch.argmax(logits, dim=-1)
        transcript = self.processor.batch_decode(pred_ids.unsqueeze(0), skip_special_tokens=True)[0]

        out = {"transcript": transcript}
        if return_logits:
            out["logits"] = logits.numpy()
        return out

    @torch.inference_mode()
    def _transcribe_chunked(self, audio: np.ndarray, sample_rate: int, return_logits: bool) -> Dict:
        chunks = chunk_audio(
            audio, sample_rate=sample_rate, chunk_duration=self.chunk_duration, stride=self.stride
        )

        chunk_results: List[Dict] = []
        all_logits: List[np.ndarray] = []

        for i, (seg, start_t, end_t) in enumerate(chunks):
            r = self._transcribe_single(seg, sample_rate, return_logits)
            chunk_results.append(
                {"transcript": r["transcript"], "start_time": start_t, "end_time": end_t}
            )
            if return_logits:
                all_logits.append(r["logits"])

        full_transcript = " ".join(r["transcript"] for r in chunk_results)
        result = {"transcript": full_transcript, "chunks": chunk_results}

        if return_logits and all_logits:
            if self.trim_overlap_for_logits:
                # proportionally trim the leading overlap frames for all chunks after the first
                trimmed = [all_logits[0]]
                for i in range(1, len(all_logits)):
                    T = all_logits[i].shape[0]
                    trim = int(T * (self.stride / self.chunk_duration))
                    trimmed.append(all_logits[i][trim:] if trim < T else all_logits[i])
                result["logits"] = np.concatenate(trimmed, axis=0)
            else:
                result["logits"] = np.concatenate(all_logits, axis=0)

        return result

    @torch.inference_mode()
    def get_ctc_posteriors(self, audio: np.ndarray, sample_rate: int = 16000) -> np.ndarray:
        r = self.transcribe(audio, sample_rate, return_logits=True)
        logits = torch.from_numpy(r["logits"])  # (T, V)
        post = torch.softmax(logits, dim=-1).numpy()
        return post
