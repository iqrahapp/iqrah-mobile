# Iqrah Audio - Task Decomposition & Development Roadmap

**Purpose:** Break down architecture into concrete, AI-agent-assignable tasks  
**Generated:** 2025-10-23

---

## Task Hierarchy Visualization

```mermaid
graph TB
    subgraph "PHASE 1: OFFLINE E2E PIPELINE (Months 1-6)"
        A[Phase 1 Complete]
        
        subgraph "M1: Audio Preprocessing"
            M1[Module 1: Preprocessing] --> M1.1[Audio Loading & Validation]
            M1 --> M1.2[Resampling & Normalization]
            M1 --> M1.3[VAD Implementation]
            M1 --> M1.4[Noise Reduction]
            M1 --> M1.5[Quality Checks]
            M1 --> M1.6[Caching System]
            
            M1.1 --> T1.1.1[Support MP3/WAV/WebM/M4A]
            M1.1 --> T1.1.2[Corrupt file detection]
            M1.1 --> T1.1.3[Metadata extraction]
            
            M1.2 --> T1.2.1[Kaiser window resampling to 16kHz]
            M1.2 --> T1.2.2[Peak normalization]
            M1.2 --> T1.2.3[LUFS normalization]
            
            M1.3 --> T1.3.1[Integrate Silero VAD ONNX]
            M1.3 --> T1.3.2[Segment extraction 250ms min]
            M1.3 --> T1.3.3[Silence trimming]
            
            M1.4 --> T1.4.1[SNR estimation]
            M1.4 --> T1.4.2[Spectral subtraction]
            M1.4 --> T1.4.3[Conditional application SNR<15dB]
            
            M1.5 --> T1.5.1[Clipping detection >5%]
            M1.5 --> T1.5.2[Dynamic range analysis]
            M1.5 --> T1.5.3[Quality flag generation]
            
            M1.6 --> T1.6.1[SHA256 hash-based cache]
            M1.6 --> T1.6.2[Cache hit/miss logging]
            M1.6 --> T1.6.3[TTL management]
        end
        
        subgraph "M2: Pitch Extraction"
            M2[Module 2: Pitch] --> M2.1[SwiftF0 Integration]
            M2 --> M2.2[RMVPE Fallback]
            M2 --> M2.3[Confidence Weighting]
            M2 --> M2.4[Pitch Smoothing]
            M2 --> M2.5[Statistics Computation]
            
            M2.1 --> T2.1.1[Install swift-f0 package]
            M2.1 --> T2.1.2[Implement extraction 10ms hop]
            M2.1 --> T2.1.3[Handle 46-2093Hz range]
            M2.1 --> T2.1.4[Median filter 5-frame]
            
            M2.2 --> T2.2.1[Install RMVPE from GitHub]
            M2.2 --> T2.2.2[Trigger on conf<0.7]
            M2.2 --> T2.2.3[Deep U-Net inference]
            
            M2.3 --> T2.3.1[Inverse variance weighting]
            M2.3 --> T2.3.2[Blending function]
            M2.3 --> T2.3.3[Confidence thresholding]
            
            M2.4 --> T2.4.1[Savitzky-Golay filter p=3, w=51ms]
            M2.4 --> T2.4.2[Linear interpolation <100ms gaps]
            M2.4 --> T2.4.3[Octave jump removal]
            
            M2.5 --> T2.5.1[Mean/std Hz]
            M2.5 --> T2.5.2[Pitch range min/max]
            M2.5 --> T2.5.3[Voiced ratio]
        end
        
        subgraph "M3: Phoneme Alignment"
            M3[Module 3: Alignment] --> M3.1[Wav2Vec2-BERT Training]
            M3 --> M3.2[CTC Forced Alignment]
            M3 --> M3.3[Tajweed Mapping]
            M3 --> M3.4[GOP Scoring]
            M3 --> M3.5[Quality Validation]
            
            M3.1 --> T3.1.1[Download Tarteel-ai-everyayah dataset]
            M3.1 --> T3.1.2[Stage 1: Continue pretrain 50-100hrs]
            M3.1 --> T3.1.3[Generate phoneme labels MSA Phonetiser]
            M3.1 --> T3.1.4[Stage 2: Fine-tune CTC head]
            M3.1 --> T3.1.5[Validate PER <1% on test set]
            
            M3.2 --> T3.2.1[Integrate ctc-forced-aligner]
            M3.2 --> T3.2.2[Windowed alignment per word]
            M3.2 --> T3.2.3[HMM boundary smoothing]
            M3.2 --> T3.2.4[GPU acceleration]
            
            M3.3 --> T3.3.1[Load quran-phoneme-tajweed.json]
            M3.3 --> T3.3.2[Fuzzy matching edit distance]
            M3.3 --> T3.3.3[Rule label assignment]
            M3.3 --> T3.3.4[Expected duration lookup]
            
            M3.4 --> T3.4.1[Extract CTC posteriors]
            M3.4 --> T3.4.2[Compute log P phoneme | audio]
            M3.4 --> T3.4.3[Per-phoneme GOP scores]
            M3.4 --> T3.4.4[Threshold calibration]
            
            M3.5 --> T3.5.1[Mean confidence >0.7 check]
            M3.5 --> T3.5.2[Phoneme count ±10% validation]
            M3.5 --> T3.5.3[Duration sanity 20-500ms]
            M3.5 --> T3.5.4[Warning generation]
        end
        
        subgraph "M4: Tajweed Validators"
            M4[Module 4: Tajweed] --> M4.1[Madd Duration Validator]
            M4 --> M4.2[Ghunnah Validator]
            M4 --> M4.3[Qalqalah Validator]
            M4 --> M4.4[Complex Rules]
            
            M4.1 --> T4.1.1[Define duration thresholds per type]
            M4.1 --> T4.1.2[Tolerance ±20% implementation]
            M4.1 --> T4.1.3[Violation severity scoring]
            M4.1 --> T4.1.4[Validate 99% accuracy on test]
            
            M4.2 --> T4.2.1[Collect 1000+ labeled examples]
            M4.2 --> T4.2.2[Extract formant F1/F2/F3 Parselmouth]
            M4.2 --> T4.2.3[Nasal energy 250-350Hz]
            M4.2 --> T4.2.4[Train MLP classifier 64-32 hidden]
            M4.2 --> T4.2.5[Validate 85% accuracy]
            
            M4.3 --> T4.3.1[Collect 500 qalqalah examples]
            M4.3 --> T4.3.2[Extract ZCR, spectral centroid, RMS]
            M4.3 --> T4.3.3[Burst detection algorithm]
            M4.3 --> T4.3.4[Train SVM classifier]
            M4.3 --> T4.3.5[Validate 80% accuracy]
            
            M4.4 --> T4.4.1[Research idghaam acoustic correlates]
            M4.4 --> T4.4.2[Research ikhfaa detection]
            M4.4 --> T4.4.3[Defer to Phase 3]
        end
        
        subgraph "M5: Voice Quality Analysis"
            M5[Module 5: Voice Quality] --> M5.1[OpenSMILE Integration]
            M5 --> M5.2[Vibrato Detection]
            M5 --> M5.3[Breathiness Features]
            M5 --> M5.4[Timbre Features]
            M5 --> M5.5[Neural Embeddings]
            
            M5.1 --> T5.1.1[Install opensmile Python]
            M5.1 --> T5.1.2[Extract eGeMAPS 88-d features]
            M5.1 --> T5.1.3[Batch processing optimization]
            
            M5.2 --> T5.2.1[Bandpass filter 2-15Hz pitch]
            M5.2 --> T5.2.2[Autocorrelation rate detection]
            M5.2 --> T5.2.3[Extent amplitude calculation]
            M5.2 --> T5.2.4[Regularity CV computation]
            
            M5.3 --> T5.3.1[H1-H2 harmonic difference]
            M5.3 --> T5.3.2[HNR computation Parselmouth]
            M5.3 --> T5.3.3[CPP cepstral prominence]
            M5.3 --> T5.3.4[Spectral tilt -10 to -15dB/oct]
            
            M5.4 --> T5.4.1[Spectral centroid 2-4kHz]
            M5.4 --> T5.4.2[Spectral flux]
            M5.4 --> T5.4.3[Spectral rolloff 85%]
            M5.4 --> T5.4.4[Formant tracking F1-F4]
            
            M5.5 --> T5.5.1[X-vector SpeechBrain 512-d]
            M5.5 --> T5.5.2[Wav2Vec2 CLS token 768-d]
            M5.5 --> T5.5.3[Embedding similarity metrics]
        end
        
        subgraph "M6: Prosodic Analysis"
            M6[Module 6: Prosody] --> M6.1[Advanced Rhythm]
            M6 --> M6.2[Melody Analysis]
            M6 --> M6.3[Style Characterization]
            
            M6.1 --> T6.1.1[nPVI computation]
            M6.1 --> T6.1.2[Varco coefficient variation]
            M6.1 --> T6.1.3[IOI distribution extraction]
            M6.1 --> T6.1.4[Isochrony scoring]
            M6.1 --> T6.1.5[Soft-DTW alignment integration]
            
            M6.2 --> T6.2.1[Fujisaki model fitting scipy]
            M6.2 --> T6.2.2[Phrase/accent command extraction]
            M6.2 --> T6.2.3[Declination exponential fit]
            M6.2 --> T6.2.4[Tilt parametrization]
            M6.2 --> T6.2.5[Contour shape classification]
            M6.2 --> T6.2.6[Maqam CNN training Maqam478]
            
            M6.3 --> T6.3.1[X-vector extraction]
            M6.3 --> T6.3.2[GST token extraction optional]
            M6.3 --> T6.3.3[Style label classification]
        end
        
        subgraph "M7: Comparison Engine"
            M7[Module 7: Comparison] --> M7.1[Tajweed Comparison]
            M7 --> M7.2[Prosody Comparison]
            M7 --> M7.3[Pronunciation Comparison]
            M7 --> M7.4[Voice Quality Comparison]
            M7 --> M7.5[Fusion Algorithm]
            
            M7.1 --> T7.1.1[Aggregate validator results]
            M7.1 --> T7.1.2[Weighted scoring madd/ghunnah/qalqalah]
            M7.1 --> T7.1.3[Violation severity ranking]
            
            M7.2 --> T7.2.1[Rhythm DTW + nPVI + IOI Wasserstein]
            M7.2 --> T7.2.2[Melody Fujisaki + tilt + maqam]
            M7.2 --> T7.2.3[Style X-vector cosine similarity]
            M7.2 --> T7.2.4[Weighted prosody score]
            
            M7.3 --> T7.3.1[GOP delta computation]
            M7.3 --> T7.3.2[Phoneme accuracy CTC confidence]
            M7.3 --> T7.3.3[Per-phoneme violation flagging]
            
            M7.4 --> T7.4.1[Timbre spectral centroid diff]
            M7.4 --> T7.4.2[Vibrato rate matching]
            M7.4 --> T7.4.3[Breathiness HNR comparison]
            
            M7.5 --> T7.5.1[Overall weighted fusion 40/30/20/10]
            M7.5 --> T7.5.2[Confidence interval computation]
            M7.5 --> T7.5.3[User profile weights beginner/advanced]
        end
        
        subgraph "M8: Feedback Generation"
            M8[Module 8: Feedback] --> M8.1[Summary Generation]
            M8 --> M8.2[Detailed Feedback]
            M8 --> M8.3[Progress Tracking]
            M8 --> M8.4[Recommendations]
            
            M8.1 --> T8.1.1[Overall score interpretation]
            M8.1 --> T8.1.2[Strongest/weakest component identification]
            M8.1 --> T8.1.3[2-3 sentence summary template]
            
            M8.2 --> T8.2.1[Per-violation feedback templates]
            M8.2 --> T8.2.2[Timestamp formatting]
            M8.2 --> T8.2.3[How-to-fix instructions]
            M8.2 --> T8.2.4[Audio snippet extraction]
            
            M8.3 --> T8.3.1[PostgreSQL schema attempts table]
            M8.3 --> T8.3.2[Record attempt function]
            M8.3 --> T8.3.3[Improvement delta calculation]
            M8.3 --> T8.3.4[Streak computation]
            M8.3 --> T8.3.5[Personal best tracking]
            
            M8.4 --> T8.4.1[Critical violation prioritization]
            M8.4 --> T8.4.2[Weakest component focus]
            M8.4 --> T8.4.3[Progressive difficulty suggestions]
            M8.4 --> T8.4.4[Consistency encouragement]
        end
        
        subgraph "V1: Validation"
            V1[Validation & Testing] --> V1.1[Accuracy Validation]
            V1 --> V1.2[Performance Benchmarking]
            V1 --> V1.3[User Testing]
            
            V1.1 --> T-V1.1.1[Create test set 100 expert-annotated ayahs]
            V1.1 --> T-V1.1.2[PER measurement <1%]
            V1.1 --> T-V1.1.3[Boundary precision 20/50ms]
            V1.1 --> T-V1.1.4[Tajweed rule accuracy per type]
            V1.1 --> T-V1.1.5[Correlation study r>0.7 vs human ratings]
            
            V1.2 --> T-V1.2.1[Latency benchmarking script]
            V1.2 --> T-V1.2.2[p50/p95/p99 percentiles]
            V1.2 --> T-V1.2.3[CPU vs GPU comparison]
            V1.2 --> T-V1.2.4[Memory profiling]
            
            V1.3 --> T-V1.3.1[Alpha testing 10 internal users]
            V1.3 --> T-V1.3.2[Qualitative feedback collection]
            V1.3 --> T-V1.3.3[Beta testing 50-100 users]
            V1.3 --> T-V1.3.4[A/B testing feedback styles]
            V1.3 --> T-V1.3.5[Formal validation study N=60-100]
        end
        
        A --> M1
        A --> M2
        A --> M3
        A --> M4
        A --> M5
        A --> M6
        A --> M7
        A --> M8
        A --> V1
    end
    
    subgraph "PHASE 2: REAL-TIME OPTIMIZATION (Months 7-12)"
        B[Phase 2 Complete]
        
        subgraph "RT1: Streaming Architecture"
            RT1[Streaming System] --> RT1.1[WebSocket Implementation]
            RT1 --> RT1.2[VAD-Based Chunking]
            RT1 --> RT1.3[Incremental Processing]
            
            RT1.1 --> T-RT1.1.1[FastAPI WebSocket endpoint]
            RT1.1 --> T-RT1.1.2[Bi-directional protocol design]
            RT1.1 --> T-RT1.1.3[Connection management]
            
            RT1.2 --> T-RT1.2.1[5s overlapping windows]
            RT1.2 --> T-RT1.2.2[Silence removal real-time]
            RT1.2 --> T-RT1.2.3[Buffer management]
            
            RT1.3 --> T-RT1.3.1[Incremental phoneme alignment]
            RT1.3 --> T-RT1.3.2[Streaming feature extraction]
            RT1.3 --> T-RT1.3.3[Progressive feedback emission]
        end
        
        subgraph "RT2: Model Optimization"
            RT2[Model Optimization] --> RT2.1[Quantization]
            RT2 --> RT2.2[ONNX Conversion]
            RT2 --> RT2.3[TensorRT]
            RT2 --> RT2.4[Model Pruning]
            
            RT2.1 --> T-RT2.1.1[INT8 quantization Wav2Vec2]
            RT2.1 --> T-RT2.1.2[Validation <2% accuracy loss]
            RT2.1 --> T-RT2.1.3[4x speedup verification]
            
            RT2.2 --> T-RT2.2.1[Export to ONNX format]
            RT2.2 --> T-RT2.2.2[ONNX Runtime integration]
            RT2.2 --> T-RT2.2.3[2-3x speedup validation]
            
            RT2.3 --> T-RT2.3.1[TensorRT optimization NVIDIA]
            RT2.3 --> T-RT2.3.2[FP16 precision testing]
            RT2.3 --> T-RT2.3.3[Inference engine integration]
            
            RT2.4 --> T-RT2.4.1[Magnitude pruning 30%]
            RT2.4 --> T-RT2.4.2[Structured pruning channels]
            RT2.4 --> T-RT2.4.3[Fine-tune after pruning]
        end
        
        subgraph "RT3: Caching & CDN"
            RT3[Caching Strategy] --> RT3.1[Reference Precomputation]
            RT3 --> RT3.2[Redis Integration]
            RT3 --> RT3.3[CDN Setup]
            
            RT3.1 --> T-RT3.1.1[Precompute all 6236 ayahs]
            RT3.1 --> T-RT3.1.2[Store features pitch/phonemes/prosody]
            RT3.1 --> T-RT3.1.3[Version control cache keys]
            
            RT3.2 --> T-RT3.2.1[Redis cluster setup]
            RT3.2 --> T-RT3.2.2[Cache hit/miss logic]
            RT3.2 --> T-RT3.2.3[TTL expiration 30 days]
            
            RT3.3 --> T-RT3.3.1[CloudFlare or AWS CloudFront]
            RT3.3 --> T-RT3.3.2[Reference audio distribution]
            RT3.3 --> T-RT3.3.3[Geographic replication]
        end
        
        subgraph "RT4: Infrastructure"
            RT4[Production Infrastructure] --> RT4.1[GPU Server Setup]
            RT4 --> RT4.2[Load Balancing]
            RT4 --> RT4.3[Monitoring]
            
            RT4.1 --> T-RT4.1.1[Provision A100/T4 AWS/CoreWeave]
            RT4.1 --> T-RT4.1.2[Docker containerization]
            RT4.1 --> T-RT4.1.3[Auto-scaling policies]
            
            RT4.2 --> T-RT4.2.1[NGINX reverse proxy]
            RT4.2 --> T-RT4.2.2[Round-robin distribution]
            RT4.2 --> T-RT4.2.3[Health check endpoints]
            
            RT4.3 --> T-RT4.3.1[Prometheus metrics collection]
            RT4.3 --> T-RT4.3.2[Grafana dashboards]
            RT4.3 --> T-RT4.3.3[Alerting rules latency>500ms]
        end
        
        subgraph "RT5: Real-Time Validation"
            RT5[RT Validation] --> RT5.1[Latency Testing]
            RT5 --> RT5.2[Stress Testing]
            RT5 --> RT5.3[User Pilot]
            
            RT5.1 --> T-RT5.1.1[End-to-end <500ms p95 validation]
            RT5.1 --> T-RT5.1.2[Per-component latency breakdown]
            RT5.1 --> T-RT5.1.3[Network latency profiling]
            
            RT5.2 --> T-RT5.2.1[Load test 100 concurrent users]
            RT5.2 --> T-RT5.2.2[Sustained load 1hr test]
            RT5.2 --> T-RT5.2.3[Failure recovery testing]
            
            RT5.3 --> T-RT5.3.1[Pilot 20 users real-time mode]
            RT5.3 --> T-RT5.3.2[UX feedback collection]
            RT5.3 --> T-RT5.3.3[Iterate on responsiveness]
        end
        
        B --> RT1
        B --> RT2
        B --> RT3
        B --> RT4
        B --> RT5
    end
    
    subgraph "PHASE 3: MOBILE DEPLOYMENT (Months 13-18)"
        C[Phase 3 Complete]
        
        subgraph "MB1: Model Distillation"
            MB1[Mobile Models] --> MB1.1[Student Model Training]
            MB1 --> MB1.2[Knowledge Distillation]
            MB1 --> MB1.3[Mobile Quantization]
            
            MB1.1 --> T-MB1.1.1[Design student architecture <100M params]
            MB1.1 --> T-MB1.1.2[Train from scratch on Quranic data]
            MB1.1 --> T-MB1.1.3[Validate PER <2%]
            
            MB1.2 --> T-MB1.2.1[Teacher Wav2Vec2-BERT outputs]
            MB1.2 --> T-MB1.2.2[Soft target distillation]
            MB1.2 --> T-MB1.2.3[Fine-tune student with KL loss]
            
            MB1.3 --> T-MB1.3.1[INT8 quantization TFLite/CoreML]
            MB1.3 --> T-MB1.3.2[Model size <50MB validation]
            MB1.3 --> T-MB1.3.3[Accuracy retention >98%]
        end
        
        subgraph "MB2: On-Device Inference"
            MB2[On-Device System] --> MB2.1[iOS CoreML]
            MB2 --> MB2.2[Android TFLite]
            MB2 --> MB2.3[Hybrid Architecture]
            
            MB2.1 --> T-MB2.1.1[CoreML conversion coremltools]
            MB2.1 --> T-MB2.1.2[Neural Engine optimization]
            MB2.1 --> T-MB2.1.3[Swift inference wrapper]
            
            MB2.2 --> T-MB2.2.1[TFLite conversion]
            MB2.2 --> T-MB2.2.2[NNAPI GPU delegate]
            MB2.2 --> T-MB2.2.3[Kotlin inference wrapper]
            
            MB2.3 --> T-MB2.3.1[On-device phoneme alignment]
            MB2.3 --> T-MB2.3.2[On-device madd validation]
            MB2.3 --> T-MB2.3.3[Server-side prosody/style]
            MB2.3 --> T-MB2.3.4[Offline mode basic feedback]
        end
        
        subgraph "MB3: Mobile SDK"
            MB3[Mobile SDK] --> MB3.1[React Native/Flutter]
            MB3 --> MB3.2[Audio Recording]
            MB3 --> MB3.3[Real-Time Visualization]
            MB3 --> MB3.4[Backend API]
            
            MB3.1 --> T-MB3.1.1[Cross-platform framework choice]
            MB3.1 --> T-MB3.1.2[Native module bindings]
            MB3.1 --> T-MB3.1.3[UI component library]
            
            MB3.2 --> T-MB3.2.1[Microphone permission handling]
            MB3.2 --> T-MB3.2.2[16kHz mono recording]
            MB3.2 --> T-MB3.2.3[Chunked upload WebRTC]
            
            MB3.3 --> T-MB3.3.1[Real-time pitch overlay]
            MB3.3 --> T-MB3.3.2[Phoneme cursor tracking]
            MB3.3 --> T-MB3.3.3[Tajweed color highlighting]
            
            MB3.4 --> T-MB3.4.1[REST API endpoints mobile-specific]
            MB3.4 --> T-MB3.4.2[WebSocket streaming]
            MB3.4 --> T-MB3.4.3[Offline sync queue]
        end
        
        subgraph "MB4: Mobile Validation"
            MB4[Mobile Testing] --> MB4.1[Device Testing]
            MB4 --> MB4.2[Performance Profiling]
            MB4 --> MB4.3[Beta Deployment]
            
            MB4.1 --> T-MB4.1.1[Test matrix 10 devices iOS/Android]
            MB4.1 --> T-MB4.1.2[OS version compatibility]
            MB4.1 --> T-MB4.1.3[Screen size adaptation]
            
            MB4.2 --> T-MB4.2.1[Latency <300ms validation]
            MB4.2 --> T-MB4.2.2[Battery consumption profiling]
            MB4.2 --> T-MB4.2.3[Memory usage <200MB]
            
            MB4.3 --> T-MB4.3.1[TestFlight beta iOS]
            MB4.3 --> T-MB4.3.2[Google Play internal testing]
            MB4.3 --> T-MB4.3.3[Crash analytics Firebase]
            MB4.3 --> T-MB4.3.4[User feedback collection]
        end
        
        C --> MB1
        C --> MB1
        C --> MB2
        C --> MB3
        C --> MB4
    end
    
    M1 --> M2
    M2 --> M3
    M3 --> M4
    M3 --> M5
    M5 --> M6
    M4 --> M7
    M6 --> M7
    M7 --> M8
    M8 --> V1
    
    V1 --> RT1
    RT1 --> RT2
    RT2 --> RT3
    RT3 --> RT4
    RT4 --> RT5
    
    RT5 --> MB1
    MB1 --> MB2
    MB2 --> MB3
    MB3 --> MB4
    
    style M1 fill:#90EE90
    style M2 fill:#90EE90
    style M3 fill:#FFD700
    style M4 fill:#FFA500
    style M5 fill:#FFA500
    style M6 fill:#FFA500
    style M7 fill:#FFD700
    style M8 fill:#FFA500
    style V1 fill:#FF6347
    
    style RT1 fill:#D3D3D3
    style RT2 fill:#D3D3D3
    style RT3 fill:#D3D3D3
    style RT4 fill:#D3D3D3
    style RT5 fill:#D3D3D3
    
    style MB1 fill:#D3D3D3
    style MB2 fill:#D3D3D3
    style MB3 fill:#D3D3D3
    style MB4 fill:#D3D3D3
```

**Legend:**
- 🟢 Green: Complete/Mostly Done
- 🟡 Yellow: In Progress
- 🟠 Orange: Not Started (Phase 1)
- ⚪ Gray: Future (Phase 2-3)
- 🔴 Red: Critical Path

---

## Priority Matrix

### Immediate Priority (Weeks 1-4)

| Task ID | Description | Effort | Dependencies | AI-Agent Friendly? |
|---------|-------------|--------|--------------|-------------------|
| T3.1.1 | Download Tarteel dataset | 1 day | None | ✅ Yes - Clear script |
| T3.1.2 | Continue pretrain Wav2Vec2-BERT | 3-5 days | T3.1.1 | ✅ Yes - Training script |
| T4.1.1-4 | Madd duration validator | 2 days | M3 complete | ✅ Yes - Pure logic |
| T5.1.1-3 | OpenSMILE integration | 1-2 days | None | ✅ Yes - Library wrapper |
| T8.1.1-3 | Summary generation templates | 1 day | None | ✅ Yes - Template strings |

### Medium Priority (Weeks 5-12)

| Task ID | Description | Effort | Dependencies | AI-Agent Friendly? |
|---------|-------------|--------|--------------|-------------------|
| T4.2.1-5 | Ghunnah validator complete | 2 weeks | T4.2.1 data collection | ⚠️ Partial - Needs data |
| T6.1.1-5 | Advanced rhythm analysis | 1 week | M5 complete | ✅ Yes - Pure math |
| T6.2.1-6 | Melody analysis Fujisaki+maqam | 2 weeks | M2 complete | ⚠️ Partial - Optimization hard |
| T7.1.1-3 | Tajweed comparison | 3 days | M4 complete | ✅ Yes - Clear logic |
| T-V1.1.1 | Create expert test set | 2-4 weeks | Human annotation | ❌ No - Human required |

### Long-Term (Weeks 13-24)

| Task ID | Description | Effort | Dependencies | AI-Agent Friendly? |
|---------|-------------|--------|--------------|-------------------|
| T4.3.1-5 | Qalqalah validator | 2-3 weeks | T4.3.1 data | ⚠️ Partial |
| RT2.1.1-3 | INT8 quantization | 1 week | M3 trained | ✅ Yes - Standard flow |
| RT3.1.1-3 | Precompute all ayahs | 1 week | Phase 1 done | ✅ Yes - Batch script |
| MB1.1.1-3 | Mobile student model | 3-4 weeks | Phase 2 done | ✅ Yes - Training script |

---

## AI Agent Task Templates

### Template 1: Pure Implementation Task

**Example: T1.1.1 - Support Multiple Audio Formats**

```markdown
## Task: Implement Multi-Format Audio Loading

**Context:**
- Module: Audio Preprocessing (Module 1)
- Input: Audio file path (any format: MP3, WAV, WebM, M4A)
- Output: NumPy array (16kHz, mono), sample rate, metadata

**Requirements:**
1. Use `soundfile` for WAV, `librosa` for all others
2. Handle corrupt files gracefully (try-except)
3. Extract metadata: original sample rate, duration, bit depth
4. Return None if loading fails, log error message

**Code Location:**
- File: `src/iqrah_audio/preprocessing/audio_loader.py`
- Function: `load_audio(file_path: str) -> tuple | None`

**Test Cases:**
1. Load WAV file → returns array, 16000, metadata
2. Load MP3 file → same
3. Load corrupt file → returns None, logs error
4. Load unsupported format → returns None

**Acceptance Criteria:**
- All test cases pass
- Type hints complete
- Docstring with examples
- No dependencies beyond librosa, soundfile, numpy

**Estimated Time:** 2-3 hours
```

### Template 2: Model Training Task

**Example: T3.1.2 - Continue Pretrain Wav2Vec2-BERT**

```markdown
## Task: Continue Pretraining Wav2Vec2-BERT on Quranic Data

**Context:**
- Goal: Adapt facebook/w2v-bert-2.0 to Quranic recitation domain
- Dataset: Tarteel-ai-everyayah (50-100 hours subset)
- Expected: Improved phoneme recognition accuracy

**Steps:**
1. Download base model from HuggingFace Hub
2. Load Tarteel dataset using `datasets` library
3. Configure training: 
   - Learning rate: 1e-5
   - Batch size: 4 (gradient accumulation 8)
   - Epochs: 3
   - Warmup: 500 steps
4. Use HuggingFace `Trainer` with `Wav2Vec2ForPreTraining`
5. Save checkpoint every 1000 steps
6. Validate on held-out 10% subset

**Code Location:**
- Script: `scripts/train_wav2vec2_continue.py`
- Config: `configs/wav2vec2_pretrain.yaml`

**Resources:**
- GPU: 8×A100 (Lambda Labs)
- Time: 1-2 days
- Memory: ~80GB VRAM total

**Deliverables:**
- Trained model checkpoint `models/wav2vec2_bert_quranic/`
- Training logs (loss curve, WER on validation)
- README with hyperparameters

**Validation:**
- Validation loss decreases compared to base model
- No NaN/inf in training
- Model loads successfully for inference

**Estimated Time:** 3-5 days (including setup)
```

### Template 3: Data Processing Task

**Example: T4.2.1 - Collect Ghunnah Training Data**

```markdown
## Task: Collect and Annotate Ghunnah Training Examples

**Context:**
- Goal: 1000+ labeled examples (ghunnah present/absent)
- Purpose: Train binary classifier for ghunnah detection
- Format: CSV with columns: audio_path, start, end, has_ghunnah

**Steps:**
1. Extract all noon saakinah, meem, tanween phonemes from existing data
2. Use existing Tajweed mapping to identify ghunnah cases
3. Sample negative examples (non-ghunnah nasals)
4. Export to CSV with file paths and timestamps
5. Verify audio snippets are extractable

**Code Location:**
- Script: `scripts/prepare_ghunnah_dataset.py`
- Output: `data/ghunnah_training_data.csv`

**Quality Checks:**
- Balance: ~50% positive, ~50% negative
- Duration: All snippets 100-500ms
- No duplicates
- All file paths valid

**Deliverables:**
- `data/ghunnah_training_data.csv` (1000+ rows)
- Statistics report (counts per category)
- Sample audio files for manual verification (10 random)

**Estimated Time:** 1-2 days
```

### Template 4: Integration Task

**Example: T7.1.1 - Aggregate Validator Results**

```markdown
## Task: Integrate Multiple Tajweed Validators into Comparison Engine

**Context:**
- Validators: MaddDurationValidator, GhunnahValidator, QalqalahValidator
- Goal: Collect violations, compute weighted overall score
- Location: `src/iqrah_audio/comparison/engine.py`

**Requirements:**
1. Call each validator on student phonemes
2. Aggregate violations list (sorted by timestamp)
3. Compute per-rule accuracy: `100 - (violations / total) * 100`
4. Apply weights: madd=0.5, ghunnah=0.25, qalqalah=0.15, complex=0.1
5. Return dict: `{overall, madd, ghunnah, qalqalah, violations}`

**Code Location:**
- Function: `ComparisonEngine._compare_tajweed()`
- Input: student_phonemes, reference_phonemes, audio
- Output: dict with scores and violations

**Edge Cases:**
- No violations found → score = 100
- Validator raises exception → log warning, skip that validator
- Zero phonemes for rule → score = N/A (not 0)

**Test Cases:**
- Perfect recitation → overall = 100
- 1 madd violation out of 5 → madd score = 80
- Validator crashes → graceful degradation

**Estimated Time:** 4-6 hours
```

---

## Dependency Graph

### Critical Path (Phase 1)

```
T3.1.1 → T3.1.2 → T3.1.3 → T3.1.4 → T3.1.5
  ↓
T3.2.1 → T3.2.2 → T3.2.3
  ↓
T4.1.1 → T4.1.2 → T4.1.3 → T4.1.4
  ↓
T7.1.1 → T7.1.2 → T7.1.3
  ↓
T8.1.1 → T8.1.2 → T8.1.3
  ↓
T-V1.1.1 → T-V1.1.2 → T-V1.1.3
```

**Duration:** ~12-16 weeks for critical path

### Parallel Workstreams

| Workstream | Tasks | Can Start When | Duration |
|------------|-------|----------------|----------|
| Voice Quality | T5.1-T5.5 | Immediately | 2 weeks |
| Prosody | T6.1-T6.3 | After M5 | 3 weeks |
| Feedback | T8.1-T8.4 | After M7 | 1 week |
| Ghunnah | T4.2.1-T4.2.5 | After data collected | 2 weeks |
| Qalqalah | T4.3.1-T4.3.5 | After data collected | 3 weeks |

---

## Milestones

### Milestone 1: Core Pipeline Complete (Week 12)
- ✅ M1-M3 fully functional
- ✅ Madd validator 99% accurate
- ✅ Basic comparison working
- **Demo:** Analyze any user recitation, return score

### Milestone 2: Advanced Tajweed (Week 18)
- ✅ Ghunnah validator 85% accurate
- ✅ Qalqalah validator 80% accurate
- ✅ Prosody analysis integrated
- **Demo:** Comprehensive feedback with all rules

### Milestone 3: Production Ready (Week 24)
- ✅ Validation study complete (r > 0.7)
- ✅ User testing 50+ users
- ✅ Documentation complete
- **Demo:** Public beta launch

### Milestone 4: Real-Time (Week 36)
- ✅ <500ms latency achieved
- ✅ GPU inference optimized
- ✅ Caching operational
- **Demo:** Live recitation analysis

### Milestone 5: Mobile Launch (Week 52)
- ✅ iOS + Android apps
- ✅ On-device inference
- ✅ App store approval
- **Demo:** Full mobile experience

---

## Resource Allocation

### Solo Developer Timeline

| Phase | Duration | Focus | Outcomes |
|-------|----------|-------|----------|
| Weeks 1-4 | 1 month | Core alignment (M3) | Phoneme recognition working |
| Weeks 5-8 | 1 month | Tajweed madd+ghunnah (M4) | Basic rules validated |
| Weeks 9-12 | 1 month | Prosody+comparison (M6-M7) | Full pipeline E2E |
| Weeks 13-16 | 1 month | Feedback+validation (M8, V1) | User-ready prototype |
| Weeks 17-20 | 1 month | Qalqalah+refinement | Advanced rules |
| Weeks 21-24 | 1 month | User testing+iteration | Production release |

### With AI Agents (Parallel Work)

| Week | Main Developer | AI Agent 1 | AI Agent 2 | AI Agent 3 |
|------|----------------|------------|------------|------------|
| 1-2 | M3.1 training | M1 preprocessing | M2 pitch | M5 OpenSMILE |
| 3-4 | M3.2 alignment | M4.1 madd | M8.1 templates | T-V1.1.1 data prep |
| 5-6 | M4.2 ghunnah train | M6.1 rhythm | M7.1 comparison | T5.5 embeddings |
| 7-8 | M4.3 qalqalah | M6.2 melody | M7.5 fusion | T8.3 progress DB |
| 9-10 | Integration testing | Documentation | Visualization | Benchmarking |
| 11-12 | User testing | Bug fixes | Refinement | Deployment |

**Speedup:** 2-3× faster with well-coordinated AI agents

---

## Next Steps

1. **Review this document** - Ensure all tasks make sense
2. **Prioritize critical path** - Focus on T3.1.x (phoneme training) first
3. **Assign first batch** - Give AI agents T1.1.1, T2.1.1, T5.1.1 (independent)
4. **Setup infrastructure** - GPU access, dataset download, repo structure
5. **Begin validation prep** - Start collecting expert annotations now (slow)

**First Sprint (Week 1):**
- Main: Setup Wav2Vec2 training pipeline
- Agent 1: Implement audio loader
- Agent 2: Integrate SwiftF0
- Agent 3: Setup OpenSMILE extraction

By end of sprint: All data preprocessing done, training ready to start.
