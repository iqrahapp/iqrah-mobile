/**
 * Hook for loading and managing audio segments with automatic cleanup
 * Handles trimming, blob URL management, and abort controller cleanup
 */

import { useState, useEffect, useRef } from 'react';
import { trimAudioBlob } from '../lib/ffmpeg';

interface UseAudioSegmentOptions {
  /** Full audio blob to trim from */
  fullAudioBlob: Blob | null;
  /** Start time of segment in seconds */
  startTime: number;
  /** End time of segment in seconds */
  endTime: number;
  /** Enable to activate this hook (useful for conditional loading) */
  enabled?: boolean;
}

interface UseAudioSegmentResult {
  /** Blob URL for the trimmed audio segment (null if not ready) */
  audioUrl: string | null;
  /** Time offset (same as startTime, useful for coordinate conversion) */
  timeOffset: number;
  /** Loading state */
  loading: boolean;
  /** Error message if trimming failed */
  error: string | null;
}

/**
 * Custom hook for extracting and managing audio segments
 *
 * Features:
 * - Automatic cleanup of old blob URLs to prevent memory leaks
 * - AbortController to cancel in-flight requests when dependencies change
 * - Safe URL revocation with delay to prevent race conditions
 *
 * @param options - Configuration for audio segment extraction
 * @returns Audio segment state and metadata
 */
export function useAudioSegment({
  fullAudioBlob,
  startTime,
  endTime,
  enabled = true,
}: UseAudioSegmentOptions): UseAudioSegmentResult {
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [timeOffset, setTimeOffset] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Track old URL for safe cleanup
  const oldUrlRef = useRef<string | null>(null);

  useEffect(() => {
    if (!enabled || !fullAudioBlob || startTime < 0 || endTime <= startTime) {
      return;
    }

    // Store old URL for cleanup AFTER new one is ready
    oldUrlRef.current = audioUrl;

    setAudioUrl(null); // Clear immediately to signal loading
    setLoading(true);
    setError(null);

    const abortController = new AbortController();

    console.log('[useAudioSegment] Trimming audio segment:', {
      startTime,
      endTime,
      duration: endTime - startTime,
      enabled,
      hasBlob: !!fullAudioBlob,
    });

    trimAudioBlob(fullAudioBlob, startTime, endTime)
      .then(result => {
        if (abortController.signal.aborted) {
          // Clean up blob created before abort was noticed
          URL.revokeObjectURL(URL.createObjectURL(result.blob));
          console.log('[useAudioSegment] Trim aborted');
          return;
        }

        const newUrl = URL.createObjectURL(result.blob);
        setAudioUrl(newUrl);
        setTimeOffset(startTime);
        console.log('[useAudioSegment] Audio segment ready, time offset:', startTime);

        // NOW safe to revoke old URL (after new one is set)
        // Small delay ensures the new URL is fully loaded before revoking old one
        if (oldUrlRef.current) {
          const urlToRevoke = oldUrlRef.current;
          setTimeout(() => {
            console.log('[useAudioSegment] Revoking old audio URL');
            URL.revokeObjectURL(urlToRevoke);
          }, 100);
          oldUrlRef.current = null;
        }
      })
      .catch(err => {
        if (abortController.signal.aborted) {
          return;
        }
        console.error('[useAudioSegment] Failed to extract audio segment:', err);
        setError('Failed to create audio segment');
      })
      .finally(() => {
        if (!abortController.signal.aborted) {
          setLoading(false);
        }
      });

    return () => {
      console.log('[useAudioSegment] Cleanup - aborting trim request');
      abortController.abort();
    };
  }, [fullAudioBlob, startTime, endTime, enabled]);

  // Cleanup blob URL on unmount
  useEffect(() => {
    return () => {
      if (audioUrl) {
        console.log('[useAudioSegment] Unmount - revoking audio URL');
        URL.revokeObjectURL(audioUrl);
      }
      if (oldUrlRef.current) {
        URL.revokeObjectURL(oldUrlRef.current);
      }
    };
  }, []); // Only run on unmount

  return {
    audioUrl,
    timeOffset,
    loading,
    error,
  };
}
