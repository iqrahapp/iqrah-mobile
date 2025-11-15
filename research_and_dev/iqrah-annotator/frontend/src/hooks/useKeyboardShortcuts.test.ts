import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useKeyboardShortcuts, formatShortcut, COMMON_SHORTCUTS } from './useKeyboardShortcuts';

describe('useKeyboardShortcuts', () => {
  beforeEach(() => {
    // Clear all event listeners before each test
    vi.clearAllMocks();
  });

  describe('basic functionality', () => {
    it('should trigger shortcut on matching key', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'a',
          description: 'Test Action',
          action: mockAction,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      // Simulate keydown event
      const event = new KeyboardEvent('keydown', { key: 'a' });
      window.dispatchEvent(event);

      expect(mockAction).toHaveBeenCalledTimes(1);
    });

    it('should trigger shortcut with Ctrl modifier', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'z',
          ctrl: true,
          description: 'Undo',
          action: mockAction,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      // Simulate Ctrl+Z
      const event = new KeyboardEvent('keydown', { key: 'z', ctrlKey: true });
      window.dispatchEvent(event);

      expect(mockAction).toHaveBeenCalledTimes(1);
    });

    it('should trigger shortcut with Shift modifier', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'z',
          ctrl: true,
          shift: true,
          description: 'Redo',
          action: mockAction,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      // Simulate Ctrl+Shift+Z
      const event = new KeyboardEvent('keydown', {
        key: 'z',
        ctrlKey: true,
        shiftKey: true,
      });
      window.dispatchEvent(event);

      expect(mockAction).toHaveBeenCalledTimes(1);
    });

    it('should not trigger when disabled', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'a',
          description: 'Test',
          action: mockAction,
          enabled: false,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      const event = new KeyboardEvent('keydown', { key: 'a' });
      window.dispatchEvent(event);

      expect(mockAction).not.toHaveBeenCalled();
    });

    it('should not trigger when hook is disabled', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'a',
          description: 'Test',
          action: mockAction,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts, enabled: false }));

      const event = new KeyboardEvent('keydown', { key: 'a' });
      window.dispatchEvent(event);

      expect(mockAction).not.toHaveBeenCalled();
    });
  });

  describe('input field handling', () => {
    it('should ignore shortcuts when typing in input field', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'a',
          description: 'Test',
          action: mockAction,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      // Create input element
      const input = document.createElement('input');
      document.body.appendChild(input);
      input.focus();

      // Simulate keydown on input
      const event = new KeyboardEvent('keydown', { key: 'a', bubbles: true });
      Object.defineProperty(event, 'target', { value: input });
      input.dispatchEvent(event);

      expect(mockAction).not.toHaveBeenCalled();

      // Cleanup
      document.body.removeChild(input);
    });

    it('should ignore shortcuts when typing in textarea', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'a',
          description: 'Test',
          action: mockAction,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      // Create textarea element
      const textarea = document.createElement('textarea');
      document.body.appendChild(textarea);
      textarea.focus();

      // Simulate keydown on textarea
      const event = new KeyboardEvent('keydown', { key: 'a', bubbles: true });
      Object.defineProperty(event, 'target', { value: textarea });
      textarea.dispatchEvent(event);

      expect(mockAction).not.toHaveBeenCalled();

      // Cleanup
      document.body.removeChild(textarea);
    });

    it('should ignore shortcuts in contentEditable', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'a',
          description: 'Test',
          action: mockAction,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      // Create contentEditable div
      const div = document.createElement('div');
      div.contentEditable = 'true';
      document.body.appendChild(div);
      div.focus();

      // Simulate keydown
      const event = new KeyboardEvent('keydown', { key: 'a', bubbles: true });
      Object.defineProperty(event, 'target', { value: div });
      div.dispatchEvent(event);

      expect(mockAction).not.toHaveBeenCalled();

      // Cleanup
      document.body.removeChild(div);
    });
  });

  describe('preventDefault behavior', () => {
    it('should preventDefault by default', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'a',
          description: 'Test',
          action: mockAction,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      const event = new KeyboardEvent('keydown', { key: 'a' });
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault');
      window.dispatchEvent(event);

      expect(preventDefaultSpy).toHaveBeenCalled();
      expect(mockAction).toHaveBeenCalled();
    });

    it('should not preventDefault when disabled', () => {
      const mockAction = vi.fn();
      const shortcuts = [
        {
          key: 'a',
          description: 'Test',
          action: mockAction,
          preventDefault: false,
        },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      const event = new KeyboardEvent('keydown', { key: 'a' });
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault');
      window.dispatchEvent(event);

      expect(preventDefaultSpy).not.toHaveBeenCalled();
      expect(mockAction).toHaveBeenCalled();
    });
  });

  describe('multiple shortcuts', () => {
    it('should handle multiple shortcuts', () => {
      const action1 = vi.fn();
      const action2 = vi.fn();
      const shortcuts = [
        { key: 'a', description: 'Action 1', action: action1 },
        { key: 'b', description: 'Action 2', action: action2 },
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }));
      expect(action1).toHaveBeenCalled();
      expect(action2).not.toHaveBeenCalled();

      vi.clearAllMocks();

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'b' }));
      expect(action1).not.toHaveBeenCalled();
      expect(action2).toHaveBeenCalled();
    });

    it('should only trigger first matching shortcut', () => {
      const action1 = vi.fn();
      const action2 = vi.fn();
      const shortcuts = [
        { key: 'a', description: 'Action 1', action: action1 },
        { key: 'a', description: 'Action 2', action: action2 }, // Duplicate
      ];

      renderHook(() => useKeyboardShortcuts({ shortcuts }));

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }));

      expect(action1).toHaveBeenCalledTimes(1);
      expect(action2).not.toHaveBeenCalled();
    });
  });

  describe('COMMON_SHORTCUTS', () => {
    it('should create undo shortcut', () => {
      const mockUndo = vi.fn();
      const shortcut = COMMON_SHORTCUTS.undo(mockUndo);

      expect(shortcut.key).toBe('z');
      expect(shortcut.ctrl).toBe(true);
      expect(shortcut.description).toBe('Undo');

      shortcut.action();
      expect(mockUndo).toHaveBeenCalled();
    });

    it('should create redo shortcut', () => {
      const mockRedo = vi.fn();
      const shortcut = COMMON_SHORTCUTS.redo(mockRedo);

      expect(shortcut.key).toBe('z');
      expect(shortcut.ctrl).toBe(true);
      expect(shortcut.shift).toBe(true);
      expect(shortcut.description).toBe('Redo');

      shortcut.action();
      expect(mockRedo).toHaveBeenCalled();
    });

    it('should create delete shortcut', () => {
      const mockDelete = vi.fn();
      const shortcut = COMMON_SHORTCUTS.delete(mockDelete);

      expect(shortcut.key).toBe('Delete');
      expect(shortcut.description).toContain('Delete');

      shortcut.action();
      expect(mockDelete).toHaveBeenCalled();
    });
  });

  describe('formatShortcut', () => {
    it('should format simple key', () => {
      const shortcut = { key: 'a', description: 'Test', action: vi.fn() };
      expect(formatShortcut(shortcut)).toBe('A');
    });

    it('should format Ctrl+key', () => {
      const shortcut = { key: 'z', ctrl: true, description: 'Undo', action: vi.fn() };
      expect(formatShortcut(shortcut)).toBe('Ctrl + Z');
    });

    it('should format Ctrl+Shift+key', () => {
      const shortcut = {
        key: 'z',
        ctrl: true,
        shift: true,
        description: 'Redo',
        action: vi.fn(),
      };
      expect(formatShortcut(shortcut)).toBe('Ctrl + Shift + Z');
    });

    it('should format special keys', () => {
      const shortcuts = [
        { key: ' ', description: 'Space', action: vi.fn() },
        { key: 'Delete', description: 'Delete', action: vi.fn() },
        { key: 'Enter', description: 'Enter', action: vi.fn() },
      ];

      expect(formatShortcut(shortcuts[0])).toBe('Space');
      expect(formatShortcut(shortcuts[1])).toBe('DELETE');
      expect(formatShortcut(shortcuts[2])).toBe('ENTER');
    });
  });

  describe('cleanup', () => {
    it('should remove event listener on unmount', () => {
      const mockAction = vi.fn();
      const shortcuts = [{ key: 'a', description: 'Test', action: mockAction }];

      const { unmount } = renderHook(() => useKeyboardShortcuts({ shortcuts }));

      // Should work before unmount
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }));
      expect(mockAction).toHaveBeenCalledTimes(1);

      // Unmount
      unmount();

      // Should not work after unmount
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }));
      expect(mockAction).toHaveBeenCalledTimes(1); // Still 1, not 2
    });
  });
});
