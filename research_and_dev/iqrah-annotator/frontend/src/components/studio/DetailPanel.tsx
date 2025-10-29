import React from 'react';
import {
  Paper,
  Typography,
  Box,
  Button,
  List,
  ListItemButton,
  ListItemText,
  ListItemIcon,
  TextField,
  Stack,
  LinearProgress,
} from '@mui/material';
import { Check, RadioButtonUnchecked, Delete, PlayArrow } from '@mui/icons-material';
import { useAnnotationStore, type TierRegion, type ActiveTier, type QPCWord } from '../../store/annotationStore';
import TajweedText from '../TajweedText';

const RegionEditor: React.FC<{ region: TierRegion; tier: ActiveTier }> = ({ region, tier }) => {
  const { updateRegion, deleteRegion } = useAnnotationStore();

  const handleLabelChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateRegion(tier, region.id, { labelText: e.target.value });
  };

  return (
    <Box>
      <Typography variant="h6" gutterBottom>
        Edit Region
      </Typography>
      <Stack spacing={2}>
        <TextField
          label="Label"
          value={region.labelText}
          onChange={handleLabelChange}
          disabled={!!region.key}
          fullWidth
          size="small"
        />
        <TextField
          label="Start (sec)"
          value={region.startTime.toFixed(3)}
          disabled
          fullWidth
          size="small"
        />
        <TextField
          label="End (sec)"
          value={region.endTime.toFixed(3)}
          disabled
          fullWidth
          size="small"
        />
        <Stack direction="row" spacing={1}>
          <Button
            variant="outlined"
            startIcon={<PlayArrow />}
            fullWidth
            onClick={() =>
              window.dispatchEvent(
                new CustomEvent('play-range', {
                  detail: { startTime: region.startTime, endTime: region.endTime },
                })
              )
            }
          >
            Play Selection
          </Button>
          <Button
            variant="outlined"
            color="error"
            startIcon={<Delete />}
            onClick={() => deleteRegion(tier, region.id)}
          >
            Delete
          </Button>
        </Stack>
      </Stack>
    </Box>
  );
};

const WordChecklist: React.FC = () => {
  const { activeInstance, activeTiers, setWordToAnnotate, wordToAnnotate } = useAnnotationStore();

  // Use actual words from backend if available, otherwise fallback to parsing
  const allWords = React.useMemo(() => {
    if (activeInstance?.words && activeInstance.words.length > 0) {
      return activeInstance.words;
    }
    // Fallback: parse from full_text
    const parsed = activeInstance?.full_text?.replace(/<[^>]+>/g, ' ').trim().split(/\s+/) || [];
    return parsed.map((text, index) => ({
      id: index,
      location: `${activeInstance?.qpc_location}:${index + 1}`,
      surah: 0,
      ayah: 0,
      word: index + 1,
      text,
      rules: [],
    }));
  }, [activeInstance?.words, activeInstance?.full_text, activeInstance?.qpc_location]);

  const annotatedKeys = new Set(activeTiers.words.map((w) => w.key).filter(Boolean));
  const done = annotatedKeys.size;
  const total = allWords.length;
  const progress = total > 0 ? (done / total) * 100 : 0;

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <Typography variant="h6" gutterBottom>
        Word Checklist
      </Typography>
      <Box sx={{ mb: 1 }}>
        <Typography variant="body2" color="text.secondary">
          Words covered: {done} / {total}
        </Typography>
        <LinearProgress variant="determinate" value={progress} sx={{ my: 1 }} />
      </Box>
      <Box sx={{ flexGrow: 1, overflowY: 'auto', border: '1px solid #ddd', borderRadius: 1 }}>
        <List dense>
          {allWords.map((word) => {
            const key = word.location; // Use QPC location as key (e.g., "1:1:3")
            const isAnnotated = annotatedKeys.has(key);
            return (
              <ListItemButton
                key={key}
                onClick={() => !isAnnotated && setWordToAnnotate(key)}
                disabled={isAnnotated}
                selected={wordToAnnotate === key}
                sx={{ flexDirection: 'column', alignItems: 'flex-start', py: 1 }}
              >
                <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
                  <ListItemIcon sx={{ minWidth: 32 }}>
                    {isAnnotated ? <Check color="success" /> : <RadioButtonUnchecked />}
                  </ListItemIcon>
                  <Box sx={{ flex: 1 }}>
                    <TajweedText htmlText={word.text} fontSize={18} />
                  </Box>
                </Box>
                <Typography variant="caption" color="text.secondary" sx={{ ml: 4 }}>
                  {word.location}
                </Typography>
              </ListItemButton>
            );
          })}
        </List>
      </Box>
    </Box>
  );
};

export const DetailPanel: React.FC = () => {
  const { selectedRegion, activeTiers, activeInstance } = useAnnotationStore();

  const getRegionDetails = () => {
    if (!selectedRegion) return null;
    const { tier, regionId } = selectedRegion;
    const region = activeTiers[tier]?.find((r) => r.id === regionId);
    return region ? { region, tier } : null;
  };

  const details = getRegionDetails();

  if (!activeInstance) return null;

  return (
    <Paper sx={{ p: 2, display: 'flex', flexDirection: 'column', flexGrow: 1 }}>
      {details ? <RegionEditor region={details.region} tier={details.tier} /> : <WordChecklist />}
    </Paper>
  );
};
