// Main wizard container with 5-stage annotation flow
import React, { useEffect, useState } from 'react';
import {
  Container,
  Paper,
  Stepper,
  Step,
  StepLabel,
  Button,
  Stack,
  Alert,
  Box,
  IconButton,
  Tooltip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Typography,
  Chip,
} from '@mui/material';
import { ArrowBack, ArrowForward, Save, Undo, Redo, RestartAlt, FolderOpen, CloudDone } from '@mui/icons-material';
import { useWizardStore } from '../store/wizardStore';
import { saveAs } from 'file-saver';
import { getValidationErrors, type AnnotationExport } from '../types/export';
import { useToast } from '../contexts/ToastContext';

// Stage components (to be created)
import { ContentSelector } from '../components/wizard/ContentSelector';
import { AudioStage } from '../components/wizard/AudioStage';
import { VerseSegmenter } from '../components/wizard/VerseSegmenter';
import { WordSegmenter } from '../components/wizard/WordSegmenter';
import { AntiPatternStage } from '../components/wizard/AntiPatternStage';
import { ErrorBoundary } from '../components/ErrorBoundary';

const STEPS = [
  'Select Content',
  'Record & Trim',
  'Segment Ayahs',
  'Segment Words',
  'Anti-Patterns',
];

const STORAGE_KEY = 'tajweed-wizard-v1';

export const StudioWizardPage: React.FC = () => {
  const {
    step,
    setStep,
    canProceed,
    nextStep,
    prevStep,
    getMissingSegments,
    exportAnnotations,
    loadExisting,
    reset,
    surah,
    ayahs,
    verses,
    words,
  } = useWizardStore();

  const toast = useToast();

  const [showResumeDialog, setShowResumeDialog] = useState(false);
  const [savedSession, setSavedSession] = useState<any>(null);

  const missing = getMissingSegments();

  // Calculate progress
  const segmentedAyahsCount = verses.length;
  const totalAyahs = ayahs.length;
  const totalWords = words.length;

  // Check for saved session on mount
  useEffect(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        // Check if there's meaningful progress (not just initial state)
        if (parsed.state && (parsed.state.surah || parsed.state.step > 0)) {
          setSavedSession(parsed.state);
          setShowResumeDialog(true);
        }
      }
    } catch (error) {
      console.error('Failed to load saved session:', error);
    }
  }, []);

  const handleResumeSession = () => {
    setShowResumeDialog(false);
    // Session is already loaded by Zustand persist middleware
  };

  const handleStartFresh = () => {
    setShowResumeDialog(false);
    reset();
    localStorage.removeItem(STORAGE_KEY);
  };

  const handleSave = () => {
    try {
      const data = exportAnnotations();

      // Validate before saving
      const errors = getValidationErrors(data);
      if (errors.length > 0) {
        toast.showError('Validation errors:\n' + errors.join('\n'), 10000);
        return;
      }

      // Export as JSON
      const blob = new Blob([JSON.stringify(data, null, 2)], {
        type: 'application/json',
      });
      saveAs(blob, `annotation_${data.recording_id}.json`);

      toast.showSuccess('Exported successfully!');
    } catch (error) {
      toast.showError(`Export failed: ${error}`);
    }
  };

  const handleReset = () => {
    if (confirm('Reset all progress? This cannot be undone.')) {
      reset();
    }
  };

  const handleLoad = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'application/json,.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      try {
        const text = await file.text();
        const data = JSON.parse(text) as AnnotationExport;

        // Validate schema
        const errors = getValidationErrors(data);
        if (errors.length > 0) {
          toast.showError('Invalid annotation file:\n' + errors.join('\n'), 10000);
          return;
        }

        // Note: Audio recording needs to be re-recorded or loaded separately
        // The JSON only contains metadata, not the audio blob
        // TODO: Replace confirm() with a proper MUI Dialog component for better UX
        if (confirm(
          'Load annotation from file? Note: You will need to record/upload the audio again.\n\n' +
          `Surah: ${data.content.surah}\n` +
          `Verses: ${data.content.verses.map(v => v.ayah).join(', ')}\n` +
          `Words: ${data.content.verses.reduce((sum, v) => sum + v.words.length, 0)}\n` +
          `Anti-patterns: ${data.content.verses.reduce((sum, v) =>
            sum + v.words.reduce((s, w) => s + w.anti_patterns.length, 0), 0)}`
        )) {
          loadExisting(data);
          toast.showSuccess('Annotation loaded! Please record or upload the audio in Stage 1.');
        }
      } catch (error) {
        toast.showError(`Failed to load annotation: ${error}`);
      }
    };
    input.click();
  };

  // Undo/Redo (zundo integration)
  const undo = () => {
    const temporal = (useWizardStore as any).temporal;
    if (temporal?.getState) {
      temporal.getState().undo();
    }
  };

  const redo = () => {
    const temporal = (useWizardStore as any).temporal;
    if (temporal?.getState) {
      temporal.getState().redo();
    }
  };

  const canUndo = () => {
    const temporal = (useWizardStore as any).temporal;
    return temporal?.getState?.()?.pastStates?.length > 0;
  };

  const canRedo = () => {
    const temporal = (useWizardStore as any).temporal;
    return temporal?.getState?.()?.futureStates?.length > 0;
  };

  return (
    <Container maxWidth="xl" sx={{ py: 3 }}>
      <Paper sx={{ p: 3 }}>
        {/* Header with controls */}
        <Stack
          direction="row"
          justifyContent="space-between"
          alignItems="center"
          sx={{ mb: 3 }}
        >
          <Stack direction="row" spacing={2} alignItems="center">
            <h1 style={{ margin: 0 }}>Tajweed Annotation Wizard</h1>
            <Tooltip title="Auto-save is enabled. Your progress is automatically saved to browser storage.">
              <Chip
                icon={<CloudDone />}
                label="Auto-save enabled"
                size="small"
                color="success"
                variant="outlined"
              />
            </Tooltip>
          </Stack>
          <Stack direction="row" spacing={1}>
            <Tooltip title="Load existing annotation">
              <IconButton onClick={handleLoad} size="small" color="primary">
                <FolderOpen />
              </IconButton>
            </Tooltip>
            <Tooltip title="Undo (Ctrl+Z)">
              <span>
                <IconButton onClick={undo} disabled={!canUndo()} size="small">
                  <Undo />
                </IconButton>
              </span>
            </Tooltip>
            <Tooltip title="Redo (Ctrl+Shift+Z)">
              <span>
                <IconButton onClick={redo} disabled={!canRedo()} size="small">
                  <Redo />
                </IconButton>
              </span>
            </Tooltip>
            <Tooltip title="Reset all progress">
              <IconButton onClick={handleReset} size="small" color="error">
                <RestartAlt />
              </IconButton>
            </Tooltip>
          </Stack>
        </Stack>

        {/* Status Display */}
        {(surah || step > 0) && (
          <Alert severity="info" sx={{ mb: 2 }}>
            <Stack direction="row" spacing={2} divider={<span>•</span>}>
              <span>Stage {step}</span>
              {surah && <span>Surah: {surah}</span>}
              {totalAyahs > 0 && step >= 2 && (
                <span>Segmented Ayahs ({segmentedAyahsCount}/{totalAyahs})</span>
              )}
              {totalWords > 0 && step >= 3 && (
                <span>Segmented Words ({totalWords})</span>
              )}
            </Stack>
          </Alert>
        )}

        {/* Stepper */}
        <Stepper activeStep={step} sx={{ mb: 3 }}>
          {STEPS.map((label, i) => (
            <Step
              key={label}
              onClick={() => setStep(i as any)}
              sx={{ cursor: 'pointer' }}
            >
              <StepLabel>{label}</StepLabel>
            </Step>
          ))}
        </Stepper>

        {/* Stage content */}
        <Box sx={{ minHeight: '500px' }}>
          <ErrorBoundary onError={(error, errorInfo) => {
            console.error('[StudioWizard] Stage error:', error, errorInfo);
            // Could send to error tracking service here (e.g., Sentry)
          }}>
            {step === 0 && <ContentSelector />}
            {step === 1 && <AudioStage />}
            {step === 2 && <VerseSegmenter />}
            {step === 3 && <WordSegmenter />}
            {step === 4 && <AntiPatternStage />}
          </ErrorBoundary>
        </Box>

        {/* Validation warnings */}
        {missing.verses.length > 0 && (
          <Alert severity="warning" sx={{ mt: 2 }}>
            Missing verse segments for ayahs: {missing.verses.join(', ')}
          </Alert>
        )}

        {Object.keys(missing.words).length > 0 && (
          <Alert severity="warning" sx={{ mt: 2 }}>
            Incomplete word segmentation:
            <ul style={{ margin: 0, paddingLeft: '20px' }}>
              {Object.entries(missing.words).map(([ayah, msgs]) => (
                <li key={ayah}>
                  Ayah {ayah}: {msgs.join(', ')}
                </li>
              ))}
            </ul>
          </Alert>
        )}

        {/* Navigation buttons */}
        <Stack direction="row" spacing={2} sx={{ mt: 3 }}>
          <Button
            startIcon={<ArrowBack />}
            onClick={prevStep}
            disabled={step === 0}
          >
            Back
          </Button>

          <Box sx={{ flexGrow: 1 }} />

          {step < 4 ? (
            <Button
              variant="contained"
              endIcon={<ArrowForward />}
              onClick={nextStep}
              disabled={!canProceed()}
            >
              Next
            </Button>
          ) : (
            <Button
              variant="contained"
              color="success"
              startIcon={<Save />}
              onClick={handleSave}
            >
              Export Annotation
            </Button>
          )}
        </Stack>

        {/* Can't proceed hint */}
        {!canProceed() && (
          <Alert severity="info" sx={{ mt: 2 }}>
            {step === 0 && 'Please select a surah and at least one ayah'}
            {step === 1 && 'Please record audio and set trim bounds'}
            {step === 2 && 'Please segment all selected ayahs'}
            {step === 3 && 'Please segment all words in all ayahs'}
            {step === 4 && 'Ready to export (anti-patterns are optional)'}
          </Alert>
        )}
      </Paper>

      {/* Session Recovery Dialog */}
      <Dialog
        open={showResumeDialog}
        onClose={() => {}}
        disableEscapeKeyDown
      >
        <DialogTitle>Resume previous session</DialogTitle>
        <DialogContent>
          <Typography gutterBottom>
            A previous annotation session was found. Would you like to resume where you left off?
          </Typography>
          {savedSession && (
            <Box sx={{ mt: 2 }}>
              <Typography variant="body2" color="text.secondary">
                Stage {savedSession.step}
                {savedSession.surah && ` • Surah: ${savedSession.surah}`}
                {savedSession.ayahs?.length > 0 && ` • Ayahs: ${savedSession.ayahs.length}`}
              </Typography>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={handleStartFresh} color="secondary">
            Start Fresh
          </Button>
          <Button onClick={handleResumeSession} variant="contained" color="primary">
            Resume
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
  );
};

export default StudioWizardPage;
