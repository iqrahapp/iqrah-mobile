# Quick Start: Comparison API

Get started with the Phase 2 comparison engine in 5 minutes.

## 1. Basic Usage

```python
from src.iqrah_audio.comparison import compare_recitations

# Analyze two ayahs
result = compare_recitations(
    student_audio_path="student_1_1.mp3",
    reference_audio_path="husary_1_1.mp3",
    student_phonemes=student_phonemes,      # From Phase 1
    reference_phonemes=reference_phonemes,  # From Phase 1
    student_pitch=student_pitch,            # From Phase 1
    reference_pitch=reference_pitch,        # From Phase 1
    student_stats=student_stats,            # From Phase 1
    reference_stats=reference_stats         # From Phase 1
)

# Get scores
print(f"Overall: {result['overall']}/100")
print(f"Rhythm: {result['rhythm']['score']}/100")
print(f"Melody: {result['melody']['score']}/100")
print(f"Duration: {result['durations']['overall']}/100")
```

## 2. Complete Example with Phase 1

```python
from src.iqrah_audio.analysis import (
    extract_pitch_swiftf0,
    extract_phonemes_wav2vec2_ctc,
    compute_full_statistics
)
from src.iqrah_audio.analysis.segments_loader import (
    get_ayah_segments,
    download_audio,
    get_word_segments_with_text
)
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.comparison import compare_recitations

def analyze_ayah(surah, ayah):
    """Complete analysis pipeline for one ayah."""
    # Get audio
    seg_data = get_ayah_segments(surah, ayah)
    audio_path = download_audio(seg_data['audio_url'])

    # Extract pitch
    pitch_data = extract_pitch_swiftf0(audio_path)

    # Get word segments and transliteration
    word_segments = get_word_segments_with_text(surah, ayah)
    trans_data = load_transliteration_data()
    transliteration = trans_data.get(f"{surah}:{ayah}", "")

    # Extract phonemes
    phonemes = extract_phonemes_wav2vec2_ctc(
        audio_path=audio_path,
        word_segments=word_segments,
        transliteration=transliteration,
        pitch_data=pitch_data,
        surah=surah,
        ayah=ayah
    )

    # Compute statistics
    statistics = compute_full_statistics(phonemes, pitch_data)

    return {
        'audio_path': audio_path,
        'phonemes': phonemes,
        'pitch': pitch_data,
        'statistics': statistics
    }

# Analyze student and reference
student = analyze_ayah(1, 1)      # Student's 1:1
reference = analyze_ayah(1, 1)    # Reference 1:1 (Husary)

# Compare
comparison = compare_recitations(
    student_audio_path=student['audio_path'],
    reference_audio_path=reference['audio_path'],
    student_phonemes=student['phonemes'],
    reference_phonemes=reference['phonemes'],
    student_pitch=student['pitch'],
    reference_pitch=reference['pitch'],
    student_stats=student['statistics'],
    reference_stats=reference['statistics']
)

# Display results
print(f"\n{'='*60}")
print(f"Comparison Results")
print(f"{'='*60}")
print(f"Overall Score: {comparison['overall']:.1f}/100")
print(f"\nComponent Scores:")
print(f"  Rhythm:   {comparison['rhythm']['score']:.1f}/100")
print(f"  Melody:   {comparison['melody']['score']:.1f}/100")
print(f"  Duration: {comparison['durations']['overall']:.1f}/100")
print(f"\nFeedback:")
for note in comparison['feedback']['all_notes'][:5]:  # Top 5 notes
    print(f"  • {note}")
print(f"{'='*60}\n")
```

## 3. HTTP API Usage

```bash
# Compare two ayahs via REST API
curl -X POST "http://localhost:8000/api/compare" \
  -H "Content-Type: application/json" \
  -d '{
    "student_surah": 1,
    "student_ayah": 1,
    "reference_surah": 1,
    "reference_ayah": 1,
    "pitch_extractor": "swiftf0"
  }'
```

Response:
```json
{
  "success": true,
  "comparison": {
    "overall": 100.0,
    "confidence": 1.0,
    "rhythm": {
      "score": 100.0,
      "divergence": 0.012,
      "notes": ["Excellent rhythm - timing matches reference very well"]
    },
    "melody": {
      "score": 100.0,
      "pitch_shift_cents": 0,
      "contour_similarity": 100.0,
      "notes": ["Reciting in similar key (shift: +0 cents)"]
    },
    "durations": {
      "overall": 100.0,
      "by_type": {},
      "critical_issues": [],
      "notes": ["No elongations (Madd) found in this ayah"]
    },
    "feedback": {
      "all_notes": [...],
      "suggestions": [],
      "top_issues": []
    },
    "metadata": {
      "tempo_ratio": 1.0,
      "student_pace": 2.31,
      "reference_pace": 2.31
    }
  }
}
```

## 4. Understanding Scores

### Overall Score (0-100)
Weighted combination:
- **40%** Rhythm (timing patterns)
- **25%** Melody (pitch contour)
- **35%** Duration (Madd elongations)

**Interpretation**:
- **90-100**: Excellent recitation ✅
- **75-89**: Good, minor improvements needed
- **60-74**: Fair, needs practice
- **0-59**: Significant differences from reference

### Rhythm Score (0-100)
Measures timing similarity using Soft-DTW divergence.

**Tempo-invariant**: Compares rhythm patterns regardless of speed.

**Interpretation**:
- **90-100**: Excellent rhythm
- **75-89**: Good rhythm, minor variations
- **60-74**: Rhythm needs work
- **0-59**: Significantly different rhythm

### Melody Score (0-100)
Measures melodic contour similarity using ΔF0.

**Key-invariant**: Compares melody regardless of pitch level.

**Pitch Shift**: Reports key difference (e.g., +200 cents = 2 semitones higher)

**Interpretation**:
- **90-100**: Excellent melodic contour
- **75-89**: Good melody, minor deviations
- **60-74**: Melody needs work
- **0-59**: Significantly different melody

### Duration Score (0-100)
Measures elongation (Madd) accuracy.

**Tempo-adaptive**: Adjusts tolerance based on recitation pace.

**Special**: Returns 100 (N/A) if no Madd found in ayah.

**Interpretation**:
- **90-100**: Excellent elongations
- **75-89**: Good, minor duration issues
- **60-74**: Some elongations need work
- **0-59**: Significant duration problems

## 5. Common Use Cases

### Self-Assessment
Compare your recitation against a master Qari:
```python
student = analyze_ayah(1, 1)      # Your recording
reference = analyze_ayah(1, 1)    # Husary reference

comparison = compare_recitations(...)

if comparison['overall'] >= 90:
    print("Excellent! ✅")
elif comparison['overall'] >= 75:
    print("Good work! Focus on these areas:")
    for issue in comparison['feedback']['top_issues']:
        print(f"  - {issue}")
else:
    print("Keep practicing! Work on:")
    for suggestion in comparison['feedback']['suggestions']:
        print(f"  - {suggestion}")
```

### Progress Tracking
Track improvement over time:
```python
import pandas as pd

# Compare weekly recordings
scores = []
for week in range(1, 13):
    student = load_recording(f"week_{week}.mp3")
    comparison = compare_recitations(...)
    scores.append({
        'week': week,
        'overall': comparison['overall'],
        'rhythm': comparison['rhythm']['score'],
        'melody': comparison['melody']['score'],
        'duration': comparison['durations']['overall']
    })

df = pd.DataFrame(scores)
print(df)

# Plot progress
df.plot(x='week', y=['overall', 'rhythm', 'melody', 'duration'])
```

### Component-Specific Practice
Focus on weak areas:
```python
comparison = compare_recitations(...)

# Check rhythm
if comparison['rhythm']['score'] < 70:
    print("🎵 Practice rhythm:")
    print(f"   Your tempo: {comparison['metadata']['student_pace']:.2f} syl/s")
    print(f"   Target: {comparison['metadata']['reference_pace']:.2f} syl/s")
    print(f"   Ratio: {comparison['metadata']['tempo_ratio']:.2f}")

# Check melody
if comparison['melody']['score'] < 70:
    shift = comparison['melody']['pitch_shift_cents']
    print(f"🎼 Practice melody:")
    print(f"   Key difference: {shift/100:.1f} semitones")
    print(f"   Contour similarity: {comparison['melody']['contour_similarity']:.1f}/100")

# Check duration
if comparison['durations']['overall'] < 70:
    print("⏱️  Practice elongations:")
    for issue in comparison['durations']['critical_issues']:
        print(f"   - {issue}")
```

## 6. Tips & Best Practices

### Audio Quality
- Use clean recordings (minimal background noise)
- Ensure consistent volume levels
- Avoid clipping or distortion
- Mono audio preferred (stereo will be downmixed)

### Comparison Selection
- Compare same ayah (student vs reference)
- Use same Qari for consistency (e.g., Husary)
- Ensure complete ayah recordings (no cuts)

### Score Interpretation
- **Self-comparison should score ~100**: Verify with test
- **Different ayahs should score <60**: Expected behavior
- **Focus on trends**: Improvement over time > absolute score
- **Components matter**: Overall score hides details

### Performance
- Cache Phase 1 analysis results
- Reuse reference analysis across students
- Batch comparisons for efficiency
- Use SwiftF0 for speed (CREPE for accuracy)

## 7. Troubleshooting

### Low Rhythm Score
- Check tempo ratio (too fast/slow?)
- Verify syllable alignment (Phase 1)
- Listen for timing inconsistencies

### Low Melody Score
- Check pitch shift (wrong key?)
- Verify pitch extraction quality
- Listen for melodic deviations

### Low Duration Score
- Check Madd types in critical_issues
- Verify tempo ratio adjustment
- Listen to elongation lengths

### Overall Score Too Low
- Check confidence value (< 0.8 = unreliable)
- Verify all components scored
- Review feedback notes

## 8. Next Steps

- Read full API docs: [comparison-api.md](comparison-api.md)
- Review session summary: [session-summary-phase2.md](session-summary-phase2.md)
- Check SOTA report: [../doc/sota-audio-recitation-comparison.md](../doc/sota-audio-recitation-comparison.md)
- Run tests: `/tmp/test_comparison_comprehensive.py`

## Support

For issues or questions:
- GitHub: [iqrah-audio issues](https://github.com/iqrah/iqrah-audio/issues)
- Documentation: `/docs/`
- API Reference: [comparison-api.md](comparison-api.md)
