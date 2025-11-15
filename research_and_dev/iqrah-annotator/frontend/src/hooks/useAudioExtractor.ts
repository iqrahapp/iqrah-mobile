import { useState, useCallback, useEffect } from 'react';
import { loadRecording } from '../store/db';
import { trimAudioBlob } from '../lib/ffmpeg';

interface AudioSegment {
  blob: Blob;
  url: string;
  offset: number;
}

export function useAudioExtractor(recordingId: string) {
  const [fullBlob, setFullBlob] = useState<Blob | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load full audio once
  const loadFullAudio = useCallback(async () => {
    if (!recordingId || fullBlob) return;

    setLoading(true);
    try {
      const result = await loadRecording(recordingId);
      if (result) {
        setFullBlob(result.blob);
      }
    } catch (err: any) {
      setError(`Failed to load audio: ${err.message}`);
    } finally {
      setLoading(false);
    }
  }, [recordingId, fullBlob]);

  // Auto-load on mount
  useEffect(() => {
    loadFullAudio();
  }, [loadFullAudio]);

  // Extract segment
  const extractSegment = useCallback(
    async (start: number, end: number): Promise<AudioSegment> => {
      if (!fullBlob) {
        throw new Error('Audio not loaded. Call loadFullAudio() first.');
      }

      const result = await trimAudioBlob(fullBlob, start, end);
      const url = URL.createObjectURL(result.blob);

      return {
        blob: result.blob,
        url,
        offset: start,
      };
    },
    [fullBlob]
  );

  return {
    fullBlob,
    loading,
    error,
    loadFullAudio,
    extractSegment,
  };
}
