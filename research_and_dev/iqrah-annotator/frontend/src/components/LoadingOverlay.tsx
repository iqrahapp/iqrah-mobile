/**
 * FIX #5 & #14: Loading overlay component with progress indicator
 * Shows during async operations to prevent user confusion
 */

import React from 'react';
import {
  Backdrop,
  CircularProgress,
  LinearProgress,
  Stack,
  Typography,
  Paper,
  Box,
} from '@mui/material';

interface LoadingOverlayProps {
  /** Whether the overlay is visible */
  visible: boolean;
  /** Loading message to display */
  message?: string;
  /** Progress value (0-1 for determinate, undefined for indeterminate) */
  progress?: number;
  /** Whether to allow clicking through the overlay */
  transparent?: boolean;
}

export const LoadingOverlay: React.FC<LoadingOverlayProps> = ({
  visible,
  message = 'Loading...',
  progress,
  transparent = false,
}) => {
  return (
    <Backdrop
      open={visible}
      sx={{
        color: '#fff',
        zIndex: (theme) => theme.zIndex.drawer + 1,
        backgroundColor: transparent ? 'transparent' : 'rgba(0, 0, 0, 0.5)',
        pointerEvents: transparent ? 'none' : 'auto',
      }}
    >
      <Paper
        elevation={8}
        sx={{
          p: 4,
          minWidth: 300,
          textAlign: 'center',
          pointerEvents: 'auto',
        }}
      >
        <Stack spacing={2} alignItems="center">
          {progress !== undefined ? (
            <>
              <CircularProgress
                variant="determinate"
                value={progress * 100}
                size={60}
                thickness={4}
              />
              <Typography variant="h6">{message}</Typography>
              <Box sx={{ width: '100%' }}>
                <LinearProgress
                  variant="determinate"
                  value={progress * 100}
                  sx={{ height: 8, borderRadius: 4 }}
                />
                <Typography variant="caption" sx={{ mt: 1 }}>
                  {(progress * 100).toFixed(0)}%
                </Typography>
              </Box>
            </>
          ) : (
            <>
              <CircularProgress size={60} thickness={4} />
              <Typography variant="h6">{message}</Typography>
            </>
          )}
        </Stack>
      </Paper>
    </Backdrop>
  );
};

export default LoadingOverlay;
