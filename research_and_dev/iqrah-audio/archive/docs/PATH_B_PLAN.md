# Path B: Real-Time Streaming - Implementation Plan

## Goal
Enable **live coaching** with <100ms visual feedback latency while maintaining accuracy from Path A.

## Current State (Path A)
- ✅ Offline analysis: RTF 0.26 (4x real-time)
- ✅ Accuracy: 0-10 cents MAE on clean audio
- ✅ Multi-dimensional features working
- ⚠️ Batch processing only (no streaming)

## Target State (Path B)
- 🎯 Streaming analysis: <100ms latency
- 🎯 Live pitch overlay (user vs reference)
- 🎯 Real-time lead/lag indicators
- 🎯 Confidence gating (freeze when uncertain)
- 🎯 Anchor-based drift correction

---

## Architecture Design

### High-Level Flow
```
Microphone → Audio Buffer → Feature Extraction → Online-DTW → Live Hints → UI
                ↓                    ↓                 ↓
          Ring Buffer         Incremental        Confidence
          (2-3s)             Processing          Gating
```

### Key Components

#### 1. Streaming Audio Buffer
```python
class StreamingAudioBuffer:
    """Ring buffer for streaming audio with configurable window."""

    def __init__(self, window_size_s=3.0, hop_size_s=0.01):
        self.window_size = window_size_s  # 2-3s sliding window
        self.hop_size = hop_size_s        # 10ms frame advance

    def push_samples(self, samples):
        """Add new audio samples."""

    def get_window(self):
        """Get current window for processing."""
```

#### 2. Incremental Feature Extraction
```python
class IncrementalPitchExtractor:
    """Extract pitch incrementally with minimal recomputation."""

    def __init__(self, method="yin", frame_size=2048, hop=512):
        self.method = method
        self.frame_cache = {}  # Cache computed frames

    def process_frame(self, audio_frame):
        """Process single frame, return F0 + confidence."""

    def process_buffer(self, audio_buffer, start_idx):
        """Process new frames only, reuse cached."""
```

#### 3. Enhanced Online-DTW
```python
class EnhancedOnlineDTW:
    """Online DTW with anchors, confidence gating, and drift control."""

    def __init__(
        self,
        window_size=300,      # ~3s at 100 fps
        band_width=50,        # Sakoe-Chiba band
        confidence_threshold=0.6,
    ):
        self.anchors = []     # Detected anchors
        self.confidence = 1.0
        self.drift_estimate = 0

    def update(self, query_frame, reference, anchors):
        """Update alignment with new frame."""

    def get_hints(self):
        """Get real-time hints (lead/lag, on-note, confidence)."""
```

#### 4. Anchor Detection
```python
class AnchorDetector:
    """Detect alignment anchors for drift correction."""

    def detect_silence(self, rms, threshold=-40):
        """Detect silence segments (>200ms)."""

    def detect_plosives(self, spectral_flatness, threshold=0.6):
        """Detect plosive bursts (qalqalah letters)."""

    def detect_long_notes(self, f0_stable, duration_threshold=0.5):
        """Detect sustained notes (madd)."""
```

#### 5. Live Feedback System
```python
class LiveFeedback:
    """Generate real-time feedback for UI."""

    def __init__(self, update_rate_hz=10):
        self.update_rate = update_rate_hz

    def generate_hints(self, dtw_result, pitch, reference):
        """Generate live hints."""
        return {
            "lead_lag_ms": int,      # -200 to +200 ms
            "on_note": bool,          # Within ±50 cents
            "pitch_error_cents": float,
            "confidence": float,      # 0-1
            "status": str,            # "good", "re-acquiring", "error"
        }
```

---

## Implementation Steps

### Phase 1: Core Streaming (Week 1)

#### Step 1.1: Audio Buffer
**File:** `src/iqrah_audio/streaming/buffer.py`

```python
class StreamingAudioBuffer:
    """
    Ring buffer for streaming audio.

    Features:
    - Configurable window size (default 3s)
    - Efficient numpy circular buffer
    - Thread-safe for async audio input
    """
```

**Tasks:**
- [x] Plan architecture
- [ ] Implement ring buffer with numpy
- [ ] Add thread safety (locks)
- [ ] Test with simulated stream

**Target:** <1ms overhead per frame

#### Step 1.2: Incremental Pitch Extraction
**File:** `src/iqrah_audio/streaming/pitch_stream.py`

```python
class IncrementalPitchExtractor:
    """
    Incremental pitch extraction with caching.

    Optimization:
    - Cache computed frames
    - Only process new samples
    - Sliding window approach
    """
```

**Tasks:**
- [ ] Implement frame caching
- [ ] Add incremental YIN extraction
- [ ] Benchmark: target <5ms per frame

**Target:** 10ms per 10ms frame → RTF 1.0 (real-time)

### Phase 2: Enhanced Online-DTW (Week 1-2)

#### Step 2.1: Anchor Detection
**File:** `src/iqrah_audio/streaming/anchors.py`

```python
class AnchorDetector:
    """Detect alignment anchors in real-time."""
```

**Anchor Types:**
1. **Silence** - RMS < threshold for >200ms
2. **Plosives** - High spectral flatness bursts
3. **Long notes** - Stable F0 for >500ms

**Tasks:**
- [ ] Implement silence detector
- [ ] Implement plosive detector
- [ ] Implement long note detector
- [ ] Test on Husary audio

#### Step 2.2: Online-DTW Enhancement
**File:** `src/iqrah_audio/streaming/online_dtw.py`

Enhance existing `OnlineDTWAligner` with:

```python
class EnhancedOnlineDTW(OnlineDTWAligner):
    """Enhanced with anchors and confidence gating."""

    def update(self, query_frame, reference, anchors=None):
        """Update with anchor-based drift correction."""

        # 1. Compute DTW alignment (existing)
        # 2. Check confidence
        # 3. Apply anchor correction if available
        # 4. Update drift estimate
```

**Tasks:**
- [ ] Add anchor integration
- [ ] Implement confidence gating
- [ ] Add drift correction
- [ ] Smooth lead/lag estimates

**Target:** <10ms per update

### Phase 3: Live Feedback (Week 2)

#### Step 3.1: Feedback Generator
**File:** `src/iqrah_audio/streaming/feedback.py`

```python
class LiveFeedback:
    """Generate real-time coaching feedback."""

    def generate_hints(self, alignment, pitch, reference):
        """Generate hints at 10-20 Hz."""

        hints = {
            "timestamp": time.time(),
            "lead_lag_ms": self._calculate_lead_lag(alignment),
            "pitch_error_cents": self._calculate_error(pitch, reference),
            "on_note": abs(error) < 50,
            "confidence": alignment.confidence,
            "status": self._determine_status(confidence),
        }

        return hints
```

**Tasks:**
- [ ] Implement hint generation
- [ ] Add smoothing (avoid jitter)
- [ ] Rate limiting (10-20 Hz max)
- [ ] Status determination logic

### Phase 4: Integration (Week 2)

#### Step 4.1: Real-Time Pipeline
**File:** `src/iqrah_audio/streaming/pipeline.py`

```python
class RealtimePipeline:
    """Complete real-time analysis pipeline."""

    def __init__(self, reference_audio, sample_rate=22050):
        self.buffer = StreamingAudioBuffer()
        self.pitch_extractor = IncrementalPitchExtractor()
        self.anchor_detector = AnchorDetector()
        self.dtw = EnhancedOnlineDTW()
        self.feedback = LiveFeedback()

        # Pre-process reference
        self.ref_pitch = self._extract_reference(reference_audio)

    def process_audio_chunk(self, audio_chunk):
        """Process incoming audio chunk (10-50ms)."""

        # 1. Add to buffer
        self.buffer.push_samples(audio_chunk)

        # 2. Extract pitch (incremental)
        user_pitch = self.pitch_extractor.process_buffer(
            self.buffer.get_window()
        )

        # 3. Detect anchors
        anchors = self.anchor_detector.detect(user_pitch, self.buffer)

        # 4. Update alignment
        alignment = self.dtw.update(
            user_pitch[-1],  # Latest frame
            self.ref_pitch,
            anchors=anchors
        )

        # 5. Generate feedback
        hints = self.feedback.generate_hints(
            alignment, user_pitch, self.ref_pitch
        )

        return hints
```

**Tasks:**
- [ ] Implement pipeline
- [ ] Add error handling
- [ ] Test with simulated stream
- [ ] Measure end-to-end latency

**Target:** <100ms total latency

### Phase 5: Optimization (Week 3)

#### Step 5.1: Profiling
```bash
python -m cProfile -o profile.stats real_time_demo.py
```

**Focus areas:**
- DTW computation (likely bottleneck)
- Feature extraction overhead
- Buffer management

#### Step 5.2: Optimizations

1. **Numba JIT for DTW**
```python
import numba

@numba.jit(nopython=True)
def dtw_cost_fast(query, reference, band_width):
    """JIT-compiled DTW cost computation."""
```

2. **Vectorized Operations**
- Replace loops with numpy ops
- Batch feature extraction

3. **Reduce Buffer Copies**
- Use memoryviews
- In-place operations

**Target:** 50% speedup → <50ms latency

### Phase 6: Testing & Demo (Week 3)

#### Step 6.1: Simulated Stream Test
```python
def simulate_audio_stream(audio_file, chunk_size_ms=50):
    """Simulate real-time audio stream from file."""

    audio, sr = sf.read(audio_file)
    chunk_samples = int(sr * chunk_size_ms / 1000)

    for i in range(0, len(audio), chunk_samples):
        chunk = audio[i:i+chunk_samples]
        yield chunk
        time.sleep(chunk_size_ms / 1000)  # Simulate real-time
```

#### Step 6.2: Interactive Demo
**File:** `examples/real_time_demo.py`

```python
"""
Real-Time Quranic Recitation Coaching Demo
===========================================

Simulates live audio stream and shows real-time feedback.
"""

def main():
    # Load reference (Husary)
    ref_audio, sr = sf.read("media/husary/01.mp3")

    # Create pipeline
    pipeline = RealtimePipeline(ref_audio, sample_rate=sr)

    # Simulate stream
    user_audio, _ = sf.read("user_recording.wav")

    print("Starting real-time analysis...")
    print("=" * 60)

    for chunk in simulate_audio_stream(user_audio, chunk_size_ms=50):
        hints = pipeline.process_audio_chunk(chunk)

        # Display feedback
        print(f"Lead/Lag: {hints['lead_lag_ms']:+4d}ms | "
              f"Error: {hints['pitch_error_cents']:+5.1f}¢ | "
              f"On-Note: {'✓' if hints['on_note'] else '✗'} | "
              f"Conf: {hints['confidence']:.2f} | "
              f"{hints['status']}")

        # Check latency
        latency_ms = (time.time() - hints['timestamp']) * 1000
        if latency_ms > 100:
            print(f"⚠️  High latency: {latency_ms:.1f}ms")
```

**Tasks:**
- [ ] Create demo script
- [ ] Test with Husary audio
- [ ] Measure actual latency
- [ ] Create visualization (optional)

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Total Latency** | <100ms | Audio in → Hints out |
| **Update Rate** | 10-20 Hz | Feedback frequency |
| **Pitch Extraction** | <10ms | Per frame |
| **DTW Update** | <10ms | Per frame |
| **Memory** | <150MB | Peak usage |
| **CPU** | <50% | Single core |

---

## Latency Budget (100ms total)

```
Audio capture:       10-20ms  (hardware + driver)
Buffer management:    2-5ms
Pitch extraction:     5-10ms
Anchor detection:     2-5ms
DTW update:           10-20ms
Feedback generation:  2-5ms
UI update:            5-10ms
Buffer/overhead:      10-20ms
────────────────────────────
TOTAL:                50-95ms ✅
```

---

## Risk Mitigation

### Risk 1: DTW too slow
**Mitigation:**
- Use smaller band width (±30 frames)
- Reduce feature dimensions
- JIT compilation with numba
- Consider FastDTW approximation

### Risk 2: Anchor detection unreliable
**Mitigation:**
- Multiple anchor types (redundancy)
- Fallback to simple lead/lag estimation
- User can disable anchors

### Risk 3: Jittery feedback
**Mitigation:**
- Smooth lead/lag with exponential moving average
- Confidence gating (freeze when low)
- Rate limiting (max 20 Hz updates)

---

## Success Criteria

### Phase 1 ✅
- [ ] Streaming buffer working
- [ ] Incremental pitch extraction <10ms
- [ ] Basic online-DTW functional

### Phase 2 ✅
- [ ] Anchor detection working
- [ ] Confidence gating implemented
- [ ] Drift correction validated

### Phase 3 ✅
- [ ] Live feedback generated
- [ ] <100ms end-to-end latency
- [ ] Stable feedback (no jitter)

### Phase 4 ✅
- [ ] Demo running with real audio
- [ ] Visual feedback working
- [ ] User testing positive

---

## Next Steps (Immediate)

1. **Create streaming module structure**
   ```bash
   mkdir -p src/iqrah_audio/streaming
   touch src/iqrah_audio/streaming/__init__.py
   touch src/iqrah_audio/streaming/buffer.py
   touch src/iqrah_audio/streaming/pitch_stream.py
   touch src/iqrah_audio/streaming/anchors.py
   touch src/iqrah_audio/streaming/online_dtw.py
   touch src/iqrah_audio/streaming/feedback.py
   touch src/iqrah_audio/streaming/pipeline.py
   ```

2. **Implement Step 1.1: Streaming Buffer**
   - Ring buffer with numpy
   - Thread-safe operations
   - Test with simulated stream

3. **Implement Step 1.2: Incremental Pitch**
   - Frame caching
   - Incremental YIN
   - Benchmark latency

Ready to start implementation! 🚀
