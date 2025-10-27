# **M7: Comparison Engine Orchestrator (v2.1 Complete Specification)**

[← Back to M7 Overview](overview.md) | [↑ Navigation](../../NAVIGATION.md)

**Purpose**: Complete implementation of the ComparisonEngine class with two-path gating logic

## **1. Purpose**

This document specifies the complete implementation of the `ComparisonEngine` class. Its sole responsibility is to orchestrate the analysis pipeline by executing a two-path gating logic. It manages dependencies, runs the content gate, and, if passed, executes the full analysis pipeline, finally fusing the results into a single, schema-compliant response.

## **2. Critical Constraints (Non-Negotiable)**

1. **ASR/Alignment Model:** All content verification (M3.5) and forced alignment (M3) **MUST** use `obadx/recitation-segmenter-v2`.
      * `NO` Montreal Forced Aligner (MFA).
      * `NO` WhisperX or other models.
2. **Alignment Method:** Phoneme alignment (M3) **MUST** be performed using a CTC-based forced alignment (e.g., Viterbi or beam-search decoding) over the `obadx` logits, forced against the **ground-truth reference text**.
      * `NEVER` align against the ASR's hypothesis (transcript).
3. **Dependencies:** All sub-modules (Verifier, Aligner, Tajweed, Prosody) **MUST** be provided via **Dependency Injection (DI)** in the `__init__` method. The orchestrator must not instantiate its own dependencies.
4. **Determinism:** The entire pipeline must be deterministic. All component models must have randomness disabled or use a fixed seed.

## **3. Core Logic: Two-Path Decision Flow**

1. **Input:** `User Audio (16kHz, mono) + Reference Text`
2. **↓**
3. **M3.5: Content Verification** (via `self.content_verifier`)
      * Calculates `WER`, `transcript`, and `word_errors` using `obadx`.
4. **↓**
5. **GATING LOGIC:**
      * `if WER > M7Config.wer_threshold_fail (0.08):`
          * **→ PATH A:** STOP. Do not run any further analysis.
          * Generate feedback from `word_errors`.
          * Return `content_error` schema.
      * `else (WER ≤ 0.08):`
          * **→ PATH B:** PROCEED to full analysis.
6. **↓**
7. **M3: Forced Alignment** (via `self.aligner`)
      * Uses `obadx` logits + `reference_text` to get precise `phoneme_boundaries`.
8. **↓**
9. **M4: Tajweed Validation** (via `self.tajweed_modules`)
      * `M4.1 MaddValidator(phoneme_boundaries, audio)`
      * `M4.2 GhunnahClassifier(phoneme_boundaries, audio)`
      * `M4.3 QalqalahDetector(phoneme_boundaries, audio)`
      * Collects all `tajweed_results` and `tajweed_violations`.
10. **↓**
11. **M2/M5/M6: Prosody & Voice** (via `self.prosody_analyzer`)
      * `M2 Pitch (SwiftF0)`, `M5 Voice (OpenSMILE)`, `M6 Prosody`
      * Collects `prosody_results` and `prosody_feedback`.
12. **↓**
13. **Score Fusion:**
      * Calculate component scores (0-100) for Tajweed, Prosody, Pronunciation, and Voice.
      * Apply `M7Config.fusion_weights` to get `overall_score`.
14. **↓**
15. **Output:** Return `pronunciation_and_prosody` schema.

## **4. Configuration (`M7Config`)**

The `ComparisonEngine` **MUST** be initialized with a configuration object (e.g., a `dataclass` or `TypedDict`) matching this schema.

```python
# Defined in a shared config file, e.g., src/iqrah_audio/config.py
from typing import TypedDict, Dict, List, Tuple

class M7AlignConfig(TypedDict):
    decode: str         # "viterbi" | "beam"
    beam_size: int
    lm_weight: float    # Should be 0.0 unless approved

class M7RuntimeConfig(TypedDict):
    sample_rate_hz: int # 16000
    max_audio_s: float  # 30.0

class M7Config(TypedDict):
    wer_threshold_fail: float              # 0.08
    wer_threshold_warn: float              # 0.05
    fusion_weights: Dict[str, float]       # {"tajweed": 0.40, ...}
    score_range: Tuple[float, float]       # [0.0, 100.0]
    align: M7AlignConfig
    runtime: M7RuntimeConfig
```

## **5. Class Structure & Interfaces**

The implementation **MUST** adhere to this class structure using dependency injection.

```python
# File: src/iqrah_audio/orchestration/engine.py
import numpy as np
from datetime import datetime
from typing import List, Dict, Any

# --- Import Dependency Protocols (Abstract Base Classes) ---
# These define the interfaces for M3.5, M3, M4, etc.
from .interfaces import (
    M7Config,
    IContentVerifier,
    IForcedAligner,
    ITajweedValidator,
    IProsodyAnalyzer,
    AlignmentResult,
    TajweedResult,
    ProsodyResult
)

class ComparisonEngine:
    """
    Orchestrates the full recitation analysis pipeline with content gating,
    as specified in doc/01-architecture/m7-orchestrator.md.
    """

    def __init__(self,
                 config: M7Config,
                 content_verifier: IContentVerifier,
                 forced_aligner: IForcedAligner,
                 tajweed_modules: Dict[str, ITajweedValidator],
                 prosody_analyzer: IProsodyAnalyzer):
        """
        Initialize the engine with injected dependencies.
        """
        self.config = config
        self.content_verifier = content_verifier
        self.forced_aligner = forced_aligner
        self.tajweed_modules = tajweed_modules
        self.prosody_analyzer = prosody_analyzer

        # Assert fusion weights sum to 1.0
        total_weight = sum(self.config['fusion_weights'].values())
        if abs(total_weight - 1.0) > 1e-6:
            raise ValueError(f"Fusion weights must sum to 1.0, but sum to {total_weight}")

    def compare(self,
                student_audio: np.ndarray,
                reference_text: str,
                surah: int,
                ayah: int) -> Dict[str, Any]:
        """
        Perform full recitation comparison with content gating.

        Precondition: student_audio MUST be 16kHz, mono, float32.
        """
        try:
            # ============================================================
            # STAGE 1: CONTENT VERIFICATION (THE GATEKEEPER)
            # ============================================================
            content_result = self.content_verifier.verify(
                student_audio,
                reference_text
            )
            wer = content_result['wer']

            # ============================================================
            # DECISION GATE: EVALUATE WER
            # ============================================================
            if wer > self.config['wer_threshold_fail']:
                # --------------------------------------------------------
                # PATH A: FAILED GATE - Report content errors only
                # --------------------------------------------------------
                feedback, reco = self._generate_content_feedback(content_result['errors'])
                return {
                    "overall_score": 0.0,
                    "analysis_type": "content_error",
                    "wer": wer,
                    "transcript": content_result['transcript'],
                    "word_errors": content_result['errors'],
                    "feedback": feedback,
                    "recommendation": reco
                }

            # --------------------------------------------------------
            # PATH B: PASSED GATE - Proceed with full analysis
            # --------------------------------------------------------
            warnings = []
            if wer <= self.config['wer_threshold_warn']:
                analysis_confidence = "high"
            else:
                analysis_confidence = "medium"
                warnings.append(
                    f"Content accuracy is borderline (WER={wer:.1%}). "
                    "Some pronunciation feedback may be inaccurate."
                )

            # ============================================================
            # STAGE 2: FORCED ALIGNMENT (using ground-truth text)
            # ============================================================
            alignment = self.forced_aligner.align(
                audio=student_audio,
                reference_text=reference_text
            )

            # ============================================================
            # STAGE 3: TAJWEED RULE VALIDATION
            # ============================================================
            tajweed_results = {}
            for rule_name, validator in self.tajweed_modules.items():
                tajweed_results[rule_name] = validator.validate(
                    phonemes=alignment['phonemes'],
                    audio=student_audio
                )

            # ============================================================
            # STAGE 4: PROSODY & VOICE ANALYSIS
            # ============================================================
            prosody_results = self.prosody_analyzer.analyze(
                audio=student_audio,
                phonemes=alignment['phonemes'],
                surah=surah,
                ayah=ayah
            )

            # ============================================================
            # STAGE 5: SCORE FUSION
            # ============================================================
            component_scores, overall_score = self._run_score_fusion(
                tajweed_results,
                prosody_results,
                alignment
            )

            # ============================================================
            # RETURN FULL ANALYSIS
            # ============================================================
            return {
                "overall_score": overall_score,
                "analysis_type": "pronunciation_and_prosody",
                "analysis_confidence": analysis_confidence,
                "wer": wer,
                "transcript": content_result['transcript'],
                "component_scores": component_scores,
                "tajweed_violations": self._extract_violations(tajweed_results),
                "prosody_feedback": prosody_results['feedback'],
                "warnings": warnings,
                "timestamp": datetime.now().isoformat()
            }

        except (ContentVerificationError, AlignmentError, DownstreamModuleError) as e:
            # Handle catastrophic failures gracefully
            return {
                "overall_score": 0.0,
                "analysis_type": "system_error",
                "error_message": str(e),
                "recommendation": "An internal error occurred. Please try again or contact support."
            }

    # --- Internal Helper Methods ---

    def _run_score_fusion(self, tajweed_results, prosody_results, alignment) -> (Dict, float):
        """Compute all component scores and the final weighted overall score."""
        # ... logic to call scoring helpers ...
        component_scores = {
            "tajweed": self._score_tajweed(tajweed_results),
            "prosody": self._score_prosody(prosody_results),
            "pronunciation": self._score_pronunciation(alignment),
            "voice_quality": self._score_voice(prosody_results)
        }

        w = self.config['fusion_weights']
        overall_score = (
            w['tajweed'] * component_scores['tajweed'] +
            w['prosody'] * component_scores['prosody'] +
            w['pronunciation'] * component_scores['pronunciation'] +
            w['voice_quality'] * component_scores['voice_quality']
        )
        # Ensure score is within [0, 100] range
        min_score, max_score = self.config['score_range']
        overall_score = max(min_score, min(max_score, overall_score))

        return component_scores, overall_score

    def _score_tajweed(self, tajweed_results: Dict) -> float:
        """Compute aggregate Tajweed score (0-100)"""
        # ... logic to aggregate scores/violations from results ...
        return 0.0 # Placeholder

    def _score_prosody(self, prosody_results: Dict) -> float:
        """Compute aggregate prosody score (0-100)"""
        # ... logic to use prosody_results['scores']['melody_score'] etc. ...
        return 0.0 # Placeholder

    def _score_pronunciation(self, alignment: Dict) -> float:
        """
        Compute GOP-style pronunciation score (0-100) from alignment.
        Formula (e.g., average log-likelihood) MUST be documented here.
        """
        # ... logic ...
        return 0.0 # Placeholder

    def _score_voice(self, prosody_results: Dict) -> float:
        """Compute voice quality score (0-100)"""
        # ... logic to use prosody_results['scores']['quality_score'] ...
        return 0.0 # Placeholder

    def _generate_content_feedback(self, errors: List[Dict]) -> (str, str):
        """Generate user-friendly feedback for content errors."""
        # ... logic from prompt 1's _generate_content_feedback ...
        return "Feedback string", "Recommendation string" # Placeholder

    def _extract_violations(self, tajweed_results: Dict) -> List[Dict]:
        """Extract user-facing violation details from raw results."""
        # ... logic from prompt 1's _extract_violations ...
        return [] # Placeholder

# --- Custom Exceptions ---

class ContentVerificationError(Exception):
    pass

class AlignmentError(Exception):
    pass

class DownstreamModuleError(Exception):
    pass

```

---

## **6. I/O Schemas (Contracts)**

The `compare` method's return value **MUST** strictly adhere to one of these two JSON schemas.

### **Path A: Content Error Response**

```json
{
  "overall_score": 0.0,
  "analysis_type": "content_error",
  "wer": 0.12,
  "transcript": "الرحمن الرحيم ملك",
  "word_errors": [
    {
      "type": "deletion",
      "reference_word": "الحمد",
      "recited_word": null,
      "position": 0
    }
  ],
  "feedback": "Content verification failed. 1 error(s) detected: Missing word: 'الحمد'",
  "recommendation": "Review the correct verse text before attempting pronunciation analysis"
}
```

### **Path B: Full Analysis Response**

```json
{
  "overall_score": 82.5,
  "analysis_type": "pronunciation_and_prosody",
  "analysis_confidence": "high",
  "wer": 0.02,
  "transcript": "الحمد لله رب العالمين",
  "component_scores": {
    "tajweed": 80.0,
    "prosody": 85.0,
    "pronunciation": 90.0,
    "voice_quality": 75.0
  },
  "tajweed_violations": [
    {
      "rule": "madd",
      "phoneme": "a:",
      "expected": { "duration_ms": 120 },
      "actual": { "duration_ms": 80 },
      "severity": "moderate",
      "time_range": [1.25, 1.33],
      "feedback": "The Madd (elongation) on 'a:' was too short."
    }
  ],
  "prosody_feedback": {
    "rhythm": { "score": 88.0, "feedback": "Good pacing." },
    "melody": { "score": 82.0, "feedback": "Melody contour generally follows the reference." },
    "style": { "score": 75.0, "feedback": "Voice quality is clear." }
  },
  "warnings": [],
  "timestamp": "2025-10-26T14:30:00.000Z"
}
```

## **7. Testing Requirements**

Tests for `tests/orchestration/test_engine.py` **MUST** cover:

1. **Gating Logic:**
      * `test_gate_blocks_on_high_wer`: Mock `IContentVerifier` to return `wer=0.10`. Assert `analysis_type == "content_error"` and `IForcedAligner.align` is **NOT** called.
      * `test_gate_warns_on_medium_wer`: Mock `wer=0.07`. Assert `analysis_type == "pronunciation_and_prosody"`, `analysis_confidence == "medium"`, and `len(warnings) > 0`.
      * `test_gate_passes_on_low_wer`: Mock `wer=0.03`. Assert `analysis_type == "pronunciation_and_prosody"` and `analysis_confidence == "high"`.
2. **Fusion Logic:**
      * `test_score_fusion_weights`: Mock all internal `_score_...` methods to return fixed values (e.g., 90, 80, 85, 75). Assert `overall_score` is exactly `(90*0.4 + 80*0.3 + 85*0.2 + 75*0.1) = 84.5`.
      * `test_fusion_weight_sum_assertion`: Assert `ComparisonEngine` raises `ValueError` if `fusion_weights` in config do not sum to 1.0.
3. **Schema Compliance:**
      * `test_path_a_schema`: The output from the high-WER test must perfectly match all keys, types, and enums in the "Path A" schema.
      * `test_path_b_schema`: The output from the low-WER test must perfectly match all keys, types, and enums in the "Path B" schema.
4. **Error Handling:**
      * `test_alignment_error_handling`: Mock `IForcedAligner.align` to raise `AlignmentError`. Assert `compare()` catches it and returns `analysis_type == "system_error"`.
      * `test_tajweed_error_handling`: Mock `ITajweedValidator.validate` to raise `DownstreamModuleError`. Assert `compare()` catches it and returns `analysis_type == "system_error"`.
5. **Latency (Phase 1):**
      * `test_latency_path_a_fast`: A 10s audio clip resulting in Path A (`wer > 0.08`) **MUST** return in `< 1.0s`.
      * `test_latency_path_b_full`: A 10s audio clip resulting in Path B (`wer <= 0.08`) **MUST** return in `< 5.0s`.

---
**Related**: [Gatekeeper Rationale](gatekeeper-rationale.md) | [Comparison Methods](comparison-methods.md) | [← M7 Overview](overview.md)
