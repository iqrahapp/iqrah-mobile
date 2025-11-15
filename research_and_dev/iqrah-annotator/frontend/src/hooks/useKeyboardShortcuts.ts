/**
 * FIX #17: Comprehensive keyboard shortcuts system
 * Provides configurable keyboard shortcuts for the annotation tool
 */

import { useEffect, useCallback, useRef } from 'react';

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean; // Command key on Mac
  description: string;
  action: () => void;
  preventDefault?: boolean;
  enabled?: boolean;
}

export interface UseKeyboardShortcutsOptions {
  /** List of keyboard shortcuts */
  shortcuts: KeyboardShortcut[];
  /** Whether shortcuts are enabled globally */
  enabled?: boolean;
  /** Whether to log shortcut activations (for debugging) */
  debug?: boolean;
}

/**
 * Hook for managing keyboard shortcuts
 *
 * Features:
 * - Configurable shortcuts with modifiers (Ctrl, Shift, Alt, Meta)
 * - Automatic preventDefault when specified
 * - Ignores shortcuts when typing in input fields
 * - Supports enabling/disabling shortcuts
 * - Debug mode for logging activations
 *
 * @param options - Configuration for keyboard shortcuts
 *
 * @example
 * ```ts
 * useKeyboardShortcuts({
 *   shortcuts: [
 *     {
 *       key: 'z',
 *       ctrl: true,
 *       description: 'Undo',
 *       action: () => undo(),
 *       preventDefault: true,
 *     },
 *     {
 *       key: 'Delete',
 *       description: 'Delete selected annotation',
 *       action: () => deleteSelected(),
 *     },
 *   ],
 * });
 * ```
 */
export function useKeyboardShortcuts({
  shortcuts,
  enabled = true,
  debug = false,
}: UseKeyboardShortcutsOptions): void {
  const shortcutsRef = useRef(shortcuts);

  // Update ref when shortcuts change (avoid stale closures)
  useEffect(() => {
    shortcutsRef.current = shortcuts;
  }, [shortcuts]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!enabled) return;

      // Ignore shortcuts when typing in input fields
      const target = e.target as HTMLElement;
      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target.isContentEditable ||
        target.contentEditable === 'true'
      ) {
        return;
      }

      // Find matching shortcut
      const matchingShortcut = shortcutsRef.current.find(shortcut => {
        if (shortcut.enabled === false) return false;

        const keyMatches = e.key.toLowerCase() === shortcut.key.toLowerCase() ||
                          e.code.toLowerCase() === shortcut.key.toLowerCase();
        const ctrlMatches = shortcut.ctrl ? (e.ctrlKey || e.metaKey) : !e.ctrlKey && !e.metaKey;
        const shiftMatches = shortcut.shift ? e.shiftKey : !e.shiftKey;
        const altMatches = shortcut.alt ? e.altKey : !e.altKey;
        const metaMatches = shortcut.meta ? e.metaKey : true; // Meta is optional

        return keyMatches && ctrlMatches && shiftMatches && altMatches && metaMatches;
      });

      if (matchingShortcut) {
        if (debug) {
          console.log('[KeyboardShortcuts] Activated:', matchingShortcut.description);
        }

        if (matchingShortcut.preventDefault !== false) {
          e.preventDefault();
        }

        matchingShortcut.action();
      }
    },
    [enabled, debug]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}

/**
 * Predefined shortcuts for common actions
 */
export const COMMON_SHORTCUTS = {
  undo: (action: () => void): KeyboardShortcut => ({
    key: 'z',
    ctrl: true,
    description: 'Undo',
    action,
    preventDefault: true,
  }),

  redo: (action: () => void): KeyboardShortcut => ({
    key: 'z',
    ctrl: true,
    shift: true,
    description: 'Redo',
    action,
    preventDefault: true,
  }),

  delete: (action: () => void): KeyboardShortcut => ({
    key: 'Delete',
    description: 'Delete selected item',
    action,
    preventDefault: true,
  }),

  save: (action: () => void): KeyboardShortcut => ({
    key: 's',
    ctrl: true,
    description: 'Save',
    action,
    preventDefault: true,
  }),

  playPause: (action: () => void): KeyboardShortcut => ({
    key: 'Space',
    description: 'Play/Pause',
    action,
    preventDefault: true,
  }),

  escape: (action: () => void): KeyboardShortcut => ({
    key: 'Escape',
    description: 'Cancel/Close',
    action,
    preventDefault: false,
  }),

  help: (action: () => void): KeyboardShortcut => ({
    key: '?',
    description: 'Show keyboard shortcuts',
    action,
    preventDefault: true,
  }),

  stageNavigation: (stageNumber: number, action: () => void): KeyboardShortcut => ({
    key: String(stageNumber),
    ctrl: true,
    description: `Jump to Stage ${stageNumber}`,
    action,
    preventDefault: true,
  }),
};

/**
 * Format a shortcut for display
 * @param shortcut - The keyboard shortcut to format
 * @returns Human-readable string representation
 */
export function formatShortcut(shortcut: KeyboardShortcut): string {
  const parts: string[] = [];

  if (shortcut.ctrl) parts.push('Ctrl');
  if (shortcut.shift) parts.push('Shift');
  if (shortcut.alt) parts.push('Alt');
  if (shortcut.meta) parts.push('Cmd');

  // Format key name
  let keyName = shortcut.key;
  if (keyName === ' ') {
    keyName = 'Space';
  } else if (keyName.length === 1) {
    keyName = keyName.toUpperCase();
  } else {
    // Uppercase special keys like Delete, Enter, Escape, etc.
    const specialKeys = ['Delete', 'Enter', 'Escape', 'Tab', 'Backspace', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'];
    if (specialKeys.includes(keyName)) {
      keyName = keyName.toUpperCase();
    }
  }

  parts.push(keyName);

  return parts.join(' + ');
}
