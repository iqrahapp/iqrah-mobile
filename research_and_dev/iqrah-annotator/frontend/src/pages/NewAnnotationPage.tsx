/**
 * Ayah-Based Annotation Workflow
 * 1. Select Surah → 2. Select Ayah Range → 3. Choose Target Rules (optional) → 4. Record → 5. Annotate
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  Box,
  Button,
  Container,
  Paper,
  Typography,
  Stack,
  Stepper,
  Step,
  StepLabel,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Alert,
  Chip,
  TextField,
  OutlinedInput,
} from '@mui/material';
import type { SelectChangeEvent } from '@mui/material/Select';
import { ArrowBack, ArrowForward, Save } from '@mui/icons-material';
import MicrophoneRecorder from '../components/MicrophoneRecorder';
import WaveformPlayer from '../components/WaveformPlayer';
import TajweedText from '../components/TajweedText';
import axios from 'axios';
import {
  createRecording,
  uploadAudio,
  createRegion,
  updateRegion,
  deleteRegion,
} from '../api/client';
import type { Recording, Region, RegionCreate } from '../api/client';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000';

interface SurahInfo {
  surah: number;
  name_arabic: string;
  name_english: string;
  ayah_count: number;
  word_count: number;
}

interface RuleInfo {
  name: string;
  display_name: string;
  description: string;
}

interface AntiPattern {
  name: string;
  display_name: string;
  description: string;
}

interface RegionLabel {
  name: string;
  display_name: string;
  description: string;
}

interface AyahText {
  surah: number;
  ayah: number;
  text: string;
  rules: string[];
}

interface RegionEdit extends Region {
  isNew?: boolean;
}

const steps = [
  'Select Surah',
  'Select Ayahs',
  'Choose Target Rules',
  'Record Audio',
  'Annotate Regions',
];

const NewAnnotationPage: React.FC = () => {
  const [activeStep, setActiveStep] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Step 0: Surah Selection
  const [availableSurahs, setAvailableSurahs] = useState<SurahInfo[]>([]);
  const [selectedSurah, setSelectedSurah] = useState<number | null>(null);

  // Step 1: Ayah Selection (visual with checkboxes)
  const [availableAyahs, setAvailableAyahs] = useState<AyahText[]>([]);
  const [selectedAyahs, setSelectedAyahs] = useState<number[]>([]);

  // Step 2: Target Rules (optional multi-select)
  const [availableRules, setAvailableRules] = useState<RuleInfo[]>([]);
  const [selectedRules, setSelectedRules] = useState<string[]>([]);
  const [availableAntiPatterns, setAvailableAntiPatterns] = useState<AntiPattern[]>([]);
  const [availableRegionLabels, setAvailableRegionLabels] = useState<RegionLabel[]>([]);

  // Step 3: Recording
  const [audioBlob, setAudioBlob] = useState<Blob | null>(null);
  const [audioDuration, setAudioDuration] = useState<number>(0);

  // Step 4: Annotation
  const [recording, setRecording] = useState<Recording | null>(null);
  const [regions, setRegions] = useState<RegionEdit[]>([]);
  const [selectedRegion, setSelectedRegion] = useState<Region | null>(null);

  // Load surahs on mount
  useEffect(() => {
    loadSurahs();
    loadRules();
  }, []);

  const loadSurahs = async () => {
    try {
      setIsLoading(true);
      const response = await axios.get<SurahInfo[]>(`${API_URL}/api/qpc/surahs`);
      setAvailableSurahs(response.data);
      setError(null);
    } catch (err: any) {
      setError(`Failed to load surahs: ${err.message}`);
    } finally {
      setIsLoading(false);
    }
  };

  const loadRules = async () => {
    try {
      const response = await axios.get<RuleInfo[]>(`${API_URL}/api/taxonomy/rules`);
      setAvailableRules(response.data);
    } catch (err: any) {
      console.error('Failed to load rules:', err);
    }
  };

  const loadAntiPatterns = async (rule: string) => {
    try {
      const response = await axios.get<AntiPattern[]>(`${API_URL}/api/taxonomy/anti-patterns/${rule}`);
      setAvailableAntiPatterns(response.data);
    } catch (err: any) {
      console.error('Failed to load anti-patterns:', err);
    }
  };

  const loadRegionLabels = async (rule: string) => {
    try {
      const response = await axios.get<RegionLabel[]>(`${API_URL}/api/taxonomy/region-labels/${rule}`);
      setAvailableRegionLabels(response.data);
    } catch (err: any) {
      console.error('Failed to load region labels:', err);
    }
  };

  const handleSurahSelect = async (surah: number) => {
    setSelectedSurah(surah);
    setSelectedAyahs([]);
    setAvailableAyahs([]);

    // Load ayahs for this surah
    try {
      setIsLoading(true);
      const response = await axios.get<AyahText[]>(`${API_URL}/api/qpc/ayahs/${surah}`);
      setAvailableAyahs(response.data);
    } catch (err: any) {
      setError(`Failed to load ayahs: ${err.message}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleAyahToggle = (ayahNumber: number) => {
    setSelectedAyahs(prev => {
      if (prev.includes(ayahNumber)) {
        return prev.filter(a => a !== ayahNumber);
      } else {
        return [...prev, ayahNumber].sort((a, b) => a - b);
      }
    });
  };

  const handleSelectAllAyahs = () => {
    if (selectedAyahs.length === availableAyahs.length) {
      setSelectedAyahs([]);
    } else {
      setSelectedAyahs(availableAyahs.map(a => a.ayah));
    }
  };

  const handleRulesChange = async (event: SelectChangeEvent<string[]>) => {
    const value = event.target.value;
    const rules = typeof value === 'string' ? value.split(',') : value;
    setSelectedRules(rules);

    // Load anti-patterns and region labels for first selected rule
    if (rules.length > 0) {
      await loadAntiPatterns(rules[0]);
      await loadRegionLabels(rules[0]);
    }
  };

  const handleRecordingComplete = (blob: Blob, duration: number) => {
    setAudioBlob(blob);
    setAudioDuration(duration);
    setSuccess('Recording complete! Moving to annotation step...');
    setTimeout(() => {
      setActiveStep(4); // Move to annotation step
      setSuccess(null);
    }, 1000);
  };

  const regionIdCounter = useRef(1);

  const handleRegionCreate = useCallback((newRegion: { start: number; end: number }) => {
    const region: RegionEdit = {
      id: regionIdCounter.current++, // Stable incrementing ID
      recording_id: recording?.id || 0,
      start_sec: newRegion.start,
      end_sec: newRegion.end,
      label: '',
      confidence: 0.8,
      notes: '',
      isNew: true,
      created_at: new Date().toISOString(),
    };
    setRegions(prev => [...prev, region]);
  }, [recording?.id]);

  const handleRegionClick = useCallback((region: Region) => {
    setSelectedRegion(region);
  }, []);

  const handleRegionUpdate = async (regionId: number, updates: Partial<Region>) => {
    const region = regions.find((r) => r.id === regionId);
    if (!region) return;

    if (region.isNew) {
      // Not saved yet, just update locally
      setRegions(regions.map((r) => (r.id === regionId ? { ...r, ...updates } : r)));
    } else {
      // Update in backend
      try {
        await updateRegion(regionId, updates);
        setRegions(regions.map((r) => (r.id === regionId ? { ...r, ...updates } : r)));
        setSuccess('Region updated successfully');
      } catch (err: any) {
        setError(`Failed to update region: ${err.message}`);
      }
    }
  };

  const handleRegionDelete = async (regionId: number) => {
    const region = regions.find((r) => r.id === regionId);
    if (!region) return;

    if (region.isNew) {
      // Not saved yet, just remove locally
      setRegions(regions.filter((r) => r.id !== regionId));
    } else {
      // Delete from backend
      try {
        await deleteRegion(regionId);
        setRegions(regions.filter((r) => r.id !== regionId));
        setSuccess('Region deleted successfully');
      } catch (err: any) {
        setError(`Failed to delete region: ${err.message}`);
      }
    }
  };

  const handleSaveAnnotation = async () => {
    if (!audioBlob || !selectedSurah) {
      setError('Missing required data');
      return;
    }

    try {
      setIsLoading(true);

      // Create location string: "surah:ayah1,ayah2,ayah3" or "surah:ayah1-ayah3" if contiguous
      const isContiguous = selectedAyahs.every((ayah, idx) =>
        idx === 0 || ayah === selectedAyahs[idx - 1] + 1
      );
      const qpcLocation = isContiguous && selectedAyahs.length > 1
        ? `${selectedSurah}:${selectedAyahs[0]}-${selectedAyahs[selectedAyahs.length - 1]}`
        : `${selectedSurah}:${selectedAyahs.join(',')}`;

      // For now, use first selected rule or 'general'
      const rule = selectedRules[0] || 'general';
      const antiPattern = 'general-violation'; // Can enhance later

      // Create recording
      const newRecording = await createRecording({
        rule,
        anti_pattern: antiPattern,
        qpc_location: qpcLocation,
        sample_rate: 16000,
        duration_sec: audioDuration,
      });

      // Upload audio (cast Blob to File for API compatibility)
      await uploadAudio(newRecording.id, new File([audioBlob], 'recording.webm', { type: audioBlob.type }));

      // Save all regions
      for (const region of regions) {
        if (region.isNew) {
          const regionData: RegionCreate = {
            recording_id: newRecording.id,
            start_sec: region.start_sec,
            end_sec: region.end_sec,
            label: region.label || 'violation',
            confidence: region.confidence,
            notes: region.notes,
          };
          await createRegion(regionData);
        }
      }

      setSuccess('Annotation saved successfully!');
      setRecording(newRecording);

      // Reset after 2 seconds
      setTimeout(() => {
        resetForm();
      }, 2000);
    } catch (err: any) {
      setError(`Failed to save annotation: ${err.message}`);
    } finally {
      setIsLoading(false);
    }
  };

  const resetForm = () => {
    setActiveStep(0);
    setSelectedSurah(null);
    setAvailableAyahs([]);
    setSelectedAyahs([]);
    setSelectedRules([]);
    setAudioBlob(null);
    setAudioDuration(0);
    setRegions([]);
    setRecording(null);
    setError(null);
    setSuccess(null);
  };

  const handleNext = () => {
    setActiveStep((prev) => prev + 1);
  };

  const handleBack = () => {
    setActiveStep((prev) => prev - 1);
  };

  const canProceed = () => {
    switch (activeStep) {
      case 0:
        return selectedSurah !== null && availableAyahs.length > 0;
      case 1:
        return selectedAyahs.length > 0;
      case 2:
        return true; // Rules are optional
      case 3:
        return audioBlob !== null;
      case 4:
        return regions.length > 0;
      default:
        return false;
    }
  };

  const selectedSurahInfo = availableSurahs.find((s) => s.surah === selectedSurah);

  return (
    <Container maxWidth="lg" sx={{ py: 4 }}>
      <Typography variant="h4" gutterBottom>
        New Annotation (Ayah-Based)
      </Typography>

      <Stepper activeStep={activeStep} sx={{ mb: 4 }}>
        {steps.map((label) => (
          <Step key={label}>
            <StepLabel>{label}</StepLabel>
          </Step>
        ))}
      </Stepper>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {success && (
        <Alert severity="success" sx={{ mb: 2 }} onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      <Paper sx={{ p: 3 }}>
        {/* Step 0: Select Surah */}
        {activeStep === 0 && (
          <Box>
            <Typography variant="h6" gutterBottom>
              Select Surah
            </Typography>
            <FormControl fullWidth sx={{ mt: 2 }}>
              <InputLabel>Surah</InputLabel>
              <Select
                value={selectedSurah || ''}
                label="Surah"
                onChange={(e) => handleSurahSelect(Number(e.target.value))}
              >
                {availableSurahs.map((surah) => (
                  <MenuItem key={surah.surah} value={surah.surah}>
                    {surah.surah}. {surah.name_arabic} ({surah.name_english}) - {surah.ayah_count} ayahs
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Box>
        )}

        {/* Step 1: Select Ayahs (Visual Selection) */}
        {activeStep === 1 && (
          <Box>
            <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
              <Typography variant="h6">
                Select Ayahs ({selectedAyahs.length} selected)
              </Typography>
              <Button
                variant="outlined"
                size="small"
                onClick={handleSelectAllAyahs}
              >
                {selectedAyahs.length === availableAyahs.length ? 'Deselect All' : 'Select All'}
              </Button>
            </Stack>
            {isLoading && <Typography>Loading ayahs...</Typography>}
            {availableAyahs.length === 0 && !isLoading && (
              <Alert severity="info">No ayahs loaded. Please go back and select a surah.</Alert>
            )}
            <Stack spacing={1} sx={{ maxHeight: '500px', overflowY: 'auto', pr: 1 }}>
              {availableAyahs.map((ayah) => (
                <Paper
                  key={ayah.ayah}
                  sx={{
                    p: 2,
                    cursor: 'pointer',
                    border: '2px solid',
                    borderColor: selectedAyahs.includes(ayah.ayah) ? 'primary.main' : 'grey.300',
                    bgcolor: selectedAyahs.includes(ayah.ayah) ? 'primary.50' : 'white',
                    '&:hover': {
                      borderColor: 'primary.main',
                      bgcolor: 'primary.50',
                    },
                  }}
                  onClick={() => handleAyahToggle(ayah.ayah)}
                >
                  <Stack direction="row" spacing={2} alignItems="flex-start">
                    <Box
                      sx={{
                        minWidth: 40,
                        height: 40,
                        borderRadius: '50%',
                        border: '2px solid',
                        borderColor: selectedAyahs.includes(ayah.ayah) ? 'primary.main' : 'grey.400',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontWeight: 'bold',
                        bgcolor: selectedAyahs.includes(ayah.ayah) ? 'primary.main' : 'transparent',
                        color: selectedAyahs.includes(ayah.ayah) ? 'white' : 'text.primary',
                      }}
                    >
                      {ayah.ayah}
                    </Box>
                    <Box sx={{ flex: 1 }}>
                      <TajweedText htmlText={ayah.text} fontSize={20} />
                    </Box>
                  </Stack>
                </Paper>
              ))}
            </Stack>
          </Box>
        )}

        {/* Step 2: Choose Target Rules */}
        {activeStep === 2 && (
          <Box>
            <Typography variant="h6" gutterBottom>
              Choose Target Rules (Optional)
            </Typography>
            <Typography variant="body2" color="text.secondary" gutterBottom>
              Select which tajweed rules you want to focus on. Leave empty for general practice.
            </Typography>
            <FormControl fullWidth sx={{ mt: 2 }}>
              <InputLabel>Target Rules</InputLabel>
              <Select
                multiple
                value={selectedRules}
                onChange={handleRulesChange}
                input={<OutlinedInput label="Target Rules" />}
                renderValue={(selected) => (
                  <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                    {selected.map((value) => {
                      const rule = availableRules.find(r => r.name === value);
                      return (
                        <Chip key={value} label={rule?.display_name || value} size="small" />
                      );
                    })}
                  </Box>
                )}
              >
                {availableRules.map((rule) => (
                  <MenuItem key={rule.name} value={rule.name}>
                    {rule.display_name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Box>
        )}

        {/* Step 3: Record Audio */}
        {activeStep === 3 && (
          <Box>
            <Typography variant="h6" gutterBottom>
              Record Your Recitation
            </Typography>
            <Alert severity="info" sx={{ mb: 2 }}>
              Recite Surah {selectedSurah}, Ayah{selectedAyahs.length === 1 ? '' : 's'}{' '}
              {selectedAyahs.length <= 3
                ? selectedAyahs.join(', ')
                : `${selectedAyahs[0]}-${selectedAyahs[selectedAyahs.length - 1]}`}
            </Alert>
            <MicrophoneRecorder onRecordingComplete={handleRecordingComplete} />
          </Box>
        )}

        {/* Step 4: Annotate Regions */}
        {activeStep === 4 && audioBlob && (
          <Box>
            <Typography variant="h6" gutterBottom>
              Annotate Violations
            </Typography>
            <Typography variant="body2" color="text.secondary" gutterBottom>
              Drag on the waveform to create annotation regions. Click regions to edit labels.
            </Typography>
            <WaveformPlayer
              audioFile={new File([audioBlob], 'recording.webm')}
              regions={regions}
              onRegionCreate={handleRegionCreate}
              onRegionClick={handleRegionClick}
            />
            <Box sx={{ mt: 2 }}>
              <Typography variant="subtitle2">Regions: {regions.length}</Typography>
              {selectedRegion && (
                <Box sx={{ mt: 1, p: 2, bgcolor: 'grey.100', borderRadius: 1 }}>
                  <Typography variant="body2">
                    Selected: {selectedRegion.start_sec.toFixed(2)}s - {selectedRegion.end_sec.toFixed(2)}s
                  </Typography>
                  <TextField
                    label="Label"
                    value={selectedRegion.label}
                    onChange={(e) => handleRegionUpdate(selectedRegion.id, { label: e.target.value })}
                    size="small"
                    sx={{ mt: 1 }}
                    fullWidth
                  />
                </Box>
              )}
            </Box>
            <Button
              variant="contained"
              color="primary"
              startIcon={<Save />}
              onClick={handleSaveAnnotation}
              disabled={isLoading || regions.length === 0}
              sx={{ mt: 2 }}
              fullWidth
            >
              Save Annotation
            </Button>
          </Box>
        )}

        {/* Navigation Buttons */}
        <Stack direction="row" spacing={2} sx={{ mt: 3 }}>
          <Button
            variant="outlined"
            startIcon={<ArrowBack />}
            onClick={handleBack}
            disabled={activeStep === 0}
          >
            Back
          </Button>
          {activeStep < 3 && (
            <Button
              variant="contained"
              endIcon={<ArrowForward />}
              onClick={handleNext}
              disabled={!canProceed()}
            >
              Next
            </Button>
          )}
          {activeStep === 3 && audioBlob && (
            <Button variant="contained" onClick={() => setActiveStep(4)}>
              Continue to Annotation
            </Button>
          )}
        </Stack>
      </Paper>
    </Container>
  );
};

export default NewAnnotationPage;
