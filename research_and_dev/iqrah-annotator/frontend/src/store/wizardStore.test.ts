import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useWizardStore } from './wizardStore';
import type { AnnotationExport } from '../types/export';

// Helper to get a fresh store for each test
const getStore = () => useWizardStore.getState();
const resetStore = () => useWizardStore.getState().reset();

describe('wizardStore', () => {
  beforeEach(() => {
    resetStore();
    localStorage.clear();
  });

  describe('Navigation', () => {
    it('should initialize with step 0', () => {
      const { step } = getStore();
      expect(step).toBe(0);
    });

    it('should set step directly', () => {
      const { setStep } = getStore();
      setStep(2);
      expect(getStore().step).toBe(2);
    });

    it('should move to next step when can proceed', () => {
      const { setSurah, setAyahRange, nextStep } = getStore();

      // Stage 0: select surah and ayahs
      setSurah(1);
      setAyahRange(1, 2, [
        { ayah: 1, text: 'بِسْمِ اللَّهِ' },
        { ayah: 2, text: 'الرَّحْمَٰنِ الرَّحِيمِ' },
      ]);

      nextStep();
      expect(getStore().step).toBe(1);
    });

    it('should not move to next step when cannot proceed', () => {
      const { nextStep } = getStore();
      nextStep(); // No surah selected
      expect(getStore().step).toBe(0);
    });

    it('should move to previous step', () => {
      const { setStep, prevStep } = getStore();
      setStep(2);
      prevStep();
      expect(getStore().step).toBe(1);
    });

    it('should not move below step 0', () => {
      const { prevStep } = getStore();
      prevStep();
      expect(getStore().step).toBe(0);
    });

    it('should not move above step 4', () => {
      const { setStep, nextStep, setSurah, setAyahRange } = getStore();
      setStep(4);
      setSurah(1);
      setAyahRange(1, 1, [{ ayah: 1, text: 'test' }]);
      nextStep();
      expect(getStore().step).toBe(4);
    });
  });

  describe('Selection (Stage 0)', () => {
    it('should set surah', () => {
      const { setSurah } = getStore();
      setSurah(1);
      expect(getStore().surah).toBe(1);
    });

    it('should clear annotations when changing surah', () => {
      const { setSurah, setAyahRange, setRecording, setTrim, addVerse } = getStore();

      // Setup initial data
      setSurah(1);
      setAyahRange(1, 1, [{ ayah: 1, text: 'test' }]);
      setRecording('rec-1', 10);
      setTrim({ start: 0, end: 10 });
      addVerse(1, 0, 5, 'test');

      // Change surah
      setSurah(2);

      const state = getStore();
      expect(state.surah).toBe(2);
      expect(state.ayahs).toEqual([]);
      expect(state.verses).toEqual([]);
      expect(state.words).toEqual([]);
      expect(state.antiPatterns).toEqual([]);
    });

    it('should set ayah range', () => {
      const { setAyahRange } = getStore();
      setAyahRange(1, 3, [
        { ayah: 1, text: 'text1' },
        { ayah: 2, text: 'text2' },
        { ayah: 3, text: 'text3' },
      ]);

      const { ayahs, ayahTexts } = getStore();
      expect(ayahs).toEqual([1, 2, 3]);
      expect(ayahTexts).toHaveLength(3);
      expect(ayahTexts[0]).toEqual({ ayah: 1, text: 'text1' });
    });

    it('should generate correct ayah range from start to end', () => {
      const { setAyahRange } = getStore();
      setAyahRange(5, 8, [
        { ayah: 5, text: 'a' },
        { ayah: 6, text: 'b' },
        { ayah: 7, text: 'c' },
        { ayah: 8, text: 'd' },
      ]);

      expect(getStore().ayahs).toEqual([5, 6, 7, 8]);
    });
  });

  describe('Audio (Stage 1)', () => {
    it('should set recording', () => {
      const { setRecording } = getStore();
      setRecording('rec-123', 45.5);

      const { recordingId, audioDuration } = getStore();
      expect(recordingId).toBe('rec-123');
      expect(audioDuration).toBe(45.5);
    });

    it('should set trim bounds', () => {
      const { setTrim } = getStore();
      setTrim({ start: 1.5, end: 8.2 });

      const { trim } = getStore();
      expect(trim).toEqual({ start: 1.5, end: 8.2 });
    });
  });

  describe('Verse Annotations (Stage 2)', () => {
    beforeEach(() => {
      const { setSurah, setAyahRange, setRecording, setTrim } = getStore();
      setSurah(1);
      setAyahRange(1, 2, [
        { ayah: 1, text: 'بِسْمِ اللَّهِ' },
        { ayah: 2, text: 'الرَّحْمَٰنِ الرَّحِيمِ' },
      ]);
      setRecording('rec-1', 10);
      setTrim({ start: 0, end: 10 });
    });

    it('should add valid verse annotation', () => {
      const { addVerse } = getStore();
      const result = addVerse(1, 0, 5, 'بِسْمِ اللَّهِ');

      expect(result.ok).toBe(true);
      expect(result.errors).toHaveLength(0);
      expect(getStore().verses).toHaveLength(1);
      expect(getStore().verses[0]).toMatchObject({
        ayah: 1,
        start: 0,
        end: 5,
        text: 'بِسْمِ اللَّهِ',
      });
    });

    it('should reject verse with invalid bounds (start >= end)', () => {
      const { addVerse } = getStore();
      const result = addVerse(1, 5, 5, 'test');

      expect(result.ok).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(getStore().verses).toHaveLength(0);
    });

    it('should reject verse outside trim bounds', () => {
      const { addVerse } = getStore();
      const result = addVerse(1, -1, 5, 'test');

      expect(result.ok).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should reject overlapping verse annotations', () => {
      const { addVerse } = getStore();

      addVerse(1, 0, 5, 'verse1');
      const result = addVerse(2, 4, 8, 'verse2'); // Overlaps with verse1

      expect(result.ok).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should replace existing verse for same ayah', () => {
      const { addVerse } = getStore();

      addVerse(1, 0, 5, 'first');
      addVerse(1, 1, 6, 'second');

      const { verses } = getStore();
      expect(verses).toHaveLength(1);
      expect(verses[0].start).toBe(1);
      expect(verses[0].end).toBe(6);
    });

    it('should update verse', () => {
      const { addVerse, updateVerse } = getStore();

      addVerse(1, 0, 5, 'test');
      updateVerse(1, { start: 0.5, end: 5.5 });

      const verse = getStore().verses[0];
      expect(verse.start).toBe(0.5);
      expect(verse.end).toBe(5.5);
    });

    it('should update verse by annotation ID', () => {
      const { addVerse, updateVerseByAnnotationId } = getStore();

      addVerse(1, 0, 5, 'test', 'annot-1');
      updateVerseByAnnotationId('annot-1', { start: 1, end: 6 });

      const verse = getStore().verses[0];
      expect(verse.start).toBe(1);
      expect(verse.end).toBe(6);
    });

    it('should delete verse and cascade to words/anti-patterns', () => {
      const { addVerse, addWord, deleteVerse, setExpectedWordCount } = getStore();

      addVerse(1, 0, 5, 'test');
      setExpectedWordCount(1, 2);
      addWord('1:1:0', 1, 0, 2, 'word1');
      addWord('1:1:1', 1, 2, 5, 'word2');

      deleteVerse(1);

      const { verses, words } = getStore();
      expect(verses).toHaveLength(0);
      expect(words).toHaveLength(0);
    });

    it('should sort verses by ayah number', () => {
      const { addVerse } = getStore();

      addVerse(2, 5, 10, 'verse2');
      addVerse(1, 0, 5, 'verse1');

      const { verses } = getStore();
      expect(verses[0].ayah).toBe(1);
      expect(verses[1].ayah).toBe(2);
    });
  });

  describe('Word Annotations (Stage 3)', () => {
    beforeEach(() => {
      const { setSurah, setAyahRange, setRecording, setTrim, addVerse } = getStore();
      setSurah(1);
      setAyahRange(1, 1, [{ ayah: 1, text: 'بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ' }]);
      setRecording('rec-1', 10);
      setTrim({ start: 0, end: 10 });
      addVerse(1, 0, 10, 'بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ');
    });

    it('should add valid word annotation', () => {
      const { addWord } = getStore();
      const result = addWord('1:1:0', 1, 0, 2.5, 'بِسْمِ');

      expect(result.ok).toBe(true);
      expect(result.errors).toHaveLength(0);
      expect(getStore().words).toHaveLength(1);
      expect(getStore().words[0]).toMatchObject({
        wordKey: '1:1:0',
        ayah: 1,
        start: 0,
        end: 2.5,
        text: 'بِسْمِ',
      });
    });

    it('should reject word when parent verse not found', () => {
      const { addWord } = getStore();
      const result = addWord('1:2:0', 2, 0, 2, 'word');

      expect(result.ok).toBe(false);
      expect(result.errors).toContain('Parent verse not found');
    });

    it('should reject word outside verse bounds', () => {
      const { addWord } = getStore();
      const result = addWord('1:1:0', 1, -1, 2, 'word');

      expect(result.ok).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should replace existing word with same key', () => {
      const { addWord } = getStore();

      addWord('1:1:0', 1, 0, 2, 'first');
      addWord('1:1:0', 1, 0, 3, 'second');

      const { words } = getStore();
      expect(words).toHaveLength(1);
      expect(words[0].end).toBe(3);
      expect(words[0].text).toBe('second');
    });

    it('should update word', () => {
      const { addWord, updateWord } = getStore();

      addWord('1:1:0', 1, 0, 2, 'word');
      updateWord('1:1:0', { start: 0.5, end: 2.5 });

      const word = getStore().words[0];
      expect(word.start).toBe(0.5);
      expect(word.end).toBe(2.5);
    });

    it('should update word by annotation ID', () => {
      const { addWord, updateWordByAnnotationId } = getStore();

      addWord('1:1:0', 1, 0, 2, 'word', 'annot-word-1');
      updateWordByAnnotationId('annot-word-1', { start: 1, end: 3 });

      const word = getStore().words[0];
      expect(word.start).toBe(1);
      expect(word.end).toBe(3);
    });

    it('should delete word and cascade to anti-patterns', () => {
      const { addWord, deleteWord } = getStore();

      addWord('1:1:0', 1, 0, 2, 'word');
      deleteWord('1:1:0');

      expect(getStore().words).toHaveLength(0);
    });

    it('should set expected word count', () => {
      const { setExpectedWordCount } = getStore();

      setExpectedWordCount(1, 4);

      expect(getStore().expectedWordCounts[1]).toBe(4);
    });
  });

  describe('Anti-Pattern Annotations (Stage 4)', () => {
    beforeEach(() => {
      const { setSurah, setAyahRange, setRecording, setTrim, addVerse, addWord } = getStore();
      setSurah(1);
      setAyahRange(1, 1, [{ ayah: 1, text: 'test' }]);
      setRecording('rec-1', 10);
      setTrim({ start: 0, end: 10 });
      addVerse(1, 0, 10, 'test');
      addWord('1:1:0', 1, 0, 5, 'word');
    });

    it('should add valid anti-pattern annotation', () => {
      const { addAntiPattern } = getStore();
      const result = addAntiPattern('1:1:0', 'weak-ghunnah', 1, 3, 0.85, 'slight nasal');

      expect(result.ok).toBe(true);
      expect(result.errors).toHaveLength(0);

      const { antiPatterns } = getStore();
      expect(antiPatterns).toHaveLength(1);
      expect(antiPatterns[0]).toMatchObject({
        wordKey: '1:1:0',
        type: 'weak-ghunnah',
        start: 1,
        end: 3,
        confidence: 0.85,
        notes: 'slight nasal',
      });
      expect(antiPatterns[0].id).toBeDefined();
    });

    it('should reject anti-pattern when parent word not found', () => {
      const { addAntiPattern } = getStore();
      const result = addAntiPattern('1:1:999', 'weak-ghunnah', 1, 2, 0.8);

      expect(result.ok).toBe(false);
      expect(result.errors).toContain('Parent word not found');
    });

    it('should reject anti-pattern outside word bounds', () => {
      const { addAntiPattern } = getStore();
      const result = addAntiPattern('1:1:0', 'weak-ghunnah', -1, 2, 0.8);

      expect(result.ok).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should update anti-pattern', () => {
      const { addAntiPattern, updateAntiPattern } = getStore();

      addAntiPattern('1:1:0', 'weak-ghunnah', 1, 3, 0.8);
      const id = getStore().antiPatterns[0].id;

      updateAntiPattern(id, { confidence: 0.9, notes: 'updated note' });

      const ap = getStore().antiPatterns[0];
      expect(ap.confidence).toBe(0.9);
      expect(ap.notes).toBe('updated note');
    });

    it('should delete anti-pattern', () => {
      const { addAntiPattern, deleteAntiPattern } = getStore();

      addAntiPattern('1:1:0', 'weak-ghunnah', 1, 3, 0.8);
      const id = getStore().antiPatterns[0].id;

      deleteAntiPattern(id);

      expect(getStore().antiPatterns).toHaveLength(0);
    });

    it('should allow multiple anti-patterns per word', () => {
      const { addAntiPattern } = getStore();

      addAntiPattern('1:1:0', 'weak-ghunnah', 1, 2, 0.8);
      addAntiPattern('1:1:0', 'no-qalqalah', 2, 3, 0.9);

      expect(getStore().antiPatterns).toHaveLength(2);
    });
  });

  describe('UI State', () => {
    it('should set active verse index', () => {
      const { setActiveVerseIdx } = getStore();
      setActiveVerseIdx(3);
      expect(getStore().activeVerseIdx).toBe(3);
    });

    it('should set active word key', () => {
      const { setActiveWordKey } = getStore();
      setActiveWordKey('1:5:2');
      expect(getStore().activeWordKey).toBe('1:5:2');
    });

    it('should allow clearing active word key', () => {
      const { setActiveWordKey } = getStore();
      setActiveWordKey('1:5:2');
      setActiveWordKey(null);
      expect(getStore().activeWordKey).toBeNull();
    });
  });

  describe('Validation (canProceed)', () => {
    it('should allow proceeding from stage 0 when surah and ayahs selected', () => {
      const { setSurah, setAyahRange, canProceed } = getStore();

      expect(canProceed()).toBe(false);

      setSurah(1);
      expect(canProceed()).toBe(false);

      setAyahRange(1, 1, [{ ayah: 1, text: 'test' }]);
      expect(canProceed()).toBe(true);
    });

    it('should allow proceeding from stage 1 when recording and trim set', () => {
      const { setSurah, setAyahRange, setStep, setRecording, setTrim, canProceed } = getStore();

      setSurah(1);
      setAyahRange(1, 1, [{ ayah: 1, text: 'test' }]);
      setStep(1);

      expect(canProceed()).toBe(false);

      setRecording('rec-1', 10);
      expect(canProceed()).toBe(false);

      setTrim({ start: 0, end: 10 });
      expect(canProceed()).toBe(true);
    });

    it('should reject invalid trim bounds (start >= end)', () => {
      const { setSurah, setAyahRange, setStep, setRecording, setTrim, canProceed } = getStore();

      setSurah(1);
      setAyahRange(1, 1, [{ ayah: 1, text: 'test' }]);
      setStep(1);
      setRecording('rec-1', 10);
      setTrim({ start: 5, end: 5 });

      expect(canProceed()).toBe(false);
    });

    it('should allow proceeding from stage 2 when all ayahs segmented', () => {
      const { setSurah, setAyahRange, setRecording, setTrim, setStep, addVerse, canProceed } = getStore();

      setSurah(1);
      setAyahRange(1, 2, [
        { ayah: 1, text: 'a' },
        { ayah: 2, text: 'b' },
      ]);
      setRecording('rec-1', 10);
      setTrim({ start: 0, end: 10 });
      setStep(2);

      expect(canProceed()).toBe(false);

      addVerse(1, 0, 5, 'a');
      expect(canProceed()).toBe(false);

      addVerse(2, 5, 10, 'b');
      expect(canProceed()).toBe(true);
    });

    it('should allow proceeding from stage 3 when all words segmented', () => {
      const {
        setSurah,
        setAyahRange,
        setRecording,
        setTrim,
        addVerse,
        setStep,
        setExpectedWordCount,
        addWord,
        canProceed,
      } = getStore();

      setSurah(1);
      setAyahRange(1, 1, [{ ayah: 1, text: 'test' }]);
      setRecording('rec-1', 10);
      setTrim({ start: 0, end: 10 });
      addVerse(1, 0, 10, 'test');
      setStep(3);

      // No expected word count yet
      expect(canProceed()).toBe(false);

      setExpectedWordCount(1, 2);
      expect(canProceed()).toBe(false);

      addWord('1:1:0', 1, 0, 5, 'word1');
      expect(canProceed()).toBe(false);

      addWord('1:1:1', 1, 5, 10, 'word2');
      expect(canProceed()).toBe(true);
    });

    it('should always allow proceeding from stage 4 (anti-patterns optional)', () => {
      const { setStep, canProceed } = getStore();
      setStep(4);
      expect(canProceed()).toBe(true);
    });
  });

  describe('Validation (getMissingSegments)', () => {
    beforeEach(() => {
      const { setSurah, setAyahRange, setRecording, setTrim } = getStore();
      setSurah(1);
      setAyahRange(1, 3, [
        { ayah: 1, text: 'a' },
        { ayah: 2, text: 'b' },
        { ayah: 3, text: 'c' },
      ]);
      setRecording('rec-1', 10);
      setTrim({ start: 0, end: 10 });
    });

    it('should return missing verses', () => {
      const { addVerse, getMissingSegments } = getStore();

      addVerse(1, 0, 3, 'a');
      addVerse(3, 6, 10, 'c');

      const missing = getMissingSegments();
      expect(missing.verses).toEqual([2]);
    });

    it('should return missing words', () => {
      const { addVerse, setExpectedWordCount, addWord, getMissingSegments } = getStore();

      addVerse(1, 0, 5, 'a');
      addVerse(2, 5, 10, 'b');

      setExpectedWordCount(1, 3);
      setExpectedWordCount(2, 2);

      addWord('1:1:0', 1, 0, 2, 'word1');
      addWord('1:2:0', 2, 5, 7, 'word2');
      addWord('1:2:1', 2, 7, 10, 'word3');

      const missing = getMissingSegments();
      expect(missing.words[1]).toEqual(['1/3 words segmented']);
      expect(missing.words[2]).toBeUndefined(); // Complete
    });

    it('should handle verses with no expected word count', () => {
      const { addVerse, getMissingSegments } = getStore();

      addVerse(1, 0, 5, 'a');

      const missing = getMissingSegments();
      expect(missing.words[1]).toEqual(['No words segmented yet']);
    });
  });

  describe('Export', () => {
    beforeEach(() => {
      const { setSurah, setAyahRange, setRecording, setTrim, addVerse, addWord, setExpectedWordCount } =
        getStore();

      setSurah(1);
      setAyahRange(1, 1, [{ ayah: 1, text: 'بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ' }]);
      setRecording('rec-123', 10);
      setTrim({ start: 0, end: 10 });
      addVerse(1, 0, 10, 'بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ');
      setExpectedWordCount(1, 4);
      addWord('1:1:0', 1, 0, 2.5, 'بِسْمِ');
      addWord('1:1:1', 1, 2.5, 5, 'اللَّهِ');
      addWord('1:1:2', 1, 5, 7.5, 'الرَّحْمَٰنِ');
      addWord('1:1:3', 1, 7.5, 10, 'الرَّحِيمِ');
    });

    it('should export complete annotations', () => {
      const { exportAnnotations } = getStore();
      const exported = exportAnnotations();

      expect(exported.version).toBe('1.0');
      expect(exported.recording_id).toBe('rec-123');
      expect(exported.created_at).toBeDefined();
      expect(exported.audio).toMatchObject({
        sample_rate: 16000,
        duration_sec: 10,
        trimmed: { start: 0, end: 10 },
      });
      expect(exported.content.surah).toBe(1);
      expect(exported.content.verses).toHaveLength(1);
    });

    it('should export verses in correct format', () => {
      const { exportAnnotations } = getStore();
      const exported = exportAnnotations();

      const verse = exported.content.verses[0];
      expect(verse.ayah).toBe(1);
      expect(verse.segment).toEqual([0, 10]);
      expect(verse.text).toBe('بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ');
      expect(verse.words).toHaveLength(4);
    });

    it('should export words sorted by index', () => {
      const { exportAnnotations } = getStore();
      const exported = exportAnnotations();

      const words = exported.content.verses[0].words;
      expect(words[0].index).toBe(0);
      expect(words[1].index).toBe(1);
      expect(words[2].index).toBe(2);
      expect(words[3].index).toBe(3);
    });

    it('should export anti-patterns with words', () => {
      const { addAntiPattern, exportAnnotations } = getStore();

      addAntiPattern('1:1:0', 'weak-ghunnah', 0.5, 1.5, 0.85, 'test note');

      const exported = exportAnnotations();
      const word = exported.content.verses[0].words[0];

      expect(word.anti_patterns).toHaveLength(1);
      expect(word.anti_patterns[0]).toMatchObject({
        type: 'weak-ghunnah',
        segment: [0.5, 1.5],
        confidence: 0.85,
        notes: 'test note',
      });
    });

    it('should throw when exporting incomplete data', () => {
      const { reset, exportAnnotations } = getStore();
      reset();

      expect(() => exportAnnotations()).toThrow('Cannot export: missing required data');
    });
  });

  describe('Import (loadExisting)', () => {
    it('should load existing annotation data', () => {
      const mockData: AnnotationExport = {
        version: '1.0',
        recording_id: 'rec-import',
        created_at: '2025-01-01T00:00:00Z',
        audio: {
          sample_rate: 16000,
          duration_sec: 15,
          trimmed: { start: 0, end: 15 },
        },
        content: {
          surah: 1,
          verses: [
            {
              ayah: 1,
              segment: [0, 7.5],
              text: 'بِسْمِ اللَّهِ',
              words: [
                {
                  index: 0,
                  location: '1:1:0',
                  segment: [0, 3],
                  text: 'بِسْمِ',
                  anti_patterns: [
                    {
                      type: 'weak-ghunnah',
                      segment: [1, 2],
                      confidence: 0.8,
                      notes: 'test',
                    },
                  ],
                },
                {
                  index: 1,
                  location: '1:1:1',
                  segment: [3, 7.5],
                  text: 'اللَّهِ',
                  anti_patterns: [],
                },
              ],
            },
          ],
        },
      };

      const { loadExisting } = getStore();
      loadExisting(mockData);

      const state = getStore();
      expect(state.surah).toBe(1);
      expect(state.recordingId).toBe('rec-import');
      expect(state.audioDuration).toBe(15);
      expect(state.trim).toEqual({ start: 0, end: 15 });
      expect(state.verses).toHaveLength(1);
      expect(state.words).toHaveLength(2);
      expect(state.antiPatterns).toHaveLength(1);
    });

    it('should override recording ID if provided', () => {
      const mockData: AnnotationExport = {
        version: '1.0',
        recording_id: 'rec-old',
        created_at: '2025-01-01T00:00:00Z',
        audio: {
          sample_rate: 16000,
          duration_sec: 10,
          trimmed: { start: 0, end: 10 },
        },
        content: {
          surah: 1,
          verses: [],
        },
      };

      const { loadExisting } = getStore();
      loadExisting(mockData, 'rec-new');

      expect(getStore().recordingId).toBe('rec-new');
    });

    it('should reset to step 0 on import', () => {
      const mockData: AnnotationExport = {
        version: '1.0',
        recording_id: 'rec-1',
        created_at: '2025-01-01T00:00:00Z',
        audio: {
          sample_rate: 16000,
          duration_sec: 10,
          trimmed: { start: 0, end: 10 },
        },
        content: {
          surah: 1,
          verses: [],
        },
      };

      const { setStep, loadExisting } = getStore();
      setStep(3);
      loadExisting(mockData);

      expect(getStore().step).toBe(0);
    });
  });

  describe('Reset', () => {
    it('should reset all state to initial values', () => {
      const {
        setSurah,
        setAyahRange,
        setRecording,
        setTrim,
        addVerse,
        addWord,
        setExpectedWordCount,
        reset,
      } = getStore();

      // Set up complex state
      setSurah(1);
      setAyahRange(1, 2, [
        { ayah: 1, text: 'a' },
        { ayah: 2, text: 'b' },
      ]);
      setRecording('rec-1', 10);
      setTrim({ start: 0, end: 10 });
      addVerse(1, 0, 5, 'a');
      setExpectedWordCount(1, 2);
      addWord('1:1:0', 1, 0, 2.5, 'word');

      // Reset
      reset();

      const state = getStore();
      expect(state.step).toBe(0);
      expect(state.surah).toBeNull();
      expect(state.ayahs).toEqual([]);
      expect(state.recordingId).toBeNull();
      expect(state.trim).toBeNull();
      expect(state.verses).toEqual([]);
      expect(state.words).toEqual([]);
      expect(state.antiPatterns).toEqual([]);
      expect(state.expectedWordCounts).toEqual({});
    });
  });
});
