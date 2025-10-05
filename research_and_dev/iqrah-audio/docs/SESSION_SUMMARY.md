# Session Summary: Shift from DTW-only to Word-Level Guidance

## What Was Accomplished

### 1. Data Discovery & Analysis
- **Discovered goldmine of annotated data** that was being completely ignored:
  - `data/husary/segments/segments.json`: 6,236 ayahs with word-level timing
  - `data/indopak.json`: Complete Quran text word-by-word
  - **100% coverage** of all 6,236 verses
  - **77,897 total words** with precise timing annotations

### 2. Core Infrastructure Built
- Created `segments_loader.py` - Clean API for accessing annotated data
  - `SegmentsLoader.get_ayah(surah, ayah)` → Returns `AyahData` with text + segments
  - `AyahData.get_word_at_time(ms)` → Find which word should be active
  - `AyahData.get_expected_word_index(ms)` → Next word to recite
- Added FastAPI endpoints:
  - `GET /api/segments/{surah}/{ayah}` → Word segments + Arabic text
  - `GET /api/segments/stats` → Coverage statistics

### 3. Problem Identified
Current system is "kinda bad" because:
- ❌ Blind pitch matching without word context
- ❌ User has no idea which word to recite next
- ❌ Generic feedback like "Pitch too low 266 cents" (unhelpful)
- ❌ No visual highlighting of current expected word
- ❌ Completely ignoring annotated segment data

### 4. Previous Work Summary (OLTW v2-v4)
**V2 Status**: 58% tracking, 0.40 confidence, <1ms latency ✓ WORKS
**V3/V4 Status**: Parameter-free approaches failed (3.5% tracking) ✗ FAILED

**Key Insight**: V2 is production-ready for pitch-only feedback, but insufficient for learning guidance.

---

## Next Steps (Prioritized)

### Phase 0: Quick Win - Word-Level UI (No ML Yet)
**Why**: Immediate UX improvement using existing data
**Effort**: 1-2 days
**Dependencies**: None (data already available)

#### Tasks:
1. **Update Web UI** (`static/index.html`):
   ```html
   <!-- Add word display section -->
   <div class="quran-text-display">
       <div class="ayah" data-verse="1:1">
           <span class="word current" data-word-id="1" data-start="0" data-end="480">بِسۡمِ</span>
           <span class="word" data-word-id="2" data-start="600" data-end="1000">اللهِ</span>
           <span class="word" data-word-id="3">الرَّحۡمٰنِ</span>
           <span class="word" data-word-id="4">الرَّحِيۡمِ</span>
       </div>
   </div>
   ```

2. **Add JavaScript word tracking** (`static/app.js`):
   ```javascript
   class WordLevelFeedback {
       async loadAyah(surah, ayah) {
           const resp = await fetch(`/api/segments/${surah}/${ayah}`);
           this.segments = await resp.json();
           this.renderWords();
       }

       updateCurrentWord(currentTimeMs) {
           // Highlight word that should be active now
           const expectedIdx = this.getExpectedWordIndex(currentTimeMs);
           this.highlightWord(expectedIdx);
       }
   }
   ```

3. **Enhance feedback messages**:
   - Instead of: "Pitch too low 266 cents"
   - Show: "✓ Good 'بِسۡمِ'! Next: 'اللهِ'"

**Output**: User sees which word to recite, with visual highlighting

---

### Phase 1: CTC vs DTW Evaluation
**Why**: Determine if ML alignment beats DTW for word boundaries
**Effort**: 3-5 days
**Dependencies**: Python ML libraries

#### Tasks:
1. **Install Dependencies**:
   ```bash
   pip install torch torchaudio transformers sherpa-onnx
   ```

2. **Implement Offline CTC Forced Alignment**:
   ```python
   # experiments/ctc_prototype.py
   from transformers import Wav2Vec2ForCTC, Wav2Vec2Processor

   class ForcedAligner:
       def __init__(self):
           self.processor = Wav2Vec2Processor.from_pretrained(
               "facebook/mms-1b-all", target_lang="ara"
           )
           self.model = Wav2Vec2ForCTC.from_pretrained(
               "facebook/mms-1b-all", target_lang="ara"
           )

       def align(self, audio_path, expected_text):
           # Returns word boundaries with confidence
           pass
   ```

3. **Run Benchmark on Al-Fatiha**:
   - Metric 1: **Word Boundary MAE** (Mean Absolute Error in ms)
     - Target: ≤60ms
     - Compare predicted boundaries vs ground truth segments
   - Metric 2: **Frame Drift** (accumulated error)
     - Target: ≤5 frames over 30s
   - Metric 3: **Real-Time Factor** (processing_time / audio_duration)
     - Target: ≤0.5 for streaming

4. **Generate Report**:
   ```markdown
   # CTC vs DTW Comparison

   | Metric              | DTW V2  | CTC Offline | CTC Streaming |
   |---------------------|---------|-------------|---------------|
   | Word Boundary MAE   | ???ms   | ???ms       | ???ms         |
   | Frame Drift (30s)   | 8 frames| ???frames   | ???frames     |
   | RTF                 | 0.02    | ???         | ???           |

   ## Recommendation
   - Use CTC when: [...]
   - Use DTW when: [...]
   ```

**Output**: Evidence-based decision on alignment method

---

### Phase 2: Hybrid Integration
**Why**: Use best tool for each scenario
**Effort**: 2-3 days
**Dependencies**: Phase 1 completion

#### Strategy:
```python
def select_alignment_method(has_known_text: bool, is_streaming: bool):
    """
    Decision tree:
    - Known text + offline → CTC Offline (best accuracy for post-analysis)
    - Known text + streaming → CTC Streaming OR DTW (depends on Phase 1 results)
    - Unknown text (free practice) → DTW V2
    """
    if has_known_text:
        if is_streaming:
            return "ctc_streaming"  # or "dtw_v2" if CTC streaming is too slow
        return "ctc_offline"
    return "dtw_v2"
```

#### Tasks:
1. Implement CTC streaming if benchmark shows it's viable
2. Add configuration flag: `alignment_method: "auto" | "ctc_offline" | "ctc_streaming" | "dtw_v2"`
3. Update pipeline to switch methods based on context
4. Add word-level confidence scoring from CTC outputs

**Output**: Intelligent method selection based on use case

---

## Key Questions to Answer

### Before Phase 1:
1. **Do we need real-time streaming CTC?**
   - Option A: Post-recitation analysis only (use CTC offline, easier)
   - Option B: Real-time word tracking (need streaming CTC, harder)

2. **What's the priority?**
   - Learning (post-analysis with detailed feedback) → CTC Offline sufficient
   - Live guidance (real-time word highlighting) → Need streaming solution

### During Phase 1:
1. Can CTC achieve <60ms word boundary accuracy on Quran audio?
2. Is CTC streaming fast enough for <500ms latency?
3. Should we fine-tune on Quran-specific data or use pre-trained models?

---

## Risk Mitigation

### Risk: CTC models too slow for real-time
**Mitigation**:
- Keep DTW V2 as fallback
- Use Sherpa-ONNX optimized models (ONNX Runtime is fast)
- Implement CPU/GPU auto-detection

### Risk: Arabic CTC models have poor accuracy
**Mitigation**:
- Test multiple models (MMS, Wav2Vec2-Arabic, etc.)
- Fine-tune on Quran segments if needed
- Hybrid: CTC for word boundaries + DTW for pitch analysis

### Risk: Segment annotations have errors
**Mitigation**:
- Validate sample of segments manually
- Use CTC offline to detect annotation errors
- Implement confidence thresholds (ignore low-confidence segments)

---

## Files Modified/Created

### Created:
- `src/iqrah_audio/core/segments_loader.py` - Data loader for segments
- `docs/ML_ALIGNMENT_PLAN.md` - Comprehensive 7-phase plan
- `docs/SESSION_SUMMARY.md` - This file

### Modified:
- `app.py` - Added API endpoints for segments
- `src/iqrah_audio/streaming/pipeline.py` - Reverted to v2 as default
- `src/iqrah_audio/streaming/online_dtw_v4.py` - Attempted parameter-free (failed)

### To Modify Next:
- `static/index.html` - Add word display UI
- `static/app.js` - Add word tracking logic

---

## Recommended Immediate Action

**Start with Phase 0 (Quick Win)** before investing in ML:

1. Update UI to fetch and display word segments
2. Add visual word highlighting based on reference audio timestamp
3. Show word-specific feedback instead of generic pitch feedback

**Why this order**:
- Immediate UX improvement with zero ML complexity
- Validates the value of word-level guidance
- UI work is needed regardless of CTC vs DTW choice
- Can be done in parallel with ML evaluation

**After Phase 0**, decide whether to invest in CTC based on:
- User feedback on word-level UI
- Whether real-time streaming is actually needed
- CTC benchmark results

---

## Code Status

### Production Ready:
- ✅ SegmentsLoader - Word segment data access
- ✅ API endpoints - Serving segment data
- ✅ OLTW V2 - Pitch-based DTW (58% tracking, 0.40 conf, <1ms)

### In Progress:
- ⏳ Word-level UI integration
- ⏳ Word tracking in WebSocket updates

### Experimental/Failed:
- ❌ OLTW V3 - Delta-pitch features (1.4% tracking)
- ❌ OLTW V4 - Parameter-free adaptation (3.5% tracking)

### Not Started:
- 🔜 CTC forced alignment prototype
- 🔜 CTC vs DTW benchmark
- 🔜 Hybrid alignment selector

---

## Success Metrics

### Phase 0 Success:
- ✅ User can see which word to recite at any moment
- ✅ Current word is visually highlighted
- ✅ Feedback mentions specific words, not just "pitch too low"

### Phase 1 Success:
- ✅ CTC achieves ≤60ms word boundary MAE
- ✅ Clear data on when to use CTC vs DTW
- ✅ Real-Time Factor ≤0.5 for streaming (if applicable)

### Phase 2 Success:
- ✅ Automatic method selection working
- ✅ No latency regression from V2 baseline
- ✅ Word-level confidence scores available

---

## Notes for Next Session

1. **Don't overfit to self-alignment**: Current v2 has 58% tracking because we tested on perfect self-alignment. Real users will have imperfect recitation, which might actually work better with DTW.

2. **Word segments are gold**: The fact that we have 100% coverage of word-level timing is huge. This makes CTC forced alignment much more attractive than pure ASR.

3. **Quick wins matter**: Getting word-level UI working quickly will show value before investing weeks in ML.

4. **Consider hybrid**: Maybe use segments for coarse alignment (which ayah/word region) and DTW for fine-grained pitch feedback. Best of both worlds.

5. **Test on real recitations**: Once Phase 0 is done, test with actual imperfect recitations to see if the experience is actually better.
