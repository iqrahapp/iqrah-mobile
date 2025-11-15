/**
 * FIX #11: Auto-save hook with session recovery
 * Automatically saves state to localStorage and detects unsaved changes
 */

import { useEffect, useRef, useState, useCallback } from 'react';

export interface UseAutoSaveOptions<T> {
  /** Unique key for localStorage */
  storageKey: string;
  /** Data to save */
  data: T;
  /** Auto-save interval in milliseconds (default: 30000 = 30s) */
  intervalMs?: number;
  /** Whether auto-save is enabled */
  enabled?: boolean;
  /** Callback when data is saved */
  onSave?: (data: T) => void;
  /** Callback when data is loaded from storage */
  onLoad?: (data: T) => void;
  /** Custom serializer (default: JSON.stringify) */
  serialize?: (data: T) => string;
  /** Custom deserializer (default: JSON.parse) */
  deserialize?: (str: string) => T;
}

export interface UseAutoSaveResult<T> {
  /** Whether there are unsaved changes */
  hasUnsavedChanges: boolean;
  /** Manually trigger a save */
  save: () => void;
  /** Load data from storage */
  load: () => T | null;
  /** Clear saved data from storage */
  clear: () => void;
  /** Last save timestamp */
  lastSaveTime: Date | null;
}

/**
 * Hook for auto-saving data to localStorage with session recovery
 *
 * Features:
 * - Automatic periodic saves
 * - Detects unsaved changes on page unload
 * - Session recovery on reload
 * - Manual save/load/clear
 * - Custom serialization
 *
 * @param options - Configuration for auto-save
 * @returns Auto-save utilities
 *
 * @example
 * ```ts
 * const { hasUnsavedChanges, save, load } = useAutoSave({
 *   storageKey: 'wizard-annotations',
 *   data: wizardState,
 *   intervalMs: 30000, // Save every 30s
 *   onSave: () => console.log('Saved!'),
 * });
 *
 * // Check for saved session on mount
 * useEffect(() => {
 *   const savedData = load();
 *   if (savedData && confirm('Resume previous session?')) {
 *     restoreState(savedData);
 *   }
 * }, []);
 * ```
 */
export function useAutoSave<T>({
  storageKey,
  data,
  intervalMs = 30000,
  enabled = true,
  onSave,
  onLoad,
  serialize = JSON.stringify,
  deserialize = JSON.parse,
}: UseAutoSaveOptions<T>): UseAutoSaveResult<T> {
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [lastSaveTime, setLastSaveTime] = useState<Date | null>(null);
  const lastSavedDataRef = useRef<string | null>(null);
  const saveIntervalRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * Save data to localStorage
   */
  const save = useCallback(() => {
    try {
      const serialized = serialize(data);
      localStorage.setItem(storageKey, serialized);
      localStorage.setItem(`${storageKey}:timestamp`, new Date().toISOString());

      lastSavedDataRef.current = serialized;
      setHasUnsavedChanges(false);
      setLastSaveTime(new Date());

      console.log('[AutoSave] Saved to', storageKey, 'at', new Date().toISOString());
      onSave?.(data);
    } catch (error) {
      console.error('[AutoSave] Failed to save:', error);
    }
  }, [data, storageKey, serialize, onSave]);

  /**
   * Load data from localStorage
   */
  const load = useCallback((): T | null => {
    try {
      const saved = localStorage.getItem(storageKey);
      if (!saved) return null;

      const deserialized = deserialize(saved);
      console.log('[AutoSave] Loaded from', storageKey);
      onLoad?.(deserialized);
      return deserialized;
    } catch (error) {
      console.error('[AutoSave] Failed to load:', error);
      return null;
    }
  }, [storageKey, deserialize, onLoad]);

  /**
   * Clear saved data from localStorage
   */
  const clear = useCallback(() => {
    localStorage.removeItem(storageKey);
    localStorage.removeItem(`${storageKey}:timestamp`);
    lastSavedDataRef.current = null;
    setHasUnsavedChanges(false);
    setLastSaveTime(null);
    console.log('[AutoSave] Cleared', storageKey);
  }, [storageKey]);

  /**
   * Detect changes by comparing current data with last saved
   */
  useEffect(() => {
    if (!enabled) return;

    const currentSerialized = serialize(data);
    const changed = lastSavedDataRef.current !== null &&
                    currentSerialized !== lastSavedDataRef.current;

    setHasUnsavedChanges(changed);
  }, [data, serialize, enabled]);

  /**
   * Auto-save on interval
   */
  useEffect(() => {
    if (!enabled) return;

    // Initial save (only if no existing data in storage)
    if (lastSavedDataRef.current === null) {
      const existing = localStorage.getItem(storageKey);
      if (!existing) {
        save();
      } else {
        // Mark existing data as "last saved" to avoid overwriting
        lastSavedDataRef.current = existing;
      }
    }

    // Set up interval
    saveIntervalRef.current = setInterval(() => {
      if (hasUnsavedChanges) {
        save();
      }
    }, intervalMs);

    return () => {
      if (saveIntervalRef.current) {
        clearInterval(saveIntervalRef.current);
      }
    };
  }, [enabled, intervalMs, hasUnsavedChanges, save, storageKey]);

  /**
   * Warn on page unload if there are unsaved changes
   */
  useEffect(() => {
    if (!enabled) return;

    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (hasUnsavedChanges) {
        e.preventDefault();
        e.returnValue = 'You have unsaved changes. Are you sure you want to leave?';
        // Save before leaving
        save();
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, [enabled, hasUnsavedChanges, save]);

  /**
   * Save on visibility change (user switches tabs)
   */
  useEffect(() => {
    if (!enabled) return;

    const handleVisibilityChange = () => {
      if (document.hidden && hasUnsavedChanges) {
        console.log('[AutoSave] Tab hidden, saving...');
        save();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, [enabled, hasUnsavedChanges, save]);

  return {
    hasUnsavedChanges,
    save,
    load,
    clear,
    lastSaveTime,
  };
}

/**
 * Get the timestamp of the last save
 */
export function getLastSaveTimestamp(storageKey: string): Date | null {
  const timestamp = localStorage.getItem(`${storageKey}:timestamp`);
  return timestamp ? new Date(timestamp) : null;
}

/**
 * Check if there's a saved session available
 */
export function hasSavedSession(storageKey: string): boolean {
  return localStorage.getItem(storageKey) !== null;
}
