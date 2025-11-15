# Quick Reference: New Utilities

## 📦 Import Cheat Sheet

```tsx
// Loading feedback
import LoadingOverlay from '../components/LoadingOverlay';

// Error messages
import { formatErrorMessage } from '../constants/errorMessages';

// Keyboard shortcuts
import { useKeyboardShortcuts, COMMON_SHORTCUTS } from '../hooks/useKeyboardShortcuts';

// Auto-save
import { useAutoSave } from '../hooks/useAutoSave';

// Export validation
import { validateExport, formatValidationErrors } from '../utils/exportValidation';

// Coordinates
import { AudioCoordinateConverter } from '../lib/coordinates';

// Accessibility
import { announceToScreenReader, ariaLabels } from '../utils/accessibility';

// Annotation restoration
import { useAnnotationRestoration } from '../hooks/useAnnotationRestoration';
```

---

## 🔧 Common Patterns

### Show Loading

```tsx
const [loading, setLoading] = useState(false);
const [progress, setProgress] = useState(0);

return (
  <LoadingOverlay
    visible={loading}
    message="Processing..."
    progress={progress} // 0-1 or omit
  />
);
```

### User-Friendly Errors

```tsx
// Replace:
setError(technicalError);

// With:
setError(formatErrorMessage(technicalError));
```

### Keyboard Shortcuts

```tsx
useKeyboardShortcuts({
  shortcuts: [
    COMMON_SHORTCUTS.undo(() => undo()),
    COMMON_SHORTCUTS.redo(() => redo()),
  ],
});
```

### Auto-Save

```tsx
const autoSave = useAutoSave({
  storageKey: 'my-data',
  data: myState,
  intervalMs: 30000,
});

// Check on mount
useEffect(() => {
  const saved = autoSave.load();
  if (saved) restoreState(saved);
}, []);
```

### Export Validation

```tsx
const data = exportData();
const result = validateExport(data);

if (!result.valid) {
  alert(formatValidationErrors(result));
  return;
}

// Safe to export
saveFile(data);
```

### Screen Reader Announcement

```tsx
announceToScreenReader('Word segmented successfully');
```

### Coordinate Conversion

```tsx
const converter = new AudioCoordinateConverter(timeOffset);
const absolute = converter.toAbsolute(relativeTime);
const relative = converter.toRelative(absoluteTime);
```

---

## 🎯 Integration Checklist

### StudioWizardPage
- [ ] Add `useAutoSave` hook
- [ ] Add session recovery on mount
- [ ] Add export validation
- [ ] Add keyboard shortcuts (Ctrl+1-5)

### WordSegmenter
- [ ] Replace errors with `formatErrorMessage()`
- [ ] Add `LoadingOverlay` during audio extraction
- [ ] Add keyboard shortcuts (Undo, Redo, Delete)
- [ ] Add screen reader announcements

### VerseSegmenter
- [ ] Replace errors with `formatErrorMessage()`
- [ ] Add `LoadingOverlay` during VAD
- [ ] Add keyboard shortcuts (Delete)
- [ ] Add screen reader announcements

### WavesurferAnnotator
- [ ] Add ARIA labels to controls
- [ ] Add screen reader announcements

---

## ⚠️ Breaking Changes

None! All utilities are opt-in additions.

---

## 🐛 Troubleshooting

| Issue | Fix |
|-------|-----|
| Auto-save not working | Check storage permissions |
| Shortcuts fire in inputs | Should already be ignored |
| Validation too strict | Adjust rules in `exportValidation.ts` |
| Restoration loops | Use `isRestoringRef.current` check |

---

## 📖 Full Documentation

- **Detailed fixes:** `FIXES_IMPLEMENTED.md`
- **Integration guide:** `UTILITY_INTEGRATION_GUIDE.md`
- **Executive summary:** `EXECUTIVE_SUMMARY.md`

---

**Last Updated:** 2025-11-01
