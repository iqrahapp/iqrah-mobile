/**
 * Keyboard Shortcuts Help Dialog
 * Shows all available keyboard shortcuts organized by category
 * Press '?' to open
 */

import React from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  IconButton,
  Typography,
  Box,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  Chip,
  Stack,
  Paper,
  Divider,
} from '@mui/material';
import { Close, Keyboard } from '@mui/icons-material';

interface ShortcutGroup {
  title: string;
  shortcuts: Array<{
    keys: string[];
    description: string;
    context?: string;
  }>;
}

const SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    title: 'Global Navigation',
    shortcuts: [
      { keys: ['?'], description: 'Show this help dialog' },
      { keys: ['Esc'], description: 'Close dialog or cancel action' },
      { keys: ['Ctrl', 'Z'], description: 'Undo last action' },
      { keys: ['Ctrl', 'Shift', 'Z'], description: 'Redo last action' },
    ],
  },
  {
    title: 'Audio Playback',
    shortcuts: [
      { keys: ['Space'], description: 'Play / Pause audio' },
      { keys: ['←'], description: 'Skip backward 100ms' },
      { keys: ['→'], description: 'Skip forward 100ms' },
      { keys: ['Shift', '←'], description: 'Skip backward 1 second' },
      { keys: ['Shift', '→'], description: 'Skip forward 1 second' },
      { keys: ['Home'], description: 'Jump to start' },
      { keys: ['End'], description: 'Jump to end' },
    ],
  },
  {
    title: 'Waveform Annotation',
    shortcuts: [
      { keys: ['Drag'], description: 'Create region segment' },
      { keys: ['Ctrl', 'Click'], description: 'Create smart segment (VAD-enhanced)' },
      { keys: ['Click'], description: 'Select/edit segment', context: 'on existing segment' },
      { keys: ['Delete'], description: 'Delete selected segment', context: 'after selecting' },
    ],
  },
  {
    title: 'Zoom Controls',
    shortcuts: [
      { keys: ['+'], description: 'Zoom in waveform' },
      { keys: ['-'], description: 'Zoom out waveform' },
    ],
  },
  {
    title: 'Speed Controls',
    shortcuts: [
      { keys: ['+'], description: 'Increase playback speed', context: 'in speed control' },
      { keys: ['-'], description: 'Decrease playback speed', context: 'in speed control' },
      { keys: ['1'], description: 'Reset to 1x speed', context: 'when speed buttons visible' },
    ],
  },
];

// macOS vs Windows/Linux modifier keys
const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;

/**
 * Format a key for display (handles Ctrl/Cmd differences)
 */
function formatKey(key: string): string {
  if (key === 'Ctrl' && isMac) return 'Cmd';
  if (key === 'Alt' && isMac) return 'Option';
  return key;
}

interface KeyboardShortcutsDialogProps {
  open: boolean;
  onClose: () => void;
}

export const KeyboardShortcutsDialog: React.FC<KeyboardShortcutsDialogProps> = ({
  open,
  onClose,
}) => {
  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="md"
      fullWidth
      aria-labelledby="keyboard-shortcuts-title"
    >
      <DialogTitle id="keyboard-shortcuts-title">
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Stack direction="row" alignItems="center" spacing={1}>
            <Keyboard />
            <span>Keyboard Shortcuts</span>
          </Stack>
          <IconButton onClick={onClose} size="small" aria-label="Close help dialog">
            <Close />
          </IconButton>
        </Stack>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={3}>
          {SHORTCUT_GROUPS.map((group, groupIndex) => (
            <Box key={groupIndex}>
              <Typography variant="h6" gutterBottom sx={{ color: 'primary.main', fontWeight: 600 }}>
                {group.title}
              </Typography>

              <Paper variant="outlined" sx={{ overflow: 'hidden' }}>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600, width: '40%' }}>Shortcut</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>Action</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {group.shortcuts.map((shortcut, index) => (
                      <TableRow key={index} hover>
                        <TableCell>
                          <Stack direction="row" spacing={0.5} flexWrap="wrap" useFlexGap>
                            {shortcut.keys.map((key, keyIndex) => (
                              <React.Fragment key={keyIndex}>
                                <Chip
                                  label={formatKey(key)}
                                  size="small"
                                  variant="outlined"
                                  sx={{
                                    fontFamily: 'monospace',
                                    fontWeight: 600,
                                    borderColor: 'primary.main',
                                    color: 'primary.main',
                                  }}
                                />
                                {keyIndex < shortcut.keys.length - 1 && (
                                  <Typography variant="caption" sx={{ alignSelf: 'center', px: 0.5 }}>
                                    +
                                  </Typography>
                                )}
                              </React.Fragment>
                            ))}
                          </Stack>
                        </TableCell>
                        <TableCell>
                          {shortcut.description}
                          {shortcut.context && (
                            <Typography variant="caption" color="text.secondary" sx={{ ml: 1 }}>
                              ({shortcut.context})
                            </Typography>
                          )}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </Paper>
            </Box>
          ))}

          <Divider />

          <Box>
            <Typography variant="body2" color="text.secondary" align="center">
              Press <Chip label="?" size="small" variant="outlined" sx={{ mx: 0.5 }} /> anytime to show this help dialog
            </Typography>
            <Typography variant="caption" color="text.secondary" align="center" sx={{ display: 'block', mt: 1 }}>
              Note: Some shortcuts may not work while typing in text fields
            </Typography>
          </Box>
        </Stack>
      </DialogContent>
    </Dialog>
  );
};

export default KeyboardShortcutsDialog;
