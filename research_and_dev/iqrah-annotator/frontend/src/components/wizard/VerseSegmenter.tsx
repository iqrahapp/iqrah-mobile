// Stage 2: Verse segmentation (one segment per ayah)
import React, { useState, useEffect, useRef } from 'react';
import {
  Stack,
  Alert,
  Chip,
  Box,
  Paper,
  IconButton,
  Tooltip,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
} from '@mui/material';
import { Delete, CheckCircle, RadioButtonUnchecked, Add } from '@mui/icons-material';
import WavesurferAnnotator from '../WavesurferAnnotator';
import TajweedText from '../TajweedText';
import { useWizardStore } from '../../store/wizardStore';
import { loadRecording } from '../../store/db';
import { detectSpeechBounds } from '../../lib/vad/silero';
import type { Annotation } from '../../annotation/types';
import type { AnnotationManager } from '../../annotation/manager';
import { useAnnotationRestoration } from '../../hooks/useAnnotationRestoration'; // FIX: Use hook for consistent restoration

export const VerseSegmenter: React.FC = () => {
  const {
    recordingId,
    ayahs,
    ayahTexts,
    verses,
    trim,
    addVerse,
    deleteVerse,
    updateVerseByAnnotationId,
  } = useWizardStore();

  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [audioBlob, setAudioBlob] = useState<Blob | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedAyah, setSelectedAyah] = useState<number | null>(null);
  const [editingSegment, setEditingSegment] = useState<{
    annotation: Annotation;
    currentAyah: number | null;
  } | null>(null);
  const [isCreatingSmartSegment, setIsCreatingSmartSegment] = useState(false);
  const annotationManagerRef = useRef<AnnotationManager | null>(null);
  const processedAnnotations = useRef<Set<string>>(new Set()); // Track processed to prevent double-call

  // BUG FIX #2.4: Use ref to track selectedAyah to avoid stale closures
  const selectedAyahRef = useRef(selectedAyah);
  useEffect(() => {
    selectedAyahRef.current = selectedAyah;
  }, [selectedAyah]);

  // BUG FIX #2.2: Clear processed annotations when verses change (prevents stale tracking)
  useEffect(() => {
    processedAnnotations.current.clear();
    console.log('[VerseSegmenter] Cleared processed annotations tracking');
  }, [verses]);

  // Load audio
  useEffect(() => {
    if (!recordingId) return;

    loadRecording(recordingId).then(result => {
      if (result) {
        setAudioUrl(result.url);
        setAudioBlob(result.blob);
      }
    });

    return () => {
      if (audioUrl) URL.revokeObjectURL(audioUrl);
    };
  }, [recordingId]);

  // Auto-select first missing ayah
  useEffect(() => {
    const missing = ayahs.find(a => !verses.some(v => v.ayah === a));
    console.log('[VerseSegmenter] Auto-select effect:', {
      ayahs,
      versesAyahs: verses.map(v => v.ayah),
      missing,
      selectedAyah
    });
    if (missing && !selectedAyah) {
      console.log('[VerseSegmenter] Auto-selecting ayah:', missing);
      setSelectedAyah(missing);
    }
  }, [ayahs, verses, selectedAyah]);

  // FIX: Use useAnnotationRestoration hook for consistent restoration (prevents visibility bug on reload)
  // Get verses with annotationId for restoration
  const versesWithAnnotations = verses.filter(v => v.annotationId);

  // Use the hook for restoration (timeOffset=0 because verses use absolute time coordinates)
  const isRestoringRef = useAnnotationRestoration({
    manager: annotationManagerRef.current,
    items: versesWithAnnotations.map(v => ({
      id: v.annotationId!,
      start: v.start,
      end: v.end,
      text: v.text,
      ayah: v.ayah,
    })),
    timeOffset: 0, // Verses use absolute time (full audio), no offset needed
    kind: 'surah',
    getLabelFn: (item) => `Ayah ${item.ayah}`,
    audioUrl: audioUrl,
    additionalDeps: [trim],
  });

  // Keyboard shortcuts for undo/redo
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const temporal = (useWizardStore as any).temporal;
      if (!temporal?.getState) return;

      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        const state = temporal.getState();
        if (state.pastStates?.length > 0) {
          state.undo();
        }
      } else if ((e.ctrlKey || e.metaKey) && (e.key === 'Z' || (e.shiftKey && e.key === 'z'))) {
        e.preventDefault();
        const state = temporal.getState();
        if (state.futureStates?.length > 0) {
          state.redo();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleCreateAnnotation = async (ann: Annotation) => {
    // Skip if we're restoring annotations (prevents infinite loop)
    if (isRestoringRef.current) {
      console.log('[VerseSegmenter] Skipping - currently restoring');
      return;
    }

    // Prevent double-call (onCreate fires from both createPoint and region-created)
    if (processedAnnotations.current.has(ann.id)) {
      console.log('[VerseSegmenter] Skipping duplicate call for:', ann.id);
      return;
    }
    processedAnnotations.current.add(ann.id);

    console.log('[VerseSegmenter] ===== handleCreateAnnotation START =====');
    console.log('[VerseSegmenter] Annotation:', ann);

    // Get FRESH state from store to avoid stale closure
    const freshState = useWizardStore.getState();
    const currentVerses = freshState.verses;
    const currentAyahs = freshState.ayahs;
    const currentAyahTexts = freshState.ayahTexts;
    const currentTrim = freshState.trim;

    // Compute which ayah to assign: find the first unassigned ayah
    const assignedAyahs = currentVerses.map(v => v.ayah);
    const targetAyah = currentAyahs.find(a => !assignedAyahs.includes(a));

    console.log('[VerseSegmenter] All Ayahs:', currentAyahs);
    console.log('[VerseSegmenter] Already Assigned:', assignedAyahs);
    console.log('[VerseSegmenter] Target Ayah (computed):', targetAyah);
    console.log('[VerseSegmenter] Current Verses:', currentVerses.map(v => ({ ayah: v.ayah, id: v.annotationId })));

    if (!targetAyah) {
      setError('All ayahs have already been segmented');
      console.warn('[VerseSegmenter] No unassigned ayahs remaining');
      if (annotationManagerRef.current) {
        annotationManagerRef.current.removeAnnotation(ann.id);
      }
      processedAnnotations.current.delete(ann.id);
      return;
    }

    const ayahText = currentAyahTexts.find(a => a.ayah === targetAyah);
    if (!ayahText) {
      setError('Ayah text not found');
      if (annotationManagerRef.current) {
        annotationManagerRef.current.removeAnnotation(ann.id);
      }
      processedAnnotations.current.delete(ann.id);
      return;
    }

    // ALWAYS expand small segments (< 1 second) using VAD
    const duration = ann.end - ann.start;
    let finalStart = ann.start;
    let finalEnd = ann.end;

    if (duration < 1.0) {
      if (!audioBlob) {
        setError('Audio not loaded yet, please wait');
        if (annotationManagerRef.current) {
          annotationManagerRef.current.removeAnnotation(ann.id);
        }
        return;
      }

      setIsCreatingSmartSegment(true);
      try {
        const bounds = await detectSpeechBounds(audioBlob, ann.start, {
          beforeSec: 3,
          afterSec: 5,
        });

        // Ensure bounds are within trim region
        if (currentTrim) {
          finalStart = Math.max(currentTrim.start, bounds.start);
          finalEnd = Math.min(currentTrim.end, bounds.end);
        } else {
          finalStart = bounds.start;
          finalEnd = bounds.end;
        }

        console.log('[VerseSegmenter] VAD expansion:', { original: ann, expanded: { finalStart, finalEnd } });

        // Update the visual annotation FIRST
        if (annotationManagerRef.current) {
          annotationManagerRef.current.updateAnnotation(ann.id, {
            start: finalStart,
            end: finalEnd,
          });
        }
      } catch (vadError) {
        console.warn('VAD failed, using default expansion:', vadError);
        // Fallback: expand to 1 second or to trim end
        finalEnd = Math.min(ann.start + 1.0, currentTrim?.end || Infinity);

        if (annotationManagerRef.current) {
          annotationManagerRef.current.updateAnnotation(ann.id, {
            start: finalStart,
            end: finalEnd,
          });
        }
      } finally {
        setIsCreatingSmartSegment(false);
      }
    }

    // Final validation before adding to store
    const MIN_DURATION = 0.05; // Minimum 50ms
    if (finalEnd - finalStart < MIN_DURATION) {
      console.error('[VerseSegmenter] Duration too short after expansion, using fallback');
      finalEnd = Math.min(finalStart + 1.0, trim?.end || Infinity);

      // Update visual
      if (annotationManagerRef.current) {
        annotationManagerRef.current.updateAnnotation(ann.id, {
          start: finalStart,
          end: finalEnd,
        });
      }
    }

    // Add to store with final expanded bounds
    console.log('[VerseSegmenter] Adding verse to store:', { ayah: targetAyah, finalStart, finalEnd, duration: finalEnd - finalStart });
    const result = addVerse(targetAyah, finalStart, finalEnd, ayahText.text, ann.id);

    if (!result.ok) {
      setError(result.errors.join('\n'));
      console.error('[VerseSegmenter] Validation failed:', result.errors);
      if (annotationManagerRef.current) {
        annotationManagerRef.current.removeAnnotation(ann.id);
      }
    } else {
      setError(null);

      // Update visual label using regions API directly for content
      if (annotationManagerRef.current) {
        const regions = (annotationManagerRef.current as any).regions;
        const region = regions.getRegions().find((r: any) => r.id === ann.id);
        if (region) {
          region.setOptions({ content: `Ayah ${targetAyah}` });
        }
      }

      console.log('[VerseSegmenter] Verse added successfully, auto-selecting next');

      // Get fresh verses from store (the verses variable above is stale)
      const updatedVerses = useWizardStore.getState().verses;
      console.log('[VerseSegmenter] Updated verses after add:', updatedVerses.map(v => ({ ayah: v.ayah, id: v.annotationId })));

      // Auto-select next unfinished ayah using fresh store data
      const nextAyah = currentAyahs.find(a => !updatedVerses.some(v => v.ayah === a));
      console.log('[VerseSegmenter] Auto-selecting next ayah:');
      console.log('  - Available ayahs:', currentAyahs);
      console.log('  - Already segmented:', updatedVerses.map(v => v.ayah));
      console.log('  - Next ayah to select:', nextAyah);

      // BUG FIX #2.4: Use ref to get fresh selectedAyah value (avoid stale closure)
      const currentSelectedAyah = selectedAyahRef.current;
      console.log('  - Previous selectedAyah (from ref):', currentSelectedAyah);

      if (nextAyah !== currentSelectedAyah) {
        console.log('[VerseSegmenter] Setting new selectedAyah:', nextAyah);
        setSelectedAyah(nextAyah || null);
      }
      console.log('[VerseSegmenter] ===== handleCreateAnnotation END =====');
    }
  };

  const handleUpdateAnnotation = (ann: Annotation) => {
    console.log('[VerseSegmenter] handleUpdateAnnotation called:', ann);

    // Get fresh verses from store
    const freshVerses = useWizardStore.getState().verses;
    const verse = freshVerses.find(v => v.annotationId === ann.id);
    console.log('[VerseSegmenter] Found verse for update:', verse);

    if (verse) {
      console.log('[VerseSegmenter] Updating verse in store:', {
        annotationId: ann.id,
        ayah: verse.ayah,
        newStart: ann.start,
        newEnd: ann.end
      });
      updateVerseByAnnotationId(ann.id, { start: ann.start, end: ann.end });
      console.log('[VerseSegmenter] Store updated, verses should re-render');
    } else {
      console.warn('[VerseSegmenter] No verse found for annotation:', ann.id);
    }
  };

  const handleClickSegment = (ann: Annotation) => {
    // Find which ayah this annotation belongs to using fresh state
    const freshVerses = useWizardStore.getState().verses;
    const verse = freshVerses.find(v => v.annotationId === ann.id);
    setEditingSegment({
      annotation: ann,
      currentAyah: verse?.ayah || null,
    });
  };

  const handleDelete = (ayah: number) => {
    console.log('[VerseSegmenter] handleDelete called for ayah:', ayah);
    const freshVerses = useWizardStore.getState().verses;
    const verse = freshVerses.find(v => v.ayah === ayah);
    console.log('[VerseSegmenter] Found verse:', verse);

    if (verse?.annotationId && annotationManagerRef.current) {
      // Remove from visual display
      console.log('[VerseSegmenter] Removing visual annotation:', verse.annotationId);
      annotationManagerRef.current.removeAnnotation(verse.annotationId);
    }
    // Remove from store
    console.log('[VerseSegmenter] Removing from store');
    deleteVerse(ayah);
  };

  const handleDeleteFromDialog = () => {
    console.log('[VerseSegmenter] handleDeleteFromDialog called', editingSegment);
    if (!editingSegment?.currentAyah) {
      console.warn('[VerseSegmenter] No current ayah in editing segment');
      return;
    }
    handleDelete(editingSegment.currentAyah);
    setEditingSegment(null);
  };

  const getAyahStatus = (ayah: number) => {
    return verses.some(v => v.ayah === ayah);
  };

  if (!audioUrl || !trim) {
    return <Alert severity="error">No audio or trim data available</Alert>;
  }

  const constraints = {
    surah: {
      disallowOverlapWithSameKind: true,
      mustBeInsideKind: null,
    },
    word: {},
    other: {},
  };

  return (
    <Stack spacing={3}>
      <Alert severity="info">
        Create one segment for each ayah. Segments cannot overlap and must be
        within the trim bounds [{trim.start.toFixed(3)}s, {trim.end.toFixed(3)}
        s].
      </Alert>

      {error && <Alert severity="error">{error}</Alert>}

      {/* Ayah selector */}
      <Box>
        <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 1 }}>
          <h3 style={{ margin: 0 }}>Select Ayah to Segment:</h3>
          <Tooltip title="Add smart segment at playhead (Ctrl+Click also works)">
            <span>
              <IconButton
                onClick={() => {
                  // This would ideally trigger smart segment creation at current playhead
                  // For now, just prompt user to Ctrl+Click
                  setError('Use Ctrl+Click on the waveform to create a smart segment at the cursor position');
                  setTimeout(() => setError(null), 3000);
                }}
                disabled={!selectedAyah || verses.length >= ayahs.length}
                color="primary"
              >
                <Add />
              </IconButton>
            </span>
          </Tooltip>
        </Stack>

        <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
          {ayahs.map(ayah => {
            const isDone = getAyahStatus(ayah);
            const isSelected = ayah === selectedAyah;

            return (
              <Chip
                key={ayah}
                label={`Ayah ${ayah}`}
                icon={
                  isDone ? (
                    <CheckCircle />
                  ) : (
                    <RadioButtonUnchecked />
                  )
                }
                color={isSelected ? 'primary' : isDone ? 'success' : 'default'}
                onClick={() => !isDone && setSelectedAyah(ayah)}
                variant={isSelected ? 'filled' : 'outlined'}
                disabled={isDone}
                sx={{ mb: 1 }}
              />
            );
          })}
        </Stack>
      </Box>

      {/* Selected ayah preview */}
      {selectedAyah && (
        <Paper sx={{ p: 2, bgcolor: 'primary.50' }}>
          <Chip
            label={`Segmenting Ayah ${selectedAyah}`}
            size="small"
            color="primary"
            sx={{ mb: 1 }}
          />
          <Box sx={{ direction: 'rtl', textAlign: 'right' }}>
            <TajweedText
              htmlText={
                ayahTexts.find(a => a.ayah === selectedAyah)?.text || ''
              }
              fontSize={18}
            />
          </Box>
        </Paper>
      )}

      {/* Waveform annotator */}
      <Box>
        <h3>Audio Waveform</h3>
        {isCreatingSmartSegment && (
          <Alert severity="info" sx={{ mb: 1 }}>
            Detecting speech boundaries...
          </Alert>
        )}
        <WavesurferAnnotator
          src={audioUrl}
          controlledKind="surah"
          constraints={constraints}
          onCreate={handleCreateAnnotation}
          onUpdate={handleUpdateAnnotation}
          onClick={handleClickSegment}
          showMinimap={true}
          managerRef={annotationManagerRef}
        />
        <Alert severity="info" sx={{ mt: 1 }}>
          Drag on the waveform to create a segment for the selected ayah. <strong>Ctrl+Click</strong> for smart segments (auto-detects speech boundaries).
          Click a segment to edit or delete it. <strong>Ctrl+Z</strong> to undo.
        </Alert>
      </Box>

      {/* Verse segments table */}
      {verses.length > 0 && (
        <Box>
          <h3>Segmented Ayahs ({verses.length}/{ayahs.length})</h3>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>Ayah</TableCell>
                <TableCell>Start</TableCell>
                <TableCell>End</TableCell>
                <TableCell>Duration</TableCell>
                <TableCell>Text Preview</TableCell>
                <TableCell align="right">Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {verses.map(v => (
                <TableRow key={v.ayah}>
                  <TableCell>
                    <Chip label={v.ayah} size="small" />
                  </TableCell>
                  <TableCell>{v.start.toFixed(3)}s</TableCell>
                  <TableCell>{v.end.toFixed(3)}s</TableCell>
                  <TableCell>{(v.end - v.start).toFixed(3)}s</TableCell>
                  <TableCell
                    sx={{
                      maxWidth: 300,
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    <Box sx={{ direction: 'rtl', textAlign: 'right' }}>
                      <TajweedText htmlText={v.text} fontSize={14} />
                    </Box>
                  </TableCell>
                  <TableCell align="right">
                    <Tooltip title="Delete segment">
                      <IconButton
                        size="small"
                        color="error"
                        onClick={() => handleDelete(v.ayah)}
                      >
                        <Delete />
                      </IconButton>
                    </Tooltip>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Box>
      )}

      {/* Edit segment dialog */}
      <Dialog open={!!editingSegment} onClose={() => setEditingSegment(null)}>
        <DialogTitle>Segment Details</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1, minWidth: 300 }}>
            <Box>
              <strong>Ayah:</strong> {editingSegment?.currentAyah}
            </Box>
            <Box>
              <strong>Time:</strong> {editingSegment?.annotation.start.toFixed(3)}s - {editingSegment?.annotation.end.toFixed(3)}s
            </Box>
            <Box>
              <strong>Duration:</strong> {editingSegment ? (editingSegment.annotation.end - editingSegment.annotation.start).toFixed(3) : '0'}s
            </Box>
            {editingSegment?.currentAyah && (
              <Box sx={{ direction: 'rtl', textAlign: 'right', p: 2, bgcolor: 'grey.50', borderRadius: 1 }}>
                <TajweedText
                  htmlText={ayahTexts.find(a => a.ayah === editingSegment.currentAyah)?.text || ''}
                  fontSize={16}
                />
              </Box>
            )}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleDeleteFromDialog} color="error" startIcon={<Delete />}>
            Delete
          </Button>
          <Button onClick={() => setEditingSegment(null)} variant="contained">
            Close
          </Button>
        </DialogActions>
      </Dialog>
    </Stack>
  );
};

export default VerseSegmenter;
