// Wizard store with undo/redo and persistence
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { temporal } from 'zundo';
import { nanoid } from 'nanoid';
import type { AnnotationExport, Verse } from '../types/export';
import { SegmentConstraints } from '../utils/segmentConstraints';

export type WizardStep = 0 | 1 | 2 | 3 | 4;

// Internal state types (exported for use in hooks)
export interface VerseSegment {
  ayah: number;
  start: number;
  end: number;
  text: string;
  annotationId?: string; // ID of the visual region in WaveSurfer
}

export interface WordSegment {
  wordKey: string; // "surah:ayah:word"
  ayah: number;
  start: number;
  end: number;
  text: string;
  annotationId?: string; // ID of the visual region in WaveSurfer
}

export interface AntiPatternSegment {
  id: string;
  wordKey: string;
  type: string;
  start: number;
  end: number;
  confidence: number;
  notes?: string;
}

interface WizardState {
  // Navigation
  step: WizardStep;

  // Selection (Stage 0)
  surah: number | null;
  ayahs: number[];
  ayahTexts: Array<{ ayah: number; text: string }>;

  // Audio (Stage 1) - persisted refs only, blobs in IndexedDB
  recordingId: string | null;
  audioDuration: number | null;
  trim: { start: number; end: number } | null;

  // Annotations (Stages 2-4)
  verses: VerseSegment[];
  words: WordSegment[];
  antiPatterns: AntiPatternSegment[];
  expectedWordCounts: Record<number, number>; // ayah -> expected word count from QPC

  // Merged boundaries (Stage 3)
  // Tracks which word boundaries are coupled (move together when dragging)
  // Key format: "ayah:wordIdx1-wordIdx2" (e.g., "7:2-3" = boundary between word 2 and 3 in ayah 7)
  mergedBoundaries: Set<string>;

  // UI state (not persisted)
  activeVerseIdx: number;
  activeWordKey: string | null;
}

interface WizardActions {
  // Navigation
  setStep(step: WizardStep): void;
  nextStep(): void;
  prevStep(): void;

  // Selection
  setSurah(surah: number): void;
  setAyahRange(start: number, end: number, texts: Array<{ ayah: number; text: string }>): void;

  // Audio
  setRecording(id: string, duration: number): void;
  setTrim(trim: { start: number; end: number }): void;

  // Verse annotations
  addVerse(
    ayah: number,
    start: number,
    end: number,
    text: string,
    annotationId?: string
  ): { ok: boolean; errors: string[] };
  updateVerse(ayah: number, updates: Partial<VerseSegment>): void;
  updateVerseByAnnotationId(annotationId: string, updates: { start: number; end: number }): void;
  deleteVerse(ayah: number): void;

  // Word annotations
  addWord(
    wordKey: string,
    ayah: number,
    start: number,
    end: number,
    text: string,
    annotationId?: string
  ): { ok: boolean; errors: string[] };
  updateWord(wordKey: string, updates: Partial<WordSegment>): void;
  updateWordByAnnotationId(annotationId: string, updates: { start: number; end: number }): void;
  deleteWord(wordKey: string): void;
  setExpectedWordCount(ayah: number, count: number): void;

  // Merged boundaries
  addMergedBoundary(ayah: number, wordIdx1: number, wordIdx2: number): void;
  removeMergedBoundary(ayah: number, wordIdx1: number, wordIdx2: number): void;
  toggleMergedBoundary(ayah: number, wordIdx1: number, wordIdx2: number): void;
  isBoundaryMerged(ayah: number, wordIdx1: number, wordIdx2: number): boolean;
  getMergedBoundaries(ayah: number): Set<string>;

  // Anti-pattern annotations
  addAntiPattern(
    wordKey: string,
    type: string,
    start: number,
    end: number,
    confidence: number,
    notes?: string
  ): { ok: boolean; errors: string[] };
  updateAntiPattern(id: string, updates: Partial<AntiPatternSegment>): void;
  deleteAntiPattern(id: string): void;

  // UI state
  setActiveVerseIdx(idx: number): void;
  setActiveWordKey(key: string | null): void;

  // Validation
  canProceed(): boolean;
  getMissingSegments(): {
    verses: number[];
    words: Record<number, string[]>;
  };

  // Export/Import
  exportAnnotations(): AnnotationExport;
  loadExisting(data: AnnotationExport, recordingId?: string): void;

  // Reset
  reset(): void;
}

type Store = WizardState & WizardActions;

const initialState: WizardState = {
  step: 0,
  surah: null,
  ayahs: [],
  ayahTexts: [],
  recordingId: null,
  audioDuration: null,
  trim: null,
  verses: [],
  words: [],
  antiPatterns: [],
  expectedWordCounts: {},
  mergedBoundaries: new Set<string>(),
  activeVerseIdx: 0,
  activeWordKey: null,
};

export const useWizardStore = create<Store>()(
  temporal(
    persist(
      (set, get) => ({
        ...initialState,

        // Navigation
        setStep(step) {
          set({ step });
        },

        nextStep() {
          const { step, canProceed } = get();
          if (canProceed() && step < 4) {
            set({ step: (step + 1) as WizardStep });
          }
        },

        prevStep() {
          const { step } = get();
          if (step > 0) {
            set({ step: (step - 1) as WizardStep });
          }
        },

        // Selection
        setSurah(surah) {
          set({
            surah,
            ayahs: [],
            ayahTexts: [],
            verses: [],
            words: [],
            antiPatterns: [],
            expectedWordCounts: {},
            mergedBoundaries: new Set<string>(),
          });
        },

        setAyahRange(start, end, texts) {
          const ayahs = Array.from({ length: end - start + 1 }, (_, i) => start + i);
          set({ ayahs, ayahTexts: texts });
        },

        // Audio
        setRecording(id, duration) {
          set({ recordingId: id, audioDuration: duration });
        },

        setTrim(trim) {
          set({ trim });
        },

        // Verse annotations
        addVerse(ayah, start, end, text, annotationId) {
          const { verses, trim } = get();
          if (!trim) return { ok: false, errors: ['No trim bounds set'] };

          const errors = SegmentConstraints.validateVerse(
            verses,
            { ayah, start, end },
            trim
          );

          if (errors.length > 0) return { ok: false, errors };

          // Replace if exists, otherwise add
          const newVerses = [
            ...verses.filter(v => v.ayah !== ayah),
            { ayah, start, end, text, annotationId },
          ].sort((a, b) => a.ayah - b.ayah);

          set({ verses: newVerses });
          return { ok: true, errors: [] };
        },

        updateVerse(ayah, updates) {
          set(state => ({
            verses: state.verses.map(v =>
              v.ayah === ayah ? { ...v, ...updates } : v
            ),
          }));
        },

        updateVerseByAnnotationId(annotationId, updates) {
          set(state => ({
            verses: state.verses.map(v =>
              v.annotationId === annotationId ? { ...v, ...updates } : v
            ),
          }));
        },

        deleteVerse(ayah) {
          set(state => ({
            verses: state.verses.filter(v => v.ayah !== ayah),
            words: state.words.filter(w => w.ayah !== ayah),
            antiPatterns: state.antiPatterns.filter(ap => {
              const wordAyah = parseInt(ap.wordKey.split(':')[1]);
              return wordAyah !== ayah;
            }),
          }));
        },

        // Word annotations
        addWord(wordKey, ayah, start, end, text, annotationId) {
          const { verses, words } = get();
          const verse = verses.find(v => v.ayah === ayah);

          if (!verse) {
            return { ok: false, errors: ['Parent verse not found'] };
          }

          const errors = SegmentConstraints.validateWord(verse, { start, end });
          if (errors.length > 0) return { ok: false, errors };

          // Replace if exists, otherwise add
          const newWords = [
            ...words.filter(w => w.wordKey !== wordKey),
            { wordKey, ayah, start, end, text, annotationId },
          ];

          set({ words: newWords });
          return { ok: true, errors: [] };
        },

        updateWord(wordKey, updates) {
          set(state => ({
            words: state.words.map(w =>
              w.wordKey === wordKey ? { ...w, ...updates } : w
            ),
          }));
        },

        updateWordByAnnotationId(annotationId, updates) {
          set(state => ({
            words: state.words.map(w =>
              w.annotationId === annotationId ? { ...w, ...updates } : w
            ),
          }));
        },

        deleteWord(wordKey) {
          set(state => ({
            words: state.words.filter(w => w.wordKey !== wordKey),
            antiPatterns: state.antiPatterns.filter(ap => ap.wordKey !== wordKey),
          }));
        },

        setExpectedWordCount(ayah, count) {
          set(state => ({
            expectedWordCounts: {
              ...state.expectedWordCounts,
              [ayah]: count,
            },
          }));
        },

        // Merged boundaries
        addMergedBoundary(ayah, wordIdx1, wordIdx2) {
          const key = `${ayah}:${wordIdx1}-${wordIdx2}`;
          set(state => ({
            mergedBoundaries: new Set([...state.mergedBoundaries, key]),
          }));
        },

        removeMergedBoundary(ayah, wordIdx1, wordIdx2) {
          const key = `${ayah}:${wordIdx1}-${wordIdx2}`;
          set(state => {
            const newBoundaries = new Set(state.mergedBoundaries);
            newBoundaries.delete(key);
            return { mergedBoundaries: newBoundaries };
          });
        },

        toggleMergedBoundary(ayah, wordIdx1, wordIdx2) {
          const key = `${ayah}:${wordIdx1}-${wordIdx2}`;
          set(state => {
            const newBoundaries = new Set(state.mergedBoundaries);
            if (newBoundaries.has(key)) {
              newBoundaries.delete(key);
            } else {
              newBoundaries.add(key);
            }
            return { mergedBoundaries: newBoundaries };
          });
        },

        isBoundaryMerged(ayah, wordIdx1, wordIdx2) {
          const key = `${ayah}:${wordIdx1}-${wordIdx2}`;
          return get().mergedBoundaries.has(key);
        },

        getMergedBoundaries(ayah) {
          const { mergedBoundaries } = get();
          const ayahBoundaries = new Set<string>();
          mergedBoundaries.forEach(key => {
            if (key.startsWith(`${ayah}:`)) {
              ayahBoundaries.add(key);
            }
          });
          return ayahBoundaries;
        },

        // Anti-pattern annotations
        addAntiPattern(wordKey, type, start, end, confidence, notes) {
          const { words } = get();
          const word = words.find(w => w.wordKey === wordKey);

          if (!word) {
            return { ok: false, errors: ['Parent word not found'] };
          }

          const errors = SegmentConstraints.validateAntiPattern(word, {
            start,
            end,
          });
          if (errors.length > 0) return { ok: false, errors };

          set(state => ({
            antiPatterns: [
              ...state.antiPatterns,
              { id: nanoid(), wordKey, type, start, end, confidence, notes },
            ],
          }));

          return { ok: true, errors: [] };
        },

        updateAntiPattern(id, updates) {
          set(state => ({
            antiPatterns: state.antiPatterns.map(ap =>
              ap.id === id ? { ...ap, ...updates } : ap
            ),
          }));
        },

        deleteAntiPattern(id) {
          set(state => ({
            antiPatterns: state.antiPatterns.filter(ap => ap.id !== id),
          }));
        },

        // UI state
        setActiveVerseIdx(idx) {
          set({ activeVerseIdx: idx });
        },

        setActiveWordKey(key) {
          set({ activeWordKey: key });
        },

        // Validation
        canProceed() {
          const { step, surah, ayahs, recordingId, trim, verses, words, ayahTexts } = get();

          switch (step) {
            case 0:
              // Must have surah and at least one ayah selected
              return surah !== null && ayahs.length > 0;

            case 1:
              // Must have recording and trim
              return recordingId !== null && trim !== null && trim.end > trim.start;

            case 2: {
              // All selected ayahs must be segmented
              const needed = new Set(ayahs);
              verses.forEach(v => needed.delete(v.ayah));
              return needed.size === 0;
            }

            case 3: {
              // All words in all verses must be segmented
              const { expectedWordCounts } = get();

              for (const verse of verses) {
                const expectedCount = expectedWordCounts[verse.ayah];
                const actualCount = words.filter(w => w.ayah === verse.ayah).length;

                console.log('[canProceed] Ayah', verse.ayah, ':', actualCount, '/', expectedCount, 'words');

                // If we don't have expected count yet, can't validate
                if (expectedCount === undefined) {
                  console.log('[canProceed] No expected count for ayah', verse.ayah);
                  return false;
                }

                // Check if all words are segmented
                if (actualCount < expectedCount) {
                  console.log('[canProceed] Missing words for ayah', verse.ayah);
                  return false;
                }
              }

              console.log('[canProceed] All words segmented!');
              return true;
            }

            case 4:
              // Anti-patterns are optional, always can proceed
              return true;

            default:
              return false;
          }
        },

        getMissingSegments() {
          const { ayahs, verses, words, expectedWordCounts } = get();

          // Missing verses
          const missingVerses = ayahs.filter(a => !verses.some(v => v.ayah === a));

          // Missing words per verse
          const missingWords: Record<number, string[]> = {};

          for (const verse of verses) {
            const ayahWords = words.filter(w => w.ayah === verse.ayah);
            const actualCount = ayahWords.length;
            const expectedCount = expectedWordCounts[verse.ayah];

            if (expectedCount !== undefined && actualCount < expectedCount) {
              missingWords[verse.ayah] = [
                `${actualCount}/${expectedCount} words segmented`,
              ];
            } else if (expectedCount === undefined && actualCount === 0) {
              missingWords[verse.ayah] = ['No words segmented yet'];
            }
          }

          return { verses: missingVerses, words: missingWords };
        },

        // Export
        exportAnnotations() {
          const {
            surah,
            verses,
            words,
            antiPatterns,
            audioDuration,
            trim,
            recordingId,
          } = get();

          if (!surah || !audioDuration || !trim || !recordingId) {
            throw new Error('Cannot export: missing required data');
          }

          const versesData: Verse[] = verses.map(v => {
            const verseWords = words
              .filter(w => w.ayah === v.ayah)
              .sort((a, b) => {
                const aIdx = parseInt(a.wordKey.split(':')[2]);
                const bIdx = parseInt(b.wordKey.split(':')[2]);
                return aIdx - bIdx;
              });

            return {
              ayah: v.ayah,
              segment: [v.start, v.end] as [number, number],
              text: v.text,
              words: verseWords.map(w => {
                const wordAntiPatterns = antiPatterns.filter(
                  ap => ap.wordKey === w.wordKey
                );

                return {
                  index: parseInt(w.wordKey.split(':')[2]),
                  location: w.wordKey,
                  segment: [w.start, w.end] as [number, number],
                  text: w.text,
                  anti_patterns: wordAntiPatterns.map(ap => ({
                    type: ap.type,
                    segment: [ap.start, ap.end] as [number, number],
                    confidence: ap.confidence,
                    notes: ap.notes,
                  })),
                };
              }),
            };
          });

          return {
            version: '1.0' as const,
            recording_id: recordingId,
            created_at: new Date().toISOString(),
            audio: {
              sample_rate: 16000 as const,
              duration_sec: audioDuration,
              trimmed: trim,
            },
            content: {
              surah,
              verses: versesData,
            },
          };
        },

        // Import
        loadExisting(data, recordingId) {
          set({
            step: 0,
            surah: data.content.surah,
            ayahs: data.content.verses.map(v => v.ayah),
            ayahTexts: data.content.verses.map(v => ({
              ayah: v.ayah,
              text: v.text,
            })),
            recordingId: recordingId || data.recording_id,
            audioDuration: data.audio.duration_sec,
            trim: data.audio.trimmed,
            verses: data.content.verses.map(v => ({
              ayah: v.ayah,
              start: v.segment[0],
              end: v.segment[1],
              text: v.text,
            })),
            words: data.content.verses.flatMap(v =>
              v.words.map(w => ({
                wordKey: w.location,
                ayah: v.ayah,
                start: w.segment[0],
                end: w.segment[1],
                text: w.text,
              }))
            ),
            antiPatterns: data.content.verses.flatMap(v =>
              v.words.flatMap(w =>
                w.anti_patterns.map(ap => ({
                  id: nanoid(),
                  wordKey: w.location,
                  type: ap.type,
                  start: ap.segment[0],
                  end: ap.segment[1],
                  confidence: ap.confidence,
                  notes: ap.notes,
                }))
              )
            ),
            activeVerseIdx: 0,
            activeWordKey: null,
          });
        },

        // Reset
        reset() {
          set(initialState);
        },
      }),
      {
        name: 'tajweed-wizard-v1',
        storage: createJSONStorage(() => localStorage),
        partialize: (state) => {
          // Don't persist UI state
          const { activeVerseIdx, activeWordKey, ...rest } = state as WizardState & WizardActions;

          // Convert Set to Array for JSON serialization
          return {
            ...rest,
            mergedBoundaries: Array.from(rest.mergedBoundaries),
          };
        },
        // Custom merge function to handle Set deserialization
        merge: (persistedState: any, currentState: WizardState & WizardActions) => {
          return {
            ...currentState,
            ...persistedState,
            // Convert Array back to Set
            mergedBoundaries: new Set(persistedState.mergedBoundaries || []),
          };
        },
      }
    ),
    {
      limit: 50,
      equality: (a, b) => JSON.stringify(a) === JSON.stringify(b),
    }
  )
);
