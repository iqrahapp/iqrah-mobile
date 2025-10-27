# ML-Based Alignment Architecture Plan

## Current Situation Assessment

### What We Have
1. **V2 DTW System** - 92.3% accuracy, <5ms latency
   - Blind pitch matching without word context
   - Generic feedback like "Pitch too low 266 cents"
   - User has no idea which word to recite next

2. **Unused Goldmine Data**
   - `data/husary/segments/segments.json` (2.0M)
     - Format: `{surah}:{ayah}` → `{surah_number, ayah_number, audio_url, segments: [[word_id, start_ms, end_ms]]}`
     - Example: `"1:1": {"surah_number":1, "ayah_number":1, "audio_url":"...", "segments":[[1,800,1960],[2,2120,2600]...]}`
   - `data/indopak.json`
     - Format: `{surah}:{ayah}` → `{id, verse_key, surah, ayah, text}`
     - Contains full Quran word-by-word Arabic text

3. **Functional Web UI**
   - Real-time pitch visualization
   - WebSocket streaming
   - Reference audio playback

### The Problem
**"The project, as is, is kinda bad"** - User has no word-level guidance. Current approach:
- ❌ Blind DTW pitch matching without knowing what word is expected
- ❌ Generic feedback that doesn't help learning
- ❌ No visual indication of which word to recite
- ❌ Completely ignoring annotated segment data

---

## Proposed Solution: Hybrid ML + DTW Architecture

### Phase 1: Data Exploration & Understanding (Week 1)
**Goal:** Understand available resources and data format

#### 1.1 Parse Existing Data
```python
# experiments/explore_data.py
import json

# Load and analyze segments
with open('data/husary/segments/segments.json') as f:
    segments = json.load(f)

# Load and analyze Quran text
with open('data/indopak.json') as f:
    quran_text = json.load(f)

# Generate report:
# - How many ayahs have segments?
# - Average words per ayah
# - Segment timing resolution (ms)
# - Coverage analysis
```

#### 1.2 Document Data Schema
- Create `docs/DATA_SCHEMA.md` with:
  - Segments structure
  - Quran text structure
  - Audio URL patterns
  - Word-to-segment mapping logic

**Deliverable:** Understanding of data coverage and quality

---

### Phase 2: CTC Prototype (Week 2-3)
**Goal:** Implement ML-based forced alignment for known text

#### 2.1 Install Dependencies
```bash
pip install torchaudio transformers sherpa-onnx
```

#### 2.2 Implement Offline Forced Alignment (MMS-FA)
```python
# src/iqrah_audio/align_ctc/offline.py
from transformers import Wav2Vec2ForCTC, Wav2Vec2Processor
import torchaudio

class ForcedAligner:
    def __init__(self):
        # Use Meta's MMS model fine-tuned for Arabic
        self.processor = Wav2Vec2Processor.from_pretrained(
            "facebook/mms-1b-all", target_lang="ara"
        )
        self.model = Wav2Vec2ForCTC.from_pretrained(
            "facebook/mms-1b-all", target_lang="ara"
        )

    def align(self, audio_path: str, expected_text: str) -> List[WordSpan]:
        """
        Returns word boundaries with confidence scores

        Returns:
            [
                WordSpan(word="بِسْمِ", start_ms=800, end_ms=1960, confidence=0.95),
                WordSpan(word="ٱللَّهِ", start_ms=2120, end_ms=2600, confidence=0.92),
                ...
            ]
        """
        # Implementation using forced alignment
        pass
```

#### 2.3 Implement Real-time CTC Streaming (Sherpa-ONNX)
```python
# src/iqrah_audio/align_ctc/streaming.py
from sherpa_onnx import OnlineRecognizer

class StreamingCTC:
    def __init__(self):
        self.recognizer = OnlineRecognizer.from_transducer(
            # Use pre-trained Arabic streaming model
            encoder="path/to/encoder.onnx",
            decoder="path/to/decoder.onnx",
            joiner="path/to/joiner.onnx"
        )

    def process_chunk(self, audio_chunk: np.ndarray) -> FrameTimestamps:
        """
        Returns frame-level timestamps for alignment anchors

        Returns:
            FrameTimestamps(
                frames=[
                    (frame_idx=10, phoneme="b", confidence=0.9),
                    (frame_idx=25, phoneme="i", confidence=0.85),
                    ...
                ]
            )
        """
        # Implementation
        pass
```

**Deliverable:** Working CTC alignment for offline and streaming

---

### Phase 3: DTW vs CTC Evaluation (Week 4)
**Goal:** Compare performance and identify best use case for each

#### 3.1 Evaluation Framework
```python
# experiments/align_ctc_vs_dtw.py
import numpy as np
from typing import List, Tuple

def evaluate_alignment(
    reference_segments: List[Tuple[int, int]],  # From segments.json
    predicted_boundaries: List[int],             # From CTC or DTW
    method: str  # "ctc" or "dtw"
) -> Dict[str, float]:
    """
    Metrics:
    1. Word Boundary MAE (Mean Absolute Error in ms)
    2. Frame Drift (accumulated error over time)
    3. Confidence Correlation (CTC confidence vs DTW path cost)
    4. Real-Time Factor (RTF - processing time / audio duration)
    """
    mae = np.mean([abs(pred - ref) for pred, ref in zip(...)])
    drift = calculate_drift(...)
    rtf = processing_time / audio_duration

    return {
        "mae_ms": mae,
        "frame_drift": drift,
        "rtf": rtf,
        "method": method
    }
```

#### 3.2 Run Experiments
- Test on 5 reference ayahs (Al-Fatiha + 4 others)
- Measure:
  - **Word boundary accuracy** (target: ≤60ms MAE)
  - **Frame drift** (target: ≤5 frames over 30s)
  - **Confidence correlation** (target: ≥0.7 correlation)
  - **Real-Time Factor** (target: RTF ≤0.5 for streaming)

#### 3.3 Generate Report
```markdown
# doc/align_ctc_vs_dtw.md

## Results Summary

| Metric            | DTW V2   | CTC Offline   | CTC Streaming |
| ----------------- | -------- | ------------- | ------------- |
| Word Boundary MAE | 120ms    | 45ms          | 80ms          |
| Frame Drift (30s) | 8 frames | 2 frames      | 4 frames      |
| Confidence Corr.  | 0.65     | 0.88          | 0.75          |
| RTF               | 0.02     | 2.5 (offline) | 0.3           |

## Recommendations
- **CTC Offline**: Best for post-analysis and grading
- **CTC Streaming**: Best for real-time word tracking with known text
- **DTW V2**: Best for pitch-only feedback when text is unknown

## Hybrid Approach
Use CTC when text is known (Quran recitation), fallback to DTW for free practice
```

**Deliverable:** Evidence-based decision on when to use each method

---

### Phase 4: Word-Level UI Redesign (Week 5)
**Goal:** Replace blind pitch matching with intelligent word guidance

#### 4.1 New UI Components
```html
<!-- static/index_v2.html -->
<div id="quran-text">
    <div class="ayah" data-ayah="1:1">
        <span class="word" data-word="1" data-start="800" data-end="1960">بِسْمِ</span>
        <span class="word" data-word="2" data-start="2120" data-end="2600">ٱللَّهِ</span>
        <span class="word current" data-word="3" data-start="2720" data-end="3360">ٱلرَّحْمَٰنِ</span>
        <!-- Current word highlighted -->
        <span class="word" data-word="4">ٱلرَّحِيمِ</span>
    </div>
</div>

<div id="feedback-panel">
    <h3>Word Progress</h3>
    <p>Current Word: <strong id="current-word-text">ٱلرَّحْمَٰنِ</strong></p>
    <p>Expected Timing: <span id="word-timing">2720-3360 ms</span></p>
    <p>Your Performance: <span id="word-score" class="score-good">Good!</span></p>
</div>

<div id="pitch-canvas-container">
    <!-- Keep existing pitch visualization -->
    <canvas id="pitch-canvas"></canvas>
</div>
```

#### 4.2 Enhanced Feedback Logic
```javascript
// static/app_v2.js
class WordLevelFeedback {
    constructor() {
        this.currentWordIndex = 0;
        this.segments = null;
        this.quranText = null;
    }

    async loadAyah(surah, ayah) {
        // Load segments from backend
        const resp = await fetch(`/api/segments/${surah}/${ayah}`);
        this.segments = await resp.json();

        // Display words with timing info
        this.renderWords();
    }

    updateProgress(currentTimeMs) {
        // Determine which word user should be on
        const expectedWord = this.segments.find(seg =>
            currentTimeMs >= seg.start && currentTimeMs <= seg.end
        );

        // Highlight current word
        document.querySelectorAll('.word').forEach(w => w.classList.remove('current'));
        const wordEl = document.querySelector(`[data-word="${expectedWord.id}"]`);
        wordEl.classList.add('current');

        // Show word-specific feedback
        this.showWordFeedback(expectedWord);
    }

    showWordFeedback(word) {
        const feedback = this.ctcAlignment.getWordScore(word.id);

        if (feedback.accuracy > 0.9) {
            this.showMessage(`✓ Excellent "${word.text}"!`, 'success');
        } else if (feedback.accuracy > 0.7) {
            this.showMessage(`⚠ "${word.text}" needs work`, 'warning');
        } else {
            this.showMessage(`✗ Retry "${word.text}"`, 'error');
        }
    }
}
```

**Deliverable:** Word-aware UI with real-time guidance

---

### Phase 5: Backend Integration (Week 6)
**Goal:** Connect CTC alignment to existing pipeline

#### 5.1 New API Endpoints
```python
# app.py additions
@app.get("/api/segments/{surah}/{ayah}")
async def get_segments(surah: int, ayah: int):
    """Return word segments for an ayah"""
    key = f"{surah}:{ayah}"
    segments_data = load_segments()
    quran_text = load_quran_text()

    return {
        "segments": segments_data[key]["segments"],
        "text": quran_text[key]["text"],
        "audio_url": segments_data[key]["audio_url"]
    }

@app.post("/api/align/ctc")
async def align_with_ctc(audio: UploadFile, expected_text: str):
    """Perform CTC forced alignment"""
    aligner = ForcedAligner()
    word_spans = await aligner.align(audio, expected_text)

    return {
        "word_boundaries": [
            {"word": ws.word, "start_ms": ws.start_ms, "end_ms": ws.end_ms, "confidence": ws.confidence}
            for ws in word_spans
        ]
    }
```

#### 5.2 Hybrid Mode Selection
```python
# core/api/alignment_auto.py
def select_alignment_method(has_known_text: bool, is_streaming: bool) -> str:
    """
    Decision tree:
    - Known text + offline → CTC Offline (best accuracy)
    - Known text + streaming → CTC Streaming (real-time word tracking)
    - Unknown text → DTW V2 (pitch-only feedback)
    """
    if has_known_text:
        return "ctc_streaming" if is_streaming else "ctc_offline"
    return "dtw_v2"
```

**Deliverable:** Seamless integration of CTC and DTW

---

### Phase 6: Pipeline Configuration (Week 7)
**Goal:** Make alignment method configurable

#### 6.1 Update Pipeline Config
```yaml
# pipeline.yaml
alignment:
  default_method: "auto"  # auto, ctc_offline, ctc_streaming, dtw_v2

  ctc_offline:
    model: "facebook/mms-1b-all"
    target_lang: "ara"
    device: "cuda"  # or "cpu"
    batch_size: 1

  ctc_streaming:
    model_path: "models/sherpa_arabic_streaming"
    chunk_size_ms: 200
    max_latency_ms: 500

  dtw_v2:
    use_delta_pitch: true
    huber_loss: true
    window_size: 50

word_feedback:
  enabled: true
  min_word_confidence: 0.7
  highlight_current_word: true
  show_timing_hints: true
```

**Deliverable:** Flexible configuration for different use cases

---

## Success Metrics

### Technical Targets
- ✅ Word boundary MAE ≤60ms (CTC offline)
- ✅ Frame drift ≤5 frames over 30s
- ✅ Real-Time Factor ≤0.5 (streaming)
- ✅ Confidence correlation ≥0.7

### User Experience Goals
- ✅ User knows which word to recite at any moment
- ✅ Feedback is word-specific, not generic
- ✅ Visual highlighting of current expected word
- ✅ Timing hints for each word

---

## Timeline Summary

| Week | Phase               | Deliverable                     |
| ---- | ------------------- | ------------------------------- |
| 1    | Data Exploration    | DATA_SCHEMA.md, coverage report |
| 2-3  | CTC Prototype       | Working offline + streaming CTC |
| 4    | Evaluation          | align_ctc_vs_dtw.md report      |
| 5    | UI Redesign         | Word-level UI with highlighting |
| 6    | Backend Integration | New API endpoints               |
| 7    | Configuration       | pipeline.yaml with hybrid mode  |

---

## Next Immediate Steps

1. **Parse segments.json** - Understand coverage and quality
2. **Parse indopak.json** - Map words to segments
3. **Create data analysis report** - Document findings
4. **Install torchaudio + sherpa-onnx** - Set up CTC dependencies
5. **Prototype offline CTC** - Test on one ayah (Al-Fatiha)

---

## Questions to Answer

1. **Coverage**: What percentage of ayahs have segment annotations?
2. **Quality**: How accurate are the manual segment timings?
3. **CTC Performance**: Can we achieve <60ms word boundary accuracy?
4. **Streaming Latency**: Can we get <500ms real-time feedback with CTC?
5. **Hybrid Logic**: When should we use CTC vs DTW?

---

## Risk Mitigation

### Risk: CTC models are too slow for real-time
**Mitigation:**
- Keep DTW V2 as fallback
- Use Sherpa-ONNX optimized streaming models
- Implement CPU/GPU auto-detection

### Risk: Arabic CTC models have poor accuracy
**Mitigation:**
- Fine-tune on Quran-specific data (segments.json)
- Hybrid approach: CTC for word boundaries + DTW for pitch

### Risk: Segment annotations are incomplete
**Mitigation:**
- Generate missing segments using CTC offline
- Crowdsource manual corrections
- Fallback to DTW when segments unavailable

---

## Open Questions for User

1. Should we **fine-tune CTC on Quran data** or use pre-trained models?
2. Do you have **manual verification** for segment accuracy?
3. Should we support **multiple Qaris** or just Husary?
4. What's the **priority**: real-time streaming vs post-recitation analysis?
