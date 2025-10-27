[↑ Navigation](../NAVIGATION.md)

# Task Overview & Dependencies

**Purpose**: Concrete, AI-agent-assignable tasks
**Generated**: 2025-10-23

This document provides a comprehensive overview of all tasks across the three phases of the IQRAH Audio project, along with their dependencies visualized in a Mermaid graph.

---

## DEPENDENCY GRAPH (MERMAID)

```mermaid
graph TB
    subgraph "PHASE 1: OFFLINE E2E (Months 1-6)"
        A[Phase 1 Complete]

        subgraph "M1: Preprocessing"
            M1[M1 Complete] --> M1.1[Audio Loading]
            M1 --> M1.2[Resampling]
            M1 --> M1.3[VAD]
            M1 --> M1.4[Noise Reduction]
            M1 --> M1.5[Quality Checks]
            M1 --> M1.6[Caching]

            M1.1 --> T1.1.1[MP3/WAV/WebM/M4A support]
            M1.1 --> T1.1.2[Corrupt file detection]
            M1.1 --> T1.1.3[Metadata extraction]

            M1.2 --> T1.2.1[Kaiser 16kHz resample]
            M1.2 --> T1.2.2[Peak normalization]
            M1.2 --> T1.2.3[LUFS normalization]

            M1.3 --> T1.3.1[Silero VAD ONNX]
            M1.3 --> T1.3.2[Segment 250ms min]
            M1.3 --> T1.3.3[Silence trimming]

            M1.4 --> T1.4.1[SNR estimation]
            M1.4 --> T1.4.2[Spectral subtraction]
            M1.4 --> T1.4.3[Apply if SNR<15dB]

            M1.5 --> T1.5.1[Clipping >5% detect]
            M1.5 --> T1.5.2[Dynamic range analysis]
            M1.5 --> T1.5.3[Quality flag gen]

            M1.6 --> T1.6.1[SHA256 cache]
            M1.6 --> T1.6.2[Cache logging]
            M1.6 --> T1.6.3[TTL management]
        end

        subgraph "M2: Pitch"
            M2[M2 Complete] --> M2.1[SwiftF0]
            M2 --> M2.2[RMVPE Fallback]
            M2 --> M2.3[Confidence Weight]
            M2 --> M2.4[Smoothing]
            M2 --> M2.5[Statistics]

            M2.1 --> T2.1.1[swift-f0 install]
            M2.1 --> T2.1.2[Extract 10ms hop]
            M2.1 --> T2.1.3[46-2093Hz range]
            M2.1 --> T2.1.4[Median filter 5-frame]

            M2.2 --> T2.2.1[RMVPE from GitHub]
            M2.2 --> T2.2.2[Trigger conf<0.7]
            M2.2 --> T2.2.3[Deep U-Net inference]

            M2.3 --> T2.3.1[Inverse variance weight]
            M2.3 --> T2.3.2[Blending function]
            M2.3 --> T2.3.3[Confidence threshold]

            M2.4 --> T2.4.1[Savgol p=3 w=51ms]
            M2.4 --> T2.4.2[Linear interp <100ms gaps]
            M2.4 --> T2.4.3[Octave jump removal]

            M2.5 --> T2.5.1[Mean/std Hz]
            M2.5 --> T2.5.2[Pitch range min/max]
            M2.5 --> T2.5.3[Voiced ratio]
        end

        subgraph "M3: Alignment"
            M3[M3 Complete] --> M3.1[Wav2Vec2-BERT Train]
            M3 --> M3.2[CTC Align]
            M3 --> M3.3[Tajweed Map]
            M3 --> M3.4[GOP Score]
            M3 --> M3.5[Quality Valid]

            M3.1 --> T3.1.1[Tarteel download]
            M3.1 --> T3.1.2[Stage1: pretrain 50-100h]
            M3.1 --> T3.1.3[MSA Phonetiser labels]
            M3.1 --> T3.1.4[Stage2: CTC finetune]
            M3.1 --> T3.1.5[PER<1% validate]

            M3.2 --> T3.2.1[ctc-forced-aligner]
            M3.2 --> T3.2.2[Windowed per word]
            M3.2 --> T3.2.3[HMM boundary smooth]
            M3.2 --> T3.2.4[GPU acceleration]

            M3.3 --> T3.3.1[Load tajweed.json]
            M3.3 --> T3.3.2[Fuzzy match edit dist]
            M3.3 --> T3.3.3[Rule label assign]
            M3.3 --> T3.3.4[Expected duration lookup]

            M3.4 --> T3.4.1[CTC posteriors extract]
            M3.4 --> T3.4.2[log P(phoneme|audio)]
            M3.4 --> T3.4.3[Per-phoneme GOP]
            M3.4 --> T3.4.4[Threshold calibration]

            M3.5 --> T3.5.1[Mean conf>0.7 check]
            M3.5 --> T3.5.2[Phoneme count ±10%]
            M3.5 --> T3.5.3[Duration 20-500ms sanity]
            M3.5 --> T3.5.4[Warning gen]
        end

        subgraph "M4: Tajweed"
            M4[M4 Complete] --> M4.1[Madd]
            M4 --> M4.2[Ghunnah]
            M4 --> M4.3[Qalqalah]
            M4 --> M4.4[Complex Rules]

            M4.1 --> T4.1.1[Duration thresholds per type]
            M4.1 --> T4.1.2[Tolerance ±20%]
            M4.1 --> T4.1.3[Violation severity]
            M4.1 --> T4.1.4[99% accuracy validate]

            M4.2 --> T4.2.1[Collect 1000+ labels]
            M4.2 --> T4.2.2[F1/F2/F3 Parselmouth]
            M4.2 --> T4.2.3[Nasal 250-350Hz]
            M4.2 --> T4.2.4[MLP 64-32 hidden]
            M4.2 --> T4.2.5[85% accuracy validate]

            M4.3 --> T4.3.1[Collect 500 examples]
            M4.3 --> T4.3.2[ZCR, centroid, RMS]
            M4.3 --> T4.3.3[Burst detection algo]
            M4.3 --> T4.3.4[SVM classifier train]
            M4.3 --> T4.3.5[80% accuracy validate]

            M4.4 --> T4.4.1[Research idghaam]
            M4.4 --> T4.4.2[Research ikhfaa]
            M4.4 --> T4.4.3[Defer Phase 3]
        end

        subgraph "M5: Voice Quality"
            M5[M5 Complete] --> M5.1[OpenSMILE]
            M5 --> M5.2[Vibrato]
            M5 --> M5.3[Breathiness]
            M5 --> M5.4[Timbre]
            M5 --> M5.5[Neural Embeddings]

            M5.1 --> T5.1.1[opensmile Python]
            M5.1 --> T5.1.2[eGeMAPS 88-d extract]
            M5.1 --> T5.1.3[Batch optimize]

            M5.2 --> T5.2.1[Bandpass 2-15Hz pitch]
            M5.2 --> T5.2.2[Autocorr rate detect]
            M5.2 --> T5.2.3[Extent amplitude]
            M5.2 --> T5.2.4[Regularity CV]

            M5.3 --> T5.3.1[H1-H2 harmonic diff]
            M5.3 --> T5.3.2[HNR Parselmouth]
            M5.3 --> T5.3.3[CPP cepstral]
            M5.3 --> T5.3.4[Spectral tilt -10 to -15dB/oct]

            M5.4 --> T5.4.1[Centroid 2-4kHz]
            M5.4 --> T5.4.2[Spectral flux]
            M5.4 --> T5.4.3[Rolloff 85%]
            M5.4 --> T5.4.4[Formant F1-F4]

            M5.5 --> T5.5.1[X-vector SpeechBrain 512-d]
            M5.5 --> T5.5.2[Wav2Vec2 CLS 768-d]
            M5.5 --> T5.5.3[Embedding similarity]
        end

        subgraph "M6: Prosody"
            M6[M6 Complete] --> M6.1[Rhythm]
            M6 --> M6.2[Melody]
            M6 --> M6.3[Style]

            M6.1 --> T6.1.1[nPVI compute]
            M6.1 --> T6.1.2[Varco coeff]
            M6.1 --> T6.1.3[IOI distribution]
            M6.1 --> T6.1.4[Isochrony score]
            M6.1 --> T6.1.5[Soft-DTW integrate]

            M6.2 --> T6.2.1[Fujisaki scipy fit]
            M6.2 --> T6.2.2[Phrase/accent extract]
            M6.2 --> T6.2.3[Declination exp fit]
            M6.2 --> T6.2.4[Tilt parametrize]
            M6.2 --> T6.2.5[Contour shape classify]
            M6.2 --> T6.2.6[Maqam CNN Maqam478]

            M6.3 --> T6.3.1[X-vector extract]
            M6.3 --> T6.3.2[GST tokens optional]
            M6.3 --> T6.3.3[Style label classify]
        end

        subgraph "M7: Comparison"
            M7[M7 Complete] --> M7.1[Tajweed Compare]
            M7 --> M7.2[Prosody Compare]
            M7 --> M7.3[Pronunciation Compare]
            M7 --> M7.4[Voice Compare]
            M7 --> M7.5[Fusion]

            M7.1 --> T7.1.1[Aggregate validators]
            M7.1 --> T7.1.2[Weighted madd/ghunnah/qalqalah]
            M7.1 --> T7.1.3[Violation severity rank]

            M7.2 --> T7.2.1[Rhythm DTW+nPVI+IOI]
            M7.2 --> T7.2.2[Melody Fujisaki+tilt+maqam]
            M7.2 --> T7.2.3[Style X-vector cosine]
            M7.2 --> T7.2.4[Weighted prosody]

            M7.3 --> T7.3.1[GOP delta compute]
            M7.3 --> T7.3.2[Phoneme CTC confidence]
            M7.3 --> T7.3.3[Per-phoneme violation]

            M7.4 --> T7.4.1[Timbre centroid diff]
            M7.4 --> T7.4.2[Vibrato rate match]
            M7.4 --> T7.4.3[Breathiness HNR compare]

            M7.5 --> T7.5.1[Fusion 40/30/20/10]
            M7.5 --> T7.5.2[Confidence interval]
            M7.5 --> T7.5.3[Explainability gen]
        end

        subgraph "M8: Feedback"
            M8[M8 Complete] --> M8.1[Text Gen]
            M8 --> M8.2[Visualization]
            M8 --> M8.3[Progress Track]
            M8 --> M8.4[Recommendations]

            M8.1 --> T8.1.1[Summary templates]
            M8.1 --> T8.1.2[Detailed feedback per rule]
            M8.1 --> T8.1.3[Timestamp formatting]

            M8.2 --> T8.2.1[Pitch overlay plot]
            M8.2 --> T8.2.2[Phoneme alignment viz]
            M8.2 --> T8.2.3[Violation markers]

            M8.3 --> T8.3.1[PostgreSQL schema]
            M8.3 --> T8.3.2[Attempt recording]
            M8.3 --> T8.3.3[Progress comparison]
            M8.3 --> T8.3.4[Streak calculation]

            M8.4 --> T8.4.1[Next steps rules]
            M8.4 --> T8.4.2[Practice suggestions]
            M8.4 --> T8.4.3[Difficulty progression]
        end

        subgraph "Validation"
            V1[Validation Complete] --> V1.1[Test Sets]
            V1 --> V1.2[Expert Annotation]
            V1 --> V1.3[User Testing]

            V1.1 --> T-V1.1.1[100 ayah phoneme boundaries]
            V1.1 --> T-V1.1.2[500 madd examples]
            V1.1 --> T-V1.1.3[300 ghunnah examples]
            V1.1 --> T-V1.1.4[200 qalqalah examples]

            V1.2 --> T-V1.2.1[Hire 3-5 Qaris]
            V1.2 --> T-V1.2.2[Annotation protocol]
            V1.2 --> T-V1.2.3[Inter-rater α>0.75]

            V1.3 --> T-V1.3.1[Alpha N=10]
            V1.3 --> T-V1.3.2[Beta N=50-100]
            V1.3 --> T-V1.3.3[Validation study N=60-100]
        end
    end
```

---

## MILESTONES SUMMARY

### Phase 1: Offline E2E (Weeks 1-24)

**M1: Core Pipeline (Week 12)**
- M1-M3 functional
- Madd 99% accuracy
- Basic comparison working

**M2: Advanced Tajweed (Week 18)**
- Ghunnah 85% accuracy
- Qalqalah 80% accuracy
- Prosody integrated

**M3: Production Ready (Week 24)**
- Validation r > 0.7
- 50+ beta users
- Documentation complete

### Phase 2: Real-Time (Weeks 25-36)

**M4: Real-Time (Week 36)**
- <500ms latency
- GPU optimized
- Caching operational

### Phase 3: Mobile (Weeks 37-52)

**M5: Mobile Launch (Week 52)**
- iOS + Android
- On-device inference
- App store approval

---

## NAVIGATION

**Phase-Specific Task Details**:
- [Phase 1: Offline E2E Tasks](./phase1-offline.md) - Weeks 1-24
- [Phase 2: Real-Time Tasks](./phase2-realtime.md) - Months 7-12
- [Phase 3: Mobile Tasks](./phase3-mobile.md) - Months 13-18

**Related Documentation**:
- [Implementation Guide](../02-implementation/guide.md)
- [AI Agent Templates](../02-implementation/ai-agent-templates.md)
- [Technical Architecture](../01-architecture/technical-architecture.md)

---

**Related**: [Implementation Guide](../02-implementation/guide.md) | [AI Agent Templates](../02-implementation/ai-agent-templates.md)
