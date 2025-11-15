# Utility Integration Guide

This guide shows how to integrate all the new utilities created during the UX/Performance fixes.

## 📦 New Utilities & Components

### 1. LoadingOverlay Component

**Purpose:** Show loading state with optional progress bar during async operations.

**Usage:**
```tsx
import LoadingOverlay from '../components/LoadingOverlay';

function MyComponent() {
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);

  return (
    <>
      <LoadingOverlay
        visible={loading}
        message="Processing audio..."
        progress={progress} // 0-1 (optional, omit for spinner)
      />
      {/* Your content */}
    </>
  );
}
```

**Integration Points:**
- `WordSegmenter`: Show during ayah audio extraction
- `VerseSegmenter`: Show during VAD speech detection
- `StudioWizardPage`: Show during recording save

---

### 2. User-Friendly Error Messages

**Purpose:** Convert technical errors to user-friendly messages with fix suggestions.

**Usage:**
```tsx
import { formatErrorMessage, getUserFriendlyError } from '../constants/errorMessages';

// Option 1: Get formatted string with "how to fix"
const errorMsg = formatErrorMessage("Validation failed: start >= end");
setError(errorMsg);

// Option 2: Get error object for custom rendering
const { title, message, howToFix } = getUserFriendlyError("Parent verse not found");
```

**Integration Points:**
- Replace all `setError(technicalError)` calls with `setError(formatErrorMessage(technicalError))`
- Add to `WordSegmenter` line 252, 321, 329, 347
- Add to `VerseSegmenter` line 185, 191, 206, 274

---

### 3. Keyboard Shortcuts Hook

**Purpose:** Add configurable keyboard shortcuts to any component.

**Usage:**
```tsx
import { useKeyboardShortcuts, COMMON_SHORTCUTS } from '../hooks/useKeyboardShortcuts';
import { useWizardStore } from '../store/wizardStore';

function WordSegmenter() {
  const { undo, redo } = useWizardStore.temporal.getState();

  useKeyboardShortcuts({
    shortcuts: [
      COMMON_SHORTCUTS.undo(() => undo()),
      COMMON_SHORTCUTS.redo(() => redo()),
      COMMON_SHORTCUTS.delete(() => handleDeleteSelected()),
      {
        key: 's',
        ctrl: true,
        description: 'Smart segment',
        action: () => createSmartSegment(),
      },
    ],
  });

  return (/* ... */);
}
```

**Integration Points:**
- Add to `WordSegmenter`: Undo, Redo, Delete, Play/Pause
- Add to `VerseSegmenter`: Undo, Redo, Delete, Play/Pause (already has Undo/Redo)
- Add to `StudioWizardPage`: Save (Ctrl+S), Stage navigation (Ctrl+1-5)

---

### 4. Auto-Save Hook

**Purpose:** Automatically save state to localStorage with session recovery.

**Usage:**
```tsx
import { useAutoSave, hasSavedSession } from '../hooks/useAutoSave';
import { useWizardStore } from '../store/wizardStore';

function StudioWizardPage() {
  const state = useWizardStore();

  // Check for saved session on mount
  useEffect(() => {
    if (hasSavedSession('wizard-session')) {
      const confirmed = confirm('Resume previous session?');
      if (confirmed) {
        const saved = autoSave.load();
        if (saved) {
          // Restore state
          useWizardStore.setState(saved);
        }
      } else {
        autoSave.clear();
      }
    }
  }, []);

  // Auto-save
  const autoSave = useAutoSave({
    storageKey: 'wizard-session',
    data: state,
    intervalMs: 30000, // 30s
    onSave: () => console.log('Auto-saved!'),
  });

  return (
    <div>
      {autoSave.hasUnsavedChanges && <span>Unsaved changes</span>}
      <button onClick={() => autoSave.save()}>Save Now</button>
    </div>
  );
}
```

**Integration Points:**
- Add to `StudioWizardPage` main component
- Save entire wizard state every 30s
- Show "Unsaved changes" indicator in header

---

### 5. Export Validation

**Purpose:** Validate annotation exports before saving.

**Usage:**
```tsx
import { validateExport, formatValidationErrors } from '../utils/exportValidation';

function ExportButton() {
  const handleExport = () => {
    const data = useWizardStore.getState().exportAnnotations();

    // Validate before export
    const validation = validateExport(data);

    if (!validation.valid) {
      alert('Export validation failed:\n\n' + formatValidationErrors(validation));
      return;
    }

    if (validation.warnings.length > 0) {
      const proceed = confirm(
        'Export has warnings:\n\n' +
        formatValidationErrors(validation) +
        '\n\nContinue anyway?'
      );
      if (!proceed) return;
    }

    // Proceed with export
    const json = JSON.stringify(data, null, 2);
    downloadFile(json, 'annotations.json');
  };

  return <button onClick={handleExport}>Export Annotations</button>;
}
```

**Integration Points:**
- Add to export button in wizard completion stage
- Show validation errors in dialog before export

---

### 6. Coordinate Conversion (Branded Types)

**Purpose:** Type-safe conversion between absolute and relative times.

**Usage:**
```tsx
import { AudioCoordinateConverter, absolute, relative } from '../lib/coordinates';

function WordSegmenter() {
  const timeOffset = currentVerse.start;
  const converter = new AudioCoordinateConverter(timeOffset);

  const handleCreateAnnotation = (ann: Annotation) => {
    // Convert relative (in ayah audio) to absolute (in full audio)
    const absoluteStart = converter.toAbsolute(ann.start as RelativeTime);
    const absoluteEnd = converter.toAbsolute(ann.end as RelativeTime);

    // Save to store with absolute times
    addWord(wordKey, ayah, absoluteStart, absoluteEnd, text);
  };

  return (/* ... */);
}
```

**Note:** This is currently optional (types can be circumvented with `as` casts). For strict type safety, update all time-related code to use branded types.

**Integration Points:**
- Gradually adopt in `WordSegmenter` and `VerseSegmenter`
- Enforce at type level by making conversion functions require branded types

---

### 7. Accessibility Helpers

**Purpose:** Improve accessibility with screen reader announcements and keyboard navigation.

**Usage:**
```tsx
import { announceToScreenReader, ariaLabels, focusManagement } from '../utils/accessibility';

function WordSegmenter() {
  const handleWordComplete = (word: string) => {
    announceToScreenReader(`Word ${word} segmented successfully`);
  };

  return (
    <button
      aria-label={ariaLabels.waveform.playButton}
      onClick={handlePlayPause}
    >
      Play
    </button>
  );
}

// Focus trap in modal
function Modal({ children, onClose }) {
  const modalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!modalRef.current) return;
    const cleanup = focusManagement.trapFocus(modalRef.current);
    return cleanup;
  }, []);

  return <div ref={modalRef}>{children}</div>;
}
```

**Integration Points:**
- Add ARIA labels to all waveform controls
- Announce state changes (ayah completed, word segmented, etc.)
- Add focus trap to edit dialogs
- Add keyboard navigation to ayah/word chip lists

---

### 8. Annotation Restoration Hook

**Purpose:** Restore annotations when switching audio segments (prevents infinite loops).

**Usage:**
```tsx
import { useAnnotationRestoration } from '../hooks/useAnnotationRestoration';

function WordSegmenter() {
  const currentVerseWords = words.filter(w => w.ayah === currentVerse.ayah);

  const isRestoringRef = useAnnotationRestoration({
    manager: annotationManagerRef.current,
    items: currentVerseWords.map(w => ({
      id: w.annotationId!,
      start: w.start,
      end: w.end,
      text: w.text,
    })),
    timeOffset,
    kind: 'word',
    getLabelFn: (item) => stripHtml(item.text),
    audioUrl: ayahAudioUrl,
    additionalDeps: [currentVerse?.ayah],
  });

  const handleCreateAnnotation = (ann: Annotation) => {
    if (isRestoringRef.current) return; // Skip during restoration
    // ... rest of logic
  };

  return (/* ... */);
}
```

**Integration Points:**
- Already integrated in `WordSegmenter` ✅
- Consider using in `VerseSegmenter` if needed

---

## 🚀 Quick Integration Checklist

### StudioWizardPage (Main Wizard)
- [ ] Add `useAutoSave` hook
- [ ] Add `useKeyboardShortcuts` for stage navigation (Ctrl+1-5)
- [ ] Add "Unsaved changes" indicator
- [ ] Add session recovery on mount
- [ ] Add export validation to export button

### WordSegmenter
- [ ] Replace technical errors with `formatErrorMessage()`
- [ ] Add `LoadingOverlay` during ayah audio extraction
- [ ] Add keyboard shortcuts (Undo, Redo, Delete, Play/Pause)
- [ ] Add screen reader announcements for state changes
- [ ] Add ARIA labels to controls

### VerseSegmenter
- [ ] Replace technical errors with `formatErrorMessage()`
- [ ] Add `LoadingOverlay` during VAD detection
- [ ] Add keyboard shortcuts (already has Undo/Redo, add Delete, Play/Pause)
- [ ] Add screen reader announcements
- [ ] Add ARIA labels

### WavesurferAnnotator
- [ ] Add ARIA labels to playback controls
- [ ] Add screen reader announcements for play/pause events

---

## 📝 Testing Guide

### Test Auto-Save
1. Start wizard, add some annotations
2. Wait 30 seconds (should auto-save)
3. Refresh page → Should show "Resume session?" prompt
4. Confirm → State should be restored

### Test Keyboard Shortcuts
1. Focus waveform
2. Press Space → Should play/pause
3. Press Ctrl+Z → Should undo
4. Press Ctrl+Shift+Z → Should redo
5. Press Delete → Should delete selected (if implemented)

### Test Export Validation
1. Create incomplete annotations (e.g., verse without words)
2. Try to export
3. Should show validation errors with specific location
4. Fix errors, export again → Should succeed

### Test Accessibility
1. Tab through UI → Focus should be visible
2. Use only keyboard to create annotation
3. Check ARIA labels with screen reader

### Test Loading States
1. Switch ayahs in word segmenter
2. Should show "Loading ayah audio..." overlay
3. Progress bar should update (if FFmpeg reports progress)

---

## 🐛 Common Issues & Fixes

### Issue: Auto-save not working
**Fix:** Check that wizard store is Zustand-compatible (should already be, since it uses `persist` middleware)

### Issue: Keyboard shortcuts fire in input fields
**Fix:** Hook already ignores input fields, check for `contentEditable` elements

### Issue: Export validation too strict
**Fix:** Adjust validation rules in `exportValidation.ts`, or use `warnings` instead of `errors`

### Issue: Restoration loops still happening
**Fix:** Ensure `useAnnotationRestoration` is used everywhere, and `isRestoringRef.current` is checked in `onCreate` callbacks

### Issue: ARIA labels not read by screen reader
**Fix:** Test with actual screen reader (NVDA, JAWS, VoiceOver), ensure elements have correct `role` attribute

---

## 🎯 Priority Integration Order

1. **High Priority (Immediate Impact):**
   - [ ] Add error message formatting to all error setters
   - [ ] Add auto-save to main wizard
   - [ ] Add export validation to export button

2. **Medium Priority (Improves UX):**
   - [ ] Add keyboard shortcuts to segmenters
   - [ ] Add loading overlays to async operations
   - [ ] Add screen reader announcements

3. **Low Priority (Polish):**
   - [ ] Add focus management to modals
   - [ ] Add branded types everywhere
   - [ ] Add comprehensive ARIA labels

---

*Last Updated: 2025-11-01*
*Ready for Integration!*
