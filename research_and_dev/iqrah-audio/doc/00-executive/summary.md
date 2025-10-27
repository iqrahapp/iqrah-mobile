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
- PER: <2% (using pre-trained Muaalem)
- Madd: 95%+ (Tier 2 probabilistic)
- Ghunnah: 70-85% (Tier 1 baseline) → 90%+ (Tier 2 formants)
- Qalqalah: 75-80% (Tier 1 baseline) → 85%+ (Tier 2 burst detection)
- Comprehensive Tajweed: 10+ rules from Day 1 (via Muaalem sifat)
- Correlation vs expert: r > 0.75

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
INPUT → M1_PREPROCESSING → M2_PITCH → M3_PHONEME → M4_TAJWEED → 
M5_VOICE_QUALITY → M6_PROSODY → M7_COMPARISON → M8_FEEDBACK → OUTPUT
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
- Muaalem v3.2 pre-trained (phonemes + sifat output)
- Phonetic-first approach (not grapheme-based)
- CTC forced alignment (phoneme-level + word-level)
- Phonetic gatekeeper using PER (Phoneme Error Rate)

### M4: Tajweed Validation
- **Two-Tier Architecture**:
  - **Tier 1 Baseline**: Free sifat from Muaalem (10+ rules, 70-85% accuracy)
  - **Tier 2 Specialized**: Pluggable modules for advanced detection
- **Madd (PRIORITY 1)**: Probabilistic duration modeling (95%+ target)
- **Ghunnah (PRIORITY 2)**: Baseline + formant analysis (90%+ target)
- **Qalqalah (PRIORITY 3)**: Baseline + burst detection (85%+ target)

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
| Model | Size | Target |
|-------|------|--------|
| Muaalem v3.2 (pre-trained) | 2.2GB | <2% PER + comprehensive sifat |
| SwiftF0 | 0.4MB | 91.8% accuracy |
| RMVPE | 50MB | Fallback |
| Maqam CNN (optional) | 10MB | 90% accuracy |
| X-vector | 20MB | Style embedding |

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

### PHASE 1: OFFLINE E2E (Months 1-4) ← START HERE
**Goal**: 90% accuracy on comprehensive Tajweed (10+ rules), prosody analysis

**Deliverables**:
- Muaalem integration: Phonemes + Sifat <2% PER (no training required)
- Two-tier Tajweed: Tier 1 baseline (10+ rules, 70-85%) + Tier 2 specialized (Madd 95%+, Ghunnah 90%+)
- Voice quality + prosody analysis
- Comparison engine
- Feedback generation
- Validation: 100 expert-rated cases (r > 0.75)

**Estimated Cost**: €500-1,000 (expert validation only, no GPU training costs)

### PHASE 2: REAL-TIME (Months 5-10)
**Goal**: <500ms latency, streaming

**Deliverables**:
- WebSocket streaming
- INT8 quantization (4× speedup, <2% accuracy loss)
- ONNX Runtime (2-3× speedup)
- GPU acceleration (A100/T4)
- Redis cache (<100ms lookup)
- 10+ concurrent users

**Estimated Cost**: €500-1,000/month (GPU) + €5,000 optimization

### PHASE 3: MOBILE (Months 11-16)
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

| Capability | SOTA Source | Implementation | Target |
|------------|-------------|----------------|--------|
| Phoneme | Muaalem v3.2: Phonemes + Sifat | Pre-trained (no training) | <2% |
| Pitch | SwiftF0 (2025): 91.8%, 42× CREPE | SwiftF0 + RMVPE | 91%+ |
| Madd | 2024 research: 99.87% | Probabilistic duration (Tier 2) | 95%+ |
| Ghunnah | Muaalem baseline + formants | Two-tier (70-85% → 90%+) | 90%+ |
| Prosody | OpenSMILE eGeMAPS | 88-d features | r>0.7 |
| Maqam | CNN on Maqam478: 90%+ | CNN chroma+MFCCs | 90%+ |

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

| Metric | Phase 1 | Phase 2 | Phase 3 |
|--------|---------|---------|---------|
| PER | <2% | <2% | <3% |
| Madd | 95%+ | 95%+ | 95%+ |
| Ghunnah | 70-85% (T1) → 90%+ (T2) | 90%+ | 85%+ |
| Qalqalah | 75-80% (T1) → 85%+ (T2) | 85%+ | 80%+ |
| Correlation | r>0.75 | r>0.75 | r>0.70 |
| Latency p95 | <5s | <500ms | <300ms |
| Memory | <4GB | <2GB | <200MB |

### User KPIs

| Metric | Alpha (N=10) | Beta (N=50) | Production (1000+) |
|--------|--------------|-------------|-------------------|
| Retention (30d) | 60%+ | 70%+ | 70%+ |
| Rating | 4.0+ | 4.3+ | 4.5+ |
| Weekly active | 70%+ | 50%+ | 30%+ |
| Improvement | Anecdotal | d>0.3 | d>0.5 |

### Business KPIs

| Metric | 6mo | 12mo | 18mo |
|--------|-----|------|------|
| B2B pilots | 1-2 schools | 5-10 schools | 20+ schools |
| Students | 50-100 | 500-1,000 | 5,000+ |
| Revenue | €0 | €1,000-5,000 | €10,000+ |

---

## WEEK 1 SETUP

### Main Developer
1. Setup infrastructure (HuggingFace, compute environment)
2. Download Muaalem model (obadx/muaalem-model-v3_2)
3. Setup PostgreSQL (for Madd distributions)
4. Extract quran_phonetizer from obadx/quran-muaalem
5. Coordinate AI agents

### AI Agent 1: T1.1.1 - Audio preprocessing (Day 3 deadline)
### AI Agent 2: T2.1.1 - SwiftF0 integration (Day 2 deadline)
### AI Agent 3: T5.1.1 - OpenSMILE wrapper (Day 4 deadline)

---

## MILESTONES

### M1: Muaalem Integration (Week 4)
Phonemes + Sifat working, PER <2%, phonetic gate functional

### M2: Tajweed MVP (Week 7)
Tier 1 baseline (10+ rules, 70-85%) + Madd Tier 2 (95%+) working

### M3: Full Pipeline (Week 14)
All 8 modules integrated, comprehensive Tajweed + prosody

### M4: Validation Study (Week 18)
100 expert-rated cases: r > 0.75

### M5: Real-Time (Month 10)
<500ms latency, 10+ concurrent users

### M6: Mobile Launch (Month 16)
iOS + Android in stores, 1000+ downloads week 1

---

## CRITICAL RISKS

| Risk | Mitigation | Check |
|------|------------|-------|
| Muaalem PER >5% on learners | Fine-tune in Phase 2 | Week 4 validation |
| Ghunnah Tier 1 <70% | Enable Tier 2 formants | Week 7 validation |
| Real-time >500ms | Aggressive caching | Monthly latency |
| GPU cost exceeds budget | Spot instances, caching | Monthly bills |

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
