/**
 * Main App Component
 */

import  { useState } from 'react';
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
import AnnotationStudioPage from './pages/AnnotationStudioPage';

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

  return (
    <ThemeProvider theme={theme}>
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
              Annotation Studio
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
          {currentPage === 'studio' && <AnnotationStudioPage />}
          {currentPage === 'list' && <RecordingsListPage />}
        </Box>
      </Box>
    </ThemeProvider>
  );
}

export default App;
