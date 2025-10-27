# Module M7: Comparison Engine (Orchestrator)

[← Back to Architecture](../overview.md) | [↑ Navigation](../../NAVIGATION.md)

## Overview

The Comparison Engine (M7) is the central orchestrator for the analysis pipeline. It implements the **"fail-fast" principle**:

1. **Verify Content (M3.5):** First, check *what* was said using ASR and WER.
2. **Analyze Pronunciation (M3, M4, M2/M5/M6):** Only if the content is correct, proceed to analyze *how* it was said.

This approach prevents invalid feedback by ensuring pronunciation rules are never applied to the wrong words.

## Architecture: Two-Path Gating

The engine's core logic is a two-path decision flow based on the Word Error Rate (WER) from the M3.5 Content Verifier.

### Decision Gate

```
User Audio + Reference Text
│
▼
M3.5: Content Verification (obadx)
│
▼
┌────────┐
│WER \> 8%│───YES──→ PATH A: STOP
└───┬────┘           Report content errors only
│
NO (WER ≤ 8%)
│
┌───▼────┐
│5% \< WER│───YES──→ PATH B: PROCEED
│  ≤ 8%  │           Flag "medium confidence"
└───┬────┘
│
NO (WER ≤ 5%)
│
▼
PATH B: PROCEED
Flag "high confidence"
```

### PATH A: Content Error Response (WER > 8%)

* **Action:** STOP analysis immediately. Do not run alignment or Tajweed modules.
* **Response:** Return `analysis_type: "content_error"` with the WER, transcript, and a list of `word_errors` (deletions, insertions, substitutions).
* **Rationale:** Prevents analyzing incorrect content, which would produce nonsensical pronunciation feedback.

### PATH B: Full Pronunciation Analysis (WER ≤ 8%)

* **Action:** PROCEED with the full analysis pipeline.
* **Confidence:**
  * `WER ≤ 5%`: Set `analysis_confidence: "high"`.
  * `5% < WER ≤ 8%`: Set `analysis_confidence: "medium"` and include a user-facing warning.
* **Pipeline:**
    1. **M3 Forced Alignment:** Generate precise phoneme boundaries using `obadx/recitation-segmenter-v2` (via CTC forced alignment) against the **ground-truth reference text**.
    2. **M4 Tajweed Validation:** Run `Madd`, `Ghunnah`, and `Qalqalah` modules on the aligned phonemes.
    3. **M2/M5/M6 Prosody & Voice:** Run pitch, quality, and rhythm analysis.
    4. **Score Fusion:** Combine all component scores into a final, weighted `overall_score`.

## M7 Module Documentation

This overview is the entry point. The detailed specifications are in the following files:

**1. ➡️ [Orchestrator Implementation](orchestrator.md)**

* **Use For:** Implementing the `ComparisonEngine` class.
* **Contains:** The complete, actionable specification, `M7Config`, I/O schemas, dependency injection contracts, fusion weights, and exact test requirements.

**2. 🧐 [Gatekeeper Rationale](gatekeeper-rationale.md)**

* **Use For:** Understanding *why* this architecture was chosen.
* **Contains:** Design decisions, rationale for the two-stage (ASR + Aligner) gate, and trade-offs.

**3. 🧮 [Comparison Methods](comparison-methods.md)**

* **Use For:** Implementing the downstream scoring algorithms.
* **Contains:** Detailed algorithms for M7.1 (Tajweed scoring), M7.2 (Prosody scoring), etc.

---
**Next**: [Module M8: Feedback](../m8-feedback.md) | [← Back](../overview.md)
