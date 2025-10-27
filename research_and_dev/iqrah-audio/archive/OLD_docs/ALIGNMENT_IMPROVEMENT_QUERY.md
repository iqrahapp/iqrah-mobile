# Query: Improving Phoneme Alignment Quality for Quranic Recitation

## Context

I'm building a Quranic recitation learning system that compares student recordings against expert reciters (Qaris). The system uses **Wav2Vec2 CTC forced alignment** to extract phoneme-level timestamps from Arabic audio, then scores pronunciation, rhythm, and Tajweed rules.

## Current Pipeline

```python
# 1. Romanize Arabic text
arabic = "بِسۡمِ ٱللَّهِ ٱلرَّحۡمَـٰنِ ٱلرَّحِيمِ"
romanized = uroman.romanize_string(arabic)  # → "bismillaahirrahmaanirrahiim"

# 2. CTC forced alignment
model = torchaudio.pipelines.MMS_FA.get_model()
emissions = model(waveform)  # [T, vocab]
tokens = tokenizer(romanized)
alignment = aligner(emissions, tokens)  # Returns character-level spans

# 3. Map characters to syllables
syllables = ['Bis', 'mil', 'laah', 'ir', 'Rah', 'maan', 'ir', 'Rah', 'eem']
# Group character spans by syllable based on length

# 4. Assign word indices based on word segment timestamps
# e.g., word 1: 0-480ms, word 2: 600-1000ms, etc.
```

## Identified Issues

### Issue #1: Low Coverage (69.4%)
**Problem**: Total phoneme duration (3.55s) only covers 69% of total audio (5.12s).

**Analysis**:
```
Word            Segment Time     Phonemes Cover      Gaps
بِسۡمِ          0-480ms         80-582ms           80ms before, -102ms after
ٱللَّهِ        600-1000ms      703-1325ms         103ms before, -325ms after (!)
ٱلرَّحۡمَـٰنِ   1800-2160ms     1446-2490ms        -354ms before (!), -330ms after (!)
ٱلرَّحِيمِ     2480-5160ms     2590-4417ms        110ms before, 743ms after (!)
```

**Observations**:
- Phonemes often start 80-350ms BEFORE word segment boundaries
- Last word has 743ms gap at end (silence or CTC error?)
- Total 1.57s of audio not covered by any phoneme

### Issue #2: Suspiciously Long Phonemes
**Problem**: 'eem' phoneme = 1446ms (nearly 1.5 seconds!)

**Context**: This is the final syllable in "Raheem" (الرَّحِيمِ). The Tajweed rule is `madda_permissible`, which expects 2-6 "counts" (~200-1400ms), so duration IS plausible. However, when comparing Husary to himself, this creates 0.14 count error (7% of 6 counts), suggesting CTC is absorbing trailing silence into the last phoneme.

### Issue #3: Moderate Boundary Alignment
**Problem**: Mean boundary energy = 0.46 (on scale 0=local_min, 1=local_max)

**Interpretation**: Boundaries are not consistently placed at energy minima. They're roughly midway between local min/max, suggesting CTC boundaries don't respect acoustic phoneme transitions.

## Data Available

### Word-Level Timestamps (High Quality)
```json
{
  "1:1": {
    "segments": [[1, 0, 480], [2, 600, 1000], [3, 1800, 2160], [4, 2480, 5160]]
  }
}
```
These are expert-annotated and very accurate.

### Transliteration (Ground Truth)
```
"Bismil laahir Rahmaanir Raheem"
```
Matches the Arabic text exactly, with Tajweed-aware transcription (long vowels, etc.)

### Audio Features Available
- RMS energy (10ms frames)
- Pitch contour (SwiftF0, very clean)
- MMS-FA CTC emissions [T, vocab]
- Character-level CTC alignment scores

## Questions for Improvement

### A. Post-Processing Strategies

**Q1**: Given that CTC alignment is deterministic but imperfect, what post-processing would best improve boundary accuracy?

Options I'm considering:
1. **Energy-based boundary snapping**: Move boundaries to nearest local energy minimum (±50ms window)
2. **Duration normalization**: Force total phoneme duration to match word segment duration by proportional scaling
3. **HMM smoothing**: Use Viterbi with acoustic model to refine CTC boundaries
4. **Median filtering**: Smooth abrupt duration changes

**Q2**: For the trailing silence problem (last phoneme absorbing 743ms gap), which approach works best?
- Voice Activity Detection (VAD) to clip silence, then re-align
- Threshold-based duration clipping (e.g., max 1.5s per phoneme)
- Backward pass: trim last phoneme to word boundary

### B. Syllabification Strategy

**Current approach**: Simple heuristic based on vowel patterns:
```python
# Long vowel → take following consonant: "heem" not "he-em"
# Short vowel + consonant → one syllable: "Bis"
```

**Q3**: Should I use **phonotactic rules** from Arabic linguistics instead?
Arabic syllable structure: (C)(C)V(C)(C) where:
- Initial cluster: rare (e.g., "st" in loanwords)
- Long vowels (aa/ee/oo): typically nucleus + coda
- Shadda (gemination): split into onset + coda?

**Q4**: Would **data-driven syllabification** (train on aligned data) outperform rule-based?

### C. CTC Decoding Parameters

**Q5**: Can I improve CTC quality by tuning:
- **Beam width**: Currently using greedy (width=1). Would beam=5-10 help?
- **Blank penalty**: CTC has blank tokens - can I penalize excessive blanks that create gaps?
- **Language model**: Add Arabic phonotactic constraints as LM rescoring?

### D. Alternative Approaches

**Q6**: Should I abandon CTC and use:
- **Frame-level classification** + Viterbi: Classify each 10ms frame, then decode with HMM
- **Hybrid approach**: CTC for initialization, then refine with acoustic model
- **Supervised fine-tuning**: Fine-tune MMS-FA on Quranic recitation data?

**Q7**: For the word-level timestamp anchoring, should I:
- **Hard constraints**: Force first/last phoneme of each word to exactly match word boundaries
- **Soft constraints**: Add penalty in objective function
- **Windowed alignment**: Run CTC separately per word (0-480ms, 600-1000ms, etc.) instead of full audio

## Constraints & Goals

### Must Have:
✅ Deterministic (same audio → same output)
✅ No training data required (zero-shot or few-shot only)
✅ Works for all 6,236 Quran ayahs
✅ Real-time capable (<2s processing for 5s audio)

### Accuracy Goals:
🎯 Coverage: >90% (currently 69%)
🎯 Boundary accuracy: ±25ms (currently ±100-300ms)
🎯 Self-comparison score: >95% (currently 88.7%)

### Use Case:
This is for **educational feedback**. Students record themselves reciting Quran, and the system provides scores on:
- Rhythm (DTW-based timing alignment)
- Melody (pitch contour matching)
- Duration (Tajweed elongation rules)
- Pronunciation (phoneme quality - future work)

Phoneme boundaries need to be accurate enough to:
1. Detect 200ms duration differences (2-count vs 4-count Madd)
2. Map Tajweed rules to correct phonemes (character → phoneme mapping)
3. Provide stable scores across recordings

## Current Results

**Husary vs Husary (self-comparison)**:
- Rhythm: 100/100 ✅
- Melody: 100/100 ✅
- Duration: 88.7/100 ⚠️ (should be ~98%+)

The 11.3% duration error comes from:
- 2-count Madd: 0.14 count error → 92.3% score
- 6-count Madd: 0.07 count error → 99.3% score

These small count errors (±50-100ms) are likely from CTC boundary inaccuracy.

## What I'm Looking For

I need concrete recommendations on:

1. **Immediate wins**: What single change would most improve coverage (69% → 90%)?

2. **Boundary refinement**: Best algorithm to move CTC boundaries to acoustic phoneme boundaries?

3. **Trailing silence**: How to prevent last phoneme from absorbing silence?

4. **Word boundary anchoring**: Should I enforce hard constraints on first/last phoneme per word?

5. **Validation**: How to measure alignment quality without ground-truth phoneme labels? (I only have word boundaries)

Please provide:
- Algorithmic details (not just "use energy-based" but HOW)
- Code pseudocode if relevant
- Tradeoffs (accuracy vs speed vs complexity)
- References to papers/implementations if applicable

## Example Output Format

```
RECOMMENDATION 1: Energy-Based Boundary Snapping
Priority: HIGH
Expected Improvement: 69% → 85% coverage

Algorithm:
1. For each CTC boundary at time t:
   - Extract RMS energy in window [t-50ms, t+50ms]
   - Find local minimum: t_min = argmin(energy)
   - Move boundary: t := t_min
2. After snapping, re-check monotonicity (t_i < t_{i+1})

Pseudocode:
```python
def snap_to_energy_minimum(phonemes, rms, times, window_ms=50):
    for i in range(1, len(phonemes)):
        t = phonemes[i]['start']
        idx = np.argmin(np.abs(times - t))

        # Window in frames
        window_frames = int(window_ms / 10)  # 10ms per frame
        start_idx = max(0, idx - window_frames)
        end_idx = min(len(rms), idx + window_frames)

        # Find local minimum
        local_min_idx = start_idx + np.argmin(rms[start_idx:end_idx])

        # Update boundary
        phonemes[i]['start'] = times[local_min_idx]
        phonemes[i-1]['end'] = times[local_min_idx]
```

Tradeoffs:
✅ Simple, fast, deterministic
✅ Leverages acoustic cues
❌ May snap to wrong minimum in noisy audio
❌ Doesn't address coverage issue

References:
- "Phoneme boundary detection using RMS energy" (Smith et al., 2015)
```

Thank you!
