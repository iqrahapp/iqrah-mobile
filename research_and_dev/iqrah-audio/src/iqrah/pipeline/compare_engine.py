"""
Comparison Engine - Grapheme MVP Orchestrator (M3)

Grapheme-based alignment pipeline with zero training:
1. ASR transcription (obadx/muaalem-model-v3_2 CTC)
2. Content verification (hybrid WER/CER gatekeeper)
3. CTC Viterbi forced alignment (graphemes)
4. LLR confidence scoring (discriminative GOP)
5. Tajweed validation (duration/energy MVP rules)

Returns structured JSON output with backward compatibility.
"""

from typing import Dict, Optional, List
import numpy as np

from ..asr import ASRModel
from ..compare import ContentGate, verify_content
from ..align import CTCForcedAligner, add_llr_scores
from ..tajweed import TajweedValidatorMVP


class ComparisonEngine:
    """
    Main orchestrator for the grapheme MVP pipeline.

    Pipeline flow:
    Audio → ASR → Gate (WER/CER check) → CTC Viterbi Align → LLR → Tajweed → Output

    Features:
    - Zero training required
    - Grapheme-level alignment (not phoneme)
    - Proper Viterbi algorithm with blank handling
    - LLR-based confidence (discriminative)
    - Duration/energy Tajweed rules (MVP)

    Examples:
        >>> engine = ComparisonEngine("obadx/muaalem-model-v3_2")
        >>> result = engine.compare(audio, "بسم الله الرحمن الرحيم")
        >>> result["status"]
        'success'
        >>> result["quality_score"]
        87.5
    """

    def __init__(
        self,
        asr_model_name: str = "obadx/muaalem-model-v3_2",
        device: Optional[str] = None,
        use_fp16: bool = True
    ):
        """
        Initialize comparison engine.

        Args:
            asr_model_name: HuggingFace ASR model (CTC grapheme model)
            device: Device to use (cuda/cpu, auto-detected if None)
            use_fp16: Use FP16 precision (recommended for RTX 3060-Ti)

        Examples:
            >>> engine = ComparisonEngine()  # Default: muaalem-model-v3_2
            >>> engine = ComparisonEngine(use_fp16=False)  # FP32 for debugging
        """
        # Initialize ASR model
        self.asr_model = ASRModel(asr_model_name, device, use_fp16)

        # Initialize CTC forced aligner (Viterbi)
        self.forced_aligner = CTCForcedAligner(self.asr_model)

        # Initialize content gate
        self.content_gate = ContentGate()

        # Initialize Tajweed validator (MVP: duration + energy rules)
        self.tajweed_validator = TajweedValidatorMVP()

    def compare(
        self,
        audio: np.ndarray,
        reference_text: str,
        sample_rate: int = 16000,
        skip_content_check: bool = False,
        surah: Optional[int] = None,
        ayah: Optional[int] = None
    ) -> Dict:
        """
        Compare user recitation to reference text.

        Args:
            audio: Audio array (mono, float32, 16kHz recommended)
            reference_text: Expected Quranic text (with diacritics)
            sample_rate: Sample rate in Hz (default: 16000)
            skip_content_check: Skip WER/CER verification (for testing)
            surah: Optional surah number (for metadata)
            ayah: Optional ayah number (for metadata)

        Returns:
            {
                "status": "success" | "content_error",
                "content_verification": {
                    "error_rate": float,
                    "confidence": "high" | "medium" | "fail",
                    "metric_type": "wer" | "cer",
                    "should_proceed": bool
                },
                "units": [                    # Grapheme tokens
                    {
                        "token": str,
                        "start": float,
                        "end": float,
                        "confidence": float,
                        "gop_score": float    # LLR confidence
                    }
                ],
                "phonemes": [...],            # Backward compat alias
                "alignment_method": str,       # "ctc_grapheme" or "ctc_grapheme_fallback"
                "alignment_validation": {...},
                "quality_score": float,        # 0-100
                "tajweed": {
                    "overall_score": float,
                    "madd_violations": [...],
                    "shadda_violations": [...],
                    "waqf_violations": [...]
                },
                "metadata": {
                    "asr_model": str,
                    "num_tokens": int,
                    "duration_seconds": float,
                    "surah": int | None,
                    "ayah": int | None
                }
            }

        Edge Cases:
            - Content verification fails → returns status="content_error", no alignment
            - Empty audio → returns empty units
            - Audio too short → may trigger fallback alignment

        Examples:
            >>> audio = np.random.randn(16000 * 3)  # 3 seconds
            >>> result = engine.compare(audio, "بسم الله الرحمن الرحيم")
            >>> print(f"Quality: {result['quality_score']:.1f}/100")
            Quality: 87.5/100
        """
        # Step 1: ASR Transcription (for content verification)
        if not skip_content_check:
            asr_result = self.asr_model.transcribe(
                audio,
                sample_rate,
                return_logits=False  # Don't need logits for gate
            )
            transcript = asr_result["transcript"]

            # Step 2: Content Verification (Gatekeeper)
            gate_result = verify_content(reference_text, transcript)

            if not gate_result["should_proceed"]:
                # WER/CER too high - stop analysis
                return {
                    "status": "content_error",
                    "content_verification": gate_result,
                    "units": [],
                    "phonemes": [],
                    "alignment_method": "none",
                    "quality_score": 0.0,
                    "tajweed": None,
                    "metadata": {
                        "asr_model": self.asr_model.model_name,
                        "num_tokens": 0,
                        "duration_seconds": len(audio) / sample_rate,
                        "surah": surah,
                        "ayah": ayah,
                        "message": "Content mismatch detected. Please recite the correct verse."
                    }
                }

            # Use normalized reference from gate for alignment
            target_text = gate_result.get("normalized_reference", reference_text)
        else:
            gate_result = {
                "error_rate": 0.0,
                "confidence": "skipped",
                "metric_type": "wer",
                "should_proceed": True,
                "normalized_reference": reference_text
            }
            target_text = reference_text

        # Step 3: CTC Viterbi Forced Alignment
        # This returns: {"units": [...], "alignment_method": str, "quality_score": float}
        alignment_result = self.forced_aligner.align(audio, target_text, sample_rate)

        units = alignment_result["units"]
        alignment_method = alignment_result["alignment_method"]

        # Step 4: Validate Alignment Quality
        alignment_validation = self.forced_aligner.validate_alignment(units)

        # Step 5: Tajweed Validation (MVP: Madd, Shadda, Waqf)
        tajweed_result = self.tajweed_validator.validate(
            units,
            audio,
            sample_rate
        )

        # Step 6: Compute Overall Quality Score
        quality_score = self._compute_quality_score(
            units,
            alignment_validation,
            tajweed_result
        )

        # Step 7: Create backward-compatible "phonemes" alias
        # (Some downstream code may expect "phoneme" key instead of "token")
        phonemes_alias = [
            {
                "phoneme": u["token"],
                "start": u["start"],
                "end": u["end"],
                "confidence": u.get("confidence", 0.0),
                "gop_score": u.get("gop_score", None)
            }
            for u in units
        ]

        # Return structured result
        return {
            "status": "success",
            "content_verification": gate_result,
            "units": units,                           # Grapheme tokens
            "phonemes": phonemes_alias,               # Backward compat
            "alignment_method": alignment_method,     # "ctc_grapheme" or fallback
            "alignment_validation": alignment_validation,
            "quality_score": quality_score,
            "tajweed": tajweed_result,
            "metadata": {
                "asr_model": self.asr_model.model_name,
                "num_tokens": len(units),
                "duration_seconds": len(audio) / sample_rate,
                "surah": surah,
                "ayah": ayah
            }
        }

    def _compute_quality_score(
        self,
        units: List[Dict],
        alignment_validation: Dict,
        tajweed_result: Dict
    ) -> float:
        """
        Compute overall quality score (0-100).

        Combines:
        - Alignment confidence (40%): Mean posterior probability
        - LLR scores (30%): Discriminative confidence
        - Tajweed score (30%): Duration/energy rule violations

        Args:
            units: Aligned grapheme units with confidence
            alignment_validation: Validation result from CTCForcedAligner
            tajweed_result: Tajweed validation result

        Returns:
            Quality score (0-100)

        Examples:
            >>> score = engine._compute_quality_score(units, validation, tajweed)
            >>> 0 <= score <= 100
            True
        """
        if not units:
            return 0.0

        # Component 1: Alignment confidence (0-100)
        # Uses mean posterior probability from CTC
        alignment_score = alignment_validation.get("mean_confidence", 0.0) * 100

        # Component 2: LLR component (0-100)
        # LLR typically ranges from -10 to +5
        # Map to [0, 100] using sigmoid-like transformation
        llr_scores = [u.get("gop_score", -10.0) for u in units if "gop_score" in u]
        if llr_scores:
            mean_llr = float(np.mean(llr_scores))
            # Map LLR to 0-100: LLR > 0 → good, LLR < -2 → poor
            # Using: score = 50 + 25 * tanh(LLR/2)
            llr_score = 50 + 25 * np.tanh(mean_llr / 2.0)
            llr_score = max(0, min(100, llr_score))
        else:
            llr_score = 50.0  # Neutral if no LLR scores

        # Component 3: Tajweed component (already 0-100)
        tajweed_score = tajweed_result.get("overall_score", 0.0)

        # Weighted combination
        # Weights: alignment 40%, LLR 30%, tajweed 30%
        overall = 0.4 * alignment_score + 0.3 * llr_score + 0.3 * tajweed_score

        return float(np.clip(overall, 0, 100))


# Backward compatibility alias
AlignmentPipeline = ComparisonEngine
