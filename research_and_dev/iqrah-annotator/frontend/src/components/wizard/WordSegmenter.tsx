// Stage 3: Word segmentation (ayah-by-ayah)
import React, { useState, useEffect, useRef, useMemo } from 'react';
import {
  Stack,
  Alert,
  Chip,
  Box,
  Paper,
  ToggleButtonGroup,
  ToggleButton,
  LinearProgress,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  IconButton,
  Tooltip,
  Button,
  Typography,
  Checkbox,
  Skeleton,
} from '@mui/material';
import { Delete, CheckCircle, NavigateNext, NavigateBefore, RadioButtonUnchecked, Link, LinkOff, DeleteSweep } from '@mui/icons-material';
import axios from 'axios';
import WavesurferAnnotator from '../WavesurferAnnotator';
import TajweedText from '../TajweedText';
import { ConfirmDialog } from '../ConfirmDialog';
import { useWizardStore } from '../../store/wizardStore';
import { loadRecording } from '../../store/db';
import { calculateAverageDuration } from '../../lib/vad/silero';
import { OVERLAP_ALLOWED_RULES, MAX_OVERLAP_MS } from '../../constants/tajweed';
import type { Annotation } from '../../annotation/types';
import type { AnnotationManager } from '../../annotation/manager';
import { stripHtml } from '../../lib/utils'; // Shared utility for HTML stripping
import { useAudioSegment } from '../../hooks/useAudioSegment'; // REFACTOR: Use shared hook for audio trimming
import { useAnnotationRestoration } from '../../hooks/useAnnotationRestoration'; // FIX #4: Use hook for consistent restoration
import { distributeProportionally, findGaps, distributeAcrossGaps } from '../../lib/arabicAnalysis'; // Auto-segmentation utilities
import { useMergedBoundaries } from '../../hooks/useMergedBoundaries'; // Merged boundaries for coupled drag
import { isDefined, ensureNumber, safeAt } from '../../utils/defensive'; // Defensive programming utilities

const API = import.meta.env.VITE_API_URL || 'http://localhost:8000';

interface QpcWord {
  id: number;
  location: string;
  surah: number;
  ayah: number;
  word: number;
  text: string;
  rules: string[];
}

export const WordSegmenter: React.FC = () => {
  const {
    recordingId,
    surah,
    verses,
    words,
    activeVerseIdx,
    setActiveVerseIdx,
    addWord,
    deleteWord,
    updateWordByAnnotationId,
    setExpectedWordCount,
  } = useWizardStore();

  const [fullAudioBlob, setFullAudioBlob] = useState<Blob | null>(null);
  const [qpcWords, setQpcWords] = useState<QpcWord[]>([]);
  const [fetchingWords, setFetchingWords] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedWordIdx, setSelectedWordIdx] = useState<number | null>(null);
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [selectedSegments, setSelectedSegments] = useState<Set<string>>(new Set());
  const [showBulkDeleteConfirm, setShowBulkDeleteConfirm] = useState(false);

  const annotationManagerRef = useRef<AnnotationManager | null>(null);
  const isCoupledUpdateRef = useRef(false); // Prevent infinite loops during coupled boundary updates
  const currentVerse = safeAt(verses, activeVerseIdx); // Safe array access

  // Merged boundaries hook for coupled drag behavior
  const {
    autoMergeOnCreate,
    isStartMerged,
    isEndMerged,
    toggleStartMerge,
    toggleEndMerge,
    getAyahMergedBoundaries,
  } = useMergedBoundaries(currentVerse?.ayah ?? 0);

  // Load full audio blob once
  useEffect(() => {
    if (!recordingId) return;

    loadRecording(recordingId).then(result => {
      if (result) {
        setFullAudioBlob(result.blob);
      }
    });
  }, [recordingId]);

  // Clear selections when changing verses
  useEffect(() => {
    setSelectedSegments(new Set());
  }, [activeVerseIdx]);

  // REFACTOR: Use shared hook for audio trimming with automatic cleanup
  const { audioUrl: ayahAudioUrl, timeOffset, loading: audioLoading, error: audioError } = useAudioSegment({
    fullAudioBlob,
    startTime: currentVerse?.start ?? 0,
    endTime: currentVerse?.end ?? 0,
    enabled: !!currentVerse && !!fullAudioBlob,
  });

  // Merge audio error into main error state
  useEffect(() => {
    if (audioError) {
      setError(audioError);
    }
  }, [audioError]);

  // Fetch expected word counts for ALL ayahs once on mount (FIX #9: Eager loading)
  useEffect(() => {
    if (!surah || verses.length === 0) return;

    console.log('[WordSegmenter] Fetching expected word counts for all ayahs');

    axios
      .get(`${API}/api/qpc/words`, {
        params: {
          surah,
          limit: 1000, // Get all words for the surah
        },
      })
      .then(r => {
        const ARABIC_NUMERAL_REGEX = /[\u0660-\u0669\u06F0-\u06F9]/;

        // Count words per ayah
        verses.forEach(verse => {
          let ayahWords = r.data.filter((w: QpcWord) => w.ayah === verse.ayah);

          // Remove verse number (last word if contains Arabic numerals)
          if (ayahWords.length > 0 && ARABIC_NUMERAL_REGEX.test(ayahWords[ayahWords.length - 1].text)) {
            ayahWords = ayahWords.slice(0, -1);
          }

          setExpectedWordCount(verse.ayah, ayahWords.length);
          console.log('[WordSegmenter] Expected word count for ayah', verse.ayah, ':', ayahWords.length);
        });
      })
      .catch(err => {
        console.error('[WordSegmenter] Failed to fetch expected word counts:', err);
      });
  }, [surah, verses]);

  // FIX #4: Use useAnnotationRestoration hook for consistent restoration (prevents infinite loops)
  // Get words for current ayah that have been segmented (have visual annotations)
  const currentVerseWords = useMemo(() => {
    if (!currentVerse) return [];
    return words.filter(w => w.ayah === currentVerse.ayah && w.annotationId);
  }, [words, currentVerse?.ayah]);

  // Use the hook for restoration
  const isRestoringRef = useAnnotationRestoration({
    manager: annotationManagerRef.current,
    items: currentVerseWords.map(w => ({
      id: w.annotationId!,
      start: w.start,
      end: w.end,
      text: w.text,
    })),
    timeOffset,
    kind: 'word',
    getLabelFn: (item) => stripHtml(item.text),
    audioUrl: ayahAudioUrl,
    additionalDeps: [currentVerse?.ayah],
  });

  // Fetch words for current verse
  useEffect(() => {
    if (!currentVerse) return;

    setFetchingWords(true);
    setError(null);

    axios
      .get(`${API}/api/qpc/words`, {
        params: {
          surah,
          limit: 1000,
        },
      })
      .then(r => {
        let ayahWords = r.data.filter((w: QpcWord) => w.ayah === currentVerse.ayah);

        // Remove verse number (last word if contains Arabic numerals)
        const ARABIC_NUMERAL_REGEX = /[\u0660-\u0669\u06F0-\u06F9]/;
        if (ayahWords.length > 0 && ARABIC_NUMERAL_REGEX.test(ayahWords[ayahWords.length - 1].text)) {
          ayahWords = ayahWords.slice(0, -1);
        }

        setQpcWords(ayahWords);

        // Store expected word count for this ayah
        setExpectedWordCount(currentVerse.ayah, ayahWords.length);
        console.log('[WordSegmenter] Set expected word count for ayah', currentVerse.ayah, ':', ayahWords.length);

        // Auto-select first unsegmented word
        const firstMissing = ayahWords.findIndex((w: QpcWord) => {
          const key = w.location;
          return !words.some(ws => ws.wordKey === key);
        });
        if (firstMissing >= 0) {
          setSelectedWordIdx(firstMissing);
        } else {
          setSelectedWordIdx(null);
        }
      })
      .catch(err => {
        console.error('Failed to fetch words:', err);
        setError(`Failed to load words for ayah ${currentVerse.ayah}`);
      })
      .finally(() => setFetchingWords(false));
  }, [currentVerse, surah, words]);

  const handleCreateAnnotation = (ann: Annotation) => {
    console.log('[WordSegmenter] handleCreateAnnotation called:', ann);

    // Skip if we're restoring annotations (prevents infinite loop)
    if (isRestoringRef.current) {
      console.log('[WordSegmenter] Skipping - currently restoring');
      return;
    }

    // Defensive: Ensure current verse exists
    if (!isDefined(currentVerse)) {
      console.error('[WordSegmenter] No current verse selected');
      setError('No verse selected. Please select a verse first.');
      return;
    }

    // At this point, TypeScript knows currentVerse is defined
    const verseEnd = currentVerse.end;
    const verseAyah = currentVerse.ayah;

    // Get FRESH state to avoid stale closure
    const freshWords = useWizardStore.getState().words;

    // Compute target word: find first unsegmented word for this ayah
    const segmentedKeys = freshWords
      .filter(w => w.ayah === verseAyah)
      .map(w => w.wordKey);
    const targetWord = qpcWords.find(w => !segmentedKeys.includes(w.location));

    console.log('[WordSegmenter] Segmented words for this ayah:', segmentedKeys);
    console.log('[WordSegmenter] Target word (computed):', targetWord?.location, targetWord?.text);

    if (!targetWord) {
      setError('All words for this ayah have been segmented');
      console.warn('[WordSegmenter] No unsegmented words remaining');
      if (isDefined(annotationManagerRef.current)) {
        annotationManagerRef.current.removeAnnotation(ann.id);
      }
      return;
    }

    const word = targetWord;
    const wordKey = word.location;

    // Convert relative times (in ayah audio) to absolute times (in full audio)
    // Use ensureNumber to prevent NaN issues
    let absoluteStart = ensureNumber(ann.start, 0) + ensureNumber(timeOffset, 0);
    let absoluteEnd = ensureNumber(ann.end, 0) + ensureNumber(timeOffset, 0);

    // Clamp end to ayah bounds to prevent exceeding audio duration
    if (absoluteEnd > verseEnd) {
      absoluteEnd = verseEnd;
      console.log('[WordSegmenter] Clamped segment end to ayah boundary:', verseEnd);

      // Update visual annotation to match clamped bounds
      if (annotationManagerRef.current) {
        const relativeEnd = absoluteEnd - timeOffset;
        annotationManagerRef.current.updateAnnotation(ann.id, {
          start: ann.start,
          end: relativeEnd,
        });
      }
    }

    // Smart sizing: if segment too small, use average duration
    if (absoluteEnd - absoluteStart < 0.05) {
      const currentAyahWords = freshWords.filter(w => w.ayah === verseAyah);
      const avgDuration = ensureNumber(calculateAverageDuration(currentAyahWords), 0.2);
      absoluteEnd = absoluteStart + avgDuration;

      // Clamp again after smart sizing
      if (absoluteEnd > verseEnd) {
        absoluteEnd = verseEnd;
        console.log('[WordSegmenter] Clamped after smart sizing to ayah boundary:', verseEnd);
      }

      // Update visual to match final bounds
      if (isDefined(annotationManagerRef.current)) {
        const relativeEnd = absoluteEnd - timeOffset;
        annotationManagerRef.current.updateAnnotation(ann.id, {
          start: ann.start,
          end: relativeEnd,
        });
      }
    }

    // Validate overlap
    const overlapping = freshWords.find(w => {
      if (w.ayah !== verseAyah) return false;
      const overlap = Math.min(absoluteEnd, w.end) - Math.max(absoluteStart, w.start);
      return overlap > 0;
    });

    if (overlapping) {
      const overlap = Math.min(absoluteEnd, overlapping.end) - Math.max(absoluteStart, overlapping.start);

      // Check if overlap is allowed based on tajweed rules
      const overlappingQpcWord = qpcWords.find(w => w.location === overlapping.wordKey);
      const canOverlap = overlappingQpcWord && (
        word.rules.some(r => OVERLAP_ALLOWED_RULES.includes(r)) ||
        overlappingQpcWord.rules.some(r => OVERLAP_ALLOWED_RULES.includes(r))
      );

      if (!canOverlap) {
        setError('Word overlap not allowed (no merging tajweed rule)');
        if (annotationManagerRef.current) {
          annotationManagerRef.current.removeAnnotation(ann.id);
        }
        return;
      }

      if (overlap > MAX_OVERLAP_MS / 1000) {
        setError(`Overlap exceeds ${MAX_OVERLAP_MS}ms limit`);
        if (annotationManagerRef.current) {
          annotationManagerRef.current.removeAnnotation(ann.id);
        }
        return;
      }
    }

    const result = addWord(
      wordKey,
      verseAyah,
      absoluteStart,
      absoluteEnd,
      word.text,
      ann.id
    );

    if (!result.ok) {
      setError(result.errors.join('\n'));
      if (isDefined(annotationManagerRef.current)) {
        annotationManagerRef.current.removeAnnotation(ann.id);
      }
    } else {
      setError(null);

      // Auto-merge boundaries with adjacent words (by default, all adjacent boundaries are merged)
      autoMergeOnCreate(wordKey);

      // Update visual label (strip HTML for plain text display in regions)
      const plainText = stripHtml(word.text);
      if (isDefined(annotationManagerRef.current)) {
        annotationManagerRef.current.updateAnnotation(ann.id, {
          meta: { label: plainText },
        });

        // Set region content as plain text (regions don't support HTML rendering)
        const regions = (annotationManagerRef.current as any).regions;
        if (isDefined(regions)) {
          const region = regions.getRegions().find((r: any) => r.id === ann.id);
          if (isDefined(region)) {
            region.setOptions({ content: plainText });
          }
        }
      }

      // Auto-select next unsegmented word using fresh store state
      const updatedWords = useWizardStore.getState().words;
      const nextIdx = qpcWords.findIndex((w) => {
        const key = w.location;
        return !updatedWords.some(ws => ws.wordKey === key);
      });
      console.log('[WordSegmenter] Auto-selecting next word index:', nextIdx);
      setSelectedWordIdx(nextIdx >= 0 ? nextIdx : null);
    }
  };

  const handleUpdateAnnotation = (ann: Annotation) => {
    // Skip if this is a programmatic coupled update to prevent infinite loops
    if (isCoupledUpdateRef.current) {
      console.log('[WordSegmenter] Skipping - coupled update in progress');
      return;
    }

    console.log('[WordSegmenter] handleUpdateAnnotation called:', ann);

    // Get fresh state to avoid stale closure
    const freshWords = useWizardStore.getState().words;
    const word = freshWords.find(w => w.annotationId === ann.id);

    if (word) {
      // Convert relative to absolute
      const absoluteStart = ann.start + timeOffset;
      const absoluteEnd = ann.end + timeOffset;

      console.log('[WordSegmenter] Updating word in store:', {
        wordKey: word.wordKey,
        oldAbsolute: { start: word.start, end: word.end },
        newAbsolute: { start: absoluteStart, end: absoluteEnd }
      });

      // Detect which edge was dragged
      const startChanged = Math.abs(word.start - absoluteStart) > 0.001;
      const endChanged = Math.abs(word.end - absoluteEnd) > 0.001;

      // Update the word
      updateWordByAnnotationId(ann.id, { start: absoluteStart, end: absoluteEnd });

      // Set flag to prevent infinite loops from manager updates
      isCoupledUpdateRef.current = true;

      try {
        // Handle coupled boundary updates
        if (startChanged && isStartMerged(word.wordKey)) {
          // Start boundary is merged - update previous word's end
          const wordIdx = parseInt(word.wordKey.split(':')[2]);
          const previousWord = freshWords.find(w => {
            const prevIdx = parseInt(w.wordKey.split(':')[2]);
            return w.ayah === word.ayah && prevIdx === wordIdx - 1;
          });

          if (previousWord && previousWord.annotationId) {
            console.log('[WordSegmenter] Coupled update: moving previous word end to', absoluteStart);

            // Update store
            updateWordByAnnotationId(previousWord.annotationId, {
              start: previousWord.start,
              end: absoluteStart
            });

            // FIX: Update visual annotation manager (convert to relative coordinates)
            if (annotationManagerRef.current) {
              const relativeEnd = absoluteStart - timeOffset;
              console.log('[WordSegmenter] Updating previous word visual end to', relativeEnd);
              annotationManagerRef.current.updateAnnotation(previousWord.annotationId, {
                end: relativeEnd
              });
            }
          }
        }

        if (endChanged && isEndMerged(word.wordKey)) {
          // End boundary is merged - update next word's start
          const wordIdx = parseInt(word.wordKey.split(':')[2]);
          const nextWord = freshWords.find(w => {
            const nextIdx = parseInt(w.wordKey.split(':')[2]);
            return w.ayah === word.ayah && nextIdx === wordIdx + 1;
          });

          if (nextWord && nextWord.annotationId) {
            console.log('[WordSegmenter] Coupled update: moving next word start to', absoluteEnd);

            // Update store
            updateWordByAnnotationId(nextWord.annotationId, {
              start: absoluteEnd,
              end: nextWord.end
            });

            // FIX: Update visual annotation manager (convert to relative coordinates)
            if (annotationManagerRef.current) {
              const relativeStart = absoluteEnd - timeOffset;
              console.log('[WordSegmenter] Updating next word visual start to', relativeStart);
              annotationManagerRef.current.updateAnnotation(nextWord.annotationId, {
                start: relativeStart
              });
            }
          }
        }
      } finally {
        // Always clear the flag, even if an error occurred
        isCoupledUpdateRef.current = false;
      }
    }
  };

  const handleDelete = (wordKey: string) => {
    const word = words.find(w => w.wordKey === wordKey);
    if (isDefined(word?.annotationId) && isDefined(annotationManagerRef.current)) {
      annotationManagerRef.current.removeAnnotation(word.annotationId);
    }
    deleteWord(wordKey);
  };

  /**
   * Auto-segment all unsegmented words using letter/madd-based heuristics
   * Respects existing segments and fills gaps proportionally
   */
  const handleAutoSegment = () => {
    if (!isDefined(annotationManagerRef.current) || !isDefined(currentVerse)) {
      setError('Cannot auto-segment: manager not ready or no verse selected');
      return;
    }

    // Get fresh state
    const freshWords = useWizardStore.getState().words;
    const currentAyahWords = freshWords.filter(w => w.ayah === currentVerse.ayah);

    // Find unsegmented words
    const segmentedKeys = new Set(currentAyahWords.map(w => w.wordKey));
    const unsegmentedWords = qpcWords.filter(w => !segmentedKeys.has(w.location));

    if (unsegmentedWords.length === 0) {
      setError('All words already segmented');
      return;
    }

    console.log('[WordSegmenter] Auto-segmenting', unsegmentedWords.length, 'words');

    let distributedWords;

    if (currentAyahWords.length === 0) {
      // No existing segments: distribute across entire verse
      distributedWords = distributeProportionally(
        unsegmentedWords,
        currentVerse.start,
        currentVerse.end
      );
    } else {
      // Some segments exist: fill gaps proportionally
      const gaps = findGaps(currentAyahWords, currentVerse.start, currentVerse.end);
      console.log('[WordSegmenter] Found gaps:', gaps);

      distributedWords = distributeAcrossGaps(unsegmentedWords, gaps);
    }

    // Create visual annotations and store segments
    let createdCount = 0;
    distributedWords.forEach((word) => {
      // Convert absolute to relative times for visual annotation
      const relativeStart = word.start - timeOffset;
      const relativeEnd = word.end - timeOffset;

      // Create visual region
      const ann = annotationManagerRef.current?.createPoint(
        relativeStart,
        'word',
        { label: stripHtml(word.text) }
      );

      if (!ann) {
        console.error('[WordSegmenter] Failed to create annotation for word:', word.location);
        return;
      }

      // Update region with correct end time
      annotationManagerRef.current?.updateAnnotation(ann.id, {
        start: relativeStart,
        end: relativeEnd,
      });

      // Add to store
      const result = addWord(
        word.location,
        currentVerse.ayah,
        word.start,
        word.end,
        word.text,
        ann.id
      );

      if (result.ok) {
        createdCount++;

        // Update visual label
        const plainText = stripHtml(word.text);
        if (isDefined(annotationManagerRef.current)) {
          annotationManagerRef.current.updateAnnotation(ann.id, {
            meta: { label: plainText },
          });

          // Set region content
          const regions = (annotationManagerRef.current as any).regions;
          if (isDefined(regions)) {
            const region = regions.getRegions().find((r: any) => r.id === ann.id);
            if (isDefined(region)) {
              region.setOptions({ content: plainText });
            }
          }
        }
      } else {
        console.error('[WordSegmenter] Failed to add word to store:', word.location, result.errors);
        // Remove visual annotation if store failed
        if (isDefined(annotationManagerRef.current)) {
          annotationManagerRef.current.removeAnnotation(ann.id);
        }
      }
    });

    if (createdCount > 0) {
      setError(null);
      console.log('[WordSegmenter] Auto-segmented', createdCount, 'words successfully');

      // Auto-merge all adjacent boundaries after auto-segmentation
      distributedWords.forEach((word) => {
        autoMergeOnCreate(word.location);
      });
    } else {
      setError('Failed to auto-segment words');
    }
  };

  /**
   * Clear all word segments for current ayah
   */
  const handleClearAll = () => {
    if (!isDefined(annotationManagerRef.current) || !isDefined(currentVerse)) return;

    const freshWords = useWizardStore.getState().words;
    const currentAyahWords = freshWords.filter(w => w.ayah === currentVerse.ayah);

    currentAyahWords.forEach(word => {
      if (isDefined(word.annotationId) && isDefined(annotationManagerRef.current)) {
        annotationManagerRef.current.removeAnnotation(word.annotationId);
      }
      deleteWord(word.wordKey);
    });

    console.log('[WordSegmenter] Cleared all segments for ayah', currentVerse.ayah);
    setShowClearConfirm(false);
  };

  /**
   * Toggle segment selection for bulk operations
   */
  const toggleSegmentSelection = (wordKey: string) => {
    setSelectedSegments(prev => {
      const newSet = new Set(prev);
      if (newSet.has(wordKey)) {
        newSet.delete(wordKey);
      } else {
        newSet.add(wordKey);
      }
      return newSet;
    });
  };

  /**
   * Select/deselect all segments
   */
  const toggleSelectAll = () => {
    if (selectedSegments.size === currentVerseWords.length) {
      setSelectedSegments(new Set());
    } else {
      setSelectedSegments(new Set(currentVerseWords.map(w => w.wordKey)));
    }
  };

  /**
   * Bulk delete selected segments
   */
  const handleBulkDelete = () => {
    if (!isDefined(annotationManagerRef.current)) return;

    const freshWords = useWizardStore.getState().words;

    selectedSegments.forEach(wordKey => {
      const word = freshWords.find(w => w.wordKey === wordKey);
      if (word && isDefined(word.annotationId) && isDefined(annotationManagerRef.current)) {
        annotationManagerRef.current.removeAnnotation(word.annotationId);
      }
      deleteWord(wordKey);
    });

    console.log('[WordSegmenter] Bulk deleted', selectedSegments.size, 'segments');
    setSelectedSegments(new Set());
    setShowBulkDeleteConfirm(false);
  };

  const getWordStatus = (word: QpcWord) => {
    const key = word.location;
    return words.some(w => w.wordKey === key);
  };

  const segmentedCount = qpcWords.filter(w => getWordStatus(w)).length;
  const progress = qpcWords.length > 0 ? (segmentedCount / qpcWords.length) * 100 : 0;

  const canGoNext = activeVerseIdx < verses.length - 1;
  const canGoPrev = activeVerseIdx > 0;

  if (!ayahAudioUrl || !currentVerse) {
    return (
      <Stack spacing={3}>
        <Skeleton variant="rectangular" height={60} />
        <Stack direction="row" spacing={1}>
          <Skeleton variant="rectangular" width={100} height={36} />
          <Skeleton variant="rectangular" width={100} height={36} />
        </Stack>
        <Skeleton variant="rectangular" height={56} />
        <Skeleton variant="rectangular" height={120} />
        <Skeleton variant="rectangular" height={200} />
        <Stack direction="row" spacing={1} flexWrap="wrap">
          {[...Array(8)].map((_, i) => (
            <Skeleton key={i} variant="rectangular" width={80} height={32} sx={{ mb: 1 }} />
          ))}
        </Stack>
        <Skeleton variant="rectangular" height={300} />
      </Stack>
    );
  }

  const constraints = {
    surah: {},
    word: {
      disallowOverlapWithSameKind: false, // Words can overlap
      mustBeInsideKind: null,
    },
    other: {},
  };

  return (
    <Stack spacing={3}>
      <Alert severity="info">
        Segment each word within the ayah. Words can overlap up to 100ms if they have merging tajweed rules.
        <br />
        <strong>You are viewing ONLY the audio for Ayah {currentVerse.ayah}.</strong> All times shown are relative to this ayah segment.
      </Alert>

      {error && <Alert severity="error">{error}</Alert>}

      {/* Verse selector */}
      <Box>
        <Stack
          direction="row"
          justifyContent="space-between"
          alignItems="center"
        >
          <h3 style={{ margin: 0 }}>Select Ayah:</h3>
          <Stack direction="row" spacing={1}>
            <Button
              startIcon={<NavigateBefore />}
              disabled={!canGoPrev}
              onClick={() => setActiveVerseIdx(activeVerseIdx - 1)}
              size="small"
            >
              Previous
            </Button>
            <Button
              endIcon={<NavigateNext />}
              disabled={!canGoNext}
              onClick={() => setActiveVerseIdx(activeVerseIdx + 1)}
              size="small"
            >
              Next
            </Button>
          </Stack>
        </Stack>

        <ToggleButtonGroup
          value={activeVerseIdx}
          exclusive
          onChange={(_, idx) => idx !== null && setActiveVerseIdx(idx)}
          sx={{ mt: 1 }}
        >
          {verses.map((v, i) => (
            <ToggleButton key={v.ayah} value={i}>
              Ayah {v.ayah}
              {i === activeVerseIdx && ` (${segmentedCount}/${qpcWords.length})`}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </Box>

      {/* Progress */}
      <Box>
        <Stack direction="row" justifyContent="space-between" sx={{ mb: 1 }}>
          <span>
            <strong>Progress:</strong> {segmentedCount} / {qpcWords.length}{' '}
            words segmented
          </span>
          <span>{progress.toFixed(0)}%</span>
        </Stack>
        <LinearProgress variant="determinate" value={progress} />
      </Box>

      {/* Verse preview */}
      <Paper sx={{ p: 2, bgcolor: 'grey.50' }}>
        <Chip
          label={`Ayah ${currentVerse.ayah}`}
          size="small"
          sx={{ mb: 1 }}
        />
        <Box sx={{ direction: 'rtl', textAlign: 'right' }}>
          <TajweedText htmlText={currentVerse.text} fontSize={18} />
        </Box>
      </Paper>

      {/* Word selector */}
      {!fetchingWords && qpcWords.length > 0 && (
        <Box>
          <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 1 }}>
            <h4 style={{ margin: 0 }}>Select Word to Segment:</h4>
            <Stack direction="row" spacing={1}>
              <Tooltip title="Automatically segment all unsegmented words using letter/madd-based duration estimation">
                <span>
                  <Button
                    variant="contained"
                    size="small"
                    onClick={handleAutoSegment}
                    disabled={segmentedCount === qpcWords.length}
                  >
                    Auto-Segment ({qpcWords.length - segmentedCount} remaining)
                  </Button>
                </span>
              </Tooltip>
              <Tooltip title="Clear all word segments for this ayah">
                <span>
                  <Button
                    variant="outlined"
                    size="small"
                    color="error"
                    onClick={() => setShowClearConfirm(true)}
                    disabled={segmentedCount === 0}
                  >
                    Clear All
                  </Button>
                </span>
              </Tooltip>
            </Stack>
          </Stack>
          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            {qpcWords.map((word, idx) => {
              const isDone = getWordStatus(word);
              const isSelected = idx === selectedWordIdx;

              return (
                <Chip
                  key={idx}
                  label={
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                      <Typography variant="caption">{word.word}.</Typography>
                      <TajweedText htmlText={word.text} fontSize={14} />
                    </Box>
                  }
                  icon={
                    isDone ? (
                      <CheckCircle />
                    ) : (
                      <RadioButtonUnchecked />
                    )
                  }
                  color={
                    isSelected ? 'primary' : isDone ? 'success' : 'default'
                  }
                  onClick={() => !isDone && setSelectedWordIdx(idx)}
                  variant={isSelected ? 'filled' : 'outlined'}
                  disabled={isDone}
                  sx={{ mb: 1 }}
                />
              );
            })}
          </Stack>
        </Box>
      )}

      {/* Waveform annotator */}
      <Box>
        <h3>Ayah {currentVerse.ayah} Audio (Isolated)</h3>
        {selectedWordIdx !== null && qpcWords[selectedWordIdx] && (
          <Alert severity="info" sx={{ mb: 1 }}>
            Segmenting word: <TajweedText htmlText={qpcWords[selectedWordIdx].text} fontSize={14} />
          </Alert>
        )}
        <WavesurferAnnotator
          src={ayahAudioUrl}
          controlledKind="word"
          constraints={constraints}
          onCreate={handleCreateAnnotation}
          onUpdate={handleUpdateAnnotation}
          showMinimap={true}
          managerRef={annotationManagerRef}
        />
        <Stack direction="row" spacing={1} sx={{ mt: 1 }}>
          <Alert severity="info" sx={{ flex: 1 }}>
            All times shown are relative to this ayah segment. Overlap allowed for merging rules (idgham, etc.).
          </Alert>
          {getAyahMergedBoundaries().length > 0 && (
            <Alert severity="success" icon={<Link />} sx={{ minWidth: 'fit-content' }}>
              <strong>{getAyahMergedBoundaries().length} merged boundaries</strong> (boundaries move together when dragged)
            </Alert>
          )}
        </Stack>
      </Box>

      {/* Segmented words table */}
      {currentVerseWords.length > 0 && (
        <Box>
          <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 1 }}>
            <h3 style={{ margin: 0 }}>Segmented Words ({currentVerseWords.length})</h3>
            {selectedSegments.size > 0 && (
              <Tooltip title={`Delete ${selectedSegments.size} selected segment${selectedSegments.size !== 1 ? 's' : ''}`}>
                <Button
                  variant="contained"
                  color="error"
                  size="small"
                  startIcon={<DeleteSweep />}
                  onClick={() => setShowBulkDeleteConfirm(true)}
                >
                  Delete Selected ({selectedSegments.size})
                </Button>
              </Tooltip>
            )}
          </Stack>
          <Alert severity="info" sx={{ mb: 2 }}>
            <strong>Merged boundaries:</strong> Adjacent words share a boundary (move together when dragging).
            Click the merge icon to toggle coupling.
            {selectedSegments.size > 0 && (
              <>
                <br />
                <strong>Bulk delete:</strong> {selectedSegments.size} segment{selectedSegments.size !== 1 ? 's' : ''} selected. Click "Delete Selected" to remove them.
              </>
            )}
          </Alert>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell padding="checkbox">
                  <Tooltip title="Select/deselect all segments">
                    <Checkbox
                      checked={selectedSegments.size === currentVerseWords.length && currentVerseWords.length > 0}
                      indeterminate={selectedSegments.size > 0 && selectedSegments.size < currentVerseWords.length}
                      onChange={toggleSelectAll}
                    />
                  </Tooltip>
                </TableCell>
                <TableCell>Word #</TableCell>
                <TableCell>Text</TableCell>
                <TableCell>Start (abs)</TableCell>
                <TableCell>End (abs)</TableCell>
                <TableCell>Duration</TableCell>
                <TableCell align="center">Boundaries</TableCell>
                <TableCell align="right">Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {currentVerseWords.map(w => {
                const startMerged = isStartMerged(w.wordKey);
                const endMerged = isEndMerged(w.wordKey);
                const isSelected = selectedSegments.has(w.wordKey);

                return (
                  <TableRow key={w.wordKey} selected={isSelected}>
                    <TableCell padding="checkbox">
                      <Checkbox
                        checked={isSelected}
                        onChange={() => toggleSegmentSelection(w.wordKey)}
                      />
                    </TableCell>
                    <TableCell>{w.wordKey.split(':')[2]}</TableCell>
                    <TableCell sx={{ direction: 'rtl', textAlign: 'right' }}>
                      <TajweedText htmlText={w.text} fontSize={14} />
                    </TableCell>
                    <TableCell>{w.start.toFixed(3)}s</TableCell>
                    <TableCell>{w.end.toFixed(3)}s</TableCell>
                    <TableCell>{(w.end - w.start).toFixed(3)}s</TableCell>
                    <TableCell align="center">
                      <Stack direction="row" spacing={0.5} justifyContent="center">
                        <Tooltip title={startMerged ? 'Start merged with previous word' : 'Start independent (first word or unmerged)'}>
                          <span>
                            <IconButton
                              size="small"
                              color={startMerged ? 'primary' : 'default'}
                              onClick={() => toggleStartMerge(w.wordKey)}
                              disabled={!startMerged && parseInt(w.wordKey.split(':')[2]) === 1}
                            >
                              {startMerged ? <Link fontSize="small" /> : <LinkOff fontSize="small" />}
                            </IconButton>
                          </span>
                        </Tooltip>
                        <Tooltip title={endMerged ? 'End merged with next word' : 'End independent (last word or unmerged)'}>
                          <span>
                            <IconButton
                              size="small"
                              color={endMerged ? 'primary' : 'default'}
                              onClick={() => toggleEndMerge(w.wordKey)}
                              disabled={!endMerged && parseInt(w.wordKey.split(':')[2]) === qpcWords.length}
                            >
                              {endMerged ? <Link fontSize="small" /> : <LinkOff fontSize="small" />}
                            </IconButton>
                          </span>
                        </Tooltip>
                      </Stack>
                    </TableCell>
                    <TableCell align="right">
                      <Tooltip title="Delete segment">
                        <IconButton
                          size="small"
                          color="error"
                          onClick={() => handleDelete(w.wordKey)}
                        >
                          <Delete />
                        </IconButton>
                      </Tooltip>
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </Box>
      )}

      {/* Confirmation dialog for Clear All */}
      <ConfirmDialog
        open={showClearConfirm}
        title="Clear All Word Segments?"
        message={`This will delete all ${segmentedCount} word segment${segmentedCount !== 1 ? 's' : ''} for Ayah ${currentVerse?.ayah}. This action cannot be undone.`}
        confirmText="Clear All"
        confirmColor="error"
        onConfirm={handleClearAll}
        onCancel={() => setShowClearConfirm(false)}
      />

      {/* Confirmation dialog for Bulk Delete */}
      <ConfirmDialog
        open={showBulkDeleteConfirm}
        title="Delete Selected Segments?"
        message={`This will delete ${selectedSegments.size} selected word segment${selectedSegments.size !== 1 ? 's' : ''} for Ayah ${currentVerse?.ayah}. This action cannot be undone.`}
        confirmText={`Delete ${selectedSegments.size} Segment${selectedSegments.size !== 1 ? 's' : ''}`}
        confirmColor="error"
        onConfirm={handleBulkDelete}
        onCancel={() => setShowBulkDeleteConfirm(false)}
      />
    </Stack>
  );
};

export default WordSegmenter;
