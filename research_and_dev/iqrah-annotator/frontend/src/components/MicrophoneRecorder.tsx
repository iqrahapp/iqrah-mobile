/**
 * Microphone Recorder Component
 * Records audio from microphone using MediaRecorder API
 */

import React, { useState, useRef, useEffect } from 'react';
import { Box, Button, Stack, Typography, Alert } from '@mui/material';
import { Mic, Stop, Replay } from '@mui/icons-material';

interface MicrophoneRecorderProps {
  onRecordingComplete: (audioBlob: Blob, duration: number) => void;
}

const MicrophoneRecorder: React.FC<MicrophoneRecorderProps> = ({
  onRecordingComplete,
}) => {
  const [isRecording, setIsRecording] = useState(false);
  const [recordingTime, setRecordingTime] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const timerRef = useRef<number | null>(null);
  const startTimeRef = useRef<number>(0);

  const startRecording = async () => {
    try {
      setError(null);

      // Request microphone access
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          channelCount: 1, // Mono
          sampleRate: 16000, // 16kHz
          echoCancellation: true,
          noiseSuppression: true,
        },
      });

      // Create MediaRecorder
      const mediaRecorder = new MediaRecorder(stream, {
        mimeType: 'audio/webm',
      });

      mediaRecorderRef.current = mediaRecorder;
      chunksRef.current = [];

      // Handle data available
      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          chunksRef.current.push(event.data);
        }
      };

      // Handle recording stop
      mediaRecorder.onstop = () => {
        const audioBlob = new Blob(chunksRef.current, { type: 'audio/webm' });
        const duration = (Date.now() - startTimeRef.current) / 1000;

        // Stop all tracks
        stream.getTracks().forEach(track => track.stop());

        // Call callback
        onRecordingComplete(audioBlob, duration);

        // Reset
        setIsRecording(false);
        if (timerRef.current) {
          clearInterval(timerRef.current);
          timerRef.current = null;
        }
      };

      // Start recording
      mediaRecorder.start();
      startTimeRef.current = Date.now();
      setIsRecording(true);
      setRecordingTime(0);

      // Update timer
      timerRef.current = window.setInterval(() => {
        setRecordingTime((Date.now() - startTimeRef.current) / 1000);
      }, 100);

    } catch (err: any) {
      setError(err.message || 'Failed to access microphone');
      console.error('Microphone error:', err);
    }
  };

  const stopRecording = () => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop();
    }
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = (seconds % 60).toFixed(1);
    return `${mins}:${secs.padStart(4, '0')}`;
  };

  // Cleanup on unmount: stop recording and clear timer
  useEffect(() => {
    return () => {
      // Clear timer to prevent memory leak
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }

      // Stop recording and release media stream
      if (mediaRecorderRef.current && mediaRecorderRef.current.state !== 'inactive') {
        mediaRecorderRef.current.stop();
        mediaRecorderRef.current.stream.getTracks().forEach(track => track.stop());
      }
    };
  }, []);

  return (
    <Box>
      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <Stack direction="row" spacing={2} alignItems="center">
        {!isRecording ? (
          <Button
            variant="contained"
            color="error"
            size="large"
            startIcon={<Mic />}
            onClick={startRecording}
            fullWidth
          >
            Start Recording
          </Button>
        ) : (
          <>
            <Button
              variant="contained"
              color="secondary"
              size="large"
              startIcon={<Stop />}
              onClick={stopRecording}
              fullWidth
            >
              Stop Recording
            </Button>
            <Box
              sx={{
                minWidth: 80,
                textAlign: 'center',
                p: 1,
                border: '2px solid',
                borderColor: 'error.main',
                borderRadius: 1,
                animation: 'pulse 1s infinite',
                '@keyframes pulse': {
                  '0%, 100%': { opacity: 1 },
                  '50%': { opacity: 0.7 },
                },
              }}
            >
              <Typography variant="h6" color="error">
                {formatTime(recordingTime)}
              </Typography>
            </Box>
          </>
        )}
      </Stack>

      <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>
        {isRecording
          ? 'Recording... Click Stop when finished'
          : 'Click Start to begin recording your recitation'}
      </Typography>
    </Box>
  );
};

export default MicrophoneRecorder;
