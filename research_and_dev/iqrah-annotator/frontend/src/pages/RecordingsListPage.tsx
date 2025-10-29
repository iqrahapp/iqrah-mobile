/**
 * Recordings List Page
 */

import React, { useState, useEffect } from 'react';
import {
  Container,
  Typography,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  IconButton,
  Chip,
  Button,
  Stack,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Alert,
} from '@mui/material';
import { Delete, Refresh, Download } from '@mui/icons-material';
import { listRecordings, deleteRecording, exportJSON } from '../api/client';
import type { Recording } from '../api/client';

const RecordingsListPage: React.FC = () => {
  const [recordings, setRecordings] = useState<Recording[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Filters
  const [ruleFilter, setRuleFilter] = useState('');
  const [antiPatternFilter, setAntiPatternFilter] = useState('');

  useEffect(() => {
    loadRecordings();
  }, []);

  const loadRecordings = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const params: any = {};
      if (ruleFilter) params.rule = ruleFilter;
      if (antiPatternFilter) params.anti_pattern = antiPatternFilter;

      const data = await listRecordings(params);
      setRecordings(data);
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to load recordings');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm('Are you sure you want to delete this recording?')) return;

    try {
      await deleteRecording(id);
      setRecordings(recordings.filter((r) => r.id !== id));
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to delete recording');
    }
  };

  const handleExport = async () => {
    try {
      const params: any = {};
      if (ruleFilter) params.rule = ruleFilter;
      if (antiPatternFilter) params.anti_pattern = antiPatternFilter;

      const data = await exportJSON(params);

      // Download as JSON file
      const blob = new Blob([JSON.stringify(data, null, 2)], {
        type: 'application/json',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `tajweed_export_${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to export data');
    }
  };

  return (
    <Container maxWidth="xl" sx={{ py: 4 }}>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 3 }}>
        <Typography variant="h4">Recordings</Typography>
        <Button variant="contained" startIcon={<Download />} onClick={handleExport}>
          Export JSON
        </Button>
      </Stack>

      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Filters */}
      <Paper sx={{ p: 2, mb: 2 }}>
        <Stack direction="row" spacing={2} alignItems="center">
          <FormControl sx={{ minWidth: 200 }}>
            <InputLabel>Rule</InputLabel>
            <Select
              value={ruleFilter}
              label="Rule"
              onChange={(e) => setRuleFilter(e.target.value)}
            >
              <MenuItem value="">All</MenuItem>
              <MenuItem value="ghunnah">Ghunnah</MenuItem>
              <MenuItem value="qalqalah">Qalqalah</MenuItem>
            </Select>
          </FormControl>

          <FormControl sx={{ minWidth: 200 }}>
            <InputLabel>Anti-Pattern</InputLabel>
            <Select
              value={antiPatternFilter}
              label="Anti-Pattern"
              onChange={(e) => setAntiPatternFilter(e.target.value)}
            >
              <MenuItem value="">All</MenuItem>
              <MenuItem value="weak-ghunnah">Weak Ghunnah</MenuItem>
              <MenuItem value="no-ghunnah">No Ghunnah</MenuItem>
              <MenuItem value="no-qalqalah">No Qalqalah</MenuItem>
              <MenuItem value="weak-qalqalah">Weak Qalqalah</MenuItem>
            </Select>
          </FormControl>

          <Button
            variant="outlined"
            startIcon={<Refresh />}
            onClick={loadRecordings}
            disabled={isLoading}
          >
            {isLoading ? 'Loading...' : 'Refresh'}
          </Button>
        </Stack>
      </Paper>

      {/* Recordings Table */}
      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>ID</TableCell>
              <TableCell>Rule</TableCell>
              <TableCell>Anti-Pattern</TableCell>
              <TableCell>QPC Location</TableCell>
              <TableCell>Duration</TableCell>
              <TableCell>Sample Rate</TableCell>
              <TableCell>Created</TableCell>
              <TableCell align="right">Actions</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {recordings.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} align="center">
                  <Typography variant="body2" color="text.secondary">
                    No recordings found
                  </Typography>
                </TableCell>
              </TableRow>
            ) : (
              recordings.map((recording) => (
                <TableRow key={recording.id}>
                  <TableCell>{recording.id}</TableCell>
                  <TableCell>
                    <Chip label={recording.rule} size="small" />
                  </TableCell>
                  <TableCell>
                    <Chip label={recording.anti_pattern} size="small" color="secondary" />
                  </TableCell>
                  <TableCell>{recording.qpc_location || '-'}</TableCell>
                  <TableCell>{recording.duration_sec.toFixed(2)}s</TableCell>
                  <TableCell>{recording.sample_rate} Hz</TableCell>
                  <TableCell>
                    {new Date(recording.created_at).toLocaleString()}
                  </TableCell>
                  <TableCell align="right">
                    <IconButton size="small" onClick={() => handleDelete(recording.id)}>
                      <Delete />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </TableContainer>
    </Container>
  );
};

export default RecordingsListPage;
