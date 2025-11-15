// Stage 1: Audio recording and trimming with auto-detection
import React, { useState, useEffect, useRef } from 'react';
import {
  Stack,
  Alert,
  CircularProgress,
  Button,
  Box,
  Chip,
  Tooltip,
  LinearProgress,
} from '@mui/material';
import { AutoAwesome, Upload, Check, Cancel } from '@mui/icons-material';
import MicrophoneRecorder from '../MicrophoneRecorder';
import WavesurferTrimmer from '../WavesurferTrimmer';
import { ConfirmDialog } from '../ConfirmDialog';
import { useWizardStore } from '../../store/wizardStore';
import { saveRecording, loadRecording } from '../../store/db';
import { autoTrim } from '../../lib/vad/silero';
import { trimAudioBlob } from '../../lib/ffmpeg';
import { isDefined, ensureNumber, clamp } from '../../utils/defensive';

export const AudioStage: React.FC = () => {
  const {
    recordingId,
    audioDuration,
    trim,
    setRecording,
    setTrim,
    surah,
    ayahs,
  } = useWizardStore();

  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [autoTrimStatus, setAutoTrimStatus] = useState<string | null>(null);
  const [autoTrimConfidence, setAutoTrimConfidence] = useState<number>(0);
  const [isTrimmed, setIsTrimmed] = useState(false);
  const [ffmpegError, setFfmpegError] = useState<string | null>(null);
  const [showResetConfirm, setShowResetConfirm] = useState(false);
  const [trimProgress, setTrimProgress] = useState<number>(0);
  const abortControllerRef = useRef<AbortController | null>(null);

  // Load existing recording if available
  useEffect(() => {
    if (!recordingId) return;

    loadRecording(recordingId).then(result => {
      if (result) {
        setAudioUrl(result.url);
      }
    });

    // NOTE: Do NOT revoke blob URLs on cleanup! This causes race conditions in React StrictMode.
    // WaveSurfer might still be loading the URL when cleanup runs. Instead, only revoke when
    // creating a NEW blob URL to replace the old one (see handleRecordingComplete and handleReset).
  }, [recordingId]);

  const handleRecordingComplete = async (blob: Blob, duration: number) => {
    setLoading(true);
    setAutoTrimStatus('Saving recording...');

    try {
      // Clean up old URL first
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
      }

      // Save to IndexedDB
      const id = await saveRecording(blob, duration);
      const url = URL.createObjectURL(blob);

      setRecording(id, duration);
      setAudioUrl(url);

      // Run auto-trim
      setAutoTrimStatus('Analyzing audio for optimal trim...');
      const trimResult = await autoTrim(blob);

      if (trimResult.confidence > 0) {
        setTrim(trimResult);
        setAutoTrimConfidence(ensureNumber(trimResult.confidence, 0));
        const dur = (ensureNumber(trimResult.end, 0) - ensureNumber(trimResult.start, 0)).toFixed(3);
        const conf = (ensureNumber(trimResult.confidence, 0) * 100).toFixed(0);
        setAutoTrimStatus(`Trimmed to ${ensureNumber(trimResult.start, 0).toFixed(3)}s - ${ensureNumber(trimResult.end, 0).toFixed(3)}s (${dur}s, ${conf}% confidence)`);
      } else {
        // Fallback: use full duration
        setTrim({ start: 0, end: ensureNumber(duration, 0) });
        setAutoTrimStatus('Auto-trim failed, using full audio');
      }
    } catch (error) {
      console.error('Failed to process recording:', error);
      setAutoTrimStatus(`Error: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    if (audioUrl) URL.revokeObjectURL(audioUrl);
    setAudioUrl(null);
    setRecording('', 0);
    if (audioDuration) {
      setTrim({ start: 0, end: audioDuration });
    }
    setAutoTrimStatus(null);
    setAutoTrimConfidence(0);
    setIsTrimmed(false);
    setFfmpegError(null);
    setShowResetConfirm(false);
  };

  const handleCancelTrim = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      setLoading(false);
      setAutoTrimStatus('Trim cancelled');
      setTrimProgress(0);
      abortControllerRef.current = null;
    }
  };

  const handleConfirmTrim = async () => {
    if (!isDefined(audioUrl) || !isDefined(trim) || !isDefined(recordingId)) {
      setAutoTrimStatus('Missing audio or trim data');
      return;
    }

    setLoading(true);
    setAutoTrimStatus('Loading FFmpeg...');
    setFfmpegError(null);
    setTrimProgress(0);
    abortControllerRef.current = new AbortController();

    try {
      // Load the original audio blob from IndexedDB
      const recording = await loadRecording(recordingId);
      if (!recording) {
        throw new Error('Failed to load recording from storage');
      }

      setAutoTrimStatus('Trimming audio with FFmpeg...');

      // Perform the actual trim with a 30-second timeout (enough for re-encoding if needed)
      const trimPromise = trimAudioBlob(
        recording.blob,
        trim.start,
        trim.end,
        30000, // 30-second timeout (generous for safety)
        (progress) => {
          setTrimProgress(progress);
          setAutoTrimStatus(`Trimming audio... ${Math.round(progress * 100)}%`);
        }
      );

      // Race against a 35-second absolute timeout and abort signal
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error('Trim operation timed out after 35 seconds')), 35000);
      });

      const abortPromise = new Promise<never>((_, reject) => {
        abortControllerRef.current?.signal.addEventListener('abort', () => {
          reject(new Error('Trim operation cancelled by user'));
        });
      });

      const trimResult = await Promise.race([trimPromise, timeoutPromise, abortPromise]);

      const trimmedDuration = ensureNumber(trim.end, 0) - ensureNumber(trim.start, 0);

      // Save the trimmed version back to IndexedDB (overwrite)
      const newId = await saveRecording(trimResult.blob, trimmedDuration);
      const newUrl = URL.createObjectURL(trimResult.blob);

      // Update store with trimmed audio
      setRecording(newId, trimmedDuration);

      // Clean up old URL
      if (audioUrl) URL.revokeObjectURL(audioUrl);
      setAudioUrl(newUrl);

      // Reset trim region to full duration of trimmed audio
      setTrim({ start: 0, end: trimmedDuration });

      setIsTrimmed(true);
      setAutoTrimStatus('Trim applied successfully');
      setTrimProgress(1);

    } catch (error) {
      console.error('FFmpeg trim failed:', error);
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      setFfmpegError(errorMsg);

      // Provide helpful message based on error type
      if (errorMsg.includes('cancelled')) {
        setAutoTrimStatus('Trim cancelled - you can retry or click "SKIP TRIM" to use full audio');
      } else if (errorMsg.includes('timed out')) {
        setAutoTrimStatus('⚠️ Trim timed out - you can click "APPLY TRIM" to retry, or click "SKIP TRIM" to use full audio');
      } else {
        setAutoTrimStatus('⚠️ Trim failed - you can retry or click "SKIP TRIM" to use full audio');
      }

      // Don't auto-proceed on error - let user decide to retry or skip
      setIsTrimmed(false);
      setTrimProgress(0);
    } finally {
      setLoading(false);
      abortControllerRef.current = null;
    }
  };

  return (
    <Stack spacing={3}>
      <Alert severity="info">
        <strong>Recording:</strong> Surah {surah}, Ayahs {ayahs.join(', ')}
        <br />
        Record yourself reciting these ayahs. The audio will be automatically
        trimmed to remove silence.
      </Alert>

      {!recordingId && (
        <>
          <MicrophoneRecorder onRecordingComplete={handleRecordingComplete} />

          <Box sx={{ textAlign: 'center', py: 2 }}>
            <Button
              variant="outlined"
              startIcon={<Upload />}
              component="label"
              disabled
            >
              Upload Audio File
              <input type="file" accept="audio/*" hidden />
            </Button>
            <div style={{ fontSize: 12, color: '#666', marginTop: 8 }}>
              (File upload coming soon)
            </div>
          </Box>
        </>
      )}

      {loading && (
        <Alert severity="info" icon={<CircularProgress size={20} />}>
          {autoTrimStatus || 'Processing...'}
        </Alert>
      )}

      {autoTrimStatus && !loading && autoTrimConfidence > 0 && (
        <Alert
          severity={
            autoTrimConfidence > 0.8
              ? 'success'
              : autoTrimConfidence > 0.6
              ? 'info'
              : 'warning'
          }
          icon={<AutoAwesome />}
        >
          <strong>Auto-trim applied:</strong> {autoTrimStatus}
          <br />
          You can adjust the trim region below by dragging the edges.
        </Alert>
      )}

      {audioUrl && trim && !isTrimmed && (
        <>
          <Box>
            <Stack
              direction="row"
              justifyContent="space-between"
              alignItems="center"
              sx={{ mb: 1 }}
            >
              <h3 style={{ margin: 0 }}>Step 2: Adjust Trim Boundaries</h3>
              <Tooltip title="Discard this recording and record a new one">
                <Button size="small" onClick={() => setShowResetConfirm(true)} variant="outlined">
                  Record Again
                </Button>
              </Tooltip>
            </Stack>

            <Box sx={{ mb: 2 }}>
              <Chip
                label={`Original Duration: ${audioDuration?.toFixed(3)}s`}
                size="small"
                sx={{ mr: 1 }}
              />
              <Chip
                label={`Trim: ${trim.start.toFixed(3)}s - ${trim.end.toFixed(
                  2
                )}s (keeping ${(trim.end - trim.start).toFixed(3)}s)`}
                size="small"
                color="primary"
              />
            </Box>

            <WavesurferTrimmer
              audioUrl={audioUrl}
              value={trim}
              onChange={setTrim}
            />
          </Box>

          <Alert severity="info">
            Drag the edges of the green region to adjust the trim. Dark areas will be removed.
            <br />
            <strong>Trim Stats:</strong> Keeping{' '}
            {audioDuration ? ((trim.end - trim.start) / audioDuration * 100).toFixed(1) : '0'}%
            {' '}of the original audio.
          </Alert>

          {ffmpegError && (
            <Alert severity="warning">
              FFmpeg Error: {ffmpegError}. You can still proceed with the original audio.
            </Alert>
          )}

          {/* Progress bar during FFmpeg processing */}
          {loading && trimProgress > 0 && (
            <Box sx={{ width: '100%' }}>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                <span style={{ fontSize: 14, fontWeight: 500 }}>Processing Audio</span>
                <span style={{ fontSize: 14 }}>{Math.round(trimProgress * 100)}%</span>
              </Box>
              <LinearProgress variant="determinate" value={trimProgress * 100} />
            </Box>
          )}

          <Box sx={{ display: 'flex', justifyContent: 'space-between', gap: 2 }}>
            <Tooltip title="Proceed without trimming - use the full audio recording">
              <span>
                <Button
                  variant="text"
                  onClick={() => {
                    setIsTrimmed(true);
                    setAutoTrimStatus('Skipped trim - using full audio');
                  }}
                  disabled={loading}
                >
                  Skip Trim (Use Full Audio)
                </Button>
              </span>
            </Tooltip>
            <Box sx={{ display: 'flex', gap: 2 }}>
              <Tooltip title="Reset trim region to full audio duration">
                <span>
                  <Button
                    variant="outlined"
                    onClick={() => {
                      setIsTrimmed(false);
                      if (audioDuration) {
                        setTrim({ start: 0, end: audioDuration });
                      }
                    }}
                    disabled={loading}
                  >
                    Reset Trim
                  </Button>
                </span>
              </Tooltip>
              <Tooltip title="Apply the trim and continue - this will permanently cut the audio using FFmpeg">
                <span>
                  <Button
                    variant="contained"
                    endIcon={loading ? <CircularProgress size={20} /> : <Check />}
                    onClick={handleConfirmTrim}
                    disabled={loading}
                    size="large"
                  >
                    {loading ? autoTrimStatus : 'Apply Trim (FFmpeg)'}
                  </Button>
                </span>
              </Tooltip>
              {loading && (
                <Tooltip title="Cancel the trim operation">
                  <Button
                    variant="outlined"
                    color="error"
                    startIcon={<Cancel />}
                    onClick={handleCancelTrim}
                    size="large"
                  >
                    Cancel
                  </Button>
                </Tooltip>
              )}
            </Box>
          </Box>
        </>
      )}

      {audioUrl && trim && isTrimmed && (
        <>
          <Alert severity="success" icon={<Check />}>
            <strong>Audio trimmed successfully!</strong> You can now proceed to segment the verses.
          </Alert>

          <Box>
            <Stack
              direction="row"
              justifyContent="space-between"
              alignItems="center"
              sx={{ mb: 1 }}
            >
              <h3 style={{ margin: 0 }}>Trimmed Audio</h3>
              <Tooltip title="Discard everything and start with a new recording">
                <Button size="small" onClick={() => setShowResetConfirm(true)} variant="outlined">
                  Start Over
                </Button>
              </Tooltip>
            </Stack>

            <Box sx={{ mb: 2 }}>
              <Chip
                label={`Duration: ${audioDuration?.toFixed(3)}s`}
                size="small"
                color="success"
              />
            </Box>

            <WavesurferTrimmer
              audioUrl={audioUrl}
              value={trim}
              onChange={() => {}} // Read-only after trim applied
            />
          </Box>

          <Alert severity="info">
            The trim has been permanently applied. This is the audio you'll be annotating.
          </Alert>
        </>
      )}

      {/* Confirmation dialog for Reset */}
      <ConfirmDialog
        open={showResetConfirm}
        title="Discard Recording?"
        message="This will delete your current audio recording and trim settings. You'll need to record again from scratch. This action cannot be undone."
        confirmText="Discard & Start Over"
        confirmColor="warning"
        onConfirm={handleReset}
        onCancel={() => setShowResetConfirm(false)}
      />
    </Stack>
  );
};

export default AudioStage;
