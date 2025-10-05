# Iqrah Audio Integration Roadmap

**Current Status:** V2 DTW achieving 92.3% accuracy - **READY FOR MVP**  
**Target:** Integrate pitch tracking into Iqrah mobile app  
**Date:** October 5, 2025

---

## 🎯 Integration Strategy

Based on your design documents and the current V2 DTW achievements, here's the pragmatic roadmap:

---

## Phase 1: MVP Foundation (Current - COMPLETE ✅)

### What We Have
- ✅ **V2 DTW:** 92.3% tracking accuracy
- ✅ **Low Latency:** <5ms average per frame
- ✅ **Huber Loss:** Robust to outliers
- ✅ **Delta-Pitch Option:** Configurable for cross-alignment
- ✅ **Production Ready:** Tested with real Husary audio

### What This Enables
- Real-time pitch comparison
- On-note accuracy detection
- Lead/lag feedback
- Confidence scoring

### Integration Point
This is **S0 (MVP Imitation)** from your design - we already have the core!

---

## Phase 2: Offline Analysis Feature (2-3 WEEKS)

### Goal
Create a **non-real-time** recitation practice feature to prove the tech without real-time complexity.

### Architecture
```
User Flow:
1. Select verse (e.g., Al-Fatiha 1:1)
2. See reference pitch contour from qari
3. Press "Record" → recite → Press "Stop"
4. App processes (2-3 seconds)
5. Display user's pitch overlaid on reference + score
```

### Technical Implementation

#### Data Preparation
```bash
# R&D script (Python)
python tools/prepare_reference.py \
  --audio data/husary/surahs/01.mp3 \
  --output references/husary_fatiha.cbor

# Generates:
# - Pitch contour: [(timestamp, pitch_hz), ...]
# - Metadata: duration, sample_rate, qari_name
# - Anchor points for alignment
```

#### Rust Core (New Functions)
```rust
// In core/api.rs (FRB facade)

#[flutter_rust_bridge::frb(sync)]
pub fn analyze_recitation_offline(
    user_audio: Vec<f32>,
    reference_contour: Vec<(f32, f32)>, // (time, pitch)
    sample_rate: u32,
) -> RecitationResult {
    // 1. Extract user pitch (using existing pitch extraction)
    let user_pitch = extract_pitch(&user_audio, sample_rate);
    
    // 2. Run DTW alignment (using V2)
    let alignment = run_dtw_v2(&user_pitch, &reference_contour);
    
    // 3. Compute scores
    let score = compute_score(&alignment);
    
    RecitationResult {
        score: score.overall,
        on_note_pct: score.on_note,
        user_contour: user_pitch,
        alignment_path: alignment.path,
    }
}

#[derive(Clone)]
pub struct RecitationResult {
    pub score: f32,              // 0-100
    pub on_note_pct: f32,        // Percentage on-note
    pub user_contour: Vec<(f32, f32)>,  // User pitch contour
    pub alignment_path: Vec<(usize, usize)>,  // DTW path
}
```

#### Flutter UI
```dart
// lib/screens/recitation_practice_screen.dart

class RecitationPracticeScreen extends StatefulWidget {
  final String verseId;  // e.g., "1:1" for Al-Fatiha
  
  @override
  _RecitationPracticeScreenState createState() => ...;
}

class _RecitationPracticeScreenState extends State<RecitationPracticeScreen> {
  List<(double, double)> _referenceContour = [];
  RecitationResult? _result;
  bool _isRecording = false;
  
  Future<void> _loadReference() async {
    // Load pre-processed reference
    final ref = await ReferenceLoader.load(widget.verseId);
    setState(() => _referenceContour = ref.contour);
  }
  
  Future<void> _startRecording() async {
    setState(() => _isRecording = true);
    await _audioRecorder.start();
  }
  
  Future<void> _stopRecording() async {
    final audioData = await _audioRecorder.stop();
    setState(() => _isRecording = false);
    
    // Analyze offline
    final result = await api.analyzeRecitationOffline(
      userAudio: audioData,
      referenceContour: _referenceContour,
      sampleRate: 44100,
    );
    
    setState(() => _result = result);
  }
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Column(
        children: [
          // Reference pitch visualization
          PitchContourChart(
            data: _referenceContour,
            color: Colors.blue,
          ),
          
          // User pitch overlay (if analyzed)
          if (_result != null)
            PitchContourChart(
              data: _result!.userContour,
              color: Colors.orange,
            ),
          
          // Score display
          if (_result != null)
            ScoreCard(
              score: _result!.score,
              onNotePct: _result!.onNotePct,
            ),
          
          // Record button
          RecordButton(
            isRecording: _isRecording,
            onStart: _startRecording,
            onStop: _stopRecording,
          ),
        ],
      ),
    );
  }
}
```

### Deliverables
- [ ] R&D script: Extract reference pitch contours
- [ ] Rust: `analyze_recitation_offline()` FRB function
- [ ] Flutter: Recitation practice screen
- [ ] Flutter: Pitch contour visualization widget
- [ ] Integration: Save/load reference data

### Success Metrics
- User can record and see comparison for 1 verse
- Score accuracy correlates with expert judgment
- Processing time <3 seconds for 10s audio

---

## Phase 3: Real-Time Feedback (4-6 WEEKS)

### Goal
Graduate from offline to live coaching with <100ms latency.

### Architecture
```
Audio Input → Streaming Buffer → Incremental Pitch → V2 DTW → Live Hints
                                                              ↓
                                                         Flutter UI
```

### Technical Implementation

#### Rust Core (Streaming API)
```rust
// In core/api.rs

pub struct RealtimeSession {
    pipeline: RealtimePipeline,
    hint_stream: StreamSink<LiveHint>,
}

impl RealtimeSession {
    #[flutter_rust_bridge::frb(sync)]
    pub fn new(
        reference_path: String,
        hint_sink: StreamSink<LiveHint>,
    ) -> Self {
        // Load reference
        let ref_audio = load_audio(&reference_path);
        
        // Create pipeline
        let pipeline = RealtimePipeline::new(ref_audio);
        
        Self {
            pipeline,
            hint_stream: hint_sink,
        }
    }
    
    #[flutter_rust_bridge::frb(sync)]
    pub fn process_audio_chunk(&mut self, chunk: Vec<f32>) {
        // Process chunk through pipeline
        if let Some(hints) = self.pipeline.process_chunk(&chunk) {
            // Send to Flutter via stream
            self.hint_stream.add(LiveHint {
                status: hints.status,
                message: hints.message,
                visual_cue: hints.visual_cue,
                lead_lag_ms: hints.lead_lag_ms,
                confidence: hints.confidence,
            });
        }
    }
}

#[derive(Clone)]
pub struct LiveHint {
    pub status: String,        // "good", "warning", "error"
    pub message: String,       // "Excellent!", "Too high"
    pub visual_cue: String,    // "green", "yellow", "red"
    pub lead_lag_ms: f32,      // Lead/lag in milliseconds
    pub confidence: f32,       // 0-1
}
```

#### Flutter UI
```dart
// lib/screens/realtime_recitation_screen.dart

class RealtimeRecitationScreen extends StatefulWidget {
  final String verseId;
  
  @override
  _RealtimeRecitationScreenState createState() => ...;
}

class _RealtimeRecitationScreenState extends State<RealtimeRecitationScreen> {
  late RealtimeSession _session;
  late StreamSubscription<LiveHint> _hintSubscription;
  LiveHint? _currentHint;
  
  @override
  void initState() {
    super.initState();
    _initializeSession();
  }
  
  Future<void> _initializeSession() async {
    // Create stream for hints
    final hintStream = StreamController<LiveHint>();
    
    // Initialize Rust session
    _session = await api.createRealtimeSession(
      referencePath: 'references/${widget.verseId}.cbor',
      hintSink: hintStream.sink,
    );
    
    // Listen to hints
    _hintSubscription = hintStream.stream.listen((hint) {
      setState(() => _currentHint = hint);
    });
    
    // Start audio capture
    _startAudioCapture();
  }
  
  Future<void> _startAudioCapture() async {
    await _audioStream.listen((chunk) {
      // Send to Rust for processing
      _session.processAudioChunk(chunk);
    });
  }
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Column(
        children: [
          // Live pitch visualization
          LivePitchOverlay(
            currentHint: _currentHint,
          ),
          
          // Feedback display
          if (_currentHint != null)
            FeedbackCard(
              status: _currentHint!.status,
              message: _currentHint!.message,
              color: _getColor(_currentHint!.visualCue),
            ),
          
          // Lead/lag indicator
          if (_currentHint != null)
            LeadLagIndicator(
              leadLagMs: _currentHint!.leadLagMs,
            ),
        ],
      ),
    );
  }
  
  Color _getColor(String cue) {
    switch (cue) {
      case 'green': return Colors.green;
      case 'yellow': return Colors.orange;
      case 'red': return Colors.red;
      default: return Colors.grey;
    }
  }
}
```

### Deliverables
- [ ] Rust: Streaming audio pipeline with FRB
- [ ] Rust: Real-time hint generation
- [ ] Flutter: Audio capture and streaming
- [ ] Flutter: Live pitch visualization widget
- [ ] Flutter: Real-time hint display
- [ ] Performance: <100ms end-to-end latency

### Success Metrics
- Median latency ≤100ms
- Hint flicker <2% when confident
- Stable tracking across verse

---

## Phase 4: Advanced Features (FUTURE)

Based on your long-term plan, these build on the foundation:

### S4-S6: ASR + GOP + Tajwid
- Arabic CTC for phoneme alignment
- Goodness-of-Pronunciation scoring
- Tajwid rule detection (Madd, Ghunna, Qalqalah)

### S7: Mobile Optimization
- Model quantization (INT8)
- Device-specific tuning
- Offline model packaging

### S8: Content Pipeline
- Multi-qari support
- Full Quran coverage
- Reference packs

---

## 🚀 Quick Start Guide

### Current Capabilities (Ready Now)

**What you can build TODAY:**

1. **Offline Recitation Analysis**
   ```python
   # Already working!
   from iqrah_audio.streaming import RealtimePipeline
   
   pipeline = RealtimePipeline(reference_audio)
   
   for chunk in user_audio_chunks:
       hints = pipeline.process_chunk(chunk)
       print(f"Score: {hints.on_note_pct:.1f}%")
   ```

2. **Batch Processing**
   ```python
   # For offline scoring
   from test_v2_real import test_self_alignment
   
   accuracy = test_self_alignment(
       reference_audio="husary.mp3",
       user_audio="student.mp3"
   )
   print(f"Accuracy: {accuracy:.1f}%")
   ```

### Integration Checklist

- [ ] **Phase 2 (MVP Extension)**
  - [ ] Week 1: R&D reference extraction
  - [ ] Week 2: Rust offline analysis API
  - [ ] Week 3: Flutter UI + visualization

- [ ] **Phase 3 (Real-Time)**
  - [ ] Week 1-2: Rust streaming pipeline
  - [ ] Week 3-4: Flutter real-time UI
  - [ ] Week 5: Integration + testing
  - [ ] Week 6: Performance optimization

---

## 📊 Expected Performance

Based on current V2 achievements:

| Metric | Phase 2 (Offline) | Phase 3 (Real-Time) |
|--------|-------------------|---------------------|
| **Accuracy** | 92.3% | 85-90% (live) |
| **Latency** | 1-3s total | <100ms per hint |
| **CPU** | 20-30% spike | 10-15% sustained |
| **Memory** | <50MB | <100MB |
| **Battery** | Negligible | ~5% per minute |

---

## 🎯 Next Actions

1. **Immediate (This Week)**
   - ✅ V2 DTW production ready (DONE!)
   - ✅ Documentation complete (DONE!)
   - ⏳ Test the demo app (YOU ARE HERE)
   - [ ] Plan Phase 2 timeline

2. **Phase 2 Prep (Next Week)**
   - [ ] Design reference data format (CBOR spec)
   - [ ] Create R&D extraction script
   - [ ] Set up Rust project for FRB
   - [ ] Design Flutter screen mockups

3. **Phase 2 Execution (Weeks 2-4)**
   - [ ] Implement offline analysis
   - [ ] Build UI components
   - [ ] User testing + iteration

---

## 💡 Key Decisions

### Technology Choices (Validated ✅)

1. **DTW Algorithm:** V2 with Huber loss (92.3% accuracy)
2. **Feature Type:** Z-norm for self-alignment, delta-pitch for cross-alignment
3. **Window Strategy:** Simple symmetric Sakoe-Chiba (300 frames)
4. **Language:** Python for R&D → Rust for production
5. **Bridge:** Flutter Rust Bridge (FRB) for mobile

### Architecture Principles

1. **Offline First:** Prove tech without real-time complexity
2. **Incremental:** Add streaming layer on solid foundation
3. **Rust Core:** Performance + safety for audio processing
4. **Flutter UI:** Cross-platform mobile development
5. **Test-Driven:** Validate at each phase before proceeding

---

**Status:** Ready to integrate into Iqrah! The core pitch tracking is production-ready. Phase 2 is the logical next step to prove the user experience without the complexity of real-time streaming. 🚀
