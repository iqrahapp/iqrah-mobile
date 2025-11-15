import { describe, it, expect } from 'vitest';
import { validateExport, formatValidationErrors } from './exportValidation';
import type { AnnotationExport } from '../types/export';

describe('exportValidation', () => {
  const createValidExport = (): AnnotationExport => ({
    version: '1.0',
    recording_id: 'rec-123',
    created_at: '2025-11-01T10:00:00Z',
    audio: {
      sample_rate: 16000,
      duration_sec: 10.0,
      trimmed: { start: 0, end: 10 },
    },
    content: {
      surah: 1,
      verses: [
        {
          ayah: 1,
          segment: [0, 5],
          text: 'بِسْمِ',
          words: [
            {
              index: 1,
              location: '1:1:1',
              segment: [0, 2],
              text: 'بِسْمِ',
              anti_patterns: [],
            },
          ],
        },
      ],
    },
  });

  describe('validateExport', () => {
    it('should pass validation for valid export', () => {
      const data = createValidExport();
      const result = validateExport(data);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should fail for missing version', () => {
      const data = createValidExport();
      delete (data as any).version;

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          field: 'version',
          severity: 'error',
        })
      );
    });

    it('should fail for missing recording_id', () => {
      const data = createValidExport();
      data.recording_id = '';

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          field: 'recording_id',
        })
      );
    });

    it('should warn for non-16kHz sample rate', () => {
      const data = createValidExport();
      data.audio.sample_rate = 44100;

      const result = validateExport(data);

      expect(result.valid).toBe(true); // Warning, not error
      expect(result.warnings).toContainEqual(
        expect.objectContaining({
          field: 'audio.sample_rate',
          severity: 'warning',
        })
      );
    });

    it('should fail for invalid surah number', () => {
      const data = createValidExport();
      data.content.surah = 115; // Invalid: > 114

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          field: 'content.surah',
          message: expect.stringContaining('1 and 114'),
        })
      );
    });

    it('should fail for verse start >= end', () => {
      const data = createValidExport();
      data.content.verses[0].segment = [5, 5]; // start === end

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          field: 'segment',
          location: 'Verse 1',
        })
      );
    });

    it('should fail for word segment exceeding verse bounds', () => {
      const data = createValidExport();
      data.content.verses[0].segment = [0, 5];
      data.content.verses[0].words[0].segment = [0, 6]; // Exceeds verse end

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          field: 'segment',
          location: 'Verse 1, Word 1',
          message: expect.stringContaining('exceeds verse bounds'),
        })
      );
    });

    it('should detect duplicate ayah numbers', () => {
      const data = createValidExport();
      data.content.verses.push({
        ayah: 1, // Duplicate
        segment: [5, 10],
        text: 'Test',
        words: [],
      });

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          message: expect.stringContaining('Duplicate'),
        })
      );
    });

    it('should detect duplicate word indices', () => {
      const data = createValidExport();
      data.content.verses[0].words.push({
        index: 1, // Duplicate
        location: '1:1:2',
        segment: [2, 4],
        text: 'Test',
        anti_patterns: [],
      });

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          field: 'index',
          location: expect.stringContaining('Word 1'),
        })
      );
    });

    it('should validate anti-pattern within word bounds', () => {
      const data = createValidExport();
      data.content.verses[0].words[0].anti_patterns = [
        {
          type: 'weak-ghunnah',
          segment: [0, 3], // Exceeds word end (2)
          confidence: 0.8,
        },
      ];

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          location: expect.stringContaining('Anti-pattern'),
          message: expect.stringContaining('exceeds word bounds'),
        })
      );
    });

    it('should validate confidence range', () => {
      const data = createValidExport();
      data.content.verses[0].words[0].anti_patterns = [
        {
          type: 'weak-ghunnah',
          segment: [0, 1],
          confidence: 1.5, // Invalid: > 1
        },
      ];

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          field: 'confidence',
        })
      );
    });

    it('should warn for verse without words', () => {
      const data = createValidExport();
      data.content.verses[0].words = [];

      const result = validateExport(data);

      expect(result.valid).toBe(true); // Warning, not error
      expect(result.warnings).toContainEqual(
        expect.objectContaining({
          field: 'words',
          message: expect.stringContaining('no words'),
        })
      );
    });

    it('should fail for empty verses array', () => {
      const data = createValidExport();
      data.content.verses = [];

      const result = validateExport(data);

      expect(result.valid).toBe(false);
      expect(result.errors).toContainEqual(
        expect.objectContaining({
          field: 'content.verses',
          message: expect.stringContaining('No verses'),
        })
      );
    });
  });

  describe('formatValidationErrors', () => {
    it('should format errors and warnings separately', () => {
      const result = {
        valid: false,
        errors: [
          {
            severity: 'error' as const,
            field: 'segment',
            message: 'Invalid segment',
            location: 'Verse 1',
          },
        ],
        warnings: [
          {
            severity: 'warning' as const,
            field: 'sample_rate',
            message: 'Non-standard sample rate',
          },
        ],
      };

      const formatted = formatValidationErrors(result);

      expect(formatted).toContain('**Errors:**');
      expect(formatted).toContain('**Warnings:**');
      expect(formatted).toContain('Invalid segment');
      expect(formatted).toContain('Verse 1');
      expect(formatted).toContain('Non-standard sample rate');
    });

    it('should format errors only when no warnings', () => {
      const result = {
        valid: false,
        errors: [
          {
            severity: 'error' as const,
            field: 'segment',
            message: 'Invalid segment',
          },
        ],
        warnings: [],
      };

      const formatted = formatValidationErrors(result);

      expect(formatted).toContain('**Errors:**');
      expect(formatted).not.toContain('**Warnings:**');
    });

    it('should format warnings only when no errors', () => {
      const result = {
        valid: true,
        errors: [],
        warnings: [
          {
            severity: 'warning' as const,
            field: 'sample_rate',
            message: 'Non-standard sample rate',
          },
        ],
      };

      const formatted = formatValidationErrors(result);

      expect(formatted).not.toContain('**Errors:**');
      expect(formatted).toContain('**Warnings:**');
    });
  });
});
