# Comparison API Documentation

The Comparison API provides a comprehensive system for comparing Quranic recitations, analyzing rhythm, melody, and pronunciation elongation (Madd).

## Quick Start

```python
from src.iqrah_audio.comparison import compare_recitations

# Compare two recitations
result = compare_recitations(
    student_audio_path="/path/to/student.mp3",
    reference_audio_path="/path/to/reference.mp3",
    student_phonemes=student_phonemes,
    reference_phonemes=reference_phonemes,
    student_pitch=student_pitch_data,
    reference_pitch=reference_pitch_data,
    student_stats=student_statistics,
    reference_stats=reference_statistics
)

print(f"Overall Score: {result['overall']}/100")
print(f"Rhythm: {result['rhythm']['score']}/100")
print(f"Melody: {result['melody']['score']}/100")
```

## HTTP API Endpoints

### 1. Basic Comparison

**Endpoint**: `POST /api/compare`

**Parameters**:
- `student_surah` (int): Surah number for student recitation (1-114)
- `student_ayah` (int): Ayah number for student recitation
- `reference_surah` (int): Surah number for reference recitation
- `reference_ayah` (int): Ayah number for reference recitation
- `pitch_extractor` (str, optional): Pitch extraction method (`"swiftf0"` or `"crepe"`, default: `"swiftf0"`)

**Example Request**:
```bash
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

**Response Structure**:
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
      "by_type": {
        "madda_necessary": {"count": 2, "avg_score": 95.5}
      },
      "critical_issues": [],
      "notes": ["Elongation accuracy is excellent"]
    },

    "feedback": {
      "all_notes": [...],
      "suggestions": [...],
      "top_issues": [...]
    },

    "metadata": {
      "tempo_ratio": 1.0,
      "student_pace": 2.31,
      "reference_pace": 2.31
    }
  },

  "student_analysis": {...},
  "reference_analysis": {...}
}
```

### 2. Comparison with Visualizations

**Endpoint**: `POST /api/compare/visualize`

**Parameters**: Same as `/api/compare`

**Example Request**:
```bash
curl -X POST "http://localhost:8004/api/compare/visualize" \
  -H "Content-Type: application/json" \
  -d '{
    "student_surah": 1,
    "student_ayah": 1,
    "reference_surah": 1,
    "reference_ayah": 2,
    "pitch_extractor": "swiftf0"
  }'
```

**Response Structure**:
```json
{
  "success": true,
  "comparison": {...},
  "visualizations": {
    "dtw_path": "data:image/png;base64,iVBORw0KGgoA...",
    "pitch_comparison": "data:image/png;base64,iVBORw0KGgoA...",
    "rhythm_comparison": "data:image/png;base64,iVBORw0KGgoA...",
    "student_spectrogram": "data:image/png;base64,iVBORw0KGgoA...",
    "reference_spectrogram": "data:image/png;base64,iVBORw0KGgoA..."
  },
  "student_analysis": {...},
  "reference_analysis": {...}
}
```

**Visualization Descriptions**:

- **dtw_path**: DTW alignment path overlaid on cost matrix
  - Shows how student timing maps to reference timing
  - Color-coded distance matrix with red path
  - ~160 KB per image

- **pitch_comparison**: Two-panel pitch contour comparison
  - Top: Pitch contours in semitones (with key shift)
  - Bottom: ΔF0 melodic contour (key-invariant)
  - Green alignment markers between panels
  - ~160 KB per image

- **rhythm_comparison**: Three-panel rhythm analysis
  - Top: Student onset strength
  - Middle: Reference onset strength
  - Bottom: Aligned comparison showing timing differences
  - ~160 KB per image

- **student_spectrogram**: Student spectrogram with annotations
  - Frequency spectrogram (0-1000 Hz)
  - Phoneme boundaries (cyan lines)
  - Phoneme labels (black boxes)
  - Pitch overlay (lime green)
  - ~270 KB per image

- **reference_spectrogram**: Reference spectrogram with annotations
  - Same format as student spectrogram
  - ~295 KB per image

**Total Payload**: ~1.2 MB for all 5 visualizations (base64-encoded)

## Component Descriptions

### 1. Rhythm Analysis
**Score Range**: 0-100 (100 = perfect match)

Measures timing and rhythmic similarity using Soft-DTW divergence:
- **Tempo-invariant**: Compares rhythm independent of overall speed
- **Features**: Onset strength, syllable boundaries, normalized time
- **Algorithm**: Soft-DTW with Sakoe-Chiba band constraint

**Interpretation**:
- 90-100: Excellent rhythm, timing matches reference very well
- 75-89: Good rhythm, minor timing variations
- 60-74: Rhythm needs work, some inconsistencies
- 0-59: Rhythm significantly differs from reference

### 2. Melody Analysis
**Score Range**: 0-100 (100 = perfect match)

Measures melodic contour similarity:
- **Key-invariant**: Compares melody independent of pitch level
- **Features**: ΔF0 (first difference of semitones), pitch range
- **Algorithm**: Soft-DTW on ΔF0 + range ratio

**Pitch Shift**: Reports key difference in cents (100 cents = 1 semitone)

**Interpretation**:
- 90-100: Excellent melodic contour
- 75-89: Good melody, minor deviations
- 60-74: Melody needs work
- 0-59: Melody significantly differs

### 3. Duration Analysis (Madd)
**Score Range**: 0-100 (100 = perfect)

Measures elongation accuracy for Madd rules:
- **Tempo-adaptive**: Adjusts tolerance based on recitation pace
- **Madd Types**:
  - `madda_normal`: 2 counts
  - `madda_permissible`: 2-4 counts
  - `madda_necessary`: 6 counts
  - `madda_obligatory_mottasel`: 4-5 counts
  - `madda_obligatory_monfasel`: 4-5 counts

**Scoring**: Laplace distribution `100 × exp(-|error| / σ)`
where `σ = 0.15 × expected_counts × tempo_ratio`

**Special Cases**:
- Returns 100 (N/A) if no Madd found in ayah
- Critical issues flagged if score < 50

### 4. Overall Score
**Score Range**: 0-100

Weighted combination:
- Rhythm: 40%
- Melody: 25%
- Duration: 35%

**Confidence**: Based on score consistency (0-1)

## Technical Details

### Feature Extraction
All features are extracted at 50 Hz (20ms frames) and resampled to length L=150 for DTW efficiency:

```python
@dataclass
class FeaturePack:
    onset_strength: np.ndarray      # [150] z-scored
    syll_onset_mask: np.ndarray     # [150] {0,1}
    norm_time: np.ndarray           # [150] in [0,1]
    f0_semitones: np.ndarray        # [150] NaN on unvoiced
    df0: np.ndarray                 # [150] z-norm per phrase
    frame_times: np.ndarray         # [150]
    duration: float                 # seconds
    tempo_estimate: float           # syllables/second
    mean_count: float               # seconds
```

### Soft-DTW Divergence
Proper divergence measure (symmetric, unbiased):

```
Divergence = 2×SoftDTW(x,y) - SoftDTW(x,x) - SoftDTW(y,y)
```

**Parameters**:
- `gamma = 0.15`: Soft-min temperature
- `bandwidth = 12%`: Sakoe-Chiba band width
- `scale = 60.0`: Divergence-to-score scaling

### Performance
- **Latency**: ~2-5 seconds per comparison (includes full analysis)
- **Accuracy**:
  - Self-comparison: 100/100 ✅
  - Different ayahs: 30-50/100 ✅
- **Scalability**: Single reference per call (FAISS ANN for multi-reference in future)

## Example Use Cases

### 1. Self-Assessment Tool
Compare student's recitation against a master Qari:
```python
comparison = compare_recitations(
    student_audio_path="student_1_1.mp3",
    reference_audio_path="husary_1_1.mp3",
    ...
)

if comparison['overall'] >= 90:
    print("Excellent recitation! ✅")
elif comparison['overall'] >= 75:
    print("Good work! Areas to improve:")
    for issue in comparison['feedback']['top_issues']:
        print(f"  - {issue}")
else:
    print("Keep practicing! Focus on:")
    for suggestion in comparison['feedback']['suggestions']:
        print(f"  - {suggestion}")
```

### 2. Progress Tracking
Track improvement over time:
```python
weekly_scores = []
for week in range(1, 13):
    audio = f"student_week_{week}.mp3"
    result = compare_recitations(student_audio_path=audio, ...)
    weekly_scores.append(result['overall'])

# Plot progress
import matplotlib.pyplot as plt
plt.plot(weekly_scores)
plt.xlabel("Week")
plt.ylabel("Score")
plt.title("Recitation Progress")
plt.show()
```

### 3. Component-Specific Feedback
Focus on specific aspects:
```python
comparison = compare_recitations(...)

if comparison['rhythm']['score'] < 70:
    print("🎵 Focus on rhythm:")
    print(f"   Current tempo: {comparison['metadata']['student_pace']:.2f} syl/s")
    print(f"   Reference tempo: {comparison['metadata']['reference_pace']:.2f} syl/s")

if comparison['melody']['score'] < 70:
    shift = comparison['melody']['pitch_shift_cents']
    print(f"🎼 Focus on melody:")
    print(f"   Key difference: {shift/100:.1f} semitones")

if comparison['durations']['overall'] < 70:
    print("⏱️  Focus on elongations:")
    for issue in comparison['durations']['critical_issues']:
        print(f"   - {issue}")
```

## Testing

Comprehensive test suite available:
```bash
# Basic self-comparison test
python /tmp/test_comparison.py

# Different ayahs test
python /tmp/test_comparison_different.py

# Comprehensive test suite
python /tmp/test_comparison_comprehensive.py
```

**Expected Results**:
- Self-comparison: 100/100 ✅
- Different ayahs (1:1 vs 1:2): 40-45/100 ✅
- Consecutive ayahs (1:2 vs 1:3): 40-50/100 ✅
- Distant ayahs (1:1 vs 1:7): 40-50/100 ✅

## Future Enhancements

### Phase 2.5 (Planned)
- [ ] SSL-GOP pronunciation scoring
- [ ] RMVPE pitch extraction (more robust than CREPE)
- [ ] HPCP/chroma fallback for melody when F0 unreliable
- [ ] Visualization endpoints (DTW path overlay, waveform comparison)

### Phase 3 (Planned)
- [ ] FAISS ANN for multi-reference comparison
- [ ] Real-time streaming comparison
- [ ] Mobile SDK integration
- [ ] Personalized feedback based on user history

## References

1. **Soft-DTW Divergence**:
   Blondel et al., "Differentiable Divergences Between Time Series" (AISTATS 2021)
   https://proceedings.mlr.press/v130/blondel21a/blondel21a.pdf

2. **Tajweed Rules**:
   Integrated from Iqrah Knowledge Graph

3. **Pitch Extraction**:
   - SwiftF0: Fast ONNX-based pitch tracker
   - CREPE: Monophonic pitch tracking

4. **Phase 1 Documentation**:
   See `/docs/session-summary-phase1.md`

## Support

For issues or questions:
- GitHub: [iqrah-audio issues](https://github.com/iqrah/iqrah-audio/issues)
- Documentation: `/docs/`
- SOTA Report: `/doc/sota-audio-recitation-comparison.md`
