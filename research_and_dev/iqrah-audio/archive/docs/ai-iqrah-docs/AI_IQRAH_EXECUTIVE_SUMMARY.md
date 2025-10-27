# IQRAH AUDIO - EXECUTIVE REFERENCE
**Generated**: 2025-10-23
**Stability Commitment**: 3-year architecture (2025-2028)

---

## DOCUMENT PACKAGE (4 DOCS)

### 1. SOTA_ARCHITECTURE (35k words)
Technical specification. All algorithms, models, interfaces, decisions.

### 2. TASK_DECOMPOSITION (15k words)
100+ concrete tasks with dependencies, estimates, priorities.

### 3. IMPLEMENTATION_GUIDE (10k words)
Decision rationale, AI agent strategies, risk mitigation.

### 4. EXECUTIVE_SUMMARY (this)
5-minute overview.

---

## CORE SPECS

**Objective**: Phoneme-level Quranic recitation analysis

**Competitive Edge**: 100× more precise than word-level systems (Tarteel.ai, TajweedMate)

**Target Accuracy**:
- PER: <1%
- Madd: 99%+
- Ghunnah: 85%+
- Qalqalah: 80%+
- Correlation vs expert: r > 0.7

**Target Latency**:
- Offline: <5s
- Real-time: <500ms
- Mobile: <300ms

**Target User Metrics**:
- Rating: 4.5+
- Retention (30-day): 70%+

---

## 8-MODULE ARCHITECTURE

```
INPUT → M1_PREPROCESSING → M2_PITCH → M3_PHONEME → M3.5_CONTENT_VERIFY →
M4_TAJWEED → M5_VOICE_QUALITY → M6_PROSODY → M7_COMPARISON → M8_FEEDBACK → OUTPUT

Note: M3.5 acts as a "gate" - if content verification fails (WER > 8%),
flow jumps directly to M8 for content error feedback, skipping M4-M7.
```

### M1: Preprocessing
- Audio normalization (16kHz)
- Silero VAD (250ms min segment)
- SNR estimation, clipping detection
- SHA256 caching

### M2: Pitch Extraction
- SwiftF0 primary (91.8% accuracy, 42× CREPE speed)
- RMVPE fallback (conf<0.7)
- 10ms hop, 46-2093Hz range
- Median filter 5-frame

### M3: Phoneme Alignment
- Wav2Vec2-BERT fine-tuned (target: <1% PER)
- CTC forced alignment
- Windowed per-word
- GOP scoring

### M4: Tajweed Validation
- **Madd**: Rule-based duration (99% target)
- **Ghunnah**: Formant + MLP (85% target)
- **Qalqalah**: Burst detection + SVM (80% target)

### M5: Voice Quality
- OpenSMILE eGeMAPS (88-d)
- Vibrato: rate, extent, regularity
- Breathiness: H1-H2, HNR, CPP
- X-vectors (512-d), Wav2Vec2-CLS (768-d)

### M6: Prosody
- **Rhythm**: Soft-DTW, nPVI, Varco, IOI
- **Melody**: Fujisaki decomposition, tilt, maqam CNN
- **Style**: X-vectors, GST tokens

### M7: Comparison Engine
Multi-dimensional fusion:
- Tajweed: 40%
- Prosody: 30%
- Pronunciation: 20%
- Voice quality: 10%

### M8: Feedback Generation
Pedagogical text, progress tracking (PostgreSQL), actionable recommendations

---

## TECHNOLOGY STACK

### Models
| Model         | Size  | Target          |
| ------------- | ----- | --------------- |
| Wav2Vec2-BERT | 2.2GB | <1% PER         |
| SwiftF0       | 0.4MB | 91.8% accuracy  |
| RMVPE         | 50MB  | Fallback        |
| Ghunnah MLP   | 1MB   | 85% accuracy    |
| Qalqalah SVM  | 5MB   | 80% accuracy    |
| Maqam CNN     | 10MB  | 90% accuracy    |
| X-vector      | 20MB  | Style embedding |

### Libraries
```python
soundfile>=0.12.1, librosa>=0.10.0
swift-f0, rmvpe
transformers>=4.35.0, ctc-forced-aligner>=0.1
opensmile>=3.0.1, praat-parselmouth>=0.4.3
speechbrain>=0.5.16, tslearn>=0.6.0
torch>=2.0.0, scipy>=1.10.0, scikit-learn>=1.3.0
fastapi>=0.100.0, sqlalchemy>=2.0.0
```

### Infrastructure
- Compute: Lambda Labs / RunPod (GPU)
- Storage: S3 / MinIO
- Database: PostgreSQL
- Cache: Redis
- Monitoring: Prometheus + Grafana

---

## 3-PHASE ROADMAP

### PHASE 1: OFFLINE E2E (Months 1-6) ← START HERE
**Goal**: 90% accuracy on basic Tajweed, comprehensive prosody

**Deliverables**:
- Wav2Vec2-BERT <1% PER
- Madd 99%, Ghunnah 85%, Qalqalah 80%
- Voice quality + prosody analysis
- Comparison engine
- Feedback generation
- Validation: 100 expert-rated cases (r > 0.7)

**Estimated Cost**: €1,000-2,000 (GPU + expert validation)

### PHASE 2: REAL-TIME (Months 7-12)
**Goal**: <500ms latency, streaming

**Deliverables**:
- WebSocket streaming
- INT8 quantization (4× speedup, <2% accuracy loss)
- ONNX Runtime (2-3× speedup)
- GPU acceleration (A100/T4)
- Redis cache (<100ms lookup)
- 10+ concurrent users

**Estimated Cost**: €500-1,000/month (GPU) + €5,000 optimization

### PHASE 3: MOBILE (Months 13-18)
**Goal**: On-device inference, <300ms

**Deliverables**:
- Model distillation (student <100MB)
- INT8/INT4 quantization
- iOS (CoreML) + Android (TFLite)
- Hybrid: on-device basic, server advanced
- Offline mode

**Estimated Cost**: €1,000-2,000 (distillation) + 3-6 months dev

---

## SOTA RESEARCH INTEGRATION

| Capability | SOTA Source                      | Implementation      | Target |
| ---------- | -------------------------------- | ------------------- | ------ |
| Phoneme    | AraS2P (Sep 2025): 0.16% PER     | Wav2Vec2-BERT + QPS | <1%    |
| Pitch      | SwiftF0 (2025): 91.8%, 42× CREPE | SwiftF0 + RMVPE     | 91%+   |
| Madd       | 2024 research: 99.87%            | Rule-based duration | 99%+   |
| Ghunnah    | 2020: 71-85%                     | Formant + MLP       | 85%+   |
| Prosody    | OpenSMILE eGeMAPS                | 88-d features       | r>0.7  |
| Maqam      | CNN on Maqam478: 90%+            | CNN chroma+MFCCs    | 90%+   |

---

## AI AGENT DELEGATION

### HIGH SUCCESS (60-70% of tasks)
**Pure implementation**: Audio loaders, feature extraction, integration
**Data processing**: Prepare datasets, precompute features
**Template-based**: Feedback generation, progress tracking

### MEDIUM SUCCESS (requires guidance)
Training scripts, optimization algorithms, multi-component integration

### HUMAN REQUIRED (30-40% of tasks)
Research, expert annotation, architecture decisions, user studies

---

## SUCCESS METRICS DASHBOARD

### Technical KPIs

| Metric      | Phase 1 | Phase 2 | Phase 3 |
| ----------- | ------- | ------- | ------- |
| PER         | <1%     | <1%     | <2%     |
| Madd        | 99%+    | 99%+    | 99%+    |
| Ghunnah     | 85%+    | 85%+    | 80%+    |
| Qalqalah    | 80%+    | 80%+    | 75%+    |
| Correlation | r>0.7   | r>0.7   | r>0.65  |
| Latency p95 | <5s     | <500ms  | <300ms  |
| Memory      | <4GB    | <2GB    | <200MB  |

### User KPIs

| Metric          | Alpha (N=10) | Beta (N=50) | Production (1000+) |
| --------------- | ------------ | ----------- | ------------------ |
| Retention (30d) | 60%+         | 70%+        | 70%+               |
| Rating          | 4.0+         | 4.3+        | 4.5+               |
| Weekly active   | 70%+         | 50%+        | 30%+               |
| Improvement     | Anecdotal    | d>0.3       | d>0.5              |

### Business KPIs

| Metric     | 6mo         | 12mo         | 18mo        |
| ---------- | ----------- | ------------ | ----------- |
| B2B pilots | 1-2 schools | 5-10 schools | 20+ schools |
| Students   | 50-100      | 500-1,000    | 5,000+      |
| Revenue    | €0          | €1,000-5,000 | €10,000+    |

---

## WEEK 1 SETUP

### Main Developer
1. Setup infrastructure (AWS/Lambda Labs, HuggingFace)
2. Download Tarteel dataset (~100GB)
3. Setup PostgreSQL
4. Begin Wav2Vec2 training
5. Coordinate AI agents

### AI Agent 1: T1.1.1 - Audio preprocessing (Day 3 deadline)
### AI Agent 2: T2.1.1 - SwiftF0 integration (Day 2 deadline)
### AI Agent 3: T5.1.1 - OpenSMILE wrapper (Day 4 deadline)

---

## MILESTONES

### M1: Training Complete (Week 4)
Wav2Vec2-BERT <1% PER, model checkpoint saved

### M2: Madd Perfect (Week 8)
99%+ accuracy on all 5 madd types

### M3: Full Pipeline (Week 16)
All 8 modules integrated, ghunnah + qalqalah working

### M4: Validation Study (Week 24)
100 expert-rated cases: r > 0.7

### M5: Real-Time (Month 12)
<500ms latency, 10+ concurrent users

### M6: Mobile Launch (Month 18)
iOS + Android in stores, 1000+ downloads week 1

---

## CRITICAL RISKS

| Risk                    | Mitigation              | Check             |
| ----------------------- | ----------------------- | ----------------- |
| Wav2Vec2 training fails | MMS fallback            | Week 2 curves     |
| Ghunnah <70%            | More training data      | Week 8 validation |
| Real-time >500ms        | Aggressive caching      | Monthly latency   |
| GPU cost exceeds budget | Spot instances, caching | Monthly bills     |

---

## STABILITY CONSTRAINTS

### FIXED (cannot change)
- 8-module architecture
- Module interfaces (input/output contracts)
- Progressive rollout (Offline → Real-time → Mobile)
- Accuracy targets (PER <1%, rules >80%)
- Validation methodology (r > 0.7)

### VARIABLE (can adjust)
- Hyperparameters, model sizes
- UI/UX, visualization styles
- Infrastructure (AWS vs GCP)
- Deployment strategies

---

## EXTERNAL RESOURCES

- HuggingFace Transformers: https://huggingface.co/docs/transformers
- Tarteel Dataset: https://huggingface.co/datasets/Salama1429/tarteel-ai-everyayah-Quran
- OpenSMILE: https://github.com/audeering/opensmile
- ArTST v2: https://arxiv.org/abs/2410.17924
- AraS2P: https://arxiv.org/abs/2509.23504
- SwiftF0: https://github.com/swift-f0
