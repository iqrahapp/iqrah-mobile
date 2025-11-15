/**
 * Hook for handling undo/redo keyboard shortcuts
 * Works with Zustand temporal middleware
 */

import { useEffect } from 'react';
import { useStore } from 'zustand';

interface TemporalState<T> {
  pastStates: T[];
  futureStates: T[];
  undo: () => void;
  redo: () => void;
}

interface UseUndoRedoOptions<T> {
  /** Zustand store with temporal middleware */
  store: any;
  /** Whether undo/redo is enabled (default: true) */
  enabled?: boolean;
}

interface UseUndoRedoResult {
  /** Whether undo is available */
  canUndo: boolean;
  /** Whether redo is available */
  canRedo: boolean;
  /** Manually trigger undo */
  undo: () => void;
  /** Manually trigger redo */
  redo: () => void;
}

/**
 * Custom hook for undo/redo keyboard shortcuts (Ctrl/Cmd+Z, Ctrl/Cmd+Shift+Z)
 *
 * Features:
 * - Cross-platform support (Ctrl on Windows/Linux, Cmd on Mac)
 * - Ignores shortcuts when typing in input fields
 * - Provides manual undo/redo functions for programmatic use
 * - Returns availability status for UI indicators
 *
 * @param options - Configuration for undo/redo behavior
 * @returns Undo/redo state and functions
 */
export function useUndoRedo<T = any>(
  options: UseUndoRedoOptions<T>
): UseUndoRedoResult {
  const { store, enabled = true } = options;

  // Subscribe to temporal state for undo/redo availability
  const canUndo = useStore(
    store.temporal,
    (state: TemporalState<T>) => state.pastStates.length > 0
  );
  const canRedo = useStore(
    store.temporal,
    (state: TemporalState<T>) => state.futureStates.length > 0
  );

  useEffect(() => {
    if (!enabled) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Only handle Ctrl/Cmd+Z
      if (!(e.ctrlKey || e.metaKey) || !['z', 'Z'].includes(e.key)) {
        return;
      }

      // Ignore if typing in input field
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      e.preventDefault();

      const temporal = store.temporal.getState();

      if (e.shiftKey && canRedo) {
        // Ctrl/Cmd+Shift+Z = Redo
        console.log('[useUndoRedo] Redo triggered');
        temporal.redo();
      } else if (!e.shiftKey && canUndo) {
        // Ctrl/Cmd+Z = Undo
        console.log('[useUndoRedo] Undo triggered');
        temporal.undo();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [enabled, canUndo, canRedo, store]);

  return {
    canUndo,
    canRedo,
    undo: () => store.temporal.getState().undo(),
    redo: () => store.temporal.getState().redo(),
  };
}
