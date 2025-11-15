/**
 * Arabic text analysis utilities for word segmentation
 *
 * Provides functions to:
 * - Count Arabic letters (excluding diacritics)
 * - Count madd (elongation) characters
 * - Estimate word duration weights
 * - Distribute verse duration proportionally
 */

import { stripHtml } from './utils';

/**
 * Arabic diacritics (tashkeel) to exclude from letter counting
 * These are pronunciation marks that don't affect duration significantly
 */
const ARABIC_DIACRITICS = [
  '\u064B', // Fathatan
  '\u064C', // Dammatan
  '\u064D', // Kasratan
  '\u064E', // Fatha
  '\u064F', // Damma
  '\u0650', // Kasra
  '\u0651', // Shadda
  '\u0652', // Sukun
  '\u0653', // Maddah
  '\u0654', // Hamza above
  '\u0655', // Hamza below
  '\u0656', // Subscript alef
  '\u0657', // Inverted damma
  '\u0658', // Mark noon ghunna
  '\u0670', // Superscript alef
];

/**
 * Madd (elongation) rule classes that affect word duration
 * These indicate prolonged sounds (2-6 counts)
 */
const MADD_RULE_CLASSES = [
  'madda_normal',              // 2 counts
  'madda_permissible',         // 2, 4, or 6 counts
  'madda_obligatory_mottasel', // 4-5 counts
  'madda_obligatory_monfasel', // 4-5 counts
  'madda_necessary',           // 6 counts
];

/**
 * Duration multipliers for different madd types
 * Based on typical count durations in Quranic recitation
 */
const MADD_DURATION_WEIGHTS = {
  'madda_normal': 1.5,              // Moderate lengthening
  'madda_permissible': 2.0,         // Variable, use average
  'madda_obligatory_mottasel': 2.5, // Long
  'madda_obligatory_monfasel': 2.5, // Long
  'madda_necessary': 3.0,           // Very long
};

/**
 * Count Arabic letters in text, excluding diacritics and HTML tags
 *
 * @param htmlText - Tajweed HTML text (e.g., "<rule class='ghunnah'>ال</rule>")
 * @returns Number of base Arabic letters
 *
 * @example
 * countLetters("<rule class='ghunnah'>الرَّحْمَـٰنِ</rule>") // Returns 6 (ا ل ر ح م ن)
 */
export function countLetters(htmlText: string): number {
  // Strip HTML tags first
  const plainText = stripHtml(htmlText);

  // Remove all diacritics
  let textWithoutDiacritics = plainText;
  for (const diacritic of ARABIC_DIACRITICS) {
    textWithoutDiacritics = textWithoutDiacritics.replace(new RegExp(diacritic, 'g'), '');
  }

  // Remove spaces and non-Arabic characters
  const arabicOnly = textWithoutDiacritics.replace(/[^\u0600-\u06FF]/g, '');

  return arabicOnly.length;
}

/**
 * Count madd (elongation) occurrences in Tajweed HTML
 * Parses HTML to find <rule class="madda_*"> tags
 *
 * @param htmlText - Tajweed HTML text
 * @returns Object with total madd count and weighted duration factor
 *
 * @example
 * countMaddChars("<rule class='madda_normal'>آ</rule>لم")
 * // Returns { count: 1, weight: 1.5 }
 */
export function countMaddChars(htmlText: string): { count: number; weight: number } {
  if (!htmlText) return { count: 0, weight: 0 };

  const parser = new DOMParser();
  const doc = parser.parseFromString(htmlText, 'text/html');
  const ruleElements = doc.querySelectorAll('rule');

  let count = 0;
  let totalWeight = 0;

  ruleElements.forEach((el) => {
    const ruleClass = el.getAttribute('class');
    if (ruleClass && MADD_RULE_CLASSES.includes(ruleClass)) {
      count++;
      // Use specific weight for this madd type
      totalWeight += MADD_DURATION_WEIGHTS[ruleClass as keyof typeof MADD_DURATION_WEIGHTS] || 1.5;
    }
  });

  return { count, weight: totalWeight };
}

/**
 * Calculate duration weight for a word based on letter and madd counts
 *
 * Formula: baseLetters + maddWeight
 * - Each base letter contributes 1.0
 * - Each madd contributes 1.5-3.0 (depending on type)
 *
 * @param htmlText - Tajweed HTML text
 * @returns Duration weight (relative units)
 *
 * @example
 * estimateWordDurationWeight("<rule class='madda_normal'>آ</rule>لم")
 * // Letters: 3, Madd weight: 1.5 → Total: 4.5
 */
export function estimateWordDurationWeight(htmlText: string): number {
  const letterCount = countLetters(htmlText);
  const { weight: maddWeight } = countMaddChars(htmlText);

  return letterCount + maddWeight;
}

/**
 * Interface for word with estimated duration
 */
export interface WordWithDuration {
  location: string;   // "surah:ayah:word"
  text: string;       // Tajweed HTML
  weight: number;     // Duration weight
  start: number;      // Calculated start time (absolute)
  end: number;        // Calculated end time (absolute)
}

/**
 * Distribute verse duration proportionally among words based on letter/madd analysis
 *
 * @param words - Array of words with Tajweed HTML text
 * @param verseStart - Verse start time (absolute, in seconds)
 * @param verseEnd - Verse end time (absolute, in seconds)
 * @returns Words with calculated start/end times
 *
 * @example
 * const words = [
 *   { location: "1:1:1", text: "بِسْمِ", ... },
 *   { location: "1:1:2", text: "ٱللَّهِ", ... },
 * ];
 * const distributed = distributeProportionally(words, 0, 3.5);
 * // Returns words with start/end times: [0, 1.2], [1.2, 2.3], ...
 */
export function distributeProportionally<T extends { location: string; text: string }>(
  words: T[],
  verseStart: number,
  verseEnd: number
): Array<T & WordWithDuration> {
  if (words.length === 0) {
    return [];
  }

  const verseDuration = verseEnd - verseStart;

  // Calculate weight for each word
  const wordsWithWeights = words.map((word) => ({
    ...word,
    weight: estimateWordDurationWeight(word.text),
  }));

  // Calculate total weight
  const totalWeight = wordsWithWeights.reduce((sum, w) => sum + w.weight, 0);

  // Handle edge case: no weight (shouldn't happen, but fallback to equal distribution)
  if (totalWeight === 0) {
    const equalDuration = verseDuration / words.length;
    return wordsWithWeights.map((word, idx) => ({
      ...word,
      start: verseStart + idx * equalDuration,
      end: verseStart + (idx + 1) * equalDuration,
    }));
  }

  // Distribute proportionally
  let currentTime = verseStart;

  return wordsWithWeights.map((word, idx) => {
    const proportion = word.weight / totalWeight;
    const duration = verseDuration * proportion;

    const start = currentTime;
    const end = idx === wordsWithWeights.length - 1
      ? verseEnd  // Last word: ensure exact match to avoid rounding errors
      : currentTime + duration;

    currentTime = end;

    return {
      ...word,
      start,
      end,
    };
  });
}

/**
 * Find gaps in existing word segments within a verse
 * Used for smart auto-segmentation that respects existing annotations
 *
 * @param existingWords - Already segmented words (sorted by start time)
 * @param verseStart - Verse start time
 * @param verseEnd - Verse end time
 * @returns Array of gaps with start/end times
 *
 * @example
 * const gaps = findGaps(
 *   [{ start: 1.0, end: 2.0 }, { start: 3.0, end: 4.0 }],
 *   0,
 *   5.0
 * );
 * // Returns: [{ start: 0, end: 1.0 }, { start: 2.0, end: 3.0 }, { start: 4.0, end: 5.0 }]
 */
export function findGaps(
  existingWords: Array<{ start: number; end: number }>,
  verseStart: number,
  verseEnd: number
): Array<{ start: number; end: number }> {
  const gaps: Array<{ start: number; end: number }> = [];

  if (existingWords.length === 0) {
    return [{ start: verseStart, end: verseEnd }];
  }

  // Sort by start time
  const sorted = [...existingWords].sort((a, b) => a.start - b.start);

  // Check gap before first word
  if (sorted[0].start > verseStart) {
    gaps.push({ start: verseStart, end: sorted[0].start });
  }

  // Check gaps between words
  for (let i = 0; i < sorted.length - 1; i++) {
    const currentEnd = sorted[i].end;
    const nextStart = sorted[i + 1].start;

    if (nextStart > currentEnd) {
      gaps.push({ start: currentEnd, end: nextStart });
    }
  }

  // Check gap after last word
  const lastEnd = sorted[sorted.length - 1].end;
  if (lastEnd < verseEnd) {
    gaps.push({ start: lastEnd, end: verseEnd });
  }

  return gaps;
}

/**
 * Distribute remaining (unsegmented) words across gaps
 * Useful for partial auto-segmentation when some words already exist
 *
 * @param remainingWords - Words that need segments
 * @param gaps - Available time gaps in verse
 * @returns Words with calculated start/end times
 */
export function distributeAcrossGaps<T extends { location: string; text: string }>(
  remainingWords: T[],
  gaps: Array<{ start: number; end: number }>
): Array<T & WordWithDuration> {
  if (remainingWords.length === 0 || gaps.length === 0) {
    return [];
  }

  // Calculate total gap duration
  const totalGapDuration = gaps.reduce((sum, gap) => sum + (gap.end - gap.start), 0);

  // Calculate weights for remaining words
  const wordsWithWeights = remainingWords.map((word) => ({
    ...word,
    weight: estimateWordDurationWeight(word.text),
  }));

  const totalWeight = wordsWithWeights.reduce((sum, w) => sum + w.weight, 0);

  // Distribute proportionally across all gaps
  const result: Array<T & WordWithDuration> = [];
  let wordIdx = 0;
  let remainingDuration = totalGapDuration;
  let remainingWeight = totalWeight;

  for (const gap of gaps) {
    const gapDuration = gap.end - gap.start;
    let currentTime = gap.start;

    // How many words should fit in this gap (proportional to gap size)
    const wordsInThisGap = Math.ceil((gapDuration / totalGapDuration) * remainingWords.length);

    for (let i = 0; i < wordsInThisGap && wordIdx < wordsWithWeights.length; i++, wordIdx++) {
      const word = wordsWithWeights[wordIdx];
      const proportion = word.weight / remainingWeight;
      const duration = remainingDuration * proportion;

      const start = currentTime;
      const end = Math.min(currentTime + duration, gap.end);

      result.push({
        ...word,
        start,
        end,
      });

      currentTime = end;
      remainingDuration -= duration;
      remainingWeight -= word.weight;

      // Stop if we've filled this gap
      if (currentTime >= gap.end) break;
    }
  }

  return result;
}
