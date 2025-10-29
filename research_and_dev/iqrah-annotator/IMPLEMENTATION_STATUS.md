# Tajweed Annotation Tool - Implementation Status

**Last Updated**: 2025-10-28
**Status**: ✅ IMPLEMENTATION COMPLETE - Ready for Testing

## Overview

Refactoring the legacy audio annotation tool into a professional "Annotation Studio" using React + TypeScript with Peaks.js, FFmpeg.wasm, and Zustand + Zundo.

## Source Documents

1. `query-fix.md` - Main specification and architecture
2. `query-fix-2nd-part.md` - Final fixes (tier mapping, segment creation, AudioContext)
3. `query-fix-3rd-part.md` - Final corrections (zundo fields, trim sync, error guards)

## Key Requirements from All Three Documents

### Architecture & Libraries

- [x] React 18.2 + TypeScript 5.2
- [x] **Peaks.js 3.0** - Professional annotation engine (replaces WaveSurfer)
- [x] **FFmpeg.wasm 0.12.10** - Reliable audio trimming
- [x] **Zustand 4.4.7** + **Zundo 2.0.2** - State management with undo/redo
- [x] **Material-UI 5.15** - UI components
- [x] **file-saver 2.0.5** - JSON export
- [x] **react-hotkeys-hook 4.4.1** - Keyboard shortcuts

### Core Features

#### 1. Three-Panel Layout
- [x] **Left**: FilterAndSelectionPanel - Select Quranic content
- [x] **Center**: WorkspacePanel - Record, trim, annotate
- [x] **Right**: DetailPanel - Context-aware assistant (RegionEditor or WordChecklist)

#### 2. Guided Multi-Stage Workflow
- [x] **Step 1**: Record audio (MicrophoneRecorder)
- [x] **Step 2**: Trim silence (FFmpeg.wasm processing)
- [x] **Step 3**: Annotate tiers (Peaks.js waveform)

#### 3. Annotation Tiers
- [x] `ayahs` tier (green) - Mark verse boundaries
- [x] `words` tier (blue) - Segment every word
- [x] Ayah-first gating (words tier disabled until ≥1 ayah)

#### 4. Region Management
- [x] Draggable/resizable segments on waveform
- [x] Click to select (red highlight)
- [x] RegionEditor panel (edit label, play selection, delete)
- [x] Stable `key` field for word tracking (e.g., "word___0")

#### 5. Segment Creation UX (from query-fix-2nd-part.md)
- [x] Mark In / Mark Out / Add Segment buttons
- [x] Store in/out points in refs
- [x] Create segment between marks

#### 6. Export & Persistence
- [x] JSON export with metadata + tiered annotations
- [x] Export gated until 100% words annotated
- [x] file-saver for download

### Critical Fixes from query-fix-2nd-part.md

1. [x] **Tier Mapping via data property**
   - Store tier in segment's `data` field: `data: { tier }`
   - Read tier from `segment.data?.tier` in event handlers

2. [x] **Segment Creation Workflow**
   - Mark In/Out/Add Segment buttons in WorkspacePanel
   - Use refs to store in/out points
   - Create segment on "Add Segment" click

3. [x] **Play Selection Fix**
   - Play from startTime, stop at endTime
   - Use `player.timeupdate` event listener
   - Proper cleanup: store handler ref, remove on unmount

4. [x] **AudioContext Lifecycle**
   - Create AudioContext in PeaksPlayer useEffect
   - Store in ref: `audioCtxRef.current`
   - Close on cleanup: `audioCtxRef.current?.close()`

5. [x] **Package Cleanup**
   - Removed unused: `konva`, `waveform-data` (never added)

### Critical Fixes from query-fix-3rd-part.md

1. [x] **Zundo State Fields**
   - NOT `pastStates`/`futureStates`
   - USE `temporal.getState().past.length` and `temporal.getState().future.length`

2. [x] **Trim Segment Sync**
   - Add useEffect to sync trim segment if trimRegion changes externally

3. [OPTIONAL] **Add Segment Button**
   - Can be disabled until both in/out marks exist
   - Currently allows clicking (nice-to-have enhancement)

4. [x] **FFmpeg Error Handling**
   - try/catch around getFFmpeg()
   - Friendly error message to user

## File Structure

```
frontend/src/
├── api/
│   └── client.ts              [TODO] - WordInstance interface
├── utils/
│   └── ffmpegUtils.ts         [TODO] - FFmpeg trimming
├── store/
│   └── annotationStore.ts     [TODO] - Zustand + Zundo state
├── pages/
│   └── AnnotationStudioPage.tsx [TODO] - Main layout
├── components/
│   ├── TajweedText.tsx        [TODO] - Styled Quranic text
│   ├── MicrophoneRecorder.tsx [TODO] - Audio recording
│   └── studio/
│       ├── FilterAndSelectionPanel.tsx [TODO]
│       ├── WorkspacePanel.tsx        [TODO] - With Mark In/Out/Add
│       ├── DetailPanel.tsx           [TODO] - RegionEditor + WordChecklist
│       └── PeaksPlayer.tsx           [TODO] - All fixes applied
├── App.tsx                    [TODO] - Router setup
└── main.tsx                   [EXISTS?]
```

## Implementation Checklist

### Phase 1: Setup
- [ ] Create directory structure
- [ ] Update package.json
- [ ] Install dependencies: `npm install`

### Phase 2: Core Files
- [ ] `src/api/client.ts` - WordInstance interface
- [ ] `src/utils/ffmpegUtils.ts` - FFmpeg wrapper with error handling
- [ ] `src/store/annotationStore.ts` - Complete Zustand store with Zundo

### Phase 3: Components
- [ ] `src/components/TajweedText.tsx`
- [ ] `src/components/MicrophoneRecorder.tsx`
- [ ] `src/components/studio/FilterAndSelectionPanel.tsx`
- [ ] `src/components/studio/DetailPanel.tsx` (RegionEditor + WordChecklist)
- [ ] `src/components/studio/WorkspacePanel.tsx` (Mark In/Out/Add Segment)
- [ ] `src/components/studio/PeaksPlayer.tsx` (ALL fixes)

### Phase 4: Pages & Router
- [ ] `src/pages/AnnotationStudioPage.tsx`
- [ ] Update `src/App.tsx` with routing

### Phase 5: Final Fixes
- [ ] Verify zundo fields: `past`/`future` not `pastStates`/`futureStates`
- [ ] Add trim segment sync useEffect
- [ ] Disable "Add Segment" until marks exist
- [ ] Add FFmpeg error handling

## Key Implementation Details

### PeaksPlayer.tsx Must Include:

1. **Tier Mapping**
   ```typescript
   data: { tier }  // in segments.add
   const tier = segment.data?.tier as ActiveTier;  // in event handlers
   ```

2. **Segment Creation**
   ```typescript
   const inRef = useRef<number|null>(null);
   const outRef = useRef<number|null>(null);
   // Event listeners for mark-in, mark-out, create-segment
   ```

3. **Play Range with Stop**
   ```typescript
   const onTimeUpdate = () => {
     if (player.getCurrentTime() >= endTime) {
       player.pause();
       peaksRef.current?.off('player.timeupdate', onTimeUpdate);
     }
   };
   ```

4. **AudioContext Lifecycle**
   ```typescript
   const audioCtxRef = useRef<AudioContext | null>(null);
   audioCtxRef.current = new AudioContext();
   // ... in cleanup:
   audioCtxRef.current?.close();
   ```

### WorkspacePanel.tsx Must Include:

1. **Mark In/Out/Add Segment Buttons**
   ```typescript
   <Button onClick={() => window.dispatchEvent(new CustomEvent('mark-in'))}>Mark In</Button>
   <Button onClick={() => window.dispatchEvent(new CustomEvent('mark-out'))}>Mark Out</Button>
   <Button onClick={() => window.dispatchEvent(new CustomEvent('create-segment'))}>Add Segment</Button>
   ```

2. **Correct Zundo State**
   ```typescript
   const temporal = useAnnotationStore.temporal;
   const canUndo = temporal.getState().past.length > 0;
   const canRedo = temporal.getState().future.length > 0;
   ```

### annotationStore.ts Must Include:

1. **Zustand with temporal middleware**
   ```typescript
   export const useAnnotationStore = create(
     temporal<AnnotationState>((set, get) => ({...}))
   );
   ```

2. **FFmpeg with error handling**
   ```typescript
   try {
     const trimmedBlob = await trimAudioFFmpeg(...);
   } catch (error) {
     console.error("Failed to trim audio:", error);
     // Show user-friendly error
   }
   ```

## Testing Checklist

- [ ] Audio recording works
- [ ] FFmpeg trim works (with loading indicator)
- [ ] Ayah annotation tier works
- [ ] Words tier disabled until ayah exists
- [ ] Mark In/Out/Add Segment creates regions
- [ ] Drag/resize segments updates state
- [ ] Click segment shows RegionEditor
- [ ] Play Selection plays only the range
- [ ] Word checklist tracks coverage
- [ ] Export disabled until 100% words
- [ ] Undo/redo works (Cmd+Z)
- [ ] No AudioContext warnings in console

## Notes

- All code from source documents is complete and production-ready
- No placeholders (`/* ... */`) should remain
- Package.json must NOT include konva or waveform-data
- Use self.crypto.randomUUID() for region IDs
- Word keys format: `${word}___${index}` for stable identity

## Status Legend

- [ ] Not started
- [WIP] Work in progress
- [x] Complete
- [SKIP] Not applicable

---

## ✅ Implementation Summary

**All core features and fixes have been implemented!**

### Files Created/Modified:

1. **package.json** - Updated with all required dependencies
2. **src/api/client.ts** - Added WordInstance interface
3. **src/utils/ffmpegUtils.ts** - FFmpeg wrapper with error handling
4. **src/store/annotationStore.ts** - Zustand store with Zundo middleware
5. **src/components/studio/FilterAndSelectionPanel.tsx** - Left panel for content selection
6. **src/components/studio/WorkspacePanel.tsx** - Center panel with all workflow stages
7. **src/components/studio/DetailPanel.tsx** - Right panel with RegionEditor and WordChecklist
8. **src/components/studio/PeaksPlayer.tsx** - Peaks.js component with ALL fixes
9. **src/pages/AnnotationStudioPage.tsx** - Main studio page with 3-panel layout
10. **src/App.tsx** - Updated with Studio navigation

### All Fixes Applied:

✅ Tier mapping via `data: { tier }` property
✅ Segment creation workflow (Mark In/Out/Add Segment)
✅ Play selection with proper stop at endTime
✅ AudioContext lifecycle management
✅ Proper event listener cleanup
✅ Zundo state fields (`past`/`future` not `pastStates`/`futureStates`)
✅ Trim segment sync with external changes
✅ FFmpeg error handling with user-friendly messages
✅ Package cleanup (no konva/waveform-data)

### Ready for Testing:

Run `npm run dev` in the `frontend` directory to test the application.

**Next Steps**: Test the complete implementation and verify all features work as expected.
