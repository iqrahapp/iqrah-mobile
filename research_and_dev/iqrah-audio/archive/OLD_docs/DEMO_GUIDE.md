# Real-Time Recitation Analysis Demo Guide

This guide shows how to use the `demo_realtime.py` script to demonstrate the complete real-time Quranic recitation analysis pipeline.

## Quick Start

```bash
# Self-test with reference audio (Husary Al-Fatiha)
python demo_realtime.py

# Analyze user recitation against reference
python demo_realtime.py --user path/to/user_recitation.mp3

# Use custom reference
python demo_realtime.py --reference path/to/reference.mp3 --user path/to/user.mp3

# Quiet mode (results only)
python demo_realtime.py --quiet
```

## What the Demo Shows

The demo demonstrates the **complete real-time streaming pipeline** with:

1. **Reference Loading** - Loads and analyzes reference recitation (default: Husary Al-Fatiha)
2. **Pipeline Initialization** - Sets up all streaming components with optimal configuration
3. **Real-Time Processing** - Processes audio in small chunks (~12ms) simulating live input
4. **Live Feedback** - Generates coaching feedback at 15 Hz with visual cues
5. **Performance Metrics** - Tracks and reports latency, accuracy, and quality metrics

## Performance Characteristics

### Latency
- **Average: 6-7ms per chunk** (well under <100ms target)
- **Breakdown:**
  - Pitch extraction: ~4.6ms
  - DTW alignment: ~1.3ms
  - Feedback generation: <0.1ms
- **P95 latency: <7ms** (consistent performance)

### Throughput
- Processes **~4920 chunks/minute** (512 samples @ 44.1kHz)
- **15 Hz feedback rate** (updates every ~67ms)
- **Real-time factor: ~2x** (processes 57s audio in 30s)

## Command-Line Options

```bash
python demo_realtime.py [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--reference PATH` | Path to reference recitation | `data/husary/surahs/01.mp3` |
| `--user PATH` | Path to user recitation | `None` (self-test mode) |
| `--chunk-size N` | Audio chunk size in samples | `512` |
| `--update-rate HZ` | Feedback update rate (Hz) | `15.0` |
| `--quiet` | Suppress detailed output | `False` |

### Chunk Size Guidelines

| Chunk Size | Latency | Use Case |
|------------|---------|----------|
| 256 | ~6ms | Ultra-low latency |
| 512 | ~12ms | **Recommended** (default) |
| 1024 | ~23ms | Lower CPU usage |
| 2048 | ~46ms | Batch processing |

## Output Interpretation

### Visual Feedback Cues

During processing, you'll see real-time feedback with visual indicators:

```
  [green ] ✓ Excellent!
  [yellow] ⚠ Slightly high
  [yellow] ⚠ Speed up (210ms ahead)
  [red   ] ✗ Lost tracking - please continue reciting
  [gray  ] ○ Starting analysis...
```

| Color | Icon | Meaning |
|-------|------|---------|
| **Green** | ✓ | Perfect recitation (on pitch, on time) |
| **Yellow** | ⚠ | Minor issue (pitch deviation or timing off) |
| **Red** | ✗ | Major issue (lost tracking, very poor quality) |
| **Gray** | ○ | Acquiring signal (starting/recovering) |

### Results Summary

After processing, you'll see comprehensive results:

#### 1. Performance Metrics
```
📊 Performance Metrics:
  Total frames processed: 4920
  Total hints generated: 327
  Audio duration: 57.12s
```

#### 2. Latency Breakdown
```
⏱  Latency Breakdown:
  Pitch extraction: 4.62ms
  Anchor detection: 0.00ms
  DTW alignment:    1.33ms
  Feedback gen:     0.00ms
  ─────────────────────────────
  TOTAL:            5.95ms
```

**Interpretation:**
- **<10ms** = Excellent (real-time capable)
- **10-50ms** = Good (acceptable for most uses)
- **50-100ms** = Acceptable (noticeable lag)
- **>100ms** = Poor (significant lag)

#### 3. Feedback Quality
```
🎯 Feedback Quality:
  ✓ good      :  120 ( 36.7%)
  ⚠ warning   :  150 ( 45.9%)
  ✗ error     :   50 ( 15.3%)
  ○ acquiring :    7 (  2.1%)

  Overall: GOOD (36.7% good)
```

**Quality Grades:**
- **EXCELLENT**: >80% good
- **GOOD**: 50-80% good
- **FAIR**: 20-50% good
- **NEEDS IMPROVEMENT**: <20% good

#### 4. Alignment State
```
🎵 Final Alignment State:
  Reference position: 4670
  Lead/lag: +125.3ms
  Confidence: 0.85
  Status: tracking
  Drift estimate: -12.50
```

**Key Metrics:**
- **Lead/lag**: Timing offset (positive = ahead, negative = behind)
- **Confidence**: 0.0-1.0 (higher is better, >0.6 is tracking)
- **Status**: `tracking`, `acquiring`, `lost`, `anchored`
- **Drift estimate**: Accumulated timing drift

## Example Use Cases

### 1. Self-Test (Validate Pipeline)
```bash
python demo_realtime.py --quiet
```

Tests the pipeline by aligning reference against itself. Should show:
- Low latency (<10ms)
- High confidence (>0.8)
- Tracking status

### 2. Analyze User Recitation
```bash
python demo_realtime.py \
  --reference data/husary/surahs/01.mp3 \
  --user recordings/student_fatiha.mp3
```

Compares student recitation to Sheikh Husary's Al-Fatiha.

### 3. Performance Benchmarking
```bash
# Test different chunk sizes
for size in 256 512 1024 2048; do
  echo "=== Chunk size: $size ==="
  python demo_realtime.py --chunk-size $size --quiet | grep "Average:"
done
```

### 4. Batch Testing
```bash
# Test multiple user recordings
for recording in recordings/*.mp3; do
  echo "Testing: $recording"
  python demo_realtime.py \
    --reference data/husary/surahs/01.mp3 \
    --user "$recording" \
    --quiet | grep "Overall:"
done
```

## Integration Guide

### Using the Pipeline in Your Code

```python
from iqrah_audio.streaming import RealtimePipeline, PipelineConfig, RealtimeHints

# 1. Load reference audio
reference_audio, sr = librosa.load("reference.mp3", sr=22050)

# 2. Configure pipeline
config = PipelineConfig(
    sample_rate=sr,
    update_rate_hz=15.0,
    enable_anchors=True,
)

# 3. Create pipeline with callback
def on_hints(hints: RealtimeHints):
    print(f"[{hints.visual_cue}] {hints.message}")

pipeline = RealtimePipeline(
    reference_audio=reference_audio,
    config=config,
    on_hints_callback=on_hints,
)

# 4. Process streaming audio
for chunk in audio_stream:
    hints = pipeline.process_chunk(chunk)
    if hints:
        # Update UI, log, etc.
        update_ui(hints)

# 5. Get statistics
stats = pipeline.get_stats()
print(f"Total latency: {stats.total_latency_ms:.2f}ms")
```

### Real-Time Audio Input (Live Microphone)

```python
import sounddevice as sd

# Callback for audio input
def audio_callback(indata, frames, time, status):
    chunk = indata[:, 0]  # Mono
    hints = pipeline.process_chunk(chunk)
    if hints:
        display_feedback(hints)

# Start audio stream
with sd.InputStream(
    samplerate=22050,
    blocksize=512,
    channels=1,
    callback=audio_callback,
):
    print("Listening... (press Ctrl+C to stop)")
    sd.sleep(60000)  # 60 seconds
```

## Troubleshooting

### Issue: High Latency (>50ms)

**Solutions:**
1. Reduce chunk size: `--chunk-size 256`
2. Disable anchors in config: `enable_anchors=False`
3. Check CPU usage (close other programs)

### Issue: "Lost tracking" Messages

**Causes:**
- Poor audio quality (noise, distortion)
- Different recitation style from reference
- Microphone issues

**Solutions:**
1. Use higher quality audio
2. Try different reference reciter
3. Adjust confidence threshold in config

### Issue: Memory Usage Growing

**Solutions:**
1. Call `pipeline.reset()` between recitations
2. Reduce `max_cache_frames` in extractor
3. Limit `buffer_size_s` in config

## Performance Tips

### For Lowest Latency
```python
config = PipelineConfig(
    hop_length=256,        # Smaller hop
    chunk_size=256,        # Smaller chunks
    enable_anchors=False,  # Skip anchor detection
    update_rate_hz=20.0,   # Higher feedback rate
)
```

### For Best Accuracy
```python
config = PipelineConfig(
    hop_length=512,              # Standard hop
    dtw_window_size=500,         # Larger window
    confidence_threshold=0.7,    # Higher threshold
    enable_anchors=True,         # Use anchors
    anchor_min_confidence=0.8,   # High anchor quality
)
```

### For Low CPU Usage
```python
config = PipelineConfig(
    hop_length=1024,       # Larger hop
    chunk_size=2048,       # Larger chunks
    enable_anchors=False,  # Skip anchors
    update_rate_hz=10.0,   # Lower feedback rate
)
```

## Next Steps

1. **Integrate with UI** - Use the callback mechanism to update visual feedback
2. **Add Recording** - Save user audio and pipeline stats for analysis
3. **Multi-Surah Support** - Extend to other surahs beyond Al-Fatiha
4. **Tajweed Rules** - Add specific tajweed rule detection
5. **Progress Tracking** - Track improvement over multiple sessions

## Technical Details

### Pipeline Components
1. **StreamingAudioBuffer** - Circular buffer for incoming audio
2. **OptimizedIncrementalPitchExtractor** - Ultra-low latency YIN pitch detection
3. **AnchorDetector** - Detects silence, plosives, long notes for drift correction
4. **EnhancedOnlineDTW** - Online dynamic time warping with confidence gating
5. **LiveFeedback** - Rate-limited feedback generation with EMA smoothing

### Algorithms
- **Pitch Detection**: Vectorized YIN (autocorrelation-based)
- **Alignment**: Online DTW with Sakoe-Chiba band constraint
- **Drift Correction**: Anchor-based position adjustment
- **Smoothing**: Exponential moving average (α=0.3)

### Performance Optimizations
- Frame-by-frame processing (no window recomputation)
- Numpy vectorization (all inner loops)
- Circular buffers (O(1) operations)
- Confidence gating (skip bad frames)
- Rate limiting (prevent UI flooding)

## Support

For issues, questions, or contributions:
- GitHub Issues: https://github.com/anthropics/claude-code/issues
- Documentation: See README.md and source code comments
