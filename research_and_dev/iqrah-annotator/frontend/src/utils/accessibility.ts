/**
 * FIX #22: Accessibility helpers for screen readers and keyboard navigation
 */

/**
 * Announce a message to screen readers
 * @param message - The message to announce
 * @param priority - 'polite' (default) or 'assertive'
 */
export function announceToScreenReader(
  message: string,
  priority: 'polite' | 'assertive' = 'polite'
): void {
  // Create or get existing live region
  let liveRegion = document.getElementById('sr-live-region');

  if (!liveRegion) {
    liveRegion = document.createElement('div');
    liveRegion.id = 'sr-live-region';
    liveRegion.setAttribute('aria-live', priority);
    liveRegion.setAttribute('aria-atomic', 'true');
    liveRegion.style.position = 'absolute';
    liveRegion.style.left = '-10000px';
    liveRegion.style.width = '1px';
    liveRegion.style.height = '1px';
    liveRegion.style.overflow = 'hidden';
    document.body.appendChild(liveRegion);
  }

  // Update the message
  liveRegion.textContent = message;

  // Clear after a delay to allow re-announcing the same message
  setTimeout(() => {
    if (liveRegion) liveRegion.textContent = '';
  }, 1000);
}

/**
 * ARIA label helpers for common UI patterns
 */
export const ariaLabels = {
  waveform: {
    container: 'Audio waveform visualization',
    playButton: 'Play audio',
    pauseButton: 'Pause audio',
    stopButton: 'Stop audio',
    seekForward: 'Skip forward 100 milliseconds',
    seekBackward: 'Skip backward 100 milliseconds',
    zoomIn: 'Zoom in waveform',
    zoomOut: 'Zoom out waveform',
    createAnnotation: 'Create annotation at playhead position',
  },
  annotation: {
    region: (kind: string, start: number, end: number) =>
      `${kind} annotation from ${start.toFixed(2)} to ${end.toFixed(2)} seconds`,
    deleteButton: (kind: string) => `Delete ${kind} annotation`,
    editButton: (kind: string) => `Edit ${kind} annotation`,
  },
  wizard: {
    step: (currentStep: number, totalSteps: number) =>
      `Step ${currentStep} of ${totalSteps}`,
    nextButton: 'Proceed to next step',
    prevButton: 'Return to previous step',
    progress: (completed: number, total: number) =>
      `${completed} of ${total} items completed`,
  },
  ayah: {
    chip: (ayahNumber: number, status: 'pending' | 'completed' | 'selected') => {
      const statusText = status === 'completed' ? 'completed' :
                        status === 'selected' ? 'currently selected' :
                        'not yet segmented';
      return `Ayah ${ayahNumber}, ${statusText}`;
    },
    selector: 'Select ayah to segment',
  },
  word: {
    chip: (wordNumber: number, text: string, status: 'pending' | 'completed' | 'selected') => {
      const statusText = status === 'completed' ? 'completed' :
                        status === 'selected' ? 'currently selected' :
                        'not yet segmented';
      return `Word ${wordNumber}: ${text}, ${statusText}`;
    },
    selector: 'Select word to segment',
  },
};

/**
 * Focus management utilities
 */
export const focusManagement = {
  /**
   * Trap focus within a container (useful for modals)
   */
  trapFocus(container: HTMLElement): () => void {
    const focusableElements = container.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        // Shift+Tab: focus last element if at first
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement?.focus();
        }
      } else {
        // Tab: focus first element if at last
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement?.focus();
        }
      }
    };

    container.addEventListener('keydown', handleKeyDown);

    // Return cleanup function
    return () => container.removeEventListener('keydown', handleKeyDown);
  },

  /**
   * Save and restore focus (useful when temporarily disabling/re-enabling)
   */
  saveFocus(): () => void {
    const previousActiveElement = document.activeElement as HTMLElement;

    return () => {
      if (previousActiveElement && previousActiveElement.focus) {
        previousActiveElement.focus();
      }
    };
  },

  /**
   * Focus first error in a form
   */
  focusFirstError(container: HTMLElement): boolean {
    const firstError = container.querySelector<HTMLElement>(
      '[aria-invalid="true"], .error'
    );

    if (firstError) {
      firstError.focus();
      return true;
    }

    return false;
  },
};

/**
 * Keyboard navigation helpers
 */
export const keyboardNav = {
  /**
   * Check if an event should be ignored (user is typing)
   */
  isTyping(event: KeyboardEvent): boolean {
    const target = event.target as HTMLElement;
    return (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target.isContentEditable
    );
  },

  /**
   * Create arrow key navigation for a list
   */
  createListNav(
    items: HTMLElement[],
    options: {
      loop?: boolean; // Loop from end to start
      onSelect?: (item: HTMLElement, index: number) => void;
    } = {}
  ): (e: KeyboardEvent) => void {
    const { loop = true, onSelect } = options;

    return (e: KeyboardEvent) => {
      if (!['ArrowUp', 'ArrowDown', 'Home', 'End', 'Enter'].includes(e.key)) {
        return;
      }

      e.preventDefault();

      const currentIndex = items.findIndex(item => item === document.activeElement);
      let nextIndex = currentIndex;

      switch (e.key) {
        case 'ArrowDown':
          nextIndex = currentIndex + 1;
          if (nextIndex >= items.length) {
            nextIndex = loop ? 0 : items.length - 1;
          }
          break;

        case 'ArrowUp':
          nextIndex = currentIndex - 1;
          if (nextIndex < 0) {
            nextIndex = loop ? items.length - 1 : 0;
          }
          break;

        case 'Home':
          nextIndex = 0;
          break;

        case 'End':
          nextIndex = items.length - 1;
          break;

        case 'Enter':
          if (currentIndex >= 0 && onSelect) {
            onSelect(items[currentIndex], currentIndex);
          }
          return;
      }

      if (items[nextIndex]) {
        items[nextIndex].focus();
      }
    };
  },
};

/**
 * Screen reader only text (visually hidden but announced)
 */
export function srOnly(text: string): string {
  return `<span class="sr-only">${text}</span>`;
}

/**
 * Generate appropriate ARIA attributes for an annotation region
 */
export function getAnnotationAriaProps(annotation: {
  kind: string;
  start: number;
  end: number;
  meta?: { label?: string };
}): {
  role: string;
  'aria-label': string;
  tabIndex: number;
} {
  const label = annotation.meta?.label || annotation.kind;
  const duration = (annotation.end - annotation.start).toFixed(2);

  return {
    role: 'button',
    'aria-label': `${label} annotation from ${annotation.start.toFixed(2)} to ${annotation.end.toFixed(2)} seconds, duration ${duration} seconds. Press Enter to edit, Delete to remove.`,
    tabIndex: 0,
  };
}
