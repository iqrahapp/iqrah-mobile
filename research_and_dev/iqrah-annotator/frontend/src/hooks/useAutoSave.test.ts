import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useAutoSave, hasSavedSession, getLastSaveTimestamp } from './useAutoSave';

describe('useAutoSave', () => {
  const STORAGE_KEY = 'test-autosave';

  beforeEach(() => {
    localStorage.clear();
    vi.useFakeTimers();
  });

  afterEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  describe('basic save functionality', () => {
    it('should save data to localStorage', async () => {
      const testData = { name: 'Test', value: 42 };

      const { result } = renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: testData,
          intervalMs: 30000,
        })
      );

      // Flush pending timers (just enough to trigger initial save, not the interval)
      act(() => {
        vi.advanceTimersByTime(0);
      });

      // Check that data was saved
      const saved = localStorage.getItem(STORAGE_KEY);
      expect(saved).not.toBeNull();
      expect(JSON.parse(saved!)).toEqual(testData);
    });

    it('should load data from localStorage', () => {
      const testData = { name: 'Test', value: 42 };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(testData));

      const { result } = renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: {},
          intervalMs: 30000,
        })
      );

      const loaded = result.current.load();
      expect(loaded).toEqual(testData);
    });

    it('should clear saved data', async () => {
      const testData = { name: 'Test' };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(testData));

      const { result } = renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: testData,
          intervalMs: 30000,
        })
      );

      act(() => {
        result.current.clear();
      });

      expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
    });
  });

  describe('auto-save interval', () => {
    it('should auto-save at specified interval', async () => {
      const testData = { value: 1 };

      const { result, rerender } = renderHook(
        ({ data }) =>
          useAutoSave({
            storageKey: STORAGE_KEY,
            data,
            intervalMs: 1000,
          }),
        { initialProps: { data: testData } }
      );

      // Initial save happens immediately
      act(() => {
        vi.advanceTimersByTime(0);
      });
      expect(result.current.hasUnsavedChanges).toBe(false);

      // Change data
      const newData = { value: 2 };
      act(() => {
        rerender({ data: newData });
      });

      // Should detect unsaved changes
      expect(result.current.hasUnsavedChanges).toBe(true);

      // Fast-forward time to trigger auto-save
      act(() => {
        vi.advanceTimersByTime(1000);
      });

      // Should have saved
      expect(result.current.hasUnsavedChanges).toBe(false);

      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
      expect(saved).toEqual(newData);
    });

    it('should not auto-save if no changes', async () => {
      const testData = { value: 1 };
      const onSave = vi.fn();

      renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: testData,
          intervalMs: 1000,
          onSave,
        })
      );

      // Trigger initial save
      act(() => {
        vi.advanceTimersByTime(0);
      });
      expect(onSave).toHaveBeenCalledTimes(1);

      // Fast-forward without changes
      act(() => {
        vi.advanceTimersByTime(1000);
      });

      // Should not have saved again
      expect(onSave).toHaveBeenCalledTimes(1);
    });
  });

  describe('unsaved changes detection', () => {
    it('should detect unsaved changes', async () => {
      const initialData = { value: 1 };

      const { result, rerender } = renderHook(
        ({ data }) =>
          useAutoSave({
            storageKey: STORAGE_KEY,
            data,
            intervalMs: 30000,
          }),
        { initialProps: { data: initialData } }
      );

      // Trigger initial save
      act(() => {
        vi.advanceTimersByTime(0);
      });
      expect(result.current.hasUnsavedChanges).toBe(false);

      // Change data
      act(() => {
        rerender({ data: { value: 2 } });
      });

      // Should detect changes
      expect(result.current.hasUnsavedChanges).toBe(true);
    });

    it('should clear unsaved changes after manual save', async () => {
      const testData = { value: 1 };

      const { result, rerender } = renderHook(
        ({ data }) =>
          useAutoSave({
            storageKey: STORAGE_KEY,
            data,
            intervalMs: 30000,
          }),
        { initialProps: { data: testData } }
      );

      // Trigger initial save
      act(() => {
        vi.advanceTimersByTime(0);
      });
      expect(result.current.hasUnsavedChanges).toBe(false);

      // Change data
      act(() => {
        rerender({ data: { value: 2 } });
      });
      expect(result.current.hasUnsavedChanges).toBe(true);

      // Manual save
      act(() => {
        result.current.save();
      });
      expect(result.current.hasUnsavedChanges).toBe(false);
    });
  });

  describe('callbacks', () => {
    it('should call onSave callback', async () => {
      const testData = { name: 'Test' };
      const onSave = vi.fn();

      renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: testData,
          intervalMs: 30000,
          onSave,
        })
      );

      // Trigger initial save
      act(() => {
        vi.advanceTimersByTime(0);
      });
      expect(onSave).toHaveBeenCalledWith(testData);
    });

    it('should call onLoad callback', () => {
      const testData = { name: 'Test' };
      const onLoad = vi.fn();
      localStorage.setItem(STORAGE_KEY, JSON.stringify(testData));

      const { result } = renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: {},
          intervalMs: 30000,
          onLoad,
        })
      );

      act(() => {
        result.current.load();
      });

      expect(onLoad).toHaveBeenCalledWith(testData);
    });
  });

  describe('custom serialization', () => {
    it('should use custom serialize/deserialize', async () => {
      const testData = new Date('2025-01-01');
      const serialize = vi.fn((data: Date) => data.toISOString());
      const deserialize = vi.fn((str: string) => new Date(str));

      const { result } = renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: testData,
          intervalMs: 30000,
          serialize,
          deserialize,
        })
      );

      // Trigger initial save
      act(() => {
        vi.advanceTimersByTime(0);
      });
      expect(serialize).toHaveBeenCalledWith(testData);

      act(() => {
        result.current.load();
      });

      expect(deserialize).toHaveBeenCalled();
    });
  });

  describe('lastSaveTime', () => {
    it('should track last save time', async () => {
      const testData = { value: 1 };

      const { result } = renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: testData,
          intervalMs: 30000,
        })
      );

      // Trigger initial save
      act(() => {
        vi.advanceTimersByTime(0);
      });

      expect(result.current.lastSaveTime).not.toBeNull();
      expect(result.current.lastSaveTime).toBeInstanceOf(Date);
    });
  });

  describe('enabled/disabled', () => {
    it('should not save when disabled', async () => {
      const testData = { value: 1 };

      const { result } = renderHook(() =>
        useAutoSave({
          storageKey: STORAGE_KEY,
          data: testData,
          intervalMs: 1000,
          enabled: false,
        })
      );

      // Wait a bit
      act(() => {
        vi.advanceTimersByTime(2000);
      });

      // Should not have saved
      expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
    });
  });

  describe('utility functions', () => {
    it('should check for saved session', () => {
      expect(hasSavedSession(STORAGE_KEY)).toBe(false);

      localStorage.setItem(STORAGE_KEY, JSON.stringify({ test: true }));

      expect(hasSavedSession(STORAGE_KEY)).toBe(true);
    });

    it('should get last save timestamp', () => {
      expect(getLastSaveTimestamp(STORAGE_KEY)).toBeNull();

      const timestamp = new Date().toISOString();
      localStorage.setItem(`${STORAGE_KEY}:timestamp`, timestamp);

      const result = getLastSaveTimestamp(STORAGE_KEY);
      expect(result).toBeInstanceOf(Date);
      expect(result?.toISOString()).toBe(timestamp);
    });
  });

  describe('beforeunload handling', () => {
    it('should save on page unload with unsaved changes', async () => {
      const testData = { value: 1 };

      const { rerender } = renderHook(
        ({ data }) =>
          useAutoSave({
            storageKey: STORAGE_KEY,
            data,
            intervalMs: 30000,
          }),
        { initialProps: { data: testData } }
      );

      // Trigger initial save
      act(() => {
        vi.advanceTimersByTime(0);
      });
      expect(localStorage.getItem(STORAGE_KEY)).not.toBeNull();

      // Change data
      act(() => {
        rerender({ data: { value: 2 } });
      });

      // Trigger beforeunload
      const event = new Event('beforeunload') as BeforeUnloadEvent;
      window.dispatchEvent(event);

      // Should have saved the new data
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
      expect(saved).toEqual({ value: 2 });
    });
  });

  describe('visibility change handling', () => {
    it('should save when tab becomes hidden with unsaved changes', async () => {
      const testData = { value: 1 };

      const { rerender } = renderHook(
        ({ data }) =>
          useAutoSave({
            storageKey: STORAGE_KEY,
            data,
            intervalMs: 30000,
          }),
        { initialProps: { data: testData } }
      );

      // Trigger initial save
      act(() => {
        vi.advanceTimersByTime(0);
      });
      expect(localStorage.getItem(STORAGE_KEY)).not.toBeNull();

      // Change data
      act(() => {
        rerender({ data: { value: 2 } });
      });

      // Simulate tab hidden
      Object.defineProperty(document, 'hidden', {
        writable: true,
        configurable: true,
        value: true,
      });

      const event = new Event('visibilitychange');
      document.dispatchEvent(event);

      // Should have saved
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
      expect(saved).toEqual({ value: 2 });
    });
  });
});
