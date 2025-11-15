// Stage 0: Content selection (surah + ayah range)
import React, { useState, useEffect } from 'react';
import {
  Stack,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  TextField,
  Button,
  Box,
  Chip,
  Alert,
  Skeleton,
} from '@mui/material';
import axios from 'axios';
import TajweedText from '../TajweedText';
import { useWizardStore } from '../../store/wizardStore';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8000';

interface Surah {
  surah: number;
  name_arabic: string;
  name_english: string;
  ayah_count: number;
  word_count: number;
}

interface Ayah {
  surah: number;
  ayah: number;
  text: string;
}

export const ContentSelector: React.FC = () => {
  const { surah, ayahs, setSurah, setAyahRange } = useWizardStore();

  const [surahs, setSurahs] = useState<Surah[]>([]);
  const [ayahTexts, setAyahTexts] = useState<Ayah[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [startAyah, setStartAyah] = useState(1);
  const [endAyah, setEndAyah] = useState(1);

  // Fetch surahs on mount
  useEffect(() => {
    setLoading(true);
    axios
      .get(`${API}/api/qpc/surahs`)
      .then(r => {
        setSurahs(r.data);
        setError(null);
      })
      .catch(err => {
        console.error('Failed to fetch surahs:', err);
        setError('Failed to load surahs. Is the backend running?');
      })
      .finally(() => setLoading(false));
  }, []);

  // Fetch ayah texts when surah changes
  useEffect(() => {
    if (!surah) return;

    setLoading(true);
    axios
      .get(`${API}/api/qpc/ayahs/${surah}`)
      .then(r => {
        setAyahTexts(r.data);
        setError(null);

        // Reset range to first ayah
        if (r.data.length > 0) {
          setStartAyah(r.data[0].ayah);
          setEndAyah(r.data[0].ayah);
        }
      })
      .catch(err => {
        console.error('Failed to fetch ayahs:', err);
        setError(`Failed to load ayahs for surah ${surah}`);
      })
      .finally(() => setLoading(false));
  }, [surah]);

  const selectedSurah = surahs.find(s => s.surah === surah);

  const handleSurahChange = (newSurah: number) => {
    setSurah(newSurah);
  };

  const handleApplyRange = () => {
    if (!surah || startAyah > endAyah) return;

    const selectedTexts = ayahTexts
      .filter(a => a.ayah >= startAyah && a.ayah <= endAyah)
      .map(a => ({ ayah: a.ayah, text: a.text }));

    setAyahRange(startAyah, endAyah, selectedTexts);
  };

  if (loading && surahs.length === 0) {
    return (
      <Stack spacing={3}>
        <Skeleton variant="rectangular" height={40} />
        <Skeleton variant="rectangular" height={56} />
        <Stack direction="row" spacing={2}>
          <Skeleton variant="rectangular" width={150} height={56} />
          <Skeleton variant="rectangular" width={150} height={56} />
          <Skeleton variant="rectangular" width={150} height={56} />
        </Stack>
      </Stack>
    );
  }

  if (error) {
    return <Alert severity="error">{error}</Alert>;
  }

  return (
    <Stack spacing={3}>
      <Alert severity="info">
        Select the Quranic content you'll be recording. You can record multiple
        consecutive ayahs in one session.
      </Alert>

      {/* Surah selector */}
      <FormControl fullWidth>
        <InputLabel>Surah</InputLabel>
        <Select
          value={surah || ''}
          onChange={e => handleSurahChange(Number(e.target.value))}
          label="Surah"
        >
          {surahs.map(s => (
            <MenuItem key={s.surah} value={s.surah}>
              {s.surah}. {s.name_arabic} ({s.name_english}) - {s.ayah_count}{' '}
              ayahs
            </MenuItem>
          ))}
        </Select>
      </FormControl>

      {/* Ayah range selector */}
      {surah && (
        <>
          <Stack direction="row" spacing={2} alignItems="center">
            <TextField
              label="Start Ayah"
              type="number"
              value={startAyah}
              onChange={e => setStartAyah(Number(e.target.value))}
              inputProps={{
                min: 1,
                max: selectedSurah?.ayah_count || 1,
              }}
              sx={{ width: 150 }}
            />

            <TextField
              label="End Ayah"
              type="number"
              value={endAyah}
              onChange={e => setEndAyah(Number(e.target.value))}
              inputProps={{
                min: startAyah,
                max: selectedSurah?.ayah_count || 1,
              }}
              sx={{ width: 150 }}
            />

            <Button variant="contained" onClick={handleApplyRange}>
              Apply Range
            </Button>
          </Stack>

          {ayahs.length > 0 && (
            <Alert severity="success">
              Selected: {ayahs.length} ayah{ayahs.length > 1 ? 's' : ''} (
              {ayahs.join(', ')})
            </Alert>
          )}
        </>
      )}

      {/* Preview selected ayahs */}
      {ayahs.length > 0 && (
        <Box
          sx={{
            p: 2,
            bgcolor: 'grey.50',
            borderRadius: 1,
            maxHeight: '400px',
            overflowY: 'auto',
          }}
        >
          <h3 style={{ marginTop: 0 }}>Preview:</h3>
          {ayahTexts
            .filter(a => ayahs.includes(a.ayah))
            .map(a => (
              <Box key={a.ayah} sx={{ mb: 2 }}>
                <Chip
                  label={`Ayah ${a.ayah}`}
                  size="small"
                  sx={{ mr: 1, mb: 1 }}
                />
                <Box sx={{ direction: 'rtl', textAlign: 'right' }}>
                  <TajweedText htmlText={a.text} fontSize={20} />
                </Box>
              </Box>
            ))}
        </Box>
      )}
    </Stack>
  );
};

export default ContentSelector;
