/**
 * Custom hook for managing merged (coupled) word boundaries
 *
 * When boundaries are merged, dragging one word's edge automatically moves
 * the adjacent word's edge, keeping them synchronized. This eliminates
 * the need for tedious double-adjustments and precision work.
 */

import { useCallback, useEffect } from 'react';
import { useWizardStore } from '../store/wizardStore';
import type { WordSegment } from '../store/wizardStore';

export interface AdjacentWords {
  previous: WordSegment | null;
  next: WordSegment | null;
}

/**
 * Hook to manage merged boundaries for word segmentation
 */
export function useMergedBoundaries(ayah: number) {
  const {
    words,
    mergedBoundaries,
    addMergedBoundary,
    removeMergedBoundary,
    toggleMergedBoundary,
    isBoundaryMerged,
    updateWord,
  } = useWizardStore();

  /**
   * Get word index from wordKey (e.g., "1:7:3" → 3)
   */
  const getWordIndex = useCallback((wordKey: string): number => {
    return parseInt(wordKey.split(':')[2]);
  }, []);

  /**
   * Get adjacent words for a given word in the same ayah
   */
  const getAdjacentWords = useCallback(
    (wordKey: string): AdjacentWords => {
      const wordIdx = getWordIndex(wordKey);
      const ayahWords = words
        .filter(w => w.ayah === ayah)
        .sort((a, b) => getWordIndex(a.wordKey) - getWordIndex(b.wordKey));

      const currentIndex = ayahWords.findIndex(w => w.wordKey === wordKey);

      return {
        previous: currentIndex > 0 ? ayahWords[currentIndex - 1] : null,
        next: currentIndex < ayahWords.length - 1 ? ayahWords[currentIndex + 1] : null,
      };
    },
    [words, ayah, getWordIndex]
  );

  /**
   * Check if the start boundary of a word is merged with its previous word
   */
  const isStartMerged = useCallback(
    (wordKey: string): boolean => {
      const { previous } = getAdjacentWords(wordKey);
      if (!previous) return false;

      const prevIdx = getWordIndex(previous.wordKey);
      const currIdx = getWordIndex(wordKey);

      return isBoundaryMerged(ayah, prevIdx, currIdx);
    },
    [ayah, getAdjacentWords, getWordIndex, isBoundaryMerged]
  );

  /**
   * Check if the end boundary of a word is merged with its next word
   */
  const isEndMerged = useCallback(
    (wordKey: string): boolean => {
      const { next } = getAdjacentWords(wordKey);
      if (!next) return false;

      const currIdx = getWordIndex(wordKey);
      const nextIdx = getWordIndex(next.wordKey);

      return isBoundaryMerged(ayah, currIdx, nextIdx);
    },
    [ayah, getAdjacentWords, getWordIndex, isBoundaryMerged]
  );

  /**
   * Auto-merge boundaries when new words are created
   * By default, all adjacent words have merged boundaries
   */
  const autoMergeOnCreate = useCallback(
    (wordKey: string) => {
      const wordIdx = getWordIndex(wordKey);
      const ayahWords = words.filter(w => w.ayah === ayah);

      // Find adjacent words by index
      const previousWord = ayahWords.find(w => getWordIndex(w.wordKey) === wordIdx - 1);
      const nextWord = ayahWords.find(w => getWordIndex(w.wordKey) === wordIdx + 1);

      // Merge with previous word if it exists
      if (previousWord) {
        const prevIdx = getWordIndex(previousWord.wordKey);
        addMergedBoundary(ayah, prevIdx, wordIdx);
      }

      // Merge with next word if it exists
      if (nextWord) {
        const nextIdx = getWordIndex(nextWord.wordKey);
        addMergedBoundary(ayah, wordIdx, nextIdx);
      }
    },
    [ayah, words, getWordIndex, addMergedBoundary]
  );

  /**
   * Handle boundary update with coupled movement
   * When a merged boundary is dragged, update both words
   *
   * @param wordKey - The word being dragged
   * @param newStart - New start time
   * @param newEnd - New end time
   * @param edge - Which edge was dragged ('start' | 'end')
   */
  const handleCoupledUpdate = useCallback(
    (wordKey: string, newStart: number, newEnd: number, edge: 'start' | 'end') => {
      const word = words.find(w => w.wordKey === wordKey);
      if (!word) return;

      // Update the current word
      updateWord(wordKey, { start: newStart, end: newEnd });

      if (edge === 'start' && isStartMerged(wordKey)) {
        // Start edge is merged with previous word's end
        const { previous } = getAdjacentWords(wordKey);
        if (previous) {
          updateWord(previous.wordKey, { end: newStart });
        }
      } else if (edge === 'end' && isEndMerged(wordKey)) {
        // End edge is merged with next word's start
        const { next } = getAdjacentWords(wordKey);
        if (next) {
          updateWord(next.wordKey, { start: newEnd });
        }
      }
    },
    [words, updateWord, isStartMerged, isEndMerged, getAdjacentWords]
  );

  /**
   * Toggle merge state for a word's start boundary
   */
  const toggleStartMerge = useCallback(
    (wordKey: string) => {
      const { previous } = getAdjacentWords(wordKey);
      if (!previous) return;

      const prevIdx = getWordIndex(previous.wordKey);
      const currIdx = getWordIndex(wordKey);

      toggleMergedBoundary(ayah, prevIdx, currIdx);
    },
    [ayah, getAdjacentWords, getWordIndex, toggleMergedBoundary]
  );

  /**
   * Toggle merge state for a word's end boundary
   */
  const toggleEndMerge = useCallback(
    (wordKey: string) => {
      const { next } = getAdjacentWords(wordKey);
      if (!next) return;

      const currIdx = getWordIndex(wordKey);
      const nextIdx = getWordIndex(next.wordKey);

      toggleMergedBoundary(ayah, currIdx, nextIdx);
    },
    [ayah, getAdjacentWords, getWordIndex, toggleMergedBoundary]
  );

  /**
   * Get all merged boundary keys for the current ayah
   */
  const getAyahMergedBoundaries = useCallback((): string[] => {
    return Array.from(mergedBoundaries).filter(key => key.startsWith(`${ayah}:`));
  }, [ayah, mergedBoundaries]);

  return {
    // Query functions
    getAdjacentWords,
    isStartMerged,
    isEndMerged,
    getAyahMergedBoundaries,

    // Actions
    autoMergeOnCreate,
    handleCoupledUpdate,
    toggleStartMerge,
    toggleEndMerge,
    addMergedBoundary,
    removeMergedBoundary,
  };
}

/**
 * Utility function to check if two timestamps are close enough to be considered "touching"
 * Used to determine if boundaries should be auto-merged
 */
export function areBoundariesTouching(
  time1: number,
  time2: number,
  threshold: number = 0.001 // 1ms tolerance
): boolean {
  return Math.abs(time1 - time2) < threshold;
}

/**
 * Utility to find which edge of a region is being dragged
 * Based on distance from mouse position to region edges
 */
export function getEdgeBeingDragged(
  regionStart: number,
  regionEnd: number,
  mouseTime: number,
  threshold: number = 0.05 // 50ms threshold
): 'start' | 'end' | null {
  const distToStart = Math.abs(mouseTime - regionStart);
  const distToEnd = Math.abs(mouseTime - regionEnd);

  if (distToStart < threshold && distToStart < distToEnd) {
    return 'start';
  } else if (distToEnd < threshold) {
    return 'end';
  }

  return null;
}
