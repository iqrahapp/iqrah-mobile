# Phoneme Segmentation Research Query

## Task Description

We need to segment Quranic recitation audio into **phoneme-level** alignments to display individual Arabic sounds on our pitch visualization graph.

## What We Have

1. **High-quality Qari recitation audio** (clear, professional)
2. **Word-level segments** with precise timestamps (from our annotated data)
3. **Exact Arabic text with full diacritics** (tashkeel/harakat)
4. **Pitch contours** extracted via CREPE/RMVPE

## What We Need

**Phoneme-level time alignments** that map each Arabic phoneme to exact timestamps in the audio:

Example for word "بِسۡمِ" (bismi):
```
[
  {sound: "b", start_ms: 0, end_ms: 80},
  {sound: "i", start_ms: 80, end_ms: 200},
  {sound: "s", start_ms: 200, end_ms: 350},
  {sound: "m", start_ms: 350, end_ms: 450},
  {sound: "i", start_ms: 450, end_ms: 480}
]
```

## Requirements

1. **Offline processing**: Speed doesn't matter, accuracy does
2. **Arabic-specific**: Must handle Arabic phonology correctly
3. **Diacritic-aware**: Use our tashkeel to guide alignment
4. **State-of-the-art**: Want best available models/methods
5. **Python-friendly**: Needs to integrate with our pipeline

## Search Queries to Run

Please search for:

### 1. Forced Alignment Models
```
"arabic forced alignment phoneme segmentation SOTA 2024"
"arabic speech phoneme alignment deep learning"
"arabic forced aligner montreal MFA wav2vec2"
```

### 2. Quranic-Specific Tools
```
"quran recitation phoneme segmentation"
"tajweed phoneme analysis alignment"
"arabic quranic speech forced alignment"
```

### 3. General SOTA Alignment
```
"wav2vec2 forced alignment phoneme"
"whisper phoneme timestamps alignment"
"self-supervised speech alignment 2024"
"CTC forced alignment phoneme segmentation"
```

### 4. Arabic Speech Recognition
```
"arabic ASR phoneme recognition wav2vec2"
"arabic speech recognition with timestamps"
"MMS (Massively Multilingual Speech) arabic phonemes"
```

## What to Look For

1. **Pre-trained models** that support Arabic
2. **Tools with phoneme-level output** (not just word-level)
3. **Alignment accuracy** metrics on Arabic data
4. **Ease of integration** (Python libraries, pip installable)
5. **Active maintenance** (recent updates, good documentation)

## Specific Models/Libraries to Investigate

- **Montreal Forced Aligner (MFA)** - Classic tool, check Arabic support
- **Wav2Vec2 + CTC** - Meta's self-supervised models
- **Whisper + WhisperX** - OpenAI Whisper with forced alignment extension
- **Hugging Face Wav2Vec2-Aligner** - Fine-tuned alignment models
- **MMS (Massively Multilingual Speech)** - Meta's 1000+ language model
- **Kaldi** - Traditional ASR with forced alignment capabilities

## Evaluation Criteria

Rank solutions by:
1. **Accuracy** on Arabic (especially with diacritics)
2. **Phoneme-level precision** (not just word boundaries)
3. **Ease of use** (Python API, clear docs)
4. **Maintenance status** (active development)
5. **Arabic dataset training** (prefer models trained on Arabic)

## Expected Output Format

For each solution found, provide:
```
**[Tool Name]**
- Stars/Popularity:
- Arabic Support: Yes/No/Partial
- Phoneme-level: Yes/No
- Accuracy: [metric if available]
- Installation: pip/github/manual
- Last Update:
- Pros:
- Cons:
- Integration Complexity:
```

## Our Use Case Context

This will enable our **killer feature**: showing the exact Arabic phoneme being pronounced on the pitch graph in real-time as the Qari's audio plays, with pitch jumps marking tajweed features.

This makes tajweed **visual and audible** simultaneously - perfect for learners with untrained ears!

---

## Additional Notes

- We're okay with multi-step pipelines (e.g., ASR → alignment → refinement)
- Can use GPU for offline processing
- Willing to fine-tune models if needed
- Open to hybrid approaches (rule-based + ML)
