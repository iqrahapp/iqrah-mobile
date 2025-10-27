[↑ Navigation](../NAVIGATION.md)

# Phase 3 Technical Details

**Purpose**: Complete task breakdown for Phase 3 mobile implementation
**Includes**: MB1 (Distillation), MB2 (On-Device), MB3 (SDK), MB4 (Validation)

---

## MB1: Model Distillation

### MB1.1: Student Model Training

- **T-MB1.1.1**: Design student architecture <100M params
  - Base: DistilHuBERT or custom small transformer
  - Layers: 6 (vs 12 in teacher)
  - Hidden size: 512 (vs 768)
  - Attention heads: 8
  - Params: ~80M

- **T-MB1.1.2**: Train from scratch on Quranic data
  - Dataset: Tarteel (50h)
  - Epochs: 20
  - Batch size: 16
  - LR: 3e-4
  - Duration: ~2 days on A100

- **T-MB1.1.3**: Validate PER <2%
  - Test set: 100 ayahs
  - Acceptable: PER 1.5-2%
  - Boundary accuracy: 80% within 50ms

### MB1.2: Knowledge Distillation

- **T-MB1.2.1**: Teacher Wav2Vec2-BERT outputs
  - Extract: CTC posteriors from teacher
  - Save: Soft targets for distillation
  - Format: NumPy arrays per audio

- **T-MB1.2.2**: Soft target distillation
  - Loss: KL divergence between student/teacher logits
  - Temperature: 2.0
  - Alpha: 0.5 (weight for distillation loss)

- **T-MB1.2.3**: Fine-tune student with KL loss
  - Combined loss: `alpha * KL + (1-alpha) * CTC`
  - Epochs: 5
  - LR: 1e-5
  - Expected: 0.2-0.5% PER improvement

### MB1.3: Mobile Quantization

- **T-MB1.3.1**: INT8 quantization TFLite/CoreML
  - TFLite: Post-training quantization
  - CoreML: Use coremltools with quantization
  - Calibration: 100 samples

- **T-MB1.3.2**: Model size <50MB validation
  - Measure: File size on disk
  - Target: 40-50MB
  - Format: .tflite or .mlmodel

- **T-MB1.3.3**: Accuracy retention >98%
  - Measure PER: Quantized vs float32
  - Acceptable: <0.2% PER increase
  - Test: 500 ayah diverse set

---

## MB2: On-Device Inference

### MB2.1: iOS CoreML

- **T-MB2.1.1**: CoreML conversion coremltools
  - Convert PyTorch → CoreML
  - Use: `ct.convert()`
  - Input: (1, seq_len) audio
  - Output: CTC logits

- **T-MB2.1.2**: Neural Engine optimization
  - Ensure ops compatible with Neural Engine
  - Profile: Xcode Instruments
  - Target: >80% Neural Engine utilization

- **T-MB2.1.3**: Swift inference wrapper
  - Create: Swift class wrapping CoreML
  - API: `predict(audio: [Float]) -> [Phoneme]`
  - Handle: Pre/post-processing

### MB2.2: Android TFLite

- **T-MB2.2.1**: TFLite conversion
  - Convert PyTorch → TFLite
  - Use: `torch.utils.mobile_optimizer`
  - Optimize: Fuse ops, remove redundant

- **T-MB2.2.2**: NNAPI GPU delegate
  - Enable GPU acceleration via NNAPI
  - Fallback: CPU if GPU unavailable
  - Profile: Android Studio Profiler

- **T-MB2.2.3**: Kotlin inference wrapper
  - Create: Kotlin class wrapping TFLite
  - API: `predict(audio: FloatArray): List<Phoneme>`
  - Thread-safe operations

### MB2.3: Hybrid Architecture

- **T-MB2.3.1**: On-device phoneme alignment
  - Run: Student model on-device
  - Latency: <200ms
  - Accuracy: PER ~2%

- **T-MB2.3.2**: On-device madd validation
  - Implement: Rule-based madd validator
  - Pure Swift/Kotlin (no ML)
  - Latency: <10ms

- **T-MB2.3.3**: Server-side prosody/style
  - Send: Audio + phonemes to server
  - Server computes: Prosody, style, advanced Tajweed
  - Return: Complete feedback

- **T-MB2.3.4**: Offline mode basic feedback
  - On-device only: Phonemes + madd
  - Show: Basic Tajweed violations
  - Banner: "Connect for advanced analysis"

---

## MB3: Mobile SDK

### MB3.1: React Native/Flutter

- **T-MB3.1.1**: Cross-platform framework choice
  - Decision: React Native or Flutter
  - Criteria: Native module support, performance
  - Recommendation: Flutter (better performance)

- **T-MB3.1.2**: Native module bindings
  - Create: Platform channels (Flutter) or Native modules (RN)
  - iOS: Swift bridge
  - Android: Kotlin bridge

- **T-MB3.1.3**: UI component library
  - Components: Waveform, phoneme cursor, score display
  - Style: Material Design (Android), Cupertino (iOS)
  - Accessibility: VoiceOver, TalkBack

### MB3.2: Audio Recording

- **T-MB3.2.1**: Microphone permission handling
  - Request: iOS `NSMicrophoneUsageDescription`
  - Request: Android `RECORD_AUDIO` permission
  - Handle: Denial gracefully

- **T-MB3.2.2**: 16kHz mono recording
  - Format: PCM 16-bit
  - Sample rate: 16kHz
  - Channels: Mono
  - Buffer: 1024 samples

- **T-MB3.2.3**: Chunked upload WebRTC
  - Use: WebRTC for low-latency streaming
  - Or: WebSocket with audio chunks
  - Chunk size: 0.5s (8000 samples)

### MB3.3: Real-Time Visualization

- **T-MB3.3.1**: Real-time pitch overlay
  - Plot: Pitch contour over time
  - Update: Every 100ms
  - Library: Custom Canvas or fl_chart

- **T-MB3.3.2**: Phoneme cursor tracking
  - Show: Current phoneme highlighted
  - Move: Cursor in sync with audio
  - Color: Green (correct), Red (violation)

- **T-MB3.3.3**: Tajweed color highlighting
  - Color scheme: Madd (blue), Ghunnah (green), Qalqalah (yellow)
  - Overlay: On Arabic text
  - Real-time: Update as violations detected

### MB3.4: Backend API

- **T-MB3.4.1**: REST API endpoints mobile-specific
  - Endpoint: `/api/mobile/analyze`
  - Payload: Audio (base64) + metadata
  - Response: Feedback JSON

- **T-MB3.4.2**: WebSocket streaming
  - Endpoint: `/ws/mobile/stream`
  - Protocol: Same as Phase 2
  - Optimization: Smaller payloads

- **T-MB3.4.3**: Offline sync queue
  - Queue: Store failed requests locally
  - Retry: When connection restored
  - Storage: SQLite

---

## MB4: Mobile Validation

### MB4.1: Device Testing

- **T-MB4.1.1**: Test matrix 10 devices iOS/Android
  - iOS: iPhone 12, 13, 14, 15 (mini, Pro)
  - Android: Samsung, Pixel, OnePlus, Xiaomi
  - Test: Latency, accuracy, crashes

- **T-MB4.1.2**: OS version compatibility
  - iOS: 14, 15, 16, 17, 18
  - Android: 10, 11, 12, 13, 14, 15
  - Ensure: No breaking issues

- **T-MB4.1.3**: Screen size adaptation
  - Responsive: UI adapts to all screen sizes
  - Test: Tablets, foldables
  - Orientation: Portrait and landscape

### MB4.2: Performance Profiling

- **T-MB4.2.1**: Latency <300ms validation
  - Measure: End-to-end on-device
  - Test: 100 ayah test set
  - Pass: p95 <300ms

- **T-MB4.2.2**: Battery consumption profiling
  - Tool: Xcode Energy Log, Android Battery Historian
  - Test: 1hr continuous use
  - Target: <20% battery drain per hour

- **T-MB4.2.3**: Memory usage <200MB
  - Monitor: Xcode Memory Graph, Android Profiler
  - Peak usage: <200MB
  - No leaks: Run 30min leak test

### MB4.3: Beta Deployment

- **T-MB4.3.1**: TestFlight beta iOS
  - Upload: Build to TestFlight
  - Invite: 50 beta users
  - Duration: 2 weeks

- **T-MB4.3.2**: Google Play internal testing
  - Upload: AAB to Play Console
  - Track: Internal testing
  - Invite: 50 beta users

- **T-MB4.3.3**: Crash analytics Firebase
  - Integrate: Firebase Crashlytics
  - Monitor: Crash-free rate >99%
  - Fix: P0 crashes immediately

- **T-MB4.3.4**: User feedback collection
  - In-app: Feedback form
  - Survey: Post-beta survey
  - Analyze: Common issues, feature requests

---

**Related**: See main [Architecture](../01-architecture/overview.md) docs
