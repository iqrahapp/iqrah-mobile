import React from 'react';
import {
  Paper,
  Typography,
  Alert,
  Button,
  ToggleButtonGroup,
  ToggleButton,
  Stack,
  CircularProgress,
} from '@mui/material';
import { Check, CloudDownload, Undo, Redo } from '@mui/icons-material';
import { useHotkeys } from 'react-hotkeys-hook';
import { useStore } from 'zustand';
import { useAnnotationStore } from '../../store/annotationStore';
import TajweedText from '../TajweedText';
import MicrophoneRecorder from '../MicrophoneRecorder';
import WavesurferTrimmer from '../WavesurferTrimmer';
import WavesurferAnnotator from '../WavesurferAnnotator';
import { DEFAULT_CONSTRAINTS } from '../../annotation/constraints';
import { saveAs } from 'file-saver';

export const WorkspacePanel: React.FC = () => {
  const store = useAnnotationStore();
  const [isTrimmed, setIsTrimmed] = React.useState(false);

  // Get undo/redo state and actions from temporal store (zundo v2)
  const pastStates = useStore(useAnnotationStore.temporal, (state) => state.pastStates);
  const futureStates = useStore(useAnnotationStore.temporal, (state) => state.futureStates);
  const canUndo = pastStates.length > 0;
  const canRedo = futureStates.length > 0;

  const undo = () => useAnnotationStore.temporal.getState().undo();
  const redo = () => useAnnotationStore.temporal.getState().redo();

  React.useEffect(() => {
    setIsTrimmed(false);
  }, [store.activeInstance]);

  const handleConfirmTrim = async () => {
    const ok = await store.applyTrim(); // always settles
    setIsTrimmed(true);
    if (!ok) {
      console.warn('FFmpeg failed — proceeding with original audio');
      // Optional: Add snackbar/toast notification here
      // enqueueSnackbar('FFmpeg failed — using original audio', { variant: 'warning' });
    }
  };

  const handleExport = () => {
    const exportData = {
      qpc_location: store.activeInstance?.qpc_location,
      audio_duration_ms: store.activeAudio ? store.activeAudio.duration * 1000 : 0,
      tiers: store.activeTiers,
    };
    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    saveAs(blob, `${store.activeInstance?.qpc_location}_annotations.json`);
  };

  const ayahsComplete = store.activeTiers.ayahs.length > 0;
  // Use backend word count if available
  const totalWords = store.activeInstance?.words?.length ||
    store.activeInstance?.full_text?.replace(/<[^>]+>/g, ' ').trim().split(/\s+/).length || 0;
  const annotatedKeys = new Set(store.activeTiers.words.map((w) => w.key).filter(Boolean));
  const wordsComplete = annotatedKeys.size === totalWords;

  // Keyboard shortcuts
  useHotkeys('mod+z', () => undo(), { enabled: canUndo });
  useHotkeys('mod+shift+z', () => redo(), { enabled: canRedo });

  // Step 0: No instance selected
  if (!store.activeInstance) {
    return (
      <Paper
        sx={{
          p: 3,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexGrow: 1,
        }}
      >
        <Typography variant="h6" color="text.secondary">
          Select content to begin
        </Typography>
      </Paper>
    );
  }

  // Step 1: Record Audio
  if (!store.activeAudio) {
    return (
      <Paper sx={{ p: 3, display: 'flex', flexDirection: 'column', flexGrow: 1 }}>
        <Typography variant="h6" gutterBottom>
          Step 1: Record Audio
        </Typography>
        <Paper variant="outlined" sx={{ p: 2, mb: 3 }}>
          <TajweedText htmlText={store.activeInstance.text} fontSize={28} />
        </Paper>
        <MicrophoneRecorder onRecordingComplete={store.handleRecordingComplete} />
      </Paper>
    );
  }

  // Step 2: Trim Silence
  if (!isTrimmed) {
    return (
      <Paper sx={{ p: 3, display: 'flex', flexDirection: 'column', flexGrow: 1 }}>
        <Stack direction="row" justifyContent="space-between" alignItems="center">
          <Typography variant="h6">Step 2: Trim Silence</Typography>
          <Button
            variant="contained"
            endIcon={store.isLoading ? <CircularProgress size={20} /> : <Check />}
            onClick={handleConfirmTrim}
            disabled={store.isLoading}
          >
            {store.isLoading ? 'Processing with FFmpeg...' : 'Confirm Trim'}
          </Button>
        </Stack>
        <Alert severity="info" sx={{ my: 2 }}>
          Drag the edges of the dark region to remove silence. This will create a new, clean audio
          clip.
        </Alert>
        {store.ffmpegError && (
          <Alert severity="warning" sx={{ mb: 2 }}>
            Audio processing issue: {store.ffmpegError}. Proceeding with original audio.
          </Alert>
        )}
        <WavesurferTrimmer
          audioUrl={store.activeAudio.url}
          value={store.trimRegion ? { start: store.trimRegion.start, end: store.trimRegion.end } : undefined}
          onChange={(r) => store.setTrimRegion({ start: r.start, end: r.end })}
        />
      </Paper>
    );
  }

  // Step 3: Annotate Tiers
  return (
    <Paper sx={{ p: 3, display: 'flex', flexDirection: 'column', flexGrow: 1 }}>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
        <Typography variant="h6">Step 3: Annotate Tiers</Typography>
        <Stack direction="row" spacing={1}>
          <Button onClick={undo} disabled={!canUndo}>
            <Undo />
          </Button>
          <Button onClick={redo} disabled={!canRedo}>
            <Redo />
          </Button>
          <Button
            variant="contained"
            startIcon={<CloudDownload />}
            onClick={handleExport}
            disabled={!ayahsComplete || !wordsComplete}
          >
            Export
          </Button>
        </Stack>
      </Stack>

      <Stack
        direction="row"
        spacing={2}
        alignItems="center"
        sx={{ p: 1, bgcolor: 'grey.100', borderRadius: 1, mb: 2 }}
      >
        <ToggleButtonGroup
          value={store.activeTier}
          exclusive
          onChange={(_, v) => v && store.setActiveTier(v)}
          color="primary"
        >
          <ToggleButton value="ayahs">Annotate Ayahs</ToggleButton>
          <ToggleButton value="words" disabled={!ayahsComplete}>
            Annotate Words
          </ToggleButton>
        </ToggleButtonGroup>
      </Stack>

      <WavesurferAnnotator
        src={store.activeAudio.url}
        constraints={DEFAULT_CONSTRAINTS}
        controlledKind={store.activeTier === 'words' ? 'word' : 'surah'}
        onViolation={(msgs) => console.warn('Annotation violation:', msgs.join(' '))}
        onCreate={(ann) => {
          if (ann.kind === 'surah') {
            store.addRegion('ayahs', {
              startTime: ann.start,
              endTime: ann.end,
              labelText: ann.meta.label ?? 'Surah',
            });
          } else if (ann.kind === 'word') {
            store.addRegion('words', {
              startTime: ann.start,
              endTime: ann.end,
              labelText: ann.meta.label ?? 'Word',
              key: store.wordToAnnotate || undefined,
            });
          }
        }}
      />
    </Paper>
  );
};
