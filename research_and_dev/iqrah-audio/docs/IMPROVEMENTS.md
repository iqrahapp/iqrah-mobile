# Iqrah Audio - SOTA Improvements Roadmap

## Current State Analysis

### Strengths ✅
- Solid offline analysis pipeline
- Good baseline with CREPE/YIN pitch tracking
- DTW alignment working
- Mobile-ready CBOR serialization
- Comprehensive scoring metrics

### Gaps to Fill for SOTA 🎯

#### 1. **Pitch Tracking Accuracy**
**Current:** CREPE (tiny model) + YIN fallback
**Target:** RMVPE/FCPE for vocals + octave correction
**Impact:** ↑ 15-25% accuracy on noisy/breathy recitations

**Issues:**
- CREPE trained on general audio, not optimized for vocals
- No octave error correction beyond median filtering
- Single-model approach (no ensemble)
- Missing confidence calibration

**Solutions:**
- Add **RMVPE** (Robust Multi-Period Variational Pitch Estimation) - SOTA for vocals
- Add **FCPE** (Fast Crepe with Extensions) - faster, more accurate
- Implement multi-octave tracking with automatic correction
- Add confidence-weighted pitch smoothing

#### 2. **Real-Time Performance**
**Current:** RTF ~0.12 offline, no streaming
**Target:** RTF < 0.05 with streaming, <100ms latency
**Impact:** Enable real-time coaching

**Issues:**
- No streaming architecture
- Full DTW recomputation each time
- No frame-level caching
- Heavy scipy dependencies

**Solutions:**
- Implement **Online-DTW** with ring buffer
- Add **anchor-based alignment** (silence, plosives, long notes)
- Use **Sakoe-Chiba band** with dynamic width
- Implement **frame-level feature caching**
- Port critical paths to numba/C

#### 3. **Noise Robustness**
**Current:** Spectral gating (noisereduce)
**Target:** Neural denoising (RNNoise/DeepFilterNet) + adaptive filtering
**Impact:** ↑ 30-40% accuracy in noisy environments (mosque, home)

**Issues:**
- Spectral gating damages pitch information
- No adaptive noise profiling
- Single-pass denoising
- No wind/breath noise handling

**Solutions:**
- Integrate **RNNoise** (C library, fast, proven)
- Add **DeepFilterNet3** (SOTA neural denoising)
- Implement **adaptive wiener filtering**
- Add **harmonic/percussive separation** (HPSS) for breath removal

#### 4. **Feature Extraction**
**Current:** F0 + confidence only
**Target:** Multi-dimensional (F0 + timbre + energy + voicing)
**Impact:** ↑ 20% alignment accuracy, enable tajweed detection

**Issues:**
- No timbre features for alignment cost
- Missing energy/loudness tracking
- No spectral features for consonants
- Can't detect qalqalah, ghunna, etc.

**Solutions:**
- Add **log-mel spectrogram** (64-80 bins) for timbre
- Add **chroma features** (12-bin) for octave-robust matching
- Add **RMS energy** + **spectral centroid**
- Add **ZCR** (zero-crossing rate) for voicing
- Extract **spectral flatness** for qalqalah detection

#### 5. **Scoring System**
**Current:** Basic weighted average (alignment + on-note + stability + tempo)
**Target:** Phoneme-level GOP + tajweed rules + explainable feedback
**Impact:** Letter-by-letter accuracy, actionable coaching

**Issues:**
- No phoneme-level alignment
- No tajweed rule detection
- Coarse scoring (overall only)
- No confidence weighting
- No octave error handling in scoring

**Solutions:**
- Integrate **Arabic CTC-ASR** (Conformer) for forced alignment
- Implement **GOP** (Goodness of Pronunciation) scores
- Add **tajweed detectors** (madd, ghunna, qalqalah, idghām)
- Implement **confidence-weighted metrics**
- Add **octave-aware pitch error** calculation

#### 6. **Mobile Optimization**
**Current:** Python only, no quantization, no ONNX
**Target:** ONNX models, INT8 quantization, <30MB total, RTF<0.3 on Snapdragon 7-series
**Impact:** Deployable on mid-range Android phones

**Issues:**
- Python-only (not mobile)
- No model quantization
- Large model files
- No ARM optimizations

**Solutions:**
- Export all models to **ONNX**
- Apply **INT8 quantization** (maintain <1pp accuracy loss)
- Implement **pruning** for pitch models
- Use **NNAPI/CoreML** delegates
- Profile on real Android devices

---

## Implementation Plan

### Phase 1: Accuracy Improvements (Week 1-2)

#### 1.1 Upgrade Pitch Tracking
- [ ] Add RMVPE model integration
- [ ] Add FCPE (Fast CREPE) variant
- [ ] Implement multi-octave tracking
- [ ] Add confidence calibration
- [ ] Benchmark: target ±10 cents median error on clean speech

**Files to modify:**
- `src/iqrah_audio/pitch.py` - Add RMVPE/FCPE extractors
- `pyproject.toml` - Add dependencies (torchcrepe, rmvpe)

#### 1.2 Multi-Dimensional Features
- [ ] Add mel-spectrogram extraction (librosa)
- [ ] Add chroma features
- [ ] Add energy (RMS) tracking
- [ ] Add spectral features (centroid, flatness, ZCR)
- [ ] Integrate into DTW cost function

**New files:**
- `src/iqrah_audio/features.py` - Feature extraction module

#### 1.3 Advanced DTW Cost Function
- [ ] Implement weighted cost: `wF*|ΔF0| + wT*(1-cos_sim(mel)) + wE*|ΔE|`
- [ ] Add confidence weighting
- [ ] Add octave-aware pitch distance
- [ ] Tune weights: start with `wF=0.6, wT=0.3, wE=0.1`

**Files to modify:**
- `src/iqrah_audio/dtw.py` - Update cost function

### Phase 2: Real-Time Optimizations (Week 2-3)

#### 2.1 Streaming Architecture
- [ ] Implement ring buffer for audio frames
- [ ] Add frame-level feature extraction
- [ ] Implement sliding window DTW
- [ ] Add anchor detection (silence, plosives)
- [ ] Add confidence gating (freeze when conf < threshold)

**Files to modify:**
- `src/iqrah_audio/dtw.py` - Enhance OnlineDTWAligner
- New: `src/iqrah_audio/anchors.py` - Anchor detection

#### 2.2 Performance Optimization
- [ ] Profile current pipeline (cProfile + line_profiler)
- [ ] Port DTW cost to numba JIT
- [ ] Implement feature caching
- [ ] Optimize memory allocations
- [ ] Target: RTF < 0.05

**New files:**
- `benchmarks/performance_profile.py`

### Phase 3: Noise Robustness (Week 3-4)

#### 3.1 Neural Denoising
- [ ] Integrate RNNoise (via rnnoise-python or cffi)
- [ ] Add DeepFilterNet3 (optional, heavier)
- [ ] Implement adaptive noise gate
- [ ] Add harmonic/percussive separation
- [ ] Benchmark SNR improvement: target >10dB

**Files to modify:**
- `src/iqrah_audio/denoise.py` - Add neural denoisers

### Phase 4: Phoneme-Level Analysis (Week 4-6)

#### 4.1 Arabic CTC ASR
- [ ] Research models: wav2vec2-arabic, Conformer-CTC
- [ ] Download/fine-tune on Quranic audio (everyayah.com)
- [ ] Export to ONNX
- [ ] Implement forced alignment
- [ ] Integrate with `ciborium` for Rust compatibility

**New files:**
- `models/asr/` - CTC models and scripts
- `src/iqrah_audio/asr.py` - CTC inference

#### 4.2 GOP Scoring
- [ ] Extract CTC posteriors
- [ ] Implement logit-based GOP
- [ ] Calibrate thresholds
- [ ] Aggregate to word/ayah level

**New files:**
- `src/iqrah_audio/gop.py`

#### 4.3 Tajweed Detectors
- [ ] Madd detector (duration-based)
- [ ] Ghunna detector (nasal energy band ~200-300 Hz)
- [ ] Qalqalah detector (spectral flatness on stops)
- [ ] Idghām/Ikhfā'/Iqlāb (context + coarticulation)

**New files:**
- `src/iqrah_audio/tajweed.py`

### Phase 5: Mobile Deployment (Week 6-7)

#### 5.1 Model Export
- [ ] Export RMVPE to ONNX
- [ ] Export CTC ASR to ONNX
- [ ] Quantize to INT8
- [ ] Validate accuracy (<1pp regression)

**New files:**
- `tools/export_onnx.py`

#### 5.2 Rust Integration Prep
- [ ] Document CBOR schema
- [ ] Create example Rust loader (ciborium)
- [ ] Benchmark ONNX inference (onnxruntime-rs)

### Phase 6: Evaluation (Week 7-8)

#### 6.1 Test Suite
- [ ] Download real Quranic audio (Husary, Minshawi)
- [ ] Collect user recordings (different skill levels)
- [ ] Add noise augmentation (crowd, AC, wind)
- [ ] Create golden test set

**New files:**
- `tests/test_real_audio.py`
- `data/test_audio/`

#### 6.2 Benchmarking
- [ ] Pitch accuracy (MAE in cents)
- [ ] Alignment accuracy (boundary MAE)
- [ ] Noise robustness (SNR vs accuracy)
- [ ] Speed (RTF on different devices)
- [ ] Memory usage

**New files:**
- `benchmarks/accuracy_benchmark.py`
- `benchmarks/performance_benchmark.py`

---

## Success Metrics

### Accuracy Targets
- **Pitch MAE**: <10 cents on clean, <25 cents at 5dB SNR
- **Alignment boundary MAE**: <40ms
- **On-note %**: >90% for accurate reciters
- **GOP correlation with experts**: >0.7

### Performance Targets
- **Real-time factor (RTF)**: <0.05 streaming, <0.3 offline on mobile
- **Latency**: <100ms visual feedback
- **Memory**: <200MB RAM peak on mobile
- **Model size**: <30MB total (all models)

### Robustness Targets
- **SNR improvement**: >10dB with denoising
- **Accuracy at 5dB SNR**: >70% of clean performance
- **Octave error rate**: <1%

---

## Key Dependencies to Add

```toml
[project.dependencies]
# SOTA pitch tracking
"torchcrepe>=0.3.0",           # Fast CREPE implementation
"torchfcpe>=1.0.0",            # FCPE (if available)
# Note: RMVPE requires custom integration

# Neural denoising
"rnnoise-python>=0.1.0",       # RNNoise wrapper
# "denoiser>=0.1.0",           # Facebook Denoiser (optional)

# ASR
"onnxruntime>=1.16.0",         # ONNX inference
"transformers>=4.35.0",        # For model loading
"phonemizer>=3.2.0",           # Phoneme conversion

# Performance
"numba>=0.58.0",               # JIT compilation
"line-profiler>=4.1.0",        # Profiling

# Features
"essentia>=2.1b6.dev",         # Advanced audio features (optional)
```

---

## Risk Mitigation

### Technical Risks
1. **RMVPE integration complexity** → Start with torchcrepe (easier), add RMVPE later
2. **Arabic CTC model quality** → Use pre-trained wav2vec2, fine-tune minimally
3. **Real-time performance on mobile** → Profile early, optimize incrementally
4. **Octave errors** → Implement chroma backup, confidence gating

### Scope Risks
1. **Feature creep** → Stick to phased plan, ship Phase 1-2 first
2. **Perfectionism** → Target 90% accuracy, not 100%
3. **Data availability** → Use everyayah.com, synthetic data for edge cases

---

## Next Immediate Actions

1. **Run current tests** to establish baseline
2. **Create performance benchmark** script
3. **Add RMVPE/torchcrepe** integration
4. **Implement multi-dimensional features**
5. **Test on real Quranic audio**

This plan balances **accuracy**, **performance**, and **mobile readiness** to create a truly SOTA Quranic recitation analysis tool.
