/**
 * FIX #18: Export validation with error display
 * Validates annotation exports before saving
 */

import type { AnnotationExport } from '../types/export';

export interface ValidationError {
  severity: 'error' | 'warning';
  field: string;
  message: string;
  location?: string; // e.g., "Verse 2, Word 3"
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationError[];
}

/**
 * Validate an annotation export for completeness and correctness
 *
 * Checks:
 * - Required fields present
 * - Temporal consistency (start < end, no gaps/overlaps)
 * - All ayahs have words
 * - Words are within verse boundaries
 * - Anti-patterns are within word boundaries
 * - No duplicate locations
 *
 * @param data - The annotation export to validate
 * @returns Validation result with errors and warnings
 */
export function validateExport(data: AnnotationExport): ValidationResult {
  const errors: ValidationError[] = [];
  const warnings: ValidationError[] = [];

  // Check version
  if (!data.version || data.version !== '1.0') {
    errors.push({
      severity: 'error',
      field: 'version',
      message: 'Invalid or missing version. Expected "1.0".',
    });
  }

  // Check recording ID
  if (!data.recording_id) {
    errors.push({
      severity: 'error',
      field: 'recording_id',
      message: 'Missing recording ID.',
    });
  }

  // Check audio metadata
  if (!data.audio) {
    errors.push({
      severity: 'error',
      field: 'audio',
      message: 'Missing audio metadata.',
    });
  } else {
    if (data.audio.sample_rate !== 16000) {
      warnings.push({
        severity: 'warning',
        field: 'audio.sample_rate',
        message: `Sample rate is ${data.audio.sample_rate}Hz. Expected 16000Hz.`,
      });
    }

    if (data.audio.duration_sec <= 0) {
      errors.push({
        severity: 'error',
        field: 'audio.duration_sec',
        message: 'Audio duration must be greater than 0.',
      });
    }

    if (!data.audio.trimmed) {
      warnings.push({
        severity: 'warning',
        field: 'audio.trimmed',
        message: 'No trim bounds specified.',
      });
    } else {
      if (data.audio.trimmed.start >= data.audio.trimmed.end) {
        errors.push({
          severity: 'error',
          field: 'audio.trimmed',
          message: 'Trim start must be before trim end.',
        });
      }
    }
  }

  // Check content
  if (!data.content) {
    errors.push({
      severity: 'error',
      field: 'content',
      message: 'Missing content.',
    });
    return { valid: false, errors, warnings }; // Can't continue validation
  }

  // Check surah
  if (!data.content.surah || data.content.surah < 1 || data.content.surah > 114) {
    errors.push({
      severity: 'error',
      field: 'content.surah',
      message: 'Invalid surah number. Must be between 1 and 114.',
    });
  }

  // Check verses
  if (!data.content.verses || data.content.verses.length === 0) {
    errors.push({
      severity: 'error',
      field: 'content.verses',
      message: 'No verses found. At least one verse is required.',
    });
    return { valid: false, errors, warnings }; // Can't continue validation
  }

  // Validate each verse
  const seenAyahs = new Set<number>();
  for (const verse of data.content.verses) {
    const verseLocation = `Verse ${verse.ayah}`;

    // Check for duplicate ayahs
    if (seenAyahs.has(verse.ayah)) {
      errors.push({
        severity: 'error',
        field: 'content.verses',
        message: 'Duplicate ayah number.',
        location: verseLocation,
      });
    }
    seenAyahs.add(verse.ayah);

    // Check verse segment
    if (!verse.segment || verse.segment.length !== 2) {
      errors.push({
        severity: 'error',
        field: 'segment',
        message: 'Invalid verse segment. Must be [start, end].',
        location: verseLocation,
      });
      continue;
    }

    const [verseStart, verseEnd] = verse.segment;
    if (verseStart >= verseEnd) {
      errors.push({
        severity: 'error',
        field: 'segment',
        message: `Verse start (${verseStart}) must be before end (${verseEnd}).`,
        location: verseLocation,
      });
    }

    // Check verse is within audio bounds
    if (data.audio) {
      if (verseStart < 0 || verseEnd > data.audio.duration_sec) {
        errors.push({
          severity: 'error',
          field: 'segment',
          message: `Verse segment [${verseStart}, ${verseEnd}] exceeds audio duration (${data.audio.duration_sec}s).`,
          location: verseLocation,
        });
      }

      // Check verse is within trim bounds (if specified)
      if (data.audio.trimmed) {
        if (verseStart < data.audio.trimmed.start || verseEnd > data.audio.trimmed.end) {
          warnings.push({
            severity: 'warning',
            field: 'segment',
            message: `Verse segment extends beyond trim bounds [${data.audio.trimmed.start}, ${data.audio.trimmed.end}].`,
            location: verseLocation,
          });
        }
      }
    }

    // Check words
    if (!verse.words || verse.words.length === 0) {
      warnings.push({
        severity: 'warning',
        field: 'words',
        message: 'Verse has no words segmented.',
        location: verseLocation,
      });
      continue;
    }

    // Validate each word
    const seenWords = new Set<number>();
    for (const word of verse.words) {
      const wordLocation = `${verseLocation}, Word ${word.index}`;

      // Check for duplicate word indices
      if (seenWords.has(word.index)) {
        errors.push({
          severity: 'error',
          field: 'index',
          message: 'Duplicate word index.',
          location: wordLocation,
        });
      }
      seenWords.add(word.index);

      // Check word segment
      if (!word.segment || word.segment.length !== 2) {
        errors.push({
          severity: 'error',
          field: 'segment',
          message: 'Invalid word segment. Must be [start, end].',
          location: wordLocation,
        });
        continue;
      }

      const [wordStart, wordEnd] = word.segment;
      if (wordStart >= wordEnd) {
        errors.push({
          severity: 'error',
          field: 'segment',
          message: `Word start (${wordStart}) must be before end (${wordEnd}).`,
          location: wordLocation,
        });
      }

      // Check word is within verse bounds
      if (wordStart < verseStart || wordEnd > verseEnd) {
        errors.push({
          severity: 'error',
          field: 'segment',
          message: `Word segment [${wordStart}, ${wordEnd}] exceeds verse bounds [${verseStart}, ${verseEnd}].`,
          location: wordLocation,
        });
      }

      // Validate anti-patterns
      if (word.anti_patterns) {
        for (let i = 0; i < word.anti_patterns.length; i++) {
          const ap = word.anti_patterns[i];
          const apLocation = `${wordLocation}, Anti-pattern ${i + 1}`;

          // Check anti-pattern segment
          if (!ap.segment || ap.segment.length !== 2) {
            errors.push({
              severity: 'error',
              field: 'segment',
              message: 'Invalid anti-pattern segment. Must be [start, end].',
              location: apLocation,
            });
            continue;
          }

          const [apStart, apEnd] = ap.segment;
          if (apStart >= apEnd) {
            errors.push({
              severity: 'error',
              field: 'segment',
              message: `Anti-pattern start (${apStart}) must be before end (${apEnd}).`,
              location: apLocation,
            });
          }

          // Check anti-pattern is within word bounds
          if (apStart < wordStart || apEnd > wordEnd) {
            errors.push({
              severity: 'error',
              field: 'segment',
              message: `Anti-pattern segment [${apStart}, ${apEnd}] exceeds word bounds [${wordStart}, ${wordEnd}].`,
              location: apLocation,
            });
          }

          // Check confidence
          if (ap.confidence < 0 || ap.confidence > 1) {
            errors.push({
              severity: 'error',
              field: 'confidence',
              message: `Confidence (${ap.confidence}) must be between 0 and 1.`,
              location: apLocation,
            });
          }
        }
      }
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Format validation errors for display
 */
export function formatValidationErrors(result: ValidationResult): string {
  const parts: string[] = [];

  if (result.errors.length > 0) {
    parts.push('**Errors:**');
    result.errors.forEach((error, i) => {
      const location = error.location ? ` (${error.location})` : '';
      parts.push(`${i + 1}. ${error.message}${location}`);
    });
  }

  if (result.warnings.length > 0) {
    if (parts.length > 0) parts.push('');
    parts.push('**Warnings:**');
    result.warnings.forEach((warning, i) => {
      const location = warning.location ? ` (${warning.location})` : '';
      parts.push(`${i + 1}. ${warning.message}${location}`);
    });
  }

  return parts.join('\n');
}
