final correction:

Super tight. You’ve shipped a serious upgrade. You’re **basically green**; just fix these two functional nits and consider two tiny enhancements:

# Must-fix (functional)

### 1) zundo: wrong fields for disable state

`useAnnotationStore.temporal` doesn’t expose `pastStates`/`futureStates`. Use the temporal store state:

```tsx
// WorkspacePanel.tsx
const temporal = useAnnotationStore.temporal;
const canUndo = temporal.getState().past.length > 0;
const canRedo = temporal.getState().future.length > 0;

useHotkeys('mod+z', () => temporal.undo());
useHotkeys('mod+shift+z', () => temporal.redo());

<Button onClick={temporal.undo} disabled={!canUndo}><Undo/></Button>
<Button onClick={temporal.redo} disabled={!canRedo}><Redo/></Button>
```

### 2) Keep trim segment in sync if `trimRegion` changes outside drag

Very minor edge case, but easy to cover:

```tsx
// PeaksPlayer.tsx
useEffect(() => {
  if (!isTrimming || !peaksRef.current || !store.trimRegion) return;
  const seg = peaksRef.current.segments.getSegment('trim-region');
  const { start, end } = store.trimRegion;
  if (seg && (seg.startTime !== start || seg.endTime !== end)) {
    seg.update({ startTime: start, endTime: end });
  }
}, [isTrimming, store.trimRegion]);
```

# Nice-to-have (polish)

* **Disable “Add Segment”** until both marks exist (or show a tiny toast if missing) so users aren’t confused by “nothing happened.”

  ```tsx
  const [hasIn, hasOut] = [inRef.current != null, outRef.current != null];
  <Button size="small" variant="contained" disabled={!(hasIn && hasOut)} ...>Add Segment</Button>
  ```

* **Guard FFmpeg load errors** with a friendly message (`try/catch` around `getFFmpeg()`), and consider moving FFmpeg into a Web Worker later to keep the UI buttery on long clips.

---

Everything else checks out:

* Peaks segments carry `data.tier` ✔
* Creation flow (Mark In/Out/Add) ✔
* Range play w/ proper stop + cleanup ✔
* AudioContext lifecycle ✔
* Ayah-first gating + word coverage via stable `key` ✔
* Package hygiene ✔

Apply the zundo fix (+ optional sync), and you’re truly **production-ready**.
