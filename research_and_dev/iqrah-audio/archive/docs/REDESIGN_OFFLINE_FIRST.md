# Iqrah Audio - Redesign: Offline Analysis First

**Date**: 2025-10-05
**Status**: 🔄 PIVOT - From Real-time to Offline Analysis

---

## Problem Statement

**Current State**: Real-time DTW matching is unreliable and confusing
- Frames accumulate across sessions
- Poor matching even when reciting correctly
- Words don't highlight properly
- System is **fundamentally broken** for real-time use

**Root Cause**: Online DTW is incredibly hard to get right
- Requires perfect pitch detection
- Needs complex temporal alignment
- Sensitive to timing variations
- Not forgiving of user mistakes

**Decision**: **STOP** trying to make real-time work. Build perfect offline analysis first.

---

## New Goal: Perfect Offline Recitation Comparison

### User Flow

```
1. User selects ayah (e.g., Al-Fatihah 1:1)
   ↓
2. User plays Qari's recitation
   → SEE: Pitch graph, word boundaries, timing, melody
   → LEARN: How it should sound
   ↓
3. User clicks "Record My Recitation"
   → Records user's voice
   → AUTO-STOPS on long silence (2-3 seconds)
   ↓
4. System analyzes (5-10 seconds processing)
   → Extract pitch
   → Align with reference using DTW (offline, no time pressure!)
   → Calculate all metrics
   ↓
5. User sees COMPREHENSIVE analysis
   → Side-by-side pitch comparison
   → Color-coded accuracy (green/orange/red)
   → Replay with synchronized highlighting
   → Detailed metrics and scores
   ↓
6. User can replay, try again, or move to next ayah
```

---

## Core Features

### Feature 1: Reference Audio Visualization ✨

**What User Sees**:
- **Pitch curve** (melody) with color gradient
- **Word boundaries** marked with vertical lines and text
- **Tempo markers** showing rhythm
- **Waveform** for visual reference
- **Play/pause controls** with scrubbing

**Technical**:
```python
def visualize_reference(ayah_data):
    # Extract pitch with CREPE
    pitch = extract_pitch(ayah_data.audio)

    # Plot pitch curve
    plt.plot(time, pitch, color='blue', linewidth=2)

    # Add word boundaries
    for word_seg in ayah_data.segments:
        plt.axvline(word_seg.start_ms/1000, color='gray', linestyle='--')
        plt.text(word_seg.start_ms/1000, max_pitch, word_seg.word)

    # Interactive with matplotlib or plotly
    return interactive_plot
```

---

### Feature 2: User Recording with Auto-Stop 🎤

**What User Sees**:
- Big red **"Recording..."** indicator
- Real-time waveform (just visual feedback, no analysis yet!)
- Silence detection countdown: "Silence: 2... 1... STOPPED"
- **No confusing real-time feedback** - just record!

**Technical**:
```python
def record_with_silence_detection(threshold_db=-40, silence_duration=2.5):
    """Record until silence_duration seconds of silence detected."""

    audio_chunks = []
    silence_frames = 0
    required_silence_frames = int(silence_duration * sample_rate / chunk_size)

    while True:
        chunk = stream.read(chunk_size)
        audio_chunks.append(chunk)

        # Check if silent
        rms = np.sqrt(np.mean(chunk**2))
        db = 20 * np.log10(rms + 1e-10)

        if db < threshold_db:
            silence_frames += 1
            if silence_frames >= required_silence_frames:
                print("✓ Silence detected - stopping recording")
                break
        else:
            silence_frames = 0  # Reset on any sound

    return np.concatenate(audio_chunks)
```

---

### Feature 3: Offline DTW Alignment 🎯

**No Time Pressure** - We can use the BEST alignment algorithms!

**Technical**:
```python
def align_user_to_reference(user_audio, ref_audio, ref_segments):
    """
    Offline DTW alignment - take as long as needed to get it RIGHT!
    """

    # Extract pitch for both
    user_pitch = extract_pitch(user_audio)  # CREPE, best quality
    ref_pitch = extract_pitch(ref_audio)

    # Normalize pitches (handle different pitch ranges)
    user_pitch_norm = normalize_pitch(user_pitch)
    ref_pitch_norm = normalize_pitch(ref_pitch)

    # Use FULL DTW (not online!) - much more accurate
    from dtaidistance import dtw

    # Get alignment path
    path = dtw.warping_path(ref_pitch_norm, user_pitch_norm)

    # Map user frames to reference frames
    user_to_ref_mapping = {}
    for user_idx, ref_idx in path:
        user_to_ref_mapping[user_idx] = ref_idx

    # Find which word each user frame corresponds to
    user_word_alignment = []
    for user_idx in range(len(user_pitch)):
        ref_idx = user_to_ref_mapping.get(user_idx, -1)
        if ref_idx >= 0:
            # Convert ref frame to time
            ref_time_ms = (ref_idx / pitch_rate) * 1000

            # Find word at this time
            word_idx = find_word_at_time(ref_time_ms, ref_segments)
            user_word_alignment.append(word_idx)

    return {
        'user_pitch': user_pitch,
        'ref_pitch': ref_pitch,
        'alignment_path': path,
        'user_word_alignment': user_word_alignment
    }
```

---

### Feature 4: Comprehensive Metrics 📊

#### 4.1 Pitch Accuracy (Melody Matching)

**Metric**: Pitch Error per word
- Green: <30 cents error (excellent)
- Orange: 30-60 cents (needs improvement)
- Red: >60 cents (practice needed)

```python
def calculate_pitch_accuracy(alignment):
    """
    Measure how well user matches the MELODY (pitch variations).
    Normalized to handle different absolute pitches.
    """

    word_scores = []

    for word_idx in range(num_words):
        # Get user and ref pitch for this word
        user_frames = [i for i, w in enumerate(alignment['user_word_alignment']) if w == word_idx]
        ref_frames = [alignment['alignment_path'][i][1] for i in user_frames]

        user_pitch_word = alignment['user_pitch'][user_frames]
        ref_pitch_word = alignment['ref_pitch'][ref_frames]

        # Normalize to compare SHAPE not absolute pitch
        user_norm = normalize_pitch_shape(user_pitch_word)
        ref_norm = normalize_pitch_shape(ref_pitch_word)

        # Calculate cents error
        error_cents = np.mean(np.abs(1200 * np.log2(user_norm / ref_norm + 1e-10)))

        word_scores.append({
            'word_idx': word_idx,
            'error_cents': error_cents,
            'status': 'good' if error_cents < 30 else 'warning' if error_cents < 60 else 'error'
        })

    return word_scores
```

#### 4.2 Pitch Range Difference

**Metric**: How different is user's absolute pitch from Qari
- "Your pitch is 2 semitones lower than the Qari (normal for different voices)"
- "Your pitch range is narrower - try using more vocal range"

```python
def calculate_pitch_range_stats(user_pitch, ref_pitch):
    """Measure absolute pitch differences (expected to differ!)"""

    user_median = np.median(user_pitch[user_pitch > 0])
    ref_median = np.median(ref_pitch[ref_pitch > 0])

    semitone_diff = 12 * np.log2(user_median / ref_median)

    user_range = np.percentile(user_pitch[user_pitch > 0], 90) - np.percentile(user_pitch[user_pitch > 0], 10)
    ref_range = np.percentile(ref_pitch[ref_pitch > 0], 90) - np.percentile(ref_pitch[ref_pitch > 0], 10)

    return {
        'semitone_difference': semitone_diff,
        'user_range_hz': user_range,
        'ref_range_hz': ref_range,
        'range_ratio': user_range / ref_range
    }
```

#### 4.3 Tempo Accuracy

**Metric**: Speed differences and consistency
- "Your recitation is 15% slower than the Qari (good for learning!)"
- "High tempo variation detected - practice maintaining steady rhythm"

```python
def calculate_tempo_metrics(alignment):
    """
    Measure tempo differences and stability.
    """

    # Calculate stretching factor from DTW path
    path = alignment['alignment_path']

    local_speeds = []
    for i in range(1, len(path)):
        user_delta = path[i][0] - path[i-1][0]
        ref_delta = path[i][1] - path[i-1][1]
        if ref_delta > 0:
            speed = user_delta / ref_delta  # >1 = user slower, <1 = user faster
            local_speeds.append(speed)

    avg_speed = np.mean(local_speeds)
    speed_std = np.std(local_speeds)

    return {
        'average_tempo_ratio': avg_speed,  # 1.0 = same speed, 1.2 = 20% slower
        'tempo_stability': speed_std,  # Low = stable, High = erratic
        'tempo_status': 'stable' if speed_std < 0.2 else 'unstable'
    }
```

#### 4.4 Pitch Stability (Voice Quality)

**Metric**: Local pitch variations (shakiness)
- "Your voice is very stable! (low vibrato)"
- "High pitch noise detected in words [X, Y] - practice breath control"

```python
def calculate_pitch_stability(pitch, word_alignment, segments):
    """
    Measure local pitch variations (jitter/shimmer).
    High local variation = shaky/breaking voice.
    """

    word_stabilities = []

    for word_idx in range(len(segments)):
        word_frames = [i for i, w in enumerate(word_alignment) if w == word_idx]
        word_pitch = pitch[word_frames]

        # Calculate local variation (difference between consecutive frames)
        pitch_deltas = np.diff(word_pitch)

        # RMS of deltas = stability metric
        stability_score = np.sqrt(np.mean(pitch_deltas**2))

        word_stabilities.append({
            'word_idx': word_idx,
            'stability_score': stability_score,
            'status': 'stable' if stability_score < 10 else 'unstable'
        })

    return word_stabilities
```

#### 4.5 Melody Complexity

**Metric**: Pitch distribution shape
- "Good focused melody (1-2 pitch peaks)"
- "Too many different pitches - simplify your melody"

```python
def calculate_melody_complexity(pitch):
    """
    Measure pitch distribution.
    Good: 1-3 distinct pitch peaks (structured melody)
    Bad: Flat distribution (random pitches)
    """

    # Histogram of pitches
    hist, bins = np.histogram(pitch[pitch > 0], bins=50)

    # Find peaks in distribution
    from scipy.signal import find_peaks
    peaks, _ = find_peaks(hist, prominence=max(hist)*0.1)

    # Calculate entropy (measure of randomness)
    prob = hist / np.sum(hist)
    entropy = -np.sum(prob * np.log(prob + 1e-10))

    return {
        'num_pitch_peaks': len(peaks),
        'pitch_entropy': entropy,
        'complexity_status': 'simple' if len(peaks) <= 3 else 'complex'
    }
```

---

### Feature 5: Visual Comparison UI 🎨

**Split-Screen View**:

```
┌─────────────────────────────────────────────────────────────┐
│  Qari's Recitation (Reference)                              │
│  ────────────────────────────────────────────────────────  │
│  Pitch: [Blue curve with word boundaries]                   │
│  بِسۡمِ    اللهِ    الرَّحۡمٰنِ    الرَّحِيۡمِ              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Your Recitation (Aligned)                                  │
│  ────────────────────────────────────────────────────────  │
│  Pitch: [Green/Orange/Red curve aligned to reference]       │
│  بِسۡمِ    اللهِ    الرَّحۡمٰنِ    الرَّحِيۡمِ              │
│   ✓        ✓         ⚠            ✗                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Metrics                                                    │
│  ────────────────────────────────────────────────────────  │
│  Overall Score: 78/100 ⭐⭐⭐                                │
│                                                              │
│  📊 Pitch Accuracy:        Good (avg 25 cents error)        │
│  🎵 Pitch Range:           2 semitones lower (normal)       │
│  ⏱️  Tempo:                15% slower (good for learning)   │
│  🎤 Voice Stability:       Excellent (low jitter)           │
│  🎶 Melody Complexity:     Good (2 pitch peaks)             │
│                                                              │
│  [▶️ Replay]  [🔄 Try Again]  [➡️ Next Ayah]                │
└─────────────────────────────────────────────────────────────┘
```

---

### Feature 6: Interactive Replay 🔁

**Synchronized Playback**:
- Play both audios side-by-side
- Highlight current word in both
- Show pitch curves moving together
- Pause/resume/scrub anywhere

```javascript
function syncedPlayback(referenceAudio, userAudio, alignment) {
    // Start both audios
    referenceAudio.play();
    userAudio.play();

    // Sync with DTW alignment
    setInterval(() => {
        const refTime = referenceAudio.currentTime;

        // Map to user time using alignment
        const userTime = mapTimeViaAlignment(refTime, alignment);

        // Force sync if drift
        if (Math.abs(userAudio.currentTime - userTime) > 0.1) {
            userAudio.currentTime = userTime;
        }

        // Update UI highlighting
        highlightCurrentWord(refTime);
    }, 50);
}
```

---

## Implementation Plan

### Phase 1: Reference Visualization (Week 1)
- [ ] Load ayah audio
- [ ] Extract pitch with CREPE
- [ ] Plot pitch curve with matplotlib/plotly
- [ ] Add word boundaries from segments
- [ ] Interactive play/pause/scrub
- [ ] **Test**: Can user see and understand Qari's recitation visually?

### Phase 2: Recording with Auto-Stop (Week 1)
- [ ] Implement microphone recording
- [ ] Add silence detection algorithm
- [ ] Visual feedback (waveform, recording indicator)
- [ ] Save recording to file
- [ ] **Test**: Can user easily record their recitation?

### Phase 3: Offline Analysis (Week 2)
- [ ] Extract user pitch with CREPE
- [ ] Implement offline DTW alignment
- [ ] Map user frames to reference words
- [ ] Calculate all 5 metrics
- [ ] Generate analysis report
- [ ] **Test**: Are metrics accurate and useful?

### Phase 4: Comparison UI (Week 2)
- [ ] Split-screen pitch visualization
- [ ] Color-coded word accuracy
- [ ] Metrics dashboard
- [ ] Synchronized replay
- [ ] Try again / Next ayah buttons
- [ ] **Test**: Can user understand their performance?

### Phase 5: ML Enhancements (Week 3-4)
- [ ] Use CTC for better word alignment (optional)
- [ ] Train pronunciation scoring model
- [ ] Add Tajweed error detection
- [ ] Personalized feedback
- [ ] **Test**: Does ML improve accuracy?

---

## Technology Stack

### Backend (Python)
- **FastAPI**: API endpoints
- **CREPE**: Pitch extraction (best quality)
- **dtaidistance**: Full DTW alignment (not online!)
- **librosa**: Audio processing
- **scipy**: Signal processing for metrics
- **numpy**: Numerical computations

### Frontend (JavaScript)
- **Vanilla JS**: Simple, no framework needed
- **Plotly.js**: Interactive plots
- **Wavesurfer.js**: Waveform visualization
- **HTML5 Audio**: Playback controls

### Data
- **Segments**: Word boundaries (already have!)
- **Indopak JSON**: Arabic text
- **Tarteel CDN**: Audio files

---

## API Design

### Endpoint 1: Get Reference Visualization

```http
GET /api/visualize/reference/{surah}/{ayah}

Response:
{
    "audio_url": "https://...",
    "pitch_data": [
        {"time": 0.0, "f0_hz": 120.5},
        {"time": 0.064, "f0_hz": 125.3},
        ...
    ],
    "segments": [
        {"word": "بِسۡمِ", "start": 0.0, "end": 0.48},
        ...
    ],
    "text": "بِسۡمِ اللهِ الرَّحۡمٰنِ الرَّحِيۡمِ"
}
```

### Endpoint 2: Analyze User Recitation

```http
POST /api/analyze/{surah}/{ayah}
Content-Type: multipart/form-data

Body:
- audio: File (user's recording)

Response:
{
    "analysis_id": "uuid",
    "reference": {
        "pitch": [...],
        "segments": [...]
    },
    "user": {
        "pitch": [...],
        "duration": 5.2
    },
    "alignment": {
        "word_accuracy": [
            {"word": "بِسۡمِ", "error_cents": 25, "status": "good"},
            ...
        ],
        "path": [[0,0], [1,1], ...]  // DTW path
    },
    "metrics": {
        "overall_score": 78,
        "pitch_accuracy": {...},
        "pitch_range": {...},
        "tempo": {...},
        "stability": {...},
        "complexity": {...}
    }
}
```

### Endpoint 3: Get Analysis Result

```http
GET /api/analysis/{analysis_id}

Response: Same as Endpoint 2 response
```

---

## File Structure

```
iqrah-audio/
├── app.py                          # FastAPI app (SIMPLIFIED!)
├── requirements.txt
├── static/
│   ├── index.html                  # New simpler UI
│   ├── app.js                      # Offline analysis flow
│   └── styles.css
├── src/
│   └── iqrah_audio/
│       ├── analysis/               # NEW!
│       │   ├── pitch.py           # CREPE extraction
│       │   ├── alignment.py       # Offline DTW
│       │   ├── metrics.py         # All 5 metrics
│       │   └── visualization.py   # Plot generation
│       ├── recording/             # NEW!
│       │   └── silence_detect.py  # Auto-stop recording
│       └── core/
│           └── segments_loader.py # Existing
├── data/
│   └── ... (existing)
└── docs/
    └── REDESIGN_OFFLINE_FIRST.md  # This file!
```

---

## Success Criteria

### Minimum Viable Product (MVP)
- [ ] User can see Qari's pitch visualization
- [ ] User can record with auto-stop
- [ ] User can see side-by-side comparison
- [ ] User can see at least 3 metrics (pitch, tempo, stability)
- [ ] User can replay and try again

### Excellent Product
- [ ] All 5 metrics implemented and accurate
- [ ] Beautiful, intuitive UI
- [ ] Synchronized playback works perfectly
- [ ] Metrics are pedagogically useful
- [ ] Users improve their recitation!

---

## Why This Will Work

### 1. No Time Pressure ✅
- Offline processing can take 10 seconds - doesn't matter!
- Can use best algorithms (full DTW, high-quality CREPE)
- No need for optimization tricks

### 2. Much Simpler ✅
- No WebSocket streaming
- No real-time state management
- No complex pipeline
- Just: Record → Analyze → Show Results

### 3. Actually Useful ✅
- Comprehensive metrics user can learn from
- Visual comparison makes mistakes obvious
- Can replay and practice specific words
- Measurable improvement over time

### 4. Leverages Existing Data ✅
- Segments for word boundaries
- Indopak for text
- Tarteel audio
- No new data needed!

---

## Next Steps

1. **Delete** all real-time code (DTW streaming, WebSocket, etc.)
2. **Create** new simplified `app.py`
3. **Implement** reference visualization (Phase 1)
4. **Test** with users ASAP
5. **Iterate** based on feedback

---

## Timeline

- **Week 1**: Phases 1-2 (Visualization + Recording)
- **Week 2**: Phases 3-4 (Analysis + UI)
- **Week 3-4**: Phase 5 (ML enhancements)
- **Total**: 1 month to excellent product

---

**Status**: Ready to pivot! Let's build something that actually works! 🚀
