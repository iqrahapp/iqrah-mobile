/**
 * FIX #7: User-friendly error messages with contextual help
 * Maps technical error messages to user-friendly explanations
 */

export interface ErrorMessage {
  title: string;
  message: string;
  howToFix?: string;
}

export const ERROR_MESSAGES: Record<string, ErrorMessage> = {
  // Validation errors
  'Validation failed: start >= end': {
    title: 'Invalid Segment',
    message: 'The start time must be before the end time.',
    howToFix: 'Drag the end handle to the right of the start handle.',
  },
  'start >= end': {
    title: 'Invalid Time Range',
    message: 'Start time must be before end time.',
    howToFix: 'Adjust the segment boundaries so start comes before end.',
  },
  'Parent verse not found': {
    title: 'Missing Parent Verse',
    message: 'Please segment the ayah before adding words.',
    howToFix: 'Go to Stage 2 (Verse Segmentation) and create a segment for this ayah first.',
  },
  'Parent word not found': {
    title: 'Missing Parent Word',
    message: 'Please segment the word before adding anti-patterns.',
    howToFix: 'Go to Stage 3 (Word Segmentation) and create a segment for this word first.',
  },

  // Audio processing errors
  'FFmpeg error': {
    title: 'Audio Processing Failed',
    message: 'Could not process the audio. Using original audio instead.',
    howToFix: 'Try uploading a different audio format (WAV or WebM recommended).',
  },
  'Failed to create audio segment': {
    title: 'Audio Extraction Failed',
    message: 'Could not extract the audio segment.',
    howToFix: 'Check that the audio file is valid and try again.',
  },
  'Audio processing failed': {
    title: 'Processing Error',
    message: 'The audio could not be processed within the time limit.',
    howToFix: 'Try with a shorter audio segment or check your internet connection.',
  },

  // Overlap errors
  'Word overlap not allowed': {
    title: 'Word Overlap Detected',
    message: 'Words cannot overlap unless they have merging tajweed rules (idgham, ikhfa).',
    howToFix: 'Adjust the word boundaries to remove the overlap, or verify that one word has a merging rule.',
  },
  'Overlap exceeds': {
    title: 'Overlap Too Large',
    message: 'The overlap between words exceeds the maximum allowed (150ms for merging rules).',
    howToFix: 'Reduce the overlap by adjusting the word boundaries.',
  },

  // Completion errors
  'All ayahs have already been segmented': {
    title: 'All Ayahs Complete',
    message: 'You have already segmented all selected ayahs.',
    howToFix: 'Proceed to the next stage or delete an existing segment to re-annotate.',
  },
  'All words for this ayah have been segmented': {
    title: 'All Words Complete',
    message: 'You have segmented all words in this ayah.',
    howToFix: 'Move to the next ayah or proceed to Stage 4 (Anti-patterns).',
  },

  // QPC errors
  'Failed to load words': {
    title: 'Database Error',
    message: 'Could not load words from the Quran database.',
    howToFix: 'Check your internet connection and refresh the page.',
  },
  'No words found': {
    title: 'No Words Found',
    message: 'No words found for this ayah in the database.',
    howToFix: 'Verify the surah and ayah numbers are correct.',
  },

  // Generic fallback
  'default': {
    title: 'Error',
    message: 'An unexpected error occurred.',
    howToFix: 'Please try again or contact support if the issue persists.',
  },
};

/**
 * Get a user-friendly error message from a technical error string
 * @param technicalError - The technical error message
 * @returns User-friendly error message object
 */
export function getUserFriendlyError(technicalError: string): ErrorMessage {
  // Check for exact match
  if (ERROR_MESSAGES[technicalError]) {
    return ERROR_MESSAGES[technicalError];
  }

  // Check for partial match (e.g., "Overlap exceeds 150ms" matches "Overlap exceeds")
  const partialMatch = Object.keys(ERROR_MESSAGES).find(key =>
    technicalError.includes(key) || key.includes(technicalError)
  );

  if (partialMatch) {
    return ERROR_MESSAGES[partialMatch];
  }

  // Return default error
  return {
    title: 'Error',
    message: technicalError,
    howToFix: 'Please try again or contact support if the issue persists.',
  };
}

/**
 * Format error for display (with optional "how to fix" section)
 * @param error - Technical error message
 * @param includeHowToFix - Whether to include the "how to fix" section
 * @returns Formatted error string
 */
export function formatErrorMessage(error: string, includeHowToFix = true): string {
  const friendly = getUserFriendlyError(error);
  let message = friendly.message;

  if (includeHowToFix && friendly.howToFix) {
    message += `\n\n💡 How to fix: ${friendly.howToFix}`;
  }

  return message;
}
