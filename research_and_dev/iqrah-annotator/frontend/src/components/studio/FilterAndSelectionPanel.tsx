import React, { useEffect, useState } from 'react';
import {
  Box,
  Paper,
  Typography,
  List,
  ListItemButton,
  ListItemText,
  Divider,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  CircularProgress,
  Alert,
} from '@mui/material';
import axios from 'axios';
import TajweedText from '../TajweedText';
import { useAnnotationStore, type WordInstance, type QPCWord } from '../../store/annotationStore';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000';

interface SurahInfo {
  surah: number;
  name_arabic: string;
  name_english: string;
  ayah_count: number;
  word_count: number;
}

interface AyahText {
  surah: number;
  ayah: number;
  text: string;
  rules: string[];
}

export const FilterAndSelectionPanel: React.FC = () => {
  const { setActiveInstance, activeInstance } = useAnnotationStore();
  const [surahs, setSurahs] = useState<SurahInfo[]>([]);
  const [selectedSurah, setSelectedSurah] = useState<number | null>(null);
  const [ayahs, setAyahs] = useState<AyahText[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load surahs on mount
  useEffect(() => {
    loadSurahs();
  }, []);

  const loadSurahs = async () => {
    try {
      setIsLoading(true);
      const response = await axios.get<SurahInfo[]>(`${API_URL}/api/qpc/surahs`);
      setSurahs(response.data);
      setError(null);
    } catch (err: any) {
      setError(`Failed to load surahs: ${err.message}`);
    } finally {
      setIsLoading(false);
    }
  };

  const loadAyahs = async (surahNumber: number) => {
    try {
      setIsLoading(true);
      const response = await axios.get<AyahText[]>(`${API_URL}/api/qpc/ayahs/${surahNumber}`);
      setAyahs(response.data);
      setError(null);
    } catch (err: any) {
      setError(`Failed to load ayahs: ${err.message}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSurahChange = (surahNumber: number) => {
    setSelectedSurah(surahNumber);
    setAyahs([]);
    loadAyahs(surahNumber);
  };

  const handleAyahSelect = async (ayah: AyahText) => {
    try {
      // Fetch words for this specific ayah
      const response = await axios.get<QPCWord[]>(
        `${API_URL}/api/qpc/words?surah=${ayah.surah}&limit=1000`
      );

      // Filter words that belong to this ayah
      const ayahWords = response.data.filter(w => w.ayah === ayah.ayah);

      const instance: WordInstance = {
        id: ayah.ayah,
        qpc_location: `${ayah.surah}:${ayah.ayah}`,
        text: ayah.text,
        full_text: ayah.text,
        words: ayahWords, // Include word-by-word data
      };
      setActiveInstance(instance);
    } catch (err: any) {
      console.error('Failed to load words:', err);
      // Fallback: create instance without words
      const instance: WordInstance = {
        id: ayah.ayah,
        qpc_location: `${ayah.surah}:${ayah.ayah}`,
        text: ayah.text,
        full_text: ayah.text,
      };
      setActiveInstance(instance);
    }
  };

  const selectedSurahInfo = surahs.find((s) => s.surah === selectedSurah);

  return (
    <Paper sx={{ p: 2, display: 'flex', flexDirection: 'column', flexGrow: 1 }}>
      <Typography variant="h6">Content to Annotate</Typography>
      <Divider sx={{ my: 1 }} />

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <FormControl fullWidth sx={{ mb: 2 }}>
        <InputLabel>Select Surah</InputLabel>
        <Select
          value={selectedSurah || ''}
          label="Select Surah"
          onChange={(e) => handleSurahChange(Number(e.target.value))}
        >
          {surahs.map((surah) => (
            <MenuItem key={surah.surah} value={surah.surah}>
              {surah.surah}. {surah.name_arabic} ({surah.name_english})
            </MenuItem>
          ))}
        </Select>
      </FormControl>

      {isLoading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
          <CircularProgress />
        </Box>
      )}

      {selectedSurahInfo && !isLoading && (
        <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
          {selectedSurahInfo.ayah_count} ayahs
        </Typography>
      )}

      <Box sx={{ flexGrow: 1, overflowY: 'auto' }}>
        <List dense>
          {ayahs.map((ayah) => (
            <ListItemButton
              key={ayah.ayah}
              onClick={() => handleAyahSelect(ayah)}
              selected={
                activeInstance?.qpc_location === `${ayah.surah}:${ayah.ayah}`
              }
              sx={{ flexDirection: 'column', alignItems: 'flex-start' }}
            >
              <ListItemText
                primary={`Ayah ${ayah.ayah}`}
                secondary={`Location: ${ayah.surah}:${ayah.ayah}`}
              />
              <Box sx={{ mt: 1, width: '100%' }}>
                <TajweedText htmlText={ayah.text} fontSize={16} />
              </Box>
            </ListItemButton>
          ))}
        </List>
      </Box>
    </Paper>
  );
};
