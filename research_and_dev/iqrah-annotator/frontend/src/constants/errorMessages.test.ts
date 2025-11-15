import { describe, it, expect } from 'vitest';
import {
  getUserFriendlyError,
  formatErrorMessage,
  ERROR_MESSAGES,
} from './errorMessages';

describe('errorMessages', () => {
  describe('getUserFriendlyError', () => {
    it('should return exact match for known error', () => {
      const result = getUserFriendlyError('Parent verse not found');

      expect(result.title).toBe('Missing Parent Verse');
      expect(result.message).toBe('Please segment the ayah before adding words.');
      expect(result.howToFix).toContain('Stage 2');
    });

    it('should return partial match for error containing known substring', () => {
      const result = getUserFriendlyError('Overlap exceeds 150ms limit');

      expect(result.title).toBe('Overlap Too Large');
      expect(result.message).toContain('150ms');
    });

    it('should return default error for unknown error', () => {
      const result = getUserFriendlyError('Unknown error xyz');

      expect(result.title).toBe('Error');
      expect(result.message).toBe('Unknown error xyz');
      expect(result.howToFix).toContain('try again');
    });

    it('should handle validation errors', () => {
      const result = getUserFriendlyError('start >= end');

      expect(result.title).toBe('Invalid Time Range');
      expect(result.message).toContain('Start time must be before end time');
    });
  });

  describe('formatErrorMessage', () => {
    it('should format error with how-to-fix by default', () => {
      const formatted = formatErrorMessage('Parent verse not found');

      expect(formatted).toContain('Please segment the ayah');
      expect(formatted).toContain('💡 How to fix:');
      expect(formatted).toContain('Stage 2');
    });

    it('should format error without how-to-fix when disabled', () => {
      const formatted = formatErrorMessage('Parent verse not found', false);

      expect(formatted).toContain('Please segment the ayah');
      expect(formatted).not.toContain('💡 How to fix:');
    });

    it('should handle error without how-to-fix suggestion', () => {
      const formatted = formatErrorMessage('default', false);

      expect(formatted).toBe('An unexpected error occurred.');
    });
  });

  describe('ERROR_MESSAGES coverage', () => {
    it('should have all critical error messages defined', () => {
      const criticalErrors = [
        'Parent verse not found',
        'Parent word not found',
        'start >= end',
        'Word overlap not allowed',
        'All ayahs have already been segmented',
      ];

      criticalErrors.forEach((error) => {
        const result = getUserFriendlyError(error);
        expect(result.message).not.toBe(error); // Should be transformed
        expect(result.title).not.toBe('Error'); // Should have specific title
      });
    });

    it('should have how-to-fix for all user-facing errors', () => {
      const userFacingErrors = [
        'Parent verse not found',
        'start >= end',
        'Word overlap not allowed',
        'FFmpeg error',
      ];

      userFacingErrors.forEach((error) => {
        const result = getUserFriendlyError(error);
        expect(result.howToFix).toBeDefined();
        expect(result.howToFix!.length).toBeGreaterThan(0);
      });
    });
  });
});
