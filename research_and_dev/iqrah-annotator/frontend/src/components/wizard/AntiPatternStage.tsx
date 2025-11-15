// Stage 4: Anti-pattern annotation (optional, fine-grained) - COMPLETELY REWORKED
import React, { useState, useEffect, useRef, useMemo } from 'react';
import {
  Stack,
  Alert,
  Chip,
  Box,
  Paper,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  TextField,
  Slider,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  IconButton,
  Tooltip,
  Skeleton,
  Divider,
  Typography,
  Badge,
} from '@mui/material';
import { Delete, Info } from '@mui/icons-material';
import axios from 'axios';
import WavesurferAnnotator from '../WavesurferAnnotator';
import TajweedText from '../TajweedText';
import { useWizardStore } from '../../store/wizardStore';
import { loadRecording } from '../../store/db';
import type { Annotation } from '../../annotation/types';
import { stripHtml } from '../../lib/utils';
import { useAudioSegment } from '../../hooks/useAudioSegment';
import { useAnnotationRestoration } from '../../hooks/useAnnotationRestoration';
import { isDefined, ensureNumber } from '../../utils/defensive';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8000';

interface AntiPatternType {
  name: string;
  display_name: string;
  description: string;
  rule: string;
}

interface WordWithAntiPatterns {
  wordKey: string;
  ayah: number;
  text: string;
  antiPatternTypes: AntiPatternType[];
}

export const AntiPatternStage: React.FC = () => {
  const {
    recordingId,
    words,
    verses,
    antiPatterns,
    activeWordKey,
    setActiveWordKey,
    addAntiPattern,
    updateAntiPattern,
    deleteAntiPattern,
  } = useWizardStore();

  const [fullAudioBlob, setFullAudioBlob] = useState<Blob | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const annotationManagerRef = useRef<any>(null);

  // Filtered words: only those with applicable anti-patterns
  const [wordsWithAntiPatterns, setWordsWithAntiPatterns] = useState<WordWithAntiPatterns[]>([]);

  // Anti-pattern creation state
  const [selectedType, setSelectedType] = useState<string>('');
  const [confidence, setConfidence] = useState(0.9);
  const [notes, setNotes] = useState('');

  const currentWord = words.find(w => w.wordKey === activeWordKey);

  // Determine current ayah - get the ayah that contains words with anti-patterns
  // Prefer the selected word's ayah, fallback to first ayah with anti-patterns
  const currentAyah = currentWord?.ayah ?? wordsWithAntiPatterns[0]?.ayah;
  const currentVerse = verses.find(v => v.ayah === currentAyah);

  // Load full audio blob from IndexedDB
  useEffect(() => {
    if (!recordingId) return;
    loadRecording(recordingId).then(result => {
      if (result) setFullAudioBlob(result.blob);
    });
  }, [recordingId]);

  // Batch fetch anti-pattern types for all words and filter
  useEffect(() => {
    if (words.length === 0) return;

    setLoading(true);
    const wordKeys = words.map(w => w.wordKey);

    axios
      .post(`${API}/api/qpc/words/batch/anti-patterns`, { locations: wordKeys })
      .then(response => {
        const batchResult: Record<string, AntiPatternType[]> = response.data;

        // Filter words that have at least one anti-pattern type
        const filtered: WordWithAntiPatterns[] = words
          .filter(word => {
            const types = batchResult[word.wordKey] || [];
            return types.length > 0;
          })
          .map(word => ({
            wordKey: word.wordKey,
            ayah: word.ayah,
            text: word.text,
            antiPatternTypes: batchResult[word.wordKey] || [],
          }));

        setWordsWithAntiPatterns(filtered);

        // Don't auto-select - let user choose to see full ayah first
        // User can click on a word when ready to annotate
      })
      .catch(err => {
        console.error('Failed to fetch anti-patterns:', err);
        setError('Failed to load anti-pattern types for words');
      })
      .finally(() => setLoading(false));
  }, [words]);

  // Update available types when word changes (memoized to prevent unnecessary resets)
  const availableTypes = useMemo(() => {
    return wordsWithAntiPatterns.find(w => w.wordKey === activeWordKey)?.antiPatternTypes || [];
  }, [wordsWithAntiPatterns, activeWordKey]);

  // Set default type when word changes
  useEffect(() => {
    if (availableTypes.length > 0) {
      setSelectedType(availableTypes[0].name);
    }
  }, [activeWordKey, availableTypes]);

  // Validate word has proper start/end times
  const isValidWord = currentWord &&
    typeof currentWord.start === 'number' &&
    typeof currentWord.end === 'number' &&
    currentWord.end > currentWord.start;

  // Validate verse has proper start/end times
  const isValidVerse = currentVerse &&
    typeof currentVerse.start === 'number' &&
    typeof currentVerse.end === 'number' &&
    currentVerse.end > currentVerse.start;

  // Audio segment for FULL AYAH (default view when no word selected)
  const { audioUrl: ayahAudioUrl, timeOffset: ayahTimeOffset, error: ayahAudioError } = useAudioSegment({
    fullAudioBlob,
    startTime: isValidVerse ? currentVerse.start : 0,
    endTime: isValidVerse ? currentVerse.end : 0,
    enabled: isValidVerse && !!fullAudioBlob && !activeWordKey, // Only when no word selected
  });

  // Audio segment for CURRENT WORD (focused view when word selected)
  const { audioUrl: wordAudioUrl, timeOffset: wordTimeOffset, error: wordAudioError } = useAudioSegment({
    fullAudioBlob,
    startTime: isValidWord ? currentWord.start : 0,
    endTime: isValidWord ? currentWord.end : 0,
    enabled: isValidWord && !!fullAudioBlob && !!activeWordKey, // Only when word selected
  });

  // Choose which audio to display based on selection state
  const displayAudioUrl = activeWordKey ? wordAudioUrl : ayahAudioUrl;
  const displayTimeOffset = activeWordKey ? wordTimeOffset : ayahTimeOffset;

  // Merge audio errors
  useEffect(() => {
    if (wordAudioError) setError(wordAudioError);
    if (ayahAudioError) setError(ayahAudioError);
  }, [wordAudioError, ayahAudioError]);

  // Get anti-patterns for current word
  const currentWordAntiPatterns = currentWord
    ? antiPatterns.filter(ap => ap.wordKey === currentWord.wordKey)
    : [];

  // FIX: Only restore when time offset actually changes, not on every word switch
  // This prevents annotations from disappearing when switching between words
  const isRestoringRef = useAnnotationRestoration({
    manager: annotationManagerRef.current,
    items: currentWordAntiPatterns,
    timeOffset: displayTimeOffset,
    kind: 'other',
    getLabelFn: (ap) => `${ap.type} (${(ap.confidence * 100).toFixed(0)}%)`,
    audioUrl: displayAudioUrl,
    additionalDeps: [displayTimeOffset, currentWord?.wordKey],
  });

  const handleCreateAnnotation = (ann: Annotation) => {
    if (isRestoringRef.current) return;

    const freshState = useWizardStore.getState();
    const freshCurrentWord = freshState.words.find(w => w.wordKey === activeWordKey);

    if (!isDefined(freshCurrentWord)) {
      setError('Please select a word first');
      return;
    }

    if (!isDefined(selectedType) || selectedType === '') {
      setError('Please select an anti-pattern type');
      return;
    }

    // Convert relative to absolute coordinates (with defensive number handling)
    const absoluteStart = ensureNumber(ann.start, 0) + ensureNumber(displayTimeOffset, 0);
    const absoluteEnd = ensureNumber(ann.end, 0) + ensureNumber(displayTimeOffset, 0);

    // FIX: Deduplication - check if annotation with same coordinates already exists
    const tolerance = 0.001; // 1ms tolerance for floating point comparison
    const isDuplicate = freshState.antiPatterns.some(
      ap =>
        ap.wordKey === freshCurrentWord.wordKey &&
        ap.type === selectedType &&
        Math.abs(ap.start - absoluteStart) < tolerance &&
        Math.abs(ap.end - absoluteEnd) < tolerance
    );

    if (isDuplicate) {
      console.warn('[AntiPatternStage] Duplicate annotation detected, ignoring');
      return;
    }

    const result = addAntiPattern(
      freshCurrentWord.wordKey,
      selectedType,
      absoluteStart,
      absoluteEnd,
      confidence,
      notes || undefined
    );

    if (!result.ok) {
      setError(result.errors.join('\n'));
    } else {
      setError(null);
      setNotes('');
    }
  };

  const handleUpdate = (ann: Annotation) => {
    const freshState = useWizardStore.getState();
    const antiPattern = freshState.antiPatterns.find(ap => ap.id === ann.id);

    if (isDefined(antiPattern)) {
      const absoluteStart = ensureNumber(ann.start, 0) + ensureNumber(displayTimeOffset, 0);
      const absoluteEnd = ensureNumber(ann.end, 0) + ensureNumber(displayTimeOffset, 0);
      updateAntiPattern(ann.id, { start: absoluteStart, end: absoluteEnd });
    }
  };

  const handleDelete = (id: string) => {
    if (isDefined(id) && id !== '') {
      deleteAntiPattern(id);
    }
  };

  if (!isDefined(fullAudioBlob)) {
    return <Alert severity="error">No audio available</Alert>;
  }

  if (words.length === 0) {
    return (
      <Alert severity="warning">
        No words have been segmented yet. Please complete Stage 3 first.
      </Alert>
    );
  }

  if (wordsWithAntiPatterns.length === 0 && !loading) {
    return (
      <Alert severity="info">
        <strong>No words with applicable anti-patterns found.</strong>
        <br />
        The selected verses don't contain words with tajweed rules that have defined anti-patterns.
        You can proceed to export.
      </Alert>
    );
  }

  const constraints = {
    surah: {},
    word: {},
    other: {
      disallowOverlapWithSameKind: true,
      mustBeInsideKind: null,
    },
  };

  // Group words by ayah
  const wordsByAyah = wordsWithAntiPatterns.reduce((acc, word) => {
    if (!acc[word.ayah]) acc[word.ayah] = [];
    acc[word.ayah].push(word);
    return acc;
  }, {} as Record<number, WordWithAntiPatterns[]>);

  return (
    <Stack spacing={2}>
      {/* Full Ayah Text Display */}
      {currentVerse && (
        <Paper sx={{ p: 2, bgcolor: 'grey.50' }}>
          <Stack direction="row" spacing={2} alignItems="center" sx={{ mb: 1 }}>
            <Chip label={`Ayah ${currentVerse.ayah}`} size="small" color="primary" />
            <Typography variant="caption" color="text.secondary">
              {activeWordKey ? 'Word selected - showing focused view' : 'No word selected - showing full ayah'}
            </Typography>
          </Stack>
          <Box sx={{ direction: 'rtl', textAlign: 'right' }}>
            <TajweedText htmlText={currentVerse.text} fontSize={18} />
          </Box>
        </Paper>
      )}

      <Box sx={{ display: 'flex', gap: 2, height: '70vh' }}>
        {/* LEFT: Waveform (60%) */}
        <Box sx={{ flex: '0 0 60%', display: 'flex', flexDirection: 'column', gap: 2 }}>
          <Alert severity="info" icon={<Info />}>
            <strong>Optional Stage:</strong> Mark specific tajweed violations within words.
            {!activeWordKey && ' Select a word below to start annotating.'}
          </Alert>

          {error && <Alert severity="error">{error}</Alert>}

          {currentWord && !isValidWord && (
            <Alert severity="error">
              <strong>Invalid word data!</strong> Please go back to Stage 3 and re-segment this word.
            </Alert>
          )}

          {/* Current word context */}
          {currentWord && (
          <Paper sx={{ p: 2, bgcolor: 'info.50', border: '1px solid', borderColor: 'info.200' }}>
            <Stack direction="row" justifyContent="space-between" alignItems="center">
              <Box>
                <Chip label={currentWord.wordKey} size="small" color="primary" sx={{ mb: 1 }} />
                <Box sx={{ direction: 'rtl', textAlign: 'right' }}>
                  <TajweedText htmlText={currentWord.text} fontSize={24} />
                </Box>
              </Box>
              <Box sx={{ textAlign: 'right', fontSize: 12, color: 'text.secondary' }}>
                <strong>Segment:</strong> [{currentWord.start.toFixed(3)}s - {currentWord.end.toFixed(3)}s]
                <br />
                <strong>Duration:</strong> {(currentWord.end - currentWord.start).toFixed(3)}s
                <br />
                <strong>Anti-patterns:</strong> {currentWordAntiPatterns.length}
              </Box>
            </Stack>
          </Paper>
        )}

        {/* Waveform */}
        <Paper sx={{ flex: 1, p: 2, overflow: 'auto' }}>
          <Typography variant="h6" gutterBottom>
            Audio Waveform {activeWordKey ? '(Word Focus)' : '(Full Ayah)'}
          </Typography>
          {displayAudioUrl ? (
            <>
              <Alert severity="info" sx={{ mb: 2, fontSize: 12 }}>
                {activeWordKey
                  ? 'Showing focused view of selected word. Drag on waveform to mark anti-pattern regions.'
                  : 'Showing full ayah audio. Select a word from the sidebar to focus and annotate.'}
              </Alert>
              <WavesurferAnnotator
                src={displayAudioUrl}
                controlledKind="other"
                constraints={constraints}
                onCreate={handleCreateAnnotation}
                onUpdate={handleUpdate}
                showMinimap={true}
                managerRef={annotationManagerRef}
              />
            </>
          ) : (
            <Stack spacing={2} sx={{ p: 2 }}>
              <Skeleton variant="rectangular" height={40} />
              <Skeleton variant="rectangular" height={250} />
            </Stack>
          )}
        </Paper>
      </Box>

      {/* RIGHT: Controls & Word List (40%) */}
      <Box sx={{ flex: '0 0 38%', display: 'flex', flexDirection: 'column', gap: 2, overflow: 'auto', maxHeight: 'calc(70vh - 16px)' }}>
        {/* Word Selector */}
        <Paper sx={{ p: 2 }}>
          <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 1 }}>
            <Typography variant="h6">
              Words with Anti-Patterns ({wordsWithAntiPatterns.length})
            </Typography>
            {activeWordKey && (
              <Chip
                label="Clear Selection"
                size="small"
                onClick={() => setActiveWordKey(null)}
                onDelete={() => setActiveWordKey(null)}
                color="secondary"
                variant="outlined"
              />
            )}
          </Stack>
          <Divider sx={{ mb: 2 }} />

          {loading ? (
            <Stack spacing={1}>
              {[...Array(3)].map((_, i) => (
                <Skeleton key={i} variant="rectangular" height={48} />
              ))}
            </Stack>
          ) : (
            <Stack spacing={2}>
              {Object.entries(wordsByAyah).map(([ayah, ayahWords]) => (
                <Box key={ayah}>
                  <Chip
                    label={`Ayah ${ayah}`}
                    size="small"
                    variant="outlined"
                    sx={{ mb: 1 }}
                  />
                  <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                    {ayahWords.map(word => {
                      const isSelected = word.wordKey === activeWordKey;
                      const apCount = antiPatterns.filter(
                        ap => ap.wordKey === word.wordKey
                      ).length;

                      return (
                        <Tooltip
                          key={word.wordKey}
                          title={`${word.antiPatternTypes.length} applicable anti-pattern type(s)`}
                          arrow
                        >
                          <Badge
                            badgeContent={apCount}
                            color="warning"
                            invisible={apCount === 0}
                          >
                            <Chip
                              label={stripHtml(word.text)}
                              color={isSelected ? 'primary' : 'default'}
                              onClick={() => setActiveWordKey(word.wordKey)}
                              variant={isSelected ? 'filled' : 'outlined'}
                              size="medium"
                            />
                          </Badge>
                        </Tooltip>
                      );
                    })}
                  </Stack>
                </Box>
              ))}
            </Stack>
          )}
        </Paper>

        {/* Anti-pattern Configuration */}
        {availableTypes.length > 0 && (
          <Paper sx={{ p: 2 }}>
            <Typography variant="h6" gutterBottom>
              Anti-Pattern Configuration
            </Typography>
            <Divider sx={{ mb: 2 }} />

            <Stack spacing={2}>
              <FormControl fullWidth size="small">
                <InputLabel>Type</InputLabel>
                <Select
                  value={selectedType}
                  onChange={e => setSelectedType(e.target.value)}
                  label="Type"
                >
                  {availableTypes.map(type => (
                    <MenuItem key={type.name} value={type.name}>
                      <Stack>
                        <Typography variant="body2">{type.display_name}</Typography>
                        <Typography variant="caption" color="text.secondary">
                          {type.rule}
                        </Typography>
                      </Stack>
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>

              {selectedType && (
                <Alert severity="info" sx={{ fontSize: 11, py: 0.5 }}>
                  {availableTypes.find(t => t.name === selectedType)?.description}
                </Alert>
              )}

              <Tooltip title="How confident are you that this anti-pattern exists? Use higher values (80-100%) for clear violations, lower values (50-70%) for uncertain cases.">
                <Box>
                  <Typography variant="caption" color="text.secondary" gutterBottom>
                    Confidence: {(confidence * 100).toFixed(0)}%
                  </Typography>
                  <Slider
                    value={confidence}
                    onChange={(_, v) => setConfidence(v as number)}
                    min={0}
                    max={1}
                    step={0.1}
                    marks={[
                      { value: 0.5, label: '50%' },
                      { value: 0.9, label: '90%' },
                    ]}
                    size="small"
                  />
                </Box>
              </Tooltip>

              <TextField
                label="Notes (optional)"
                multiline
                rows={2}
                value={notes}
                onChange={e => setNotes(e.target.value)}
                placeholder="Additional context..."
                size="small"
              />
            </Stack>
          </Paper>
        )}

        {/* Current Word Anti-Patterns */}
        {currentWordAntiPatterns.length > 0 && (
          <Paper sx={{ p: 2, flex: 1, overflow: 'auto' }}>
            <Typography variant="h6" gutterBottom>
              Annotations ({currentWordAntiPatterns.length})
            </Typography>
            <Divider sx={{ mb: 2 }} />

            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>Type</TableCell>
                  <TableCell>Time</TableCell>
                  <TableCell>Conf.</TableCell>
                  <TableCell align="right">Actions</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {currentWordAntiPatterns.map(ap => {
                  const typeInfo = availableTypes.find(t => t.name === ap.type);

                  return (
                    <TableRow key={ap.id}>
                      <TableCell>
                        <Tooltip title={ap.notes || 'No notes'}>
                          <Chip
                            label={typeInfo?.display_name || ap.type}
                            size="small"
                            color="warning"
                            sx={{ fontSize: 10 }}
                          />
                        </Tooltip>
                      </TableCell>
                      <TableCell sx={{ fontSize: 11 }}>
                        {ap.start.toFixed(3)}-{ap.end.toFixed(3)}s
                        <br />
                        <Typography variant="caption" color="text.secondary">
                          ({(ap.end - ap.start).toFixed(3)}s)
                        </Typography>
                      </TableCell>
                      <TableCell>{(ap.confidence * 100).toFixed(0)}%</TableCell>
                      <TableCell align="right">
                        <Tooltip title="Delete">
                          <IconButton
                            size="small"
                            color="error"
                            onClick={() => handleDelete(ap.id)}
                          >
                            <Delete fontSize="small" />
                          </IconButton>
                        </Tooltip>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </Paper>
        )}

        {/* Summary */}
        <Alert severity="success" icon={<Info />}>
          <strong>Total:</strong> {antiPatterns.length} anti-pattern(s) across{' '}
          {new Set(antiPatterns.map(ap => ap.wordKey)).size} word(s)
        </Alert>
      </Box>
    </Box>
    </Stack>
  );
};

export default AntiPatternStage;
