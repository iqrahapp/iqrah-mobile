/**
 * API Client for Tajweed Annotation Tool
 */

import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Types for Annotation Studio
export interface WordInstance {
  id: number;
  qpc_location: string;
  text: string;
  full_text?: string;
}

// Types for Backend API
export interface Recording {
  id: number;
  rule: string;
  anti_pattern: string;
  qpc_location?: string;
  sample_rate: number;
  duration_sec: number;
  audio_path: string;
  created_at: string;
}

export interface Region {
  id: number;
  recording_id: number;
  start_sec: number;
  end_sec: number;
  label: string;
  confidence?: number;
  notes?: string;
  created_at: string;
}

export interface RecordingCreate {
  rule: string;
  anti_pattern: string;
  qpc_location?: string;
  sample_rate: number;
  duration_sec: number;
}

export interface RegionCreate {
  recording_id: number;
  start_sec: number;
  end_sec: number;
  label: string;
  confidence?: number;
  notes?: string;
}

export interface ExportData {
  version: string;
  export_date: string;
  recordings: Array<Recording & { regions: Region[] }>;
}

// API Functions

// Recordings
export const createRecording = async (data: RecordingCreate): Promise<Recording> => {
  const response = await api.post('/api/recordings', data);
  return response.data;
};

export const listRecordings = async (params?: {
  rule?: string;
  anti_pattern?: string;
  qpc_location?: string;
}): Promise<Recording[]> => {
  const response = await api.get('/api/recordings', { params });
  return response.data;
};

export const getRecording = async (id: number): Promise<Recording> => {
  const response = await api.get(`/api/recordings/${id}`);
  return response.data;
};

export const deleteRecording = async (id: number): Promise<void> => {
  await api.delete(`/api/recordings/${id}`);
};

export const uploadAudio = async (id: number, file: File): Promise<void> => {
  const formData = new FormData();
  formData.append('file', file);

  await api.post(`/api/recordings/${id}/upload`, formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
};

// Regions
export const getRecordingRegions = async (recordingId: number): Promise<Region[]> => {
  const response = await api.get(`/api/recordings/${recordingId}/regions`);
  return response.data;
};

export const createRegion = async (data: RegionCreate): Promise<Region> => {
  const response = await api.post('/api/regions', data);
  return response.data;
};

export const updateRegion = async (
  id: number,
  data: Partial<RegionCreate>
): Promise<Region> => {
  const response = await api.patch(`/api/regions/${id}`, data);
  return response.data;
};

export const deleteRegion = async (id: number): Promise<void> => {
  await api.delete(`/api/regions/${id}`);
};

// Export
export const exportJSON = async (params?: {
  rule?: string;
  anti_pattern?: string;
  from?: string;
  to?: string;
}): Promise<ExportData> => {
  const response = await api.get('/api/export/json', { params });
  return response.data;
};

export default api;
