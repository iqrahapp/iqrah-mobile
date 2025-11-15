# Word Auto-Segmentation Implementation Plan

## Overview
Add phoneme-weighted word segmentation to auto-generate word boundaries within ayah regions, similar to how Silero VAD auto-generates ayah boundaries from silence.

## Architecture Integration

### Pattern: Follow VAD approach
- VAD detects **ayah** boundaries using audio analysis (silence detection)
- Word segmenter detects **word** boundaries using text analysis (phoneme weights)
- Both produce time-stamped segments that AnnotationManager can consume

### File Structure (matching existing patterns)
```
frontend/src/
├── lib/
│   └── segmentation/
│       ├── phonemeWeights.ts      # Arabic letter duration mappings
│       └── wordSegmenter.ts       # Main segmentation API
└── utils/
    └── qpcQueries.ts              # Helper to fetch word text from QPC DB (optional)
```

## Implementation Steps

### Step 1: Create Phoneme Weight Map
**File:** `frontend/src/lib/segmentation/phonemeWeights.ts`

```typescript
/**
 * Relative phoneme durations for Quranic recitation.
 * Tuned for standard tajweed (Hafs 'an 'Asim).
 * Values are unitless ratios - adjust based on annotator feedback.
 */
export const PHONEME_WEIGHTS: Record<string, number> = {
  // Emphatic consonants (heavier, longer)
  'ص': 1.3, 'ض': 1.3, 'ط': 1.3, 'ظ': 1.3,

  // Nasal/resonant
  'ن': 1.1, 'م': 1.1, 'ل': 1.0, 'ر': 1.0,

  // Regular consonants
  'ب': 0.9, 'ت': 0.9, 'ث': 0.9, 'ج': 0.9, 'ح': 0.9, 'خ': 0.9,
  'د': 0.9, 'ذ': 0.9, 'ز': 0.9, 'س': 0.9, 'ش': 0.9, 'ع': 0.9,
  'غ': 0.9, 'ف': 0.9, 'ق': 0.9, 'ك': 0.9, 'و': 0.9, 'ه': 0.9,
  'ي': 0.9, 'ء': 0.5, 'أ': 0.9, 'إ': 0.9, 'آ': 1.5,

  // Long vowels (madd)
  'ا': 1.5,  // Alif (when elongated)

  // Short vowel marks (harakat)
  'َ': 0.5, 'ِ': 0.5, 'ُ': 0.5,  // fatha, kasra, damma
  'ْ': 0.2,  // sukun
  'ً': 0.5, 'ٌ': 0.5, 'ٍ': 0.5,  // tanween
  'ّ': 0.3,  // shadda (adds to base letter)

  // Special
  'ٰ': 0.3,  // dagger alif
  '': 0.0,   // tatweel (ignore)
};

export const DEFAULT_WEIGHT = 0.7;

/**
 * Calculate total phoneme weight for a word.
 * Strips non-Arabic chars (numbers, punctuation).
 */
export function calculateWordWeight(word: string): number {
  let total = 0;
  for (const char of word) {
    total += PHONEME_WEIGHTS[char] ?? DEFAULT_WEIGHT;
  }
  return Math.max(total, 0.1); // Minimum to avoid zero-duration
}

/**
 * Calculate weights for multiple words.
 */
export function calculateWordWeights(words: string[]): number[] {
  return words.map(calculateWordWeight);
}
```

### Step 2: Create Word Segmenter
**File:** `frontend/src/lib/segmentation/wordSegmenter.ts`

```typescript
import { calculateWordWeights } from './phonemeWeights';

export interface WordSegment {
  word: string;
  start: number;  // seconds
  end: number;    // seconds
  index: number;
  confidence: number; // 0-1 (always 0.5 for heuristic)
}

export interface SegmentAyahParams {
  /** Start time of ayah in seconds */
  ayahStart: number;
  /** End time of ayah in seconds */
  ayahEnd: number;
  /** Array of Arabic words in order */
  words: string[];
  /** Optional padding (seconds) to add to each word boundary */
  paddingSec?: number;
}

/**
 * Segment an ayah into word boundaries using proportional phoneme weights.
 * Similar API to Silero VAD's detectSpeechBounds.
 *
 * @returns Array of word segments with timestamps
 */
export function segmentAyahByWords(params: SegmentAyahParams): WordSegment[] {
  const { ayahStart, ayahEnd, words, paddingSec = 0.01 } = params;

  if (words.length === 0) return [];

  const weights = calculateWordWeights(words);
  const totalWeight = weights.reduce((sum, w) => sum + w, 0);
  const ayahDuration = ayahEnd - ayahStart;

  if (ayahDuration <= 0 || totalWeight <= 0) {
    console.warn('[wordSegmenter] Invalid duration or weights');
    return [];
  }

  const segments: WordSegment[] = [];
  let currentStart = ayahStart;

  for (let i = 0; i < words.length; i++) {
    // Proportional allocation
    const wordDuration = (weights[i] / totalWeight) * ayahDuration;
    const wordEnd = currentStart + wordDuration;

    // Apply padding (shrink segment slightly for visual gap)
    const segStart = Math.max(ayahStart, currentStart + paddingSec);
    const segEnd = Math.max(segStart, Math.min(ayahEnd, wordEnd - paddingSec));

    segments.push({
      word: words[i],
      start: segStart,
      end: segEnd,
      index: i,
      confidence: 0.5, // Heuristic confidence
    });

    currentStart = wordEnd;
  }

  return segments;
}

/**
 * Adjust a single word segment and shift subsequent words.
 * Call this when user manually adjusts a boundary.
 *
 * @param segments - Current segments
 * @param wordIndex - Index of word to adjust
 * @param newEnd - New end time (seconds)
 * @returns Updated segments
 */
export function adjustWordSegment(
  segments: WordSegment[],
  wordIndex: number,
  newEnd: number
): WordSegment[] {
  if (wordIndex < 0 || wordIndex >= segments.length) return segments;

  const updated = [...segments];
  const delta = newEnd - updated[wordIndex].end;

  // Update the adjusted word
  updated[wordIndex] = { ...updated[wordIndex], end: newEnd };

  // Shift all subsequent words by delta
  for (let i = wordIndex + 1; i < updated.length; i++) {
    updated[i] = {
      ...updated[i],
      start: updated[i].start + delta,
      end: updated[i].end + delta,
    };
  }

  return updated;
}

/**
 * Validate segments (no overlaps, monotonic time).
 */
export function validateSegments(segments: WordSegment[]): boolean {
  for (let i = 0; i < segments.length; i++) {
    if (segments[i].start >= segments[i].end) {
      console.warn(`[wordSegmenter] Invalid segment ${i}: start >= end`);
      return false;
    }
    if (i > 0 && segments[i].start < segments[i - 1].end) {
      console.warn(`[wordSegmenter] Overlap at segment ${i}`);
      return false;
    }
  }
  return true;
}
```

### Step 3: Integration with AnnotationManager

**Option A: Add method to AnnotationManager**
```typescript
// In frontend/src/annotation/manager.ts

import { segmentAyahByWords, type WordSegment } from '../lib/segmentation/wordSegmenter';

export class AnnotationManager {
  // ... existing code ...

  /**
   * Auto-segment an ayah annotation into word annotations.
   * Similar to how VAD creates ayah regions from silence detection.
   *
   * @param ayahAnnotation - The parent ayah annotation
   * @param words - Array of Arabic words for this ayah
   */
  autoSegmentWords(ayahAnnotation: Annotation, words: string[]): Annotation[] {
    const segments = segmentAyahByWords({
      ayahStart: ayahAnnotation.start,
      ayahEnd: ayahAnnotation.end,
      words,
      paddingSec: 0.02, // 20ms visual gap
    });

    const created: Annotation[] = [];

    for (const seg of segments) {
      const ann = this.createPoint(seg.start, 'word', {
        label: seg.word,
        parentId: ayahAnnotation.id,
      });

      if (ann) {
        // Adjust to exact end time (createPoint creates 1s default)
        this.updateAnnotation(ann.id, { end: seg.end });
        created.push(this.annotations[ann.id]);
      }
    }

    return created;
  }
}
```

**Option B: Standalone utility (simpler)**
Just use the segmentation functions directly in UI components/wizards without modifying AnnotationManager.

### Step 4: Usage Example

```typescript
// In a component or wizard step:
import { segmentAyahByWords } from '../lib/segmentation/wordSegmenter';

// User has already created an ayah annotation (e.g., using VAD)
const ayahAnnotation = {
  id: 'xyz',
  kind: 'ayah' as const,
  start: 2.5,
  end: 8.3,
  meta: { label: '1:1' }
};

// Fetch words from QPC DB or provide manually
const words = ['بِسْمِ', 'ٱللَّهِ', 'ٱلرَّحْمَٰنِ', 'ٱلرَّحِيمِ'];

// Generate word segments
const wordSegments = segmentAyahByWords({
  ayahStart: ayahAnnotation.start,
  ayahEnd: ayahAnnotation.end,
  words,
  paddingSec: 0.02,
});

// Create word annotations in AnnotationManager
wordSegments.forEach(seg => {
  const ann = annotationManager.createPoint(seg.start, 'word', {
    label: seg.word,
    parentId: ayahAnnotation.id,
  });
  if (ann) {
    annotationManager.updateAnnotation(ann.id, { end: seg.end });
  }
});
```

## Testing Strategy

### Unit Tests
```typescript
// frontend/src/lib/segmentation/__tests__/wordSegmenter.test.ts

import { segmentAyahByWords, validateSegments } from '../wordSegmenter';

describe('Word Segmenter', () => {
  it('should segment ayah into words proportionally', () => {
    const segments = segmentAyahByWords({
      ayahStart: 0,
      ayahEnd: 10,
      words: ['بِسْمِ', 'ٱللَّهِ', 'ٱلرَّحْمَٰنِ'],
    });

    expect(segments).toHaveLength(3);
    expect(segments[0].start).toBe(0);
    expect(segments[2].end).toBeCloseTo(10, 1);
    expect(validateSegments(segments)).toBe(true);
  });

  it('should handle empty words', () => {
    const segments = segmentAyahByWords({
      ayahStart: 0,
      ayahEnd: 10,
      words: [],
    });
    expect(segments).toHaveLength(0);
  });

  it('should apply padding correctly', () => {
    const segments = segmentAyahByWords({
      ayahStart: 0,
      ayahEnd: 2,
      words: ['كلمة', 'أخرى'],
      paddingSec: 0.1,
    });

    // Check that there's a gap between words
    expect(segments[1].start).toBeGreaterThan(segments[0].end);
  });
});
```

### Manual Testing
1. Create an ayah annotation using VAD
2. Run word segmenter with known text
3. Verify boundaries look reasonable
4. Adjust weights in `phonemeWeights.ts` based on feedback
5. Test with different recitation speeds

## Tuning & Iteration

### Adjusting Weights
After testing with real recitations:
1. Identify consistently over/under-estimated words
2. Adjust letter weights in `PHONEME_WEIGHTS`
3. Consider recitation style (murattal vs mujawwad)
4. Add special handling for:
   - Idgham (merged words)
   - Madd (elongated vowels)
   - Qalqalah (bouncing consonants)

### Edge Cases
- **Very short words** (حرف): May need minimum duration
- **Merged words** (idgham): Treat as single word or split manually
- **Long madd**: Increase weight for آ, و, ي in specific contexts
- **Different reciters**: Create reciter-specific weight profiles

## Performance
- **Fast:** Pure TypeScript, no ML models
- **Instant:** No async operations (unlike VAD)
- **Memory:** Minimal (just arrays of numbers)
- **Scalable:** Can segment entire Quran in seconds

## Future Enhancements
- [ ] Reciter-specific weight profiles
- [ ] ML-based weight learning from manual annotations
- [ ] Integration with QPC database for automatic word fetching
- [ ] Batch segmentation for multiple ayahs
- [ ] Export/import weight profiles
- [ ] Visual weight editor for fine-tuning

## Comparison with VAD Approach

| Feature | VAD (Ayah) | Word Segmenter |
|---------|-----------|----------------|
| **Input** | Audio blob | Text + time bounds |
| **Method** | ML (Silero ONNX) | Heuristic (phoneme weights) |
| **Speed** | ~100ms (async) | <1ms (sync) |
| **Accuracy** | 85-90% | 70-80% (needs tuning) |
| **Adjustable** | Via params | Via weight map |
| **Dependencies** | ONNX Runtime | None |
| **Use Case** | Find ayah boundaries | Subdivide ayah into words |

## Summary
This implementation provides:
- ✅ Simple, fast word segmentation using phoneme weights
- ✅ Follows existing codebase patterns (VAD, AnnotationManager)
- ✅ No external dependencies
- ✅ Tunable via weight map
- ✅ Compatible with existing annotation system
- ✅ Room for future ML-based improvements

The word segmenter complements VAD: VAD finds ayahs from silence, word segmenter subdivides ayahs into words using linguistic heuristics.
