"""
M3 Pipeline: Phoneme Recognition & Alignment

Orchestrates the complete M3 workflow:
1. Text phonetization
2. Muaalem ASR inference (phonemes + sifat)
3. Phonetic gatekeeper (PER-based content verification)
4. CTC forced alignment (phoneme-level timestamps)

Output matches the M3 schema from doc/01-architecture/m3-phoneme-alignment.md
"""

from typing import Dict, List, Optional
from dataclasses import dataclass, asdict
import numpy as np
import torch

from ..text import phonetize_ayah, Phonetizer, IqrahPhoneticOutput
from ..asr import MuaalemASR, MuaalemInferenceOutput
from ..compare import PhoneticGatekeeper
from ..align import PhonemeCTCAligner, PhonemeAlignment, WordAlignment


@dataclass
class PhonemeOutput:
    """Phoneme with timing, confidence, and sifat."""
    phoneme: str
    start: float
    end: float
    confidence: float
    sifa: Optional[Dict]


@dataclass
class WordOutput:
    """Word-level segment."""
    word: str
    start: float
    end: float
    phonemes: List[int]  # Indices into phonemes array


@dataclass
class GateResult:
    """Gatekeeper result."""
    passed: bool
    per: float
    confidence: float
    errors: List[Dict]


@dataclass
class M3Output:
    """
    M3 module output matching the documented schema.

    Schema from doc/01-architecture/m3-phoneme-alignment.md:
    {
        "phonemes": [
            {
                "phoneme": str,
                "start": float,
                "end": float,
                "confidence": float,
                "sifa": Sifa
            }
        ],
        "words": [
            {
                "word": str,
                "start": float,
                "end": float,
                "phonemes": list[int]
            }
        ],
        "gate_result": {
            "passed": bool,
            "per": float,
            "confidence": float
        },
        "alignment_method": str
    }
    """
    phonemes: List[PhonemeOutput]
    words: List[WordOutput]
    gate_result: GateResult
    alignment_method: str

    def to_dict(self) -> Dict:
        """Convert to dictionary matching M3 schema."""
        return asdict(self)


class M3Pipeline:
    """
    Complete M3 pipeline for phoneme recognition and alignment.

    This pipeline implements the phonetic-first architecture with:
    - Pre-trained Muaalem model (no training required)
    - Phoneme-level analysis and alignment
    - PER-based content verification
    - Automatic Tajweed sifat extraction

    Examples:
        >>> from iqrah.pipeline import M3Pipeline
        >>> import numpy as np
        >>>
        >>> # Initialize pipeline
        >>> pipeline = M3Pipeline(device="cuda")
        >>>
        >>> # Process audio
        >>> audio = np.random.randn(16000 * 3)  # 3 seconds
        >>> result = pipeline.process(
        ...     audio=audio,
        ...     reference_text="بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ",
        ...     sample_rate=16000
        ... )
        >>>
        >>> # Check gate status
        >>> if result.gate_result.passed:
        ...     print(f"PER: {result.gate_result.per:.2%}")
        ...     print(f"Phonemes: {len(result.phonemes)}")
    """

    def __init__(
        self,
        device: Optional[str] = None,
        dtype = torch.bfloat16,
        rewaya: str = "hafs",
        per_threshold_high: float = 0.02,
        per_threshold_medium: float = 0.05,
        chunk_duration: float = 20.0,
        stride: float = 0.4
    ):
        """
        Initialize M3 pipeline with all components.

        Args:
            device: Device for Muaalem model ("cuda", "cpu", or None for auto)
            dtype: torch dtype for Muaalem (bfloat16 recommended)
            rewaya: Quranic recitation tradition (default: "hafs")
            per_threshold_high: PER threshold for high confidence (default: 0.02)
            per_threshold_medium: PER threshold for medium confidence (default: 0.05)
            chunk_duration: Max audio chunk size in seconds (default: 20s)
            stride: Overlap between chunks in seconds (default: 0.4s)

        Examples:
            >>> # CPU with default settings
            >>> pipeline = M3Pipeline(device="cpu")
            >>>
            >>> # CUDA with custom PER thresholds
            >>> pipeline = M3Pipeline(
            ...     device="cuda",
            ...     per_threshold_high=0.01,
            ...     per_threshold_medium=0.03
            ... )
        """
        self.device = device or ("cuda" if torch.cuda.is_available() else "cpu")
        self.dtype = dtype
        self.rewaya = rewaya

        # Initialize components
        self.phonetizer = Phonetizer(rewaya=rewaya)
        self.asr_model = MuaalemASR(
            device=self.device,
            dtype=dtype,
            chunk_duration=chunk_duration,
            stride=stride
        )
        self.gatekeeper = PhoneticGatekeeper(
            threshold_high=per_threshold_high,
            threshold_medium=per_threshold_medium
        )
        self.aligner = PhonemeCTCAligner(self.asr_model)

        print(f"M3 Pipeline initialized on {self.device}")
        print(f"  Rewaya: {rewaya}")
        print(f"  PER thresholds: high={per_threshold_high}, medium={per_threshold_medium}")

    def process(
        self,
        audio: np.ndarray,
        reference_text: str,
        sample_rate: int = 16000,
        skip_gate: bool = False
    ) -> M3Output:
        """
        Process audio through complete M3 pipeline.

        Pipeline steps:
        1. Phonetize reference text
        2. Run Muaalem ASR (phonemes + sifat)
        3. Verify content with phonetic gatekeeper (PER)
        4. If gate passes, perform CTC forced alignment
        5. Return M3Output with phonemes, words, gate result

        Args:
            audio: Audio waveform (mono, 16kHz recommended)
            reference_text: Quranic reference text (with diacritics)
            sample_rate: Audio sample rate in Hz (must be 16000)
            skip_gate: Skip gatekeeper validation (for testing)

        Returns:
            M3Output with complete phoneme-level analysis

        Raises:
            ValueError: If sample_rate != 16000
            ValueError: If audio is empty or invalid
            RuntimeError: If gatekeeper fails and skip_gate=False

        Examples:
            >>> result = pipeline.process(audio, "بِسْمِ اللَّهِ")
            >>> result.gate_result.passed
            True
            >>> len(result.phonemes)
            10
        """
        if sample_rate != 16000:
            raise ValueError(f"Sample rate must be 16kHz, got {sample_rate}Hz")

        if audio.size == 0:
            raise ValueError("Audio cannot be empty")

        # Step 1: Phonetize reference text
        print(f"[M3] Step 1/4: Phonetizing reference text...")
        phonetic_ref = self.phonetizer.phonetize(reference_text, remove_space=True)
        print(f"  Phonetic length: {len(phonetic_ref.text)} phonemes")

        # Step 2: Run Muaalem ASR
        print(f"[M3] Step 2/4: Running Muaalem ASR inference...")
        muaalem_result = self.asr_model.infer(
            audio=audio,
            phonetic_ref=phonetic_ref,
            sample_rate=sample_rate,
            return_ctc_logits=True
        )
        print(f"  Predicted: {len(muaalem_result.phonemes.text)} phonemes")
        print(f"  Sifat extracted: {len(muaalem_result.sifat)} groups")

        # Step 3: Verify content with phonetic gatekeeper
        print(f"[M3] Step 3/4: Verifying content (PER)...")
        ref_phonemes = list(phonetic_ref.text)
        pred_phonemes = list(muaalem_result.phonemes.text)

        gate_result_dict = self.gatekeeper.verify(ref_phonemes, pred_phonemes)

        gate_result = GateResult(
            passed=gate_result_dict['should_proceed'],
            per=gate_result_dict['per'],
            confidence=1.0 - gate_result_dict['per'],  # Convert PER to confidence
            errors=[asdict(e) for e in gate_result_dict['errors']]
        )

        print(f"  PER: {gate_result.per:.2%}")
        print(f"  Confidence: {gate_result_dict['confidence']}")
        print(f"  Gate: {'PASSED' if gate_result.passed else 'FAILED'}")

        if not gate_result.passed and not skip_gate:
            raise RuntimeError(
                f"Gatekeeper failed: PER={gate_result.per:.2%} > threshold. "
                f"Content mismatch detected. Use skip_gate=True to bypass."
            )

        # Step 4: Perform CTC forced alignment
        print(f"[M3] Step 4/4: Performing CTC forced alignment...")
        alignment_result = self.aligner.align(
            audio=audio,
            phonetic_ref=phonetic_ref,
            sample_rate=sample_rate
        )

        print(f"  Aligned: {len(alignment_result['phonemes'])} phonemes")
        print(f"  Words: {len(alignment_result['words'])}")
        print(f"  Method: {alignment_result['alignment_method']}")
        print(f"  Quality: {alignment_result['quality_score']:.2%}")

        # Convert to M3 output format
        phonemes = [
            PhonemeOutput(
                phoneme=p.phoneme,
                start=p.start,
                end=p.end,
                confidence=p.confidence,
                sifa=p.sifa
            )
            for p in alignment_result['phonemes']
        ]

        words = [
            WordOutput(
                word=w.word,
                start=w.start,
                end=w.end,
                phonemes=w.phoneme_indices
            )
            for w in alignment_result['words']
        ]

        return M3Output(
            phonemes=phonemes,
            words=words,
            gate_result=gate_result,
            alignment_method=alignment_result['alignment_method']
        )

    def process_batch(
        self,
        audio_list: List[np.ndarray],
        reference_texts: List[str],
        sample_rate: int = 16000,
        skip_gate: bool = False
    ) -> List[M3Output]:
        """
        Process multiple audio samples in batch.

        Note: Currently processes sequentially. Future optimization:
        batch processing in Muaalem model.

        Args:
            audio_list: List of audio waveforms
            reference_texts: List of reference texts (same length as audio_list)
            sample_rate: Audio sample rate
            skip_gate: Skip gatekeeper for all samples

        Returns:
            List of M3Output objects

        Examples:
            >>> audios = [audio1, audio2, audio3]
            >>> texts = ["بِسْمِ اللَّهِ", "الرَّحْمَٰنِ", "الرَّحِيمِ"]
            >>> results = pipeline.process_batch(audios, texts)
            >>> len(results)
            3
        """
        if len(audio_list) != len(reference_texts):
            raise ValueError(
                f"Audio list length ({len(audio_list)}) must match "
                f"reference texts length ({len(reference_texts)})"
            )

        results = []
        for i, (audio, text) in enumerate(zip(audio_list, reference_texts)):
            print(f"\n[M3 Batch] Processing {i+1}/{len(audio_list)}...")
            try:
                result = self.process(audio, text, sample_rate, skip_gate)
                results.append(result)
            except Exception as e:
                print(f"  ERROR: {e}")
                # Append None or empty result for failed samples
                results.append(None)

        return results

    def get_statistics(self, output: M3Output) -> Dict:
        """
        Extract statistics from M3 output.

        Args:
            output: M3Output from process()

        Returns:
            Dictionary with statistics:
            - total_phonemes: int
            - total_words: int
            - duration: float (seconds)
            - mean_confidence: float
            - per: float
            - sifat_count: int

        Examples:
            >>> stats = pipeline.get_statistics(result)
            >>> stats['mean_confidence']
            0.92
        """
        if not output.phonemes:
            return {
                "total_phonemes": 0,
                "total_words": 0,
                "duration": 0.0,
                "mean_confidence": 0.0,
                "per": output.gate_result.per,
                "sifat_count": 0
            }

        duration = output.phonemes[-1].end if output.phonemes else 0.0
        mean_conf = np.mean([p.confidence for p in output.phonemes])
        sifat_count = sum(1 for p in output.phonemes if p.sifa is not None)

        return {
            "total_phonemes": len(output.phonemes),
            "total_words": len(output.words),
            "duration": float(duration),
            "mean_confidence": float(mean_conf),
            "per": output.gate_result.per,
            "sifat_count": sifat_count
        }
