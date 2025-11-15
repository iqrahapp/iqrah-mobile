/**
 * Main App Component
 */

import  { useState, useEffect } from 'react';
import {
  CssBaseline,
  ThemeProvider,
  createTheme,
  AppBar,
  Toolbar,
  Typography,
  Button,
  Box,
} from '@mui/material';
import { List, Edit } from '@mui/icons-material';
import RecordingsListPage from './pages/RecordingsListPage';
import StudioWizardPage from './pages/StudioWizardPage';
import { ErrorBoundary } from './components/ErrorBoundary';
import { ToastProvider } from './contexts/ToastContext';
import { KeyboardShortcutsDialog } from './components/KeyboardShortcutsDialog';

const theme = createTheme({
  palette: {
    primary: {
      main: '#1976d2',
    },
    secondary: {
      main: '#dc004e',
    },
  },
});

type Page = 'list' | 'studio';

function App() {
  const [currentPage, setCurrentPage] = useState<Page>('studio');
  const [showShortcutsDialog, setShowShortcutsDialog] = useState(false);

  // Global keyboard listener for '?' to show shortcuts dialog
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      // Ignore if typing in input field
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      // Press '?' or 'Shift+/' to show shortcuts
      if (e.key === '?' || (e.key === '/' && e.shiftKey)) {
        e.preventDefault();
        setShowShortcutsDialog(true);
      }

      // Press 'Esc' to close shortcuts
      if (e.key === 'Escape' && showShortcutsDialog) {
        setShowShortcutsDialog(false);
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [showShortcutsDialog]);

  return (
    <ThemeProvider theme={theme}>
      <ToastProvider maxToasts={3} defaultDuration={5000}>
        <CssBaseline />
        <Box sx={{ flexGrow: 1 }}>
          <AppBar position="static">
            <Toolbar>
              <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
                Tajweed Annotation Tool
              </Typography>
              <Button
                color="inherit"
                startIcon={<Edit />}
                onClick={() => setCurrentPage('studio')}
                variant={currentPage === 'studio' ? 'outlined' : 'text'}
              >
                Annotation Wizard
              </Button>
              <Button
                color="inherit"
                startIcon={<List />}
                onClick={() => setCurrentPage('list')}
                variant={currentPage === 'list' ? 'outlined' : 'text'}
                sx={{ ml: 1 }}
              >
                Recordings
              </Button>
            </Toolbar>
          </AppBar>

          <Box sx={{ minHeight: 'calc(100vh - 64px)', backgroundColor: '#f5f5f5' }}>
            <ErrorBoundary onError={(error, errorInfo) => {
              console.error('[App] Top-level error:', error, errorInfo);
              // Could send to error tracking service here (e.g., Sentry, LogRocket)
            }}>
              {currentPage === 'studio' && <StudioWizardPage />}
              {currentPage === 'list' && <RecordingsListPage />}
            </ErrorBoundary>
          </Box>
        </Box>

        {/* Global keyboard shortcuts help dialog */}
        <KeyboardShortcutsDialog
          open={showShortcutsDialog}
          onClose={() => setShowShortcutsDialog(false)}
        />
      </ToastProvider>
    </ThemeProvider>
  );
}

export default App;
