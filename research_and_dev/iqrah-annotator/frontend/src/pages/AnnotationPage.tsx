/**
 * Main Annotation Interface
 */

import React, { useState, useEffect } from 'react';
import {
  Box,
  Button,
  Container,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Paper,
  Typography,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Alert,
  Chip,
  LinearProgress,
} from '@mui/material';
import {
  CloudUpload,
  Delete,
  Edit,
  Save,
} from '@mui/icons-material';
import WaveformPlayer from '../components/WaveformPlayer';
import {
  createRecording,
  uploadAudio,
  getRecordingRegions,
  createRegion,
  updateRegion,
  deleteRegion,
} from '../api/client';
import type { Recording, Region, RegionCreate } from '../api/client';

// Constants
const RULES = ['ghunnah', 'qalqalah'];
const ANTI_PATTERNS: Record<string, string[]> = {
  ghunnah: ['weak-ghunnah', 'no-ghunnah'],
  qalqalah: ['no-qalqalah', 'weak-qalqalah'],
};
const REGION_LABELS: Record<string, string[]> = {
  ghunnah: ['weak-ghunnah-onset', 'weak-ghunnah-sustain'],
  qalqalah: ['no-qalqalah', 'burst-misaligned'],
};
const SAMPLE_RATES = [16000, 22050, 44100];

interface RegionEdit extends Region {
  isNew?: boolean;
}

const AnnotationPage: React.FC = () => {
  // Form state
  const [rule, setRule] = useState('ghunnah');
  const [antiPattern, setAntiPattern] = useState('weak-ghunnah');
  const [qpcLocation, setQpcLocation] = useState('');
  const [sampleRate, setSampleRate] = useState(16000);

  // Audio state
  const [audioFile, setAudioFile] = useState<File | null>(null);
  const [audioDuration, setAudioDuration] = useState(0);

  // Recording state
  const [recording, setRecording] = useState<Recording | null>(null);
  const [regions, setRegions] = useState<RegionEdit[]>([]);
  const [selectedRegion, setSelectedRegion] = useState<RegionEdit | null>(null);

  // UI state
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [editDialog, setEditDialog] = useState(false);

  // Region edit form
  const [editLabel, setEditLabel] = useState('');
  const [editConfidence, setEditConfidence] = useState(0.9);
  const [editNotes, setEditNotes] = useState('');

  // Load audio duration when file changes
  useEffect(() => {
    if (!audioFile) return;

    const audio = new Audio();
    audio.src = URL.createObjectURL(audioFile);

    audio.addEventListener('loadedmetadata', () => {
      setAudioDuration(audio.duration);
    });

    return () => URL.revokeObjectURL(audio.src);
  }, [audioFile]);

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setAudioFile(file);
      setError(null);
    }
  };

  const handleCreateRecording = async () => {
    if (!audioFile) {
      setError('Please select an audio file');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      // Create recording
      const newRecording = await createRecording({
        rule,
        anti_pattern: antiPattern,
        qpc_location: qpcLocation || undefined,
        sample_rate: sampleRate,
        duration_sec: audioDuration,
      });

      // Upload audio
      await uploadAudio(newRecording.id, audioFile);

      setRecording(newRecording);
      setSuccess('Recording created successfully! Now add annotation regions by dragging on the waveform.');
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to create recording');
    } finally {
      setIsLoading(false);
    }
  };

  const handleRegionCreate = async (regionData: { start: number; end: number }) => {
    if (!recording) return;

    // Add temporary region
    const tempRegion: RegionEdit = {
      id: Date.now(), // temporary ID
      recording_id: recording.id,
      start_sec: regionData.start,
      end_sec: regionData.end,
      label: REGION_LABELS[rule][0], // Default label
      confidence: 0.9,
      created_at: new Date().toISOString(),
      isNew: true,
    };

    setRegions([...regions, tempRegion]);
    setSelectedRegion(tempRegion);
    setEditLabel(tempRegion.label);
    setEditConfidence(0.9);
    setEditNotes('');
    setEditDialog(true);
  };

  const handleRegionClick = (region: Region) => {
    const regionEdit = regions.find((r) => r.id === region.id);
    if (regionEdit) {
      setSelectedRegion(regionEdit);
      setEditLabel(regionEdit.label);
      setEditConfidence(regionEdit.confidence || 0.9);
      setEditNotes(regionEdit.notes || '');
      setEditDialog(true);
    }
  };

  const handleSaveRegion = async () => {
    if (!selectedRegion || !recording) return;

    setIsLoading(true);
    setError(null);

    try {
      if (selectedRegion.isNew) {
        // Create new region
        const newRegion = await createRegion({
          recording_id: recording.id,
          start_sec: selectedRegion.start_sec,
          end_sec: selectedRegion.end_sec,
          label: editLabel,
          confidence: editConfidence,
          notes: editNotes || undefined,
        });

        setRegions(regions.map((r) => (r.id === selectedRegion.id ? newRegion : r)));
      } else {
        // Update existing region
        const updated = await updateRegion(selectedRegion.id, {
          label: editLabel,
          confidence: editConfidence,
          notes: editNotes || undefined,
        });

        setRegions(regions.map((r) => (r.id === selectedRegion.id ? updated : r)));
      }

      setEditDialog(false);
      setSelectedRegion(null);
      setSuccess('Region saved successfully!');
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to save region');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDeleteRegion = async (regionId: number) => {
    if (!confirm('Are you sure you want to delete this region?')) return;

    setIsLoading(true);
    setError(null);

    try {
      const region = regions.find((r) => r.id === regionId);

      if (!region?.isNew) {
        await deleteRegion(regionId);
      }

      setRegions(regions.filter((r) => r.id !== regionId));
      setSuccess('Region deleted successfully!');
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to delete region');
    } finally {
      setIsLoading(false);
    }
  };

  const handleReset = () => {
    setRecording(null);
    setRegions([]);
    setAudioFile(null);
    setQpcLocation('');
    setError(null);
    setSuccess(null);
  };

  return (
    <Container maxWidth="xl" sx={{ py: 4 }}>
      <Typography variant="h4" gutterBottom>
        Tajweed Annotation Tool
      </Typography>

      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {success && (
        <Alert severity="success" onClose={() => setSuccess(null)} sx={{ mb: 2 }}>
          {success}
        </Alert>
      )}

      <Stack spacing={3}>
        {/* Recording Setup */}
        {!recording && (
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              1. Create Recording
            </Typography>

            <Stack spacing={2}>
              <Stack direction="row" spacing={2}>
                <FormControl fullWidth>
                  <InputLabel>Rule</InputLabel>
                  <Select value={rule} label="Rule" onChange={(e) => {
                    setRule(e.target.value);
                    setAntiPattern(ANTI_PATTERNS[e.target.value][0]);
                  }}>
                    {RULES.map((r) => (
                      <MenuItem key={r} value={r}>
                        {r}
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>

                <FormControl fullWidth>
                  <InputLabel>Anti-Pattern</InputLabel>
                  <Select
                    value={antiPattern}
                    label="Anti-Pattern"
                    onChange={(e) => setAntiPattern(e.target.value)}
                  >
                    {ANTI_PATTERNS[rule].map((ap) => (
                      <MenuItem key={ap} value={ap}>
                        {ap}
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>
              </Stack>

              <Stack direction="row" spacing={2}>
                <TextField
                  label="QPC Location (optional)"
                  placeholder="e.g., 89:27:3"
                  value={qpcLocation}
                  onChange={(e) => setQpcLocation(e.target.value)}
                  fullWidth
                />

                <FormControl fullWidth>
                  <InputLabel>Sample Rate</InputLabel>
                  <Select
                    value={sampleRate}
                    label="Sample Rate"
                    onChange={(e) => setSampleRate(Number(e.target.value))}
                  >
                    {SAMPLE_RATES.map((sr) => (
                      <MenuItem key={sr} value={sr}>
                        {sr} Hz
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>
              </Stack>

              <Box>
                <Button
                  variant="outlined"
                  component="label"
                  startIcon={<CloudUpload />}
                  fullWidth
                >
                  {audioFile ? audioFile.name : 'Select Audio File (WAV/WebM)'}
                  <input
                    type="file"
                    hidden
                    accept="audio/wav,audio/webm"
                    onChange={handleFileSelect}
                  />
                </Button>
                {audioFile && (
                  <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>
                    Duration: {audioDuration.toFixed(2)}s
                  </Typography>
                )}
              </Box>

              <Button
                variant="contained"
                onClick={handleCreateRecording}
                disabled={!audioFile || isLoading}
                fullWidth
              >
                {isLoading ? 'Creating...' : 'Create Recording & Upload Audio'}
              </Button>
            </Stack>
          </Paper>
        )}

        {/* Annotation Interface */}
        {recording && (
          <>
            <Paper sx={{ p: 3 }}>
              <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
                <Box>
                  <Typography variant="h6">
                    2. Annotate Regions
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Recording #{recording.id} - {recording.rule} / {recording.anti_pattern}
                  </Typography>
                </Box>
                <Button variant="outlined" onClick={handleReset}>
                  New Recording
                </Button>
              </Stack>

              {isLoading && <LinearProgress sx={{ mb: 2 }} />}

              <WaveformPlayer
                audioFile={audioFile || undefined}
                regions={regions}
                onRegionCreate={handleRegionCreate}
                onRegionClick={handleRegionClick}
                height={150}
              />
            </Paper>

            {/* Regions List */}
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                Annotation Regions ({regions.length})
              </Typography>

              {regions.length === 0 ? (
                <Typography variant="body2" color="text.secondary">
                  No regions yet. Click and drag on the waveform to create regions.
                </Typography>
              ) : (
                <TableContainer>
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell>Start</TableCell>
                        <TableCell>End</TableCell>
                        <TableCell>Label</TableCell>
                        <TableCell>Confidence</TableCell>
                        <TableCell>Notes</TableCell>
                        <TableCell>Status</TableCell>
                        <TableCell align="right">Actions</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {regions.map((region) => (
                        <TableRow key={region.id}>
                          <TableCell>{region.start_sec.toFixed(2)}s</TableCell>
                          <TableCell>{region.end_sec.toFixed(2)}s</TableCell>
                          <TableCell>{region.label}</TableCell>
                          <TableCell>
                            {region.confidence ? (region.confidence * 100).toFixed(0) + '%' : '-'}
                          </TableCell>
                          <TableCell>{region.notes || '-'}</TableCell>
                          <TableCell>
                            {region.isNew ? (
                              <Chip label="Unsaved" color="warning" size="small" />
                            ) : (
                              <Chip label="Saved" color="success" size="small" />
                            )}
                          </TableCell>
                          <TableCell align="right">
                            <IconButton
                              size="small"
                              onClick={() => handleRegionClick(region)}
                            >
                              <Edit />
                            </IconButton>
                            <IconButton
                              size="small"
                              onClick={() => handleDeleteRegion(region.id)}
                            >
                              <Delete />
                            </IconButton>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              )}
            </Paper>
          </>
        )}
      </Stack>

      {/* Edit Region Dialog */}
      <Dialog open={editDialog} onClose={() => setEditDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>
          {selectedRegion?.isNew ? 'Create Region' : 'Edit Region'}
        </DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <TextField
              label="Start Time (seconds)"
              type="number"
              value={selectedRegion?.start_sec.toFixed(2) || 0}
              disabled
              fullWidth
            />
            <TextField
              label="End Time (seconds)"
              type="number"
              value={selectedRegion?.end_sec.toFixed(2) || 0}
              disabled
              fullWidth
            />
            <FormControl fullWidth>
              <InputLabel>Label</InputLabel>
              <Select
                value={editLabel}
                label="Label"
                onChange={(e) => setEditLabel(e.target.value)}
              >
                {REGION_LABELS[rule]?.map((label) => (
                  <MenuItem key={label} value={label}>
                    {label}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
            <TextField
              label="Confidence"
              type="number"
              value={editConfidence}
              onChange={(e) => setEditConfidence(Number(e.target.value))}
              inputProps={{ min: 0, max: 1, step: 0.1 }}
              fullWidth
            />
            <TextField
              label="Notes (optional)"
              value={editNotes}
              onChange={(e) => setEditNotes(e.target.value)}
              multiline
              rows={3}
              fullWidth
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setEditDialog(false)}>Cancel</Button>
          <Button onClick={handleSaveRegion} variant="contained" startIcon={<Save />}>
            Save Region
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
  );
};

export default AnnotationPage;
