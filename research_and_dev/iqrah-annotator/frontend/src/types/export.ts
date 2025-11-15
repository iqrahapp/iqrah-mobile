// Export schema for hierarchical Tajweed annotations

export interface AnnotationExport {
  version: '1.0';
  recording_id: string;
  created_at: string; // ISO timestamp
  audio: {
    sample_rate: 16000;
    duration_sec: number;
    trimmed: { start: number; end: number };
  };
  content: {
    surah: number;
    verses: Verse[];
  };
}

export interface Verse {
  ayah: number; // 1-indexed
  segment: [number, number]; // [start_sec, end_sec] within trimmed audio
  text: string; // Tajweed HTML
  words: Word[];
}

export interface Word {
  index: number; // 1-indexed within ayah
  location: string; // "surah:ayah:word" (e.g., "1:3:5")
  segment: [number, number]; // [start_sec, end_sec] within verse segment
  text: string; // Tajweed HTML for this word
  anti_patterns: AntiPattern[];
}

export interface AntiPattern {
  type: string; // "weak-ghunnah", "no-qalqalah", etc.
  segment: [number, number]; // [start_sec, end_sec] within word segment
  confidence: number; // 0.0-1.0
  notes?: string;
}

/**
 * Validates that all segments nest correctly in the hierarchy
 * audio.duration ⊇ trimmed ⊇ verses ⊇ words ⊇ anti_patterns
 */
export function validateExport(data: any): data is AnnotationExport {
  if (!data?.content?.verses) return false;
  if (!data?.audio?.trimmed) return false;
  if (data.version !== '1.0') return false;

  const { duration_sec, trimmed } = data.audio;

  // Validate trim bounds
  if (trimmed.start < 0 || trimmed.end > duration_sec || trimmed.start >= trimmed.end) {
    return false;
  }

  // Validate hierarchical nesting
  for (const v of data.content.verses) {
    // Verses must be within trim bounds
    if (v.segment[0] < trimmed.start || v.segment[1] > trimmed.end) {
      return false;
    }
    if (v.segment[0] >= v.segment[1]) return false;

    for (const w of v.words) {
      // Words must be within verse bounds
      if (w.segment[0] < v.segment[0] || w.segment[1] > v.segment[1]) {
        return false;
      }
      if (w.segment[0] >= w.segment[1]) return false;

      for (const ap of w.anti_patterns) {
        // Anti-patterns must be within word bounds
        if (ap.segment[0] < w.segment[0] || ap.segment[1] > w.segment[1]) {
          return false;
        }
        if (ap.segment[0] >= ap.segment[1]) return false;

        // Confidence must be valid
        if (ap.confidence < 0 || ap.confidence > 1) return false;
      }
    }
  }

  return true;
}

/**
 * Get validation errors for debugging
 */
export function getValidationErrors(data: any): string[] {
  const errors: string[] = [];

  if (!data?.content?.verses) {
    errors.push('Missing content.verses');
    return errors;
  }
  if (!data?.audio?.trimmed) {
    errors.push('Missing audio.trimmed');
    return errors;
  }
  if (data.version !== '1.0') {
    errors.push(`Invalid version: ${data.version}`);
  }

  const { duration_sec, trimmed } = data.audio;

  if (trimmed.start < 0) {
    errors.push(`Trim start ${trimmed.start} is negative`);
  }
  if (trimmed.end > duration_sec) {
    errors.push(`Trim end ${trimmed.end} exceeds duration ${duration_sec}`);
  }
  if (trimmed.start >= trimmed.end) {
    errors.push(`Trim start ${trimmed.start} >= end ${trimmed.end}`);
  }

  for (const v of data.content.verses) {
    if (v.segment[0] < trimmed.start) {
      errors.push(`Verse ${v.ayah} start ${v.segment[0]} before trim start ${trimmed.start}`);
    }
    if (v.segment[1] > trimmed.end) {
      errors.push(`Verse ${v.ayah} end ${v.segment[1]} after trim end ${trimmed.end}`);
    }
    if (v.segment[0] >= v.segment[1]) {
      errors.push(`Verse ${v.ayah} invalid segment [${v.segment[0]}, ${v.segment[1]}]`);
    }

    for (const w of v.words) {
      if (w.segment[0] < v.segment[0]) {
        errors.push(`Word ${w.location} start ${w.segment[0]} before verse start ${v.segment[0]}`);
      }
      if (w.segment[1] > v.segment[1]) {
        errors.push(`Word ${w.location} end ${w.segment[1]} after verse end ${v.segment[1]}`);
      }
      if (w.segment[0] >= w.segment[1]) {
        errors.push(`Word ${w.location} invalid segment [${w.segment[0]}, ${w.segment[1]}]`);
      }

      for (const ap of w.anti_patterns) {
        if (ap.segment[0] < w.segment[0]) {
          errors.push(`Anti-pattern in ${w.location} start ${ap.segment[0]} before word start ${w.segment[0]}`);
        }
        if (ap.segment[1] > w.segment[1]) {
          errors.push(`Anti-pattern in ${w.location} end ${ap.segment[1]} after word end ${w.segment[1]}`);
        }
        if (ap.segment[0] >= ap.segment[1]) {
          errors.push(`Anti-pattern in ${w.location} invalid segment [${ap.segment[0]}, ${ap.segment[1]}]`);
        }
        if (ap.confidence < 0 || ap.confidence > 1) {
          errors.push(`Anti-pattern in ${w.location} invalid confidence ${ap.confidence}`);
        }
      }
    }
  }

  return errors;
}
