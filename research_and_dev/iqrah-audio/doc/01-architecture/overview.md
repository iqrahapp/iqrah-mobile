# IQRAH AUDIO - Architecture Overview

[← Back to Project Root](../../README.md) | [↑ Navigation](../NAVIGATION.md)

---

## SYSTEM SPECIFICATIONS

**Timeline**: 2025-2028 (3-year commitment)
**Phase**: Offline E2E → Real-time → Mobile
**Generated**: 2025-10-23

**Targets**:
- Accuracy: 90%+ all basic Tajweed
- Latency: <5s offline, <500ms real-time, <300ms mobile
- Modularity: 8 black-box components
- Rollout: Basic → Advanced → Prosody → Real-time

---

## 8-MODULE ARCHITECTURE

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   M1: Audio     │───▶│   M2: Pitch     │───▶│  M3: Phoneme    │
│  Preprocessing  │    │   Extraction    │    │   Alignment     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                                                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  M4: Tajweed    │◀───│   M5: Voice     │◀───│  M3.5: Content  │
│   Validators    │    │    Quality      │    │   Verification  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        │                      │
        └──────────┬───────────┘
                   ▼
         ┌─────────────────┐
         │  M6: Prosodic   │
         │    Analysis     │
         └─────────────────┘
                   │
                   ▼
         ┌─────────────────┐
         │  M7: Comparison │
         │     Engine      │
         │  (Orchestrator) │
         └─────────────────┘
                   │
                   ▼
         ┌─────────────────┐
         │  M8: Feedback   │
         │   Generation    │
         └─────────────────┘
```

**Data Flow**:
1. Raw audio → M1 (preprocessing) → Clean 16kHz audio
2. Clean audio → M2 (pitch) → F0 contour
3. Reference text → Phonetizer → Phonetic reference
4. Clean audio + phonetic reference → M3 (Muaalem) → Phonemes + Sifat (Tajweed properties)
5. M3.5 (Phonetic gatekeeper) → Verify content accuracy (PER check)
6. If PER > 5%: STOP, report content errors
7. If PER ≤ 5%: Proceed to M4-M6 (Tajweed, voice quality, prosody)
8. M4 (Tajweed) → Two-tier validation (Baseline sifat + Specialized modules)
9. M7 (comparison engine) → Orchestrates all analyses, fuses scores
10. M8 (feedback) → User-friendly output with recommendations

---

## MODULE INTERFACE CONTRACTS

| Module | Input | Output | Detailed Docs |
|--------|-------|--------|---------------|
| **M1: Audio Preprocessing** | Raw audio (MP3/WAV/WebM), optional metadata | Clean 16kHz audio, VAD segments, quality metrics | [m1-preprocessing.md](m1-preprocessing.md) |
| **M2: Pitch Extraction** | Preprocessed 16kHz audio | F0 contour, confidence, voicing mask, stats | [m2-pitch.md](m2-pitch.md) |
| **M3: Phoneme Alignment** | Audio + phonetic reference | Aligned phonemes with timestamps + sifat, word boundaries, gate result (PER) | [m3-phoneme-alignment.md](m3-phoneme-alignment.md) |
| **M4: Tajweed Validators** | Aligned phonemes + sifat + audio | Two-tier violations: Tier 1 baseline (10+ rules, 70-85%) + Tier 2 specialized (Madd 95%+, Ghunnah 90%+) | [m4-tajweed.md](m4-tajweed.md) |
| **M5: Voice Quality** | Audio + phonemes | Timbre, breathiness, vibrato, embeddings | [m5-voice-quality.md](m5-voice-quality.md) |
| **M6: Prosodic Analysis** | Audio + phonemes + pitch | Rhythm metrics, melody models, style features | [m6-prosody.md](m6-prosody.md) |
| **M7: Comparison Engine** | Student features + reference | Overall score, component scores, violations | [m7-comparison-engine.md](m7-comparison-engine.md) |
| **M8: Feedback Generation** | Comparison results + proficiency level | Summary, detailed feedback, next steps | [m8-feedback.md](m8-feedback.md) |

---

## TECHNOLOGY STACK SUMMARY

### Core Libraries

| Purpose | Library | Version | Notes |
|---------|---------|---------|-------|
| Audio I/O | soundfile, librosa | >=0.12.1, >=0.10.0 | Fast load/resample |
| Pitch | swift-f0, rmvpe | Latest | SwiftF0 primary |
| Alignment | transformers | >=4.35.0 | Muaalem (pre-trained) |
| Prosody | opensmile, praat-parselmouth | >=3.0.1, >=0.4.3 | eGeMAPS, formants |
| Style | speechbrain | >=0.5.16 | X-vectors |
| Rhythm | tslearn | >=0.6.0 | Soft-DTW |
| ML/DL | torch, scipy, scikit-learn | >=2.0.0, >=1.10.0, >=1.3.0 | Core ML |
| Web | fastapi, uvicorn | >=0.100.0, >=0.23.0 | REST API |
| DB | sqlalchemy, psycopg2 | >=2.0.0, >=2.9.0 | PostgreSQL |

### Model Zoo

| Model | Size | Purpose | Source |
|-------|------|---------|--------|
| Muaalem v3.2 (pre-trained) | 2.2GB | Phoneme + Sifat recognition | HuggingFace Hub (obadx) |
| SwiftF0 | 0.4MB | Pitch extraction | PyPI |
| RMVPE | 50MB | Pitch fallback | GitHub |
| Maqam CNN | 10MB | Maqam recognition | Custom (optional) |
| X-vector | 20MB | Style embeddings | SpeechBrain |

---

## PROGRESSIVE ROLLOUT

### PHASE 1: OFFLINE E2E (Months 1-4) ← START

**Goal**: 90% accuracy comprehensive Tajweed (10+ rules), prosody analysis

**Deliverables**:
1. Preprocessing pipeline (M1)
2. SwiftF0 + RMVPE (M2)
3. Muaalem integration: Phonemes + Sifat <2% PER (M3)
4. Two-tier Tajweed: Tier 1 baseline (10+ rules, 70-85%) + Tier 2 specialized (Madd 95%+, Ghunnah 90%+) (M4)
5. Voice quality + prosody (M5-M6)
6. Comparison engine (M7)
7. Feedback generation (M8)
8. Validation: 100 expert cases r > 0.75

**Outcome**: Production-ready desktop system with comprehensive Tajweed
**Cost**: €500-1,000 (expert validation only, no GPU training needed)

### PHASE 2: REAL-TIME (Months 7-12)

**Goal**: <500ms latency, streaming
**Key Techniques**: INT8 quantization, ONNX Runtime, Redis caching, GPU acceleration

### PHASE 3: MOBILE (Months 13-18)

**Goal**: On-device inference <300ms
**Key Techniques**: Model distillation, TFLite/CoreML, hybrid architecture

---

## ARCHITECTURE STRENGTHS

✅ **Modularity**: Each component independent, testable, swappable
✅ **SOTA Integration**: Latest research (Wav2Vec2-BERT, SwiftF0, OpenSMILE)
✅ **Progressive Rollout**: Start simple, scale to real-time and mobile
✅ **AI-Agent Friendly**: Clear interfaces, minimal context bleed
✅ **Validation-First**: Accuracy targets, expert validation, user studies
✅ **Production-Ready**: Edge cases, graceful degradation, monitoring

**Commitment**: This design is stable for 3 years. Focus on execution, not redesign.

---

## DETAILED MODULE DOCUMENTATION

- [Module M1: Audio Preprocessing](m1-preprocessing.md)
- [Module M2: Pitch Extraction](m2-pitch.md)
- [Module M3: Phoneme Alignment](m3-phoneme-alignment.md)
- [Module M4: Tajweed Validators](m4-tajweed.md)
- [Module M5: Voice Quality Analysis](m5-voice-quality.md)
- [Module M6: Prosodic Analysis](m6-prosody.md)
- [Module M7: Comparison Engine](m7-comparison-engine.md)
- [Module M8: Feedback Generation](m8-feedback.md)

---

**Next**: [Module M1: Audio Preprocessing](m1-preprocessing.md) | [↑ Navigation](../NAVIGATION.md)
