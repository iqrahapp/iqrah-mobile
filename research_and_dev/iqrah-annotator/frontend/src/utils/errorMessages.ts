/**
 * User-friendly error messages with actionable guidance
 * Replaces technical error messages with helpful explanations
 */

export interface FriendlyError {
  title: string;
  message: string;
  suggestions: string[];
  severity: 'error' | 'warning' | 'info';
}

/**
 * Convert FFmpeg errors to user-friendly messages
 */
export function getFriendlyFFmpegError(error: Error | string): FriendlyError {
  const errorMessage = typeof error === 'string' ? error : error.message;

  // Timeout errors
  if (errorMessage.includes('timeout') || errorMessage.includes('timed out')) {
    return {
      title: 'Audio Processing Timeout',
      message: 'The audio file took too long to process.',
      suggestions: [
        'Try recording a shorter audio segment (under 30 seconds)',
        'Close other browser tabs to free up resources',
        'If the problem persists, try reloading the page',
      ],
      severity: 'error',
    };
  }

  // File too large
  if (errorMessage.includes('too large') || errorMessage.includes('size limit')) {
    return {
      title: 'Audio File Too Large',
      message: 'The audio file exceeds the maximum size limit.',
      suggestions: [
        'Record a shorter segment',
        'Use the trim feature to reduce the audio length',
        'Maximum recommended duration: 2 minutes per ayah',
      ],
      severity: 'error',
    };
  }

  // Invalid format
  if (errorMessage.includes('format') || errorMessage.includes('codec')) {
    return {
      title: 'Unsupported Audio Format',
      message: 'The audio format is not supported.',
      suggestions: [
        'Try recording again using the built-in microphone',
        'Supported formats: WebM, WAV, MP3',
        'If uploading, ensure the file is a valid audio file',
      ],
      severity: 'error',
    };
  }

  // Memory errors
  if (errorMessage.includes('memory') || errorMessage.includes('allocation')) {
    return {
      title: 'Insufficient Memory',
      message: 'Not enough memory to process the audio.',
      suggestions: [
        'Close other browser tabs and applications',
        'Try reloading the page',
        'Record shorter segments (under 1 minute)',
      ],
      severity: 'error',
    };
  }

  // Generic FFmpeg error
  return {
    title: 'Audio Processing Error',
    message: 'Failed to process the audio file.',
    suggestions: [
      'Try recording again',
      'Reload the page and try again',
      'If the problem persists, please report this issue',
    ],
    severity: 'error',
  };
}

/**
 * Convert VAD (Voice Activity Detection) errors to user-friendly messages
 */
export function getFriendlyVADError(error: Error | string): FriendlyError {
  const errorMessage = typeof error === 'string' ? error : error.message;

  // Model loading errors
  if (errorMessage.includes('model') || errorMessage.includes('load')) {
    return {
      title: 'Voice Detection Model Not Loaded',
      message: 'Failed to load the speech detection model.',
      suggestions: [
        'Check your internet connection',
        'Reload the page to retry loading the model',
        'You can still create segments manually by dragging on the waveform',
      ],
      severity: 'warning',
    };
  }

  // No speech detected
  if (errorMessage.includes('no speech') || errorMessage.includes('silence')) {
    return {
      title: 'No Speech Detected',
      message: 'Could not detect speech in the selected region.',
      suggestions: [
        'Try selecting a region that contains speech',
        'Ensure your microphone was working during recording',
        'You can create segments manually by dragging on the waveform',
      ],
      severity: 'info',
    };
  }

  // Timeout
  if (errorMessage.includes('timeout')) {
    return {
      title: 'Speech Detection Timeout',
      message: 'Speech detection took too long to complete.',
      suggestions: [
        'Try again with a shorter audio segment',
        'You can create segments manually by dragging on the waveform',
      ],
      severity: 'warning',
    };
  }

  // Generic VAD error
  return {
    title: 'Speech Detection Error',
    message: 'Failed to automatically detect speech boundaries.',
    suggestions: [
      'You can create segments manually by dragging on the waveform',
      'Try clicking in a different location',
      'Reload the page if the problem persists',
    ],
    severity: 'warning',
  };
}

/**
 * Convert network errors to user-friendly messages
 */
export function getFriendlyNetworkError(error: Error | string, context?: string): FriendlyError {
  const errorMessage = typeof error === 'string' ? error : error.message;

  // Connection refused / Server not running
  if (errorMessage.includes('ECONNREFUSED') || errorMessage.includes('Failed to fetch')) {
    return {
      title: 'Cannot Connect to Server',
      message: 'The annotation server is not responding.',
      suggestions: [
        'Check if the backend server is running',
        'Verify the server URL in settings',
        'Try reloading the page',
      ],
      severity: 'error',
    };
  }

  // Timeout
  if (errorMessage.includes('timeout')) {
    return {
      title: 'Connection Timeout',
      message: 'The server took too long to respond.',
      suggestions: [
        'Check your internet connection',
        'The server might be busy - try again in a moment',
        'Verify the server is running properly',
      ],
      severity: 'error',
    };
  }

  // 404 Not Found
  if (errorMessage.includes('404') || errorMessage.includes('not found')) {
    const resource = context || 'resource';
    return {
      title: 'Resource Not Found',
      message: `The requested ${resource} could not be found.`,
      suggestions: [
        'The resource may have been deleted',
        'Reload the page to refresh the data',
        'If you just created this resource, try waiting a moment',
      ],
      severity: 'error',
    };
  }

  // 500 Server Error
  if (errorMessage.includes('500') || errorMessage.includes('Internal Server Error')) {
    return {
      title: 'Server Error',
      message: 'The server encountered an error while processing your request.',
      suggestions: [
        'Try again in a moment',
        'If the problem persists, check the server logs',
        'Your data is safe - the error occurred on the server',
      ],
      severity: 'error',
    };
  }

  // Generic network error
  return {
    title: 'Network Error',
    message: 'Failed to communicate with the server.',
    suggestions: [
      'Check your internet connection',
      'Verify the server is running',
      'Try reloading the page',
    ],
    severity: 'error',
  };
}

/**
 * Convert file system errors to user-friendly messages
 */
export function getFriendlyFileError(error: Error | string): FriendlyError {
  const errorMessage = typeof error === 'string' ? error : error.message;

  // Permission denied
  if (errorMessage.includes('permission') || errorMessage.includes('denied')) {
    return {
      title: 'Permission Denied',
      message: 'You do not have permission to access this file.',
      suggestions: [
        'Check file permissions',
        'Try saving to a different location',
        'Ensure you have write access to the directory',
      ],
      severity: 'error',
    };
  }

  // Disk full
  if (errorMessage.includes('ENOSPC') || errorMessage.includes('disk') || errorMessage.includes('space')) {
    return {
      title: 'Insufficient Storage',
      message: 'Not enough disk space to save the file.',
      suggestions: [
        'Free up disk space and try again',
        'Delete old recordings you no longer need',
        'Check your available storage',
      ],
      severity: 'error',
    };
  }

  // File not found
  if (errorMessage.includes('ENOENT') || errorMessage.includes('not found')) {
    return {
      title: 'File Not Found',
      message: 'The requested file could not be found.',
      suggestions: [
        'The file may have been moved or deleted',
        'Try reloading the page',
        'Re-record the audio if necessary',
      ],
      severity: 'error',
    };
  }

  // Generic file error
  return {
    title: 'File Error',
    message: 'An error occurred while accessing the file.',
    suggestions: [
      'Try again',
      'Reload the page',
      'Check file permissions and disk space',
    ],
    severity: 'error',
  };
}

/**
 * Convert validation errors to user-friendly messages
 */
export function getFriendlyValidationError(errors: string[]): FriendlyError {
  if (errors.length === 0) {
    return {
      title: 'Validation Passed',
      message: 'No validation errors found.',
      suggestions: [],
      severity: 'info',
    };
  }

  const firstError = errors[0];

  // Missing segments
  if (firstError.includes('segment')) {
    return {
      title: 'Incomplete Segmentation',
      message: 'Some segments are missing or invalid.',
      suggestions: [
        'Ensure all ayahs are segmented',
        'Ensure all words within each ayah are segmented',
        'Check for overlapping or invalid segment boundaries',
      ],
      severity: 'error',
    };
  }

  // Missing audio
  if (firstError.includes('audio') || firstError.includes('recording')) {
    return {
      title: 'Missing Audio',
      message: 'No audio recording found.',
      suggestions: [
        'Record audio in Stage 1 (Record & Trim)',
        'Ensure the audio was properly saved',
        'Try re-recording if the audio is missing',
      ],
      severity: 'error',
    };
  }

  // Invalid timestamps
  if (firstError.includes('time') || firstError.includes('duration')) {
    return {
      title: 'Invalid Timestamps',
      message: 'Some segment timestamps are invalid.',
      suggestions: [
        'Ensure all segments have valid start and end times',
        'Segment end time must be greater than start time',
        'Segments must be within the audio duration',
      ],
      severity: 'error',
    };
  }

  // Generic validation error
  return {
    title: 'Validation Error',
    message: `${errors.length} validation error${errors.length > 1 ? 's' : ''} found.`,
    suggestions: [
      ...errors.slice(0, 3).map(e => `• ${e}`),
      errors.length > 3 ? `... and ${errors.length - 3} more` : '',
    ].filter(Boolean),
    severity: 'error',
  };
}

/**
 * Format a friendly error for display in a toast or alert
 */
export function formatFriendlyError(friendlyError: FriendlyError, includeTitle: boolean = true): string {
  const parts: string[] = [];

  if (includeTitle) {
    parts.push(`**${friendlyError.title}**`);
  }

  parts.push(friendlyError.message);

  if (friendlyError.suggestions.length > 0) {
    parts.push('');
    parts.push('**Suggestions:**');
    friendlyError.suggestions.forEach(suggestion => {
      parts.push(`• ${suggestion}`);
    });
  }

  return parts.join('\n');
}

/**
 * Create a toast-friendly error message (single line with primary suggestion)
 */
export function formatCompactError(friendlyError: FriendlyError): string {
  const suggestion = friendlyError.suggestions[0];
  return suggestion
    ? `${friendlyError.message} ${suggestion}`
    : friendlyError.message;
}
