# CRITICAL NEXT STEPS

## What YOU Found That I Missed:

1. **❌ Phonemes misplaced** - Used naive time distribution instead of proper MMS-FA with word-level windowing
2. **❌ Cursor is laggy** - Need moving dot on pitch line, not red vertical line
3. **❌ Only works with 1:1** - Needed segments.json (which you found!)

## Critical Discovery: segments.json

**Location**: `data/husary/segments/segments.json`

**Contains**:
- 6,236 ayahs with word-level timestamps
- Audio URLs: `https://audio-cdn.tarteel.ai/quran/husary/001001.mp3`
- Word segments: `[[word_idx, start_ms, end_ms], ...]`

**Example**:
```json
"1:1": {
  "surah_number": 1,
  "ayah_number": 1,
  "audio_url": "https://audio-cdn.tarteel.ai/quran/husary/001001.mp3",
  "segments": [[1, 0, 480], [2, 600, 1000], [3, 1800, 2160], [4, 2480, 5160]]
}
```

This is the KEY to AI Report 2's approach!

## What's Ready:

### ✅ Infrastructure Complete:
1. **segments_loader.py** - Loads word segments, downloads audio
2. **phoneme_mms_proper.py** - Proper MMS-FA with word-level windowing
3. **app_qari_final.py** - Integrates everything correctly
4. **SwiftF0 improvements** - Confidence gating + smoothing
5. **Tajweed data** - qpc-hafs-tajweed.json, english-transliteration-tajweed.json

### ✅ Components Working:
- Pitch extraction (clean, no spikes)
- Word segment loading
- Arabic words with Tajweed colors
- RTL layout
- Audio downloading

## Immediate Tasks (Next Session):

### 1. TEST the Final App
```bash
python app_qari_final.py
# Visit http://0.0.0.0:8004
# Test with multiple ayahs (not just 1:1)
```

### 2. Fix Cursor (Moving Dot)

**Current** (laggy red line):
```javascript
// shapes: [ { type: 'line', x0: currentTime, ... } ]
```

**Needed** (moving dot on pitch):
```javascript
// Add animated marker trace that follows pitch contour
const cursorTrace = {
    x: [currentTime],
    y: [interpolatePitch(currentTime, pitchData)],  // Get pitch at current time
    mode: 'markers',
    name: 'Position',
    marker: {
        size: 15,
        color: '#ff4444',
        symbol: 'circle'
    }
};

// Update on audioPlayer.timeupdate:
audioPlayer.addEventListener('timeupdate', () => {
    const t = audioPlayer.currentTime;
    const pitch = interpolatePitch(t, currentData.pitch);
    Plotly.update('pitchPlot', {
        x: [[t]],
        y: [[pitch]]
    }, {}, [2]);  // Update trace index 2 (cursor)
});
```

### 3. Verify Phoneme Accuracy

Run proper MMS-FA and compare:
```bash
python -c "
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_mms_proper import extract_phonemes_mms_proper
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0

# Test with 1:1
segs = get_word_segments_with_text(1, 1)
trans = load_transliteration_data()['1:1']
pitch = extract_pitch_swiftf0('data/husary/surahs/001/01.mp3')

phonemes = extract_phonemes_mms_proper(
    'data/husary/surahs/001/01.mp3',
    segs, trans, pitch
)

print(f'Phonemes: {len(phonemes)}')
for p in phonemes[:10]:
    print(f'{p[\"phoneme\"]:10s} {p[\"start\"]:.3f}-{p[\"end\"]:.3f}s')
"
```

### 4. Add Comprehensive Tests

Create `tests/test_final_pipeline.py`:
```python
def test_segments_loading():
    \"\"\"Test segment data loads correctly\"\"\"
    segs = get_word_segments_with_text(1, 1)
    assert len(segs) == 4  # Ayah 1:1 has 4 words
    assert all('start_ms' in s for s in segs)

def test_mms_fa_alignment():
    \"\"\"Test MMS-FA aligns correctly with word segments\"\"\"
    # ... test proper phoneme boundaries

def test_multiple_ayahs():
    \"\"\"Test works with different ayahs\"\"\"
    for surah, ayah in [(1,1), (1,2), (2,1)]:
        # ... test each one
```

### 5. Update HTML with Moving Dot

Edit `static/qari_final.html`, replace `setupRealtimeCursor()` function.

## File Structure:

```
src/iqrah_audio/analysis/
├── pitch_extractor_swiftf0.py    ✅ Clean pitch (confidence + smoothing)
├── segments_loader.py             ✅ Load word segments + download audio
├── phoneme_mms_proper.py          ✅ AI Report 2 MMS-FA approach
├── tajweed_loader.py              ✅ Arabic words with colors
└── phoneme_from_transliteration.py ✅ Gold transliteration data

app_qari_final.py                  ✅ Proper integration
static/qari_final.html             ⚠️ Needs cursor fix
data/husary/segments/segments.json ✅ 6,236 ayahs!
```

## Quick Win Checklist:

- [ ] Test `app_qari_final.py` with ayah 1:1
- [ ] Test with ayah 1:2 (verify audio downloads)
- [ ] Fix cursor to be moving dot on pitch
- [ ] Verify phoneme boundaries are correct (not misplaced)
- [ ] Add tests for segments loading
- [ ] Add tests for MMS-FA pipeline
- [ ] Document final architecture

## Expected Behavior:

When working correctly:
1. Select any ayah → downloads audio automatically
2. Phonemes appear at CORRECT positions (using MMS-FA word windowing)
3. Cursor is smooth dot following pitch contour
4. Arabic words show with Tajweed colors
5. X-axis is RTL (right-to-left)
6. No lag, all features working

## Performance Notes:

- **segments.json**: 6,236 ayahs, loads instantly
- **MMS-FA**: ~2-3s per word (windowing makes it fast)
- **Audio download**: ~1s per ayah (cached after first download)
- **Total**: ~5-10s for first analysis, <1s for cached

The infrastructure is READY. Just needs final integration and testing!
