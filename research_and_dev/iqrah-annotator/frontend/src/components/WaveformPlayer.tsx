/**
 * Waveform Player with Regions for Annotation
 */

import React, { useEffect, useRef, useState } from 'react';
import WaveSurfer from 'wavesurfer.js';
import RegionsPlugin from 'wavesurfer.js/dist/plugins/regions.js';
import { Box, IconButton, Stack, Typography } from '@mui/material';
import { PlayArrow, Pause, Stop, ZoomIn, ZoomOut } from '@mui/icons-material';
import type { Region } from '../api/client';

interface WaveformPlayerProps {
  audioUrl?: string;
  audioFile?: File;
  regions?: Region[];
  onRegionCreate?: (region: { start: number; end: number }) => void;
  onRegionUpdate?: (regionId: number, data: Partial<Region>) => void;
  onRegionClick?: (region: Region) => void;
  height?: number;
}

const REGION_COLORS = [
  'rgba(255, 87, 34, 0.3)',   // Deep Orange
  'rgba(255, 152, 0, 0.3)',   // Orange
  'rgba(255, 193, 7, 0.3)',   // Amber
  'rgba(76, 175, 80, 0.3)',   // Green
  'rgba(33, 150, 243, 0.3)',  // Blue
  'rgba(156, 39, 176, 0.3)',  // Purple
];

const WaveformPlayer: React.FC<WaveformPlayerProps> = ({
  audioUrl,
  audioFile,
  regions = [],
  onRegionCreate,
  onRegionUpdate,
  onRegionClick,
  height = 128,
}) => {
  const waveformRef = useRef<HTMLDivElement>(null);
  const wavesurferRef = useRef<WaveSurfer | null>(null);
  const regionsPluginRef = useRef<any>(null);

  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [zoom, setZoom] = useState(50);
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    if (!waveformRef.current) return;

    // Initialize WaveSurfer
    const wavesurfer = WaveSurfer.create({
      container: waveformRef.current,
      waveColor: '#4285F4',
      progressColor: '#1967D2',
      cursorColor: '#D32F2F',
      barWidth: 2,
      barGap: 1,
      barRadius: 3,
      height: height,
      normalize: true,
      plugins: [],
    });

    // Initialize Regions Plugin
    const regionsPlugin = wavesurfer.registerPlugin(RegionsPlugin.create());

    wavesurferRef.current = wavesurfer;
    regionsPluginRef.current = regionsPlugin;

    // Event listeners
    wavesurfer.on('ready', () => {
      setDuration(wavesurfer.getDuration());
      setIsReady(true);
    });

    wavesurfer.on('play', () => setIsPlaying(true));
    wavesurfer.on('pause', () => setIsPlaying(false));
    wavesurfer.on('finish', () => setIsPlaying(false));

    wavesurfer.on('timeupdate', (time) => {
      setCurrentTime(time);
    });

    // Region events
    regionsPlugin.on('region-created', (region: any) => {
      if (onRegionCreate) {
        onRegionCreate({
          start: region.start,
          end: region.end,
        });
      }
    });

    regionsPlugin.on('region-updated', (region: any) => {
      const regionData = regions.find((r) => r.id.toString() === region.id);
      if (regionData && onRegionUpdate) {
        onRegionUpdate(regionData.id, {
          start_sec: region.start,
          end_sec: region.end,
        });
      }
    });

    regionsPlugin.on('region-clicked', (region: any) => {
      const regionData = regions.find((r) => r.id.toString() === region.id);
      if (regionData && onRegionClick) {
        onRegionClick(regionData);
      }
    });

    // Enable drag to create regions
    regionsPlugin.enableDragSelection({
      color: 'rgba(255, 87, 34, 0.3)',
    });

    return () => {
      wavesurfer.destroy();
    };
  }, []);

  // Load audio when URL or file changes
  useEffect(() => {
    if (!wavesurferRef.current) return;

    // Reset ready state when loading new audio
    setIsReady(false);
    setDuration(0);

    let objectUrl: string | null = null;

    if (audioUrl) {
      wavesurferRef.current.load(audioUrl);
    } else if (audioFile) {
      objectUrl = URL.createObjectURL(audioFile);
      wavesurferRef.current.load(objectUrl);
    }

    // Cleanup: revoke blob URL only when component unmounts or audio changes
    return () => {
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [audioUrl, audioFile]);

  // Update regions when they change
  useEffect(() => {
    if (!regionsPluginRef.current || !isReady) return;

    // Clear existing regions
    regionsPluginRef.current.clearRegions();

    // Add regions
    regions.forEach((region, index) => {
      regionsPluginRef.current.addRegion({
        id: region.id.toString(),
        start: region.start_sec,
        end: region.end_sec,
        color: REGION_COLORS[index % REGION_COLORS.length],
        drag: true,
        resize: true,
        content: region.label,
      });
    });
  }, [regions, isReady]);

  // Update zoom
  useEffect(() => {
    if (wavesurferRef.current && isReady && duration > 0) {
      // Only zoom if audio is fully loaded and ready
      try {
        wavesurferRef.current.zoom(zoom);
      } catch (err) {
        console.warn('Failed to zoom:', err);
      }
    }
  }, [zoom, duration, isReady]);

  const handlePlayPause = () => {
    if (wavesurferRef.current && isReady && duration > 0) {
      wavesurferRef.current.playPause();
    }
  };

  const handleStop = () => {
    if (wavesurferRef.current) {
      wavesurferRef.current.stop();
      setIsPlaying(false);
    }
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <Box sx={{ width: '100%' }}>
      {/* Controls */}
      <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 2 }}>
        <IconButton onClick={handlePlayPause} color="primary" disabled={!isReady}>
          {isPlaying ? <Pause /> : <PlayArrow />}
        </IconButton>
        <IconButton onClick={handleStop} color="secondary" disabled={!isReady}>
          <Stop />
        </IconButton>

        <Box sx={{ flex: 1, textAlign: 'center' }}>
          <Typography variant="body2">
            {formatTime(currentTime)} / {formatTime(duration)}
          </Typography>
        </Box>

        <IconButton onClick={() => setZoom(Math.max(10, zoom - 10))} disabled={!isReady}>
          <ZoomOut />
        </IconButton>
        <Typography variant="body2">{zoom}x</Typography>
        <IconButton onClick={() => setZoom(Math.min(200, zoom + 10))} disabled={!isReady}>
          <ZoomIn />
        </IconButton>
      </Stack>

      {/* Waveform */}
      <Box
        ref={waveformRef}
        sx={{
          width: '100%',
          border: '1px solid #ddd',
          borderRadius: 1,
          backgroundColor: '#fafafa',
        }}
      />

      {/* Instructions */}
      <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>
        Click and drag on the waveform to create annotation regions. Drag regions to move, drag edges to resize.
      </Typography>
    </Box>
  );
};

export default WaveformPlayer;
