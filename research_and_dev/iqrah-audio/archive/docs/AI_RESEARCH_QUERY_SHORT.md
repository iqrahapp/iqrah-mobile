# Quick Research Query: Audio Recitation Comparison SOTA

## What We're Building
A system to compare student Quranic recitations against expert recordings, with phoneme-level timing and Tajweed rule verification.

## What We Have (Phase 1 Complete)
- Wav2Vec2 CTC phoneme alignment
- CREPE pitch extraction
- Statistical features: tempo (ISI), pitch (GMM), count duration, elongation accuracy, rhythm (IOI)
- 18 Tajweed rules mapped to phonemes

## What We Need (Phase 2)

### Research Questions

**1. Tempo-Invariant Rhythm Comparison**
- Current plan: DTW on tempo-normalized onset times
- Q: Is DTW still SOTA (2025)? Alternatives? (Soft-DTW, neural alignment, multi-dimensional DTW)
- Q: Best distance metrics for speech rhythm?

**2. Key-Invariant Pitch Contour Matching**
- Current plan: GMM normalization + KL divergence
- Q: Should we use chroma features? Neural melodic similarity?
- Q: Query-by-humming or cover song identification techniques applicable?

**3. Elongation Scoring**
- Current: Gaussian scoring `100 × exp(-(error²/2σ²))`
- Q: Better probability models? Adaptive tolerance?

**4. Pronunciation Quality**
- Current: None (only timing)
- Q: Wav2Vec2 embeddings for pronunciation? GOP scores? Pretrained Arabic models?

**5. Overall Scoring**
- Current: Weighted average of 5 metrics
- Q: Learn weights from expert annotations? Multi-task learning?

**6. Real-time Feedback**
- Q: Best practices for educational feedback in music/speech apps?
- Q: Gamification that works?

## Use Case Example
- Expert: 7s, 10 phonemes, 245 Hz
- Student: 9s (slower), same phonemes, 220 Hz (different key)
- Goal: Score 95/100 if rhythm/melody match despite tempo/pitch differences

## Specific Requests
1. **SOTA methods** for tempo-invariant rhythm + key-invariant melody (2020-2025)
2. **Top papers** on singing voice assessment, pronunciation evaluation, sequence alignment
3. **Python libraries** for DTW variants, pitch analysis, pronunciation assessment
4. **Alternative paradigms**: End-to-end neural similarity? Contrastive learning? Transformers?

## Constraints
- Language: Classical Arabic
- Performance: <30s analysis, <5s comparison
- Users: Beginners learning Quranic recitation

## Desired Output Format
1. Top 3 methods with pros/cons
2. 5-10 key papers (2020-2025)
3. Specific Python packages
4. Pitfalls to avoid
5. Novel ideas we haven't considered

## Why This Matters
- 1.8B potential users
- Unique: Combines prosody + pronunciation + strict timing rules
- Not pure speech/singing - melodic recitation with religious significance
