import React from 'react';
import { Box, Stack, Typography, LinearProgress } from '@mui/material';

interface SegmentationProgressProps {
  total: number;
  completed: number;
  itemType: 'ayah' | 'word' | 'anti-pattern';
}

export const SegmentationProgress: React.FC<SegmentationProgressProps> = ({
  total,
  completed,
  itemType,
}) => {
  const progress = total > 0 ? (completed / total) * 100 : 0;

  return (
    <Box>
      <Stack direction="row" justifyContent="space-between" sx={{ mb: 1 }}>
        <Typography variant="body2">
          <strong>Progress:</strong> {completed} / {total} {itemType}s segmented
        </Typography>
        <Typography variant="body2">{progress.toFixed(0)}%</Typography>
      </Stack>
      <LinearProgress
        variant="determinate"
        value={progress}
        color={progress === 100 ? 'success' : 'primary'}
      />
    </Box>
  );
};
