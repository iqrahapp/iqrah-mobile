# Comparison Visualization UI Guide

Access the interactive comparison visualization tool at: **http://localhost:8004/comparison**

## Quick Start

1. **Start the server**:
   ```bash
   cd /home/shared/ws/iqrah/research_and_dev/iqrah-audio
   conda activate iqrah
   python app_qari_final.py
   ```

2. **Open browser**: Navigate to http://localhost:8004/comparison

3. **Fill in the form**:
   - **Student Recitation**: Enter surah and ayah numbers (e.g., 1:1)
   - **Reference Recitation**: Enter surah and ayah numbers (e.g., 1:2)
   - **Pitch Extractor**: Choose SwiftF0 (fast) or CREPE (accurate)

4. **Click "Compare Recitations"**: Wait 30-60 seconds for analysis

5. **View Results**: Scroll through scores, visualizations, and feedback

## Features

### Score Cards

**Overall Score** (0-100)
- Weighted combination of all components
- Confidence percentage shows reliability

**Component Scores**:
- 🎵 **Rhythm** (40% weight): Timing and tempo patterns
- 🎼 **Melody** (25% weight): Pitch contour similarity
- ⏱️ **Duration** (35% weight): Madd elongation accuracy

### Visualizations

1. **DTW Alignment Path**
   - Shows how student timing maps to reference
   - Color-coded distance matrix
   - Red line shows optimal alignment path

2. **Pitch Comparison**
   - **Top panel**: Pitch contours in semitones
   - **Bottom panel**: ΔF0 melodic contour (key-invariant)
   - Shows pitch shift between recordings

3. **Rhythm Comparison**
   - **Top**: Student onset strength
   - **Middle**: Reference onset strength
   - **Bottom**: Aligned comparison
   - Shows tempo ratio

4. **Student Spectrogram**
   - Frequency spectrogram (0-1000 Hz)
   - Cyan lines mark phoneme boundaries
   - Black boxes show phoneme labels
   - Lime green line shows pitch overlay

5. **Reference Spectrogram**
   - Same format as student spectrogram
   - For side-by-side comparison

### Feedback Section

- Actionable suggestions for improvement
- Component-specific notes
- Critical issues highlighted

## Example Comparisons

### Self-Comparison (Expected: 100/100)
```
Student: Surah 1, Ayah 1
Reference: Surah 1, Ayah 1
```
Should score 100/100 across all components.

### Different Ayahs (Expected: 30-50/100)
```
Student: Surah 1, Ayah 1
Reference: Surah 1, Ayah 2
```
Should show clear differences in all visualizations.

### Same Surah, Distant Ayahs (Expected: 30-45/100)
```
Student: Surah 1, Ayah 1
Reference: Surah 1, Ayah 7
```
Should show very different patterns.

## Navigation

- **📊 Analysis Tab**: Main phoneme analysis tool (http://localhost:8004/)
- **📈 Comparison Tab**: This comparison tool (http://localhost:8004/comparison)

Switch between tabs using the navigation buttons at the top.

## Tips

### For Best Results

1. **Use SwiftF0** for fast iteration
2. **Switch to CREPE** for final accurate analysis
3. **Compare same surah** for meaningful comparison
4. **Start with self-comparison** to verify system works
5. **Try consecutive ayahs** to see gradual differences

### Understanding Scores

- **90-100**: Excellent match
- **75-89**: Good, minor differences
- **60-74**: Fair, needs practice
- **30-59**: Significant differences (expected for different ayahs)
- **0-29**: Very different (rare, even for different ayahs)

### Interpreting Visualizations

**DTW Path**:
- Straight diagonal = perfect tempo match
- Horizontal stretches = student slower than reference
- Vertical stretches = student faster than reference

**Pitch Comparison**:
- Parallel lines = same melody, different key
- Diverging lines = different melodic contour
- ΔF0 panel shows key-invariant melodic shape

**Rhythm Comparison**:
- Bottom panel alignment shows tempo correction
- Large gaps = timing problems
- Tempo ratio shows overall speed difference

**Spectrograms**:
- Vertical cyan lines = syllable boundaries
- Lime overlay = pitch trajectory
- Labels show Arabic phonemes
- Brightness = energy/loudness

## Troubleshooting

### "Failed to compare recitations"
- Check that surah/ayah numbers are valid
- Ensure server is running on port 8004
- Check console for error messages

### Visualizations not loading
- Wait for analysis to complete (30-60s)
- Check browser console for errors
- Verify /api/compare/visualize endpoint is working

### Scores seem wrong
- Verify you're comparing meaningful ayahs
- Self-comparison should always score ~100
- Different ayahs should score 30-50
- Check confidence score (low = unreliable)

### Page is slow
- SwiftF0 is faster than CREPE (use for testing)
- Analysis happens on server, not in browser
- First request may be slower (model loading)
- Subsequent requests use cached models

## Technical Details

### Processing Pipeline

1. **Download audio** from Tarteel API
2. **Extract pitch** using SwiftF0/CREPE
3. **Align phonemes** with Wav2Vec2 CTC
4. **Compute statistics** (tempo, pitch, duration)
5. **Extract features** for comparison
6. **Run comparison** (rhythm, melody, duration)
7. **Generate visualizations** (5 PNG images)
8. **Return results** as JSON + base64 images

### Performance

- **Analysis time**: 30-60 seconds per comparison
- **Visualization size**: ~1.2 MB total (5 images)
- **Format**: Base64-encoded PNG (web-ready)
- **Caching**: Audio files cached in /tmp

### API Endpoint

The UI calls: `POST /api/compare/visualize`

For programmatic access, see [comparison-api.md](comparison-api.md)

## Examples of Good vs Bad Comparisons

### ✅ Good Comparisons

**Same reciter, same ayah**: Should score 95-100
```
Student: 1:1 (Husary)
Reference: 1:1 (Husary)
```

**Same reciter, consecutive ayahs**: Meaningful comparison
```
Student: 1:1 (Husary)
Reference: 1:2 (Husary)
```

**Student vs master, same ayah**: Real use case
```
Student: 1:1 (your recording)
Reference: 1:1 (Husary)
```

### ❌ Misleading Comparisons

**Different reciters, different style**: Not meaningful
```
Student: 1:1 (Reciter A, slow style)
Reference: 1:1 (Reciter B, fast style)
```

**Different surahs**: Not comparable
```
Student: 1:1 (Al-Fatiha)
Reference: 112:1 (Al-Ikhlas)
```

## Support

For issues or questions:
- GitHub: [iqrah-audio issues](https://github.com/iqrah/iqrah-audio/issues)
- Documentation: [comparison-api.md](comparison-api.md)
- Session Summary: [session-summary-phase2.md](session-summary-phase2.md)
