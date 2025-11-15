/**
 * Hook for restoring annotations to the visual annotation manager
 * Handles coordinate conversion and prevents callback loops
 */

import { useEffect, useRef } from 'react';
import type { AnnotationManager } from '../annotation/manager';
import type { AnnotationKind } from '../annotation/types';

interface UseAnnotationRestorationOptions<T extends { id: string; start: number; end: number }> {
  /** Annotation manager instance */
  manager: AnnotationManager | null;
  /** Items to restore (from store/state) */
  items: T[];
  /** Time offset for coordinate conversion (absolute to relative) */
  timeOffset: number;
  /** Annotation kind to restore */
  kind: AnnotationKind;
  /** Function to extract label from item */
  getLabelFn: (item: T) => string;
  /** Trigger restoration when audioUrl changes (ensures manager is ready) */
  audioUrl: string | null;
  /** Additional dependencies that should trigger restoration */
  additionalDeps?: any[];
}

/**
 * Custom hook for restoring annotations visually after audio segment loads
 *
 * Features:
 * - Prevents onCreate callback during restoration (using ref flag)
 * - Handles coordinate conversion from absolute to relative times
 * - Clears existing annotations before restoring to avoid duplicates
 * - Delays restoration to ensure WaveSurfer is fully initialized
 *
 * @param options - Configuration for annotation restoration
 * @returns Ref flag indicating if restoration is in progress
 */
export function useAnnotationRestoration<T extends { id: string; start: number; end: number }>(
  options: UseAnnotationRestorationOptions<T>
): React.MutableRefObject<boolean> {
  const {
    manager,
    items,
    timeOffset,
    kind,
    getLabelFn,
    audioUrl,
    additionalDeps = [],
  } = options;

  const isRestoringRef = useRef(false);

  useEffect(() => {
    // Wait for audio to load AND manager to be ready
    // Note: timeOffset can legitimately be 0 for first verse, so only check for undefined/null
    if (!audioUrl || !manager || timeOffset === undefined || timeOffset === null) {
      console.log('[useAnnotationRestoration] Restore skipped - not ready:', {
        hasAudio: !!audioUrl,
        hasManager: !!manager,
        timeOffset,
      });
      return;
    }

    // Small delay to ensure WaveSurfer is fully initialized
    const timer = setTimeout(() => {
      if (!manager) return;

      console.log(`[useAnnotationRestoration] Restoring ${items.length} ${kind} annotations`);

      // Set flag to prevent onCreate callback during restoration
      isRestoringRef.current = true;

      // Clear existing annotations of this kind from manager first
      const existingAnnotations = manager.queryByKind(kind);
      existingAnnotations.forEach(ann => {
        manager.removeAnnotation(ann.id);
      });

      // Restore each item as visual annotation (convert absolute to relative coordinates)
      items.forEach(item => {
        const relativeStart = item.start - timeOffset;
        const relativeEnd = item.end - timeOffset;
        const label = getLabelFn(item);

        console.log('[useAnnotationRestoration] Restoring:', {
          id: item.id,
          absolute: [item.start, item.end],
          relative: [relativeStart, relativeEnd],
          timeOffset,
          label,
        });

        manager.restoreAnnotation(item.id, relativeStart, relativeEnd, kind, {
          label,
        });
      });

      // Clear flag after all restorations complete
      isRestoringRef.current = false;
      console.log('[useAnnotationRestoration] Restoration complete');
    }, 100); // Small delay for initialization

    return () => clearTimeout(timer);
  }, [manager, audioUrl, timeOffset, items.length, kind, ...additionalDeps]);

  return isRestoringRef;
}
