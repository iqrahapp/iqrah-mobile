import React, { useEffect, useRef, useState } from 'react';
import { Box, Stack, Chip, Tooltip, IconButton, Typography, Button } from '@mui/material';
import { Add, ZoomIn, ZoomOut, ContentCopy, PlayArrow, Pause, Stop, Remove } from '@mui/icons-material';
import type { Annotation, AnnotationKind, ConstraintTable, ExportPayload } from '../annotation/types';
import { DEFAULT_CONSTRAINTS } from '../annotation/constraints';
import { AnnotationManager } from '../annotation/manager';

type Props = {
  src: string | Blob | File;
  constraints?: ConstraintTable;
  /** If provided, component becomes controlled and hides its internal type toggle */
  controlledKind?: AnnotationKind;
  onCreate?: (a: Annotation) => void;
  onUpdate?: (a: Annotation) => void;
  onDelete?: (id: string) => void;
  onViolation?: (messages: string[]) => void;
  onClick?: (a: Annotation) => void;
  showMinimap?: boolean;
  /** Ref to access the AnnotationManager instance */
  managerRef?: React.MutableRefObject<AnnotationManager | null>;
};

const WavesurferAnnotator: React.FC<Props> = ({
  src,
  constraints = DEFAULT_CONSTRAINTS,
  controlledKind,
  onCreate,
  onUpdate,
  onDelete,
  onViolation,
  onClick,
  showMinimap = true,
  managerRef,
}) => {
  const waveRef = useRef<HTMLDivElement>(null);
  const timelineRef = useRef<HTMLDivElement>(null);
  const mgrRef = useRef<AnnotationManager | null>(null);

  const [zoom, setZoom] = useState(100);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [playbackRate, setPlaybackRate] = useState(1.0);

  useEffect(() => {
    // CRITICAL: Synchronously destroy existing manager FIRST to prevent memory leaks
    // This prevents multiple WaveSurfer instances if src changes rapidly
    const existingManager = mgrRef.current;
    if (existingManager) {
      console.log('[WavesurferAnnotator] Immediate sync destroy of existing manager');
      existingManager.destroy();
      mgrRef.current = null;
      if (managerRef) managerRef.current = null;
    }

    // Guard: Don't create if dependencies missing
    if (!waveRef.current || !timelineRef.current || !src) {
      return;
    }

    // FIX #1: Add abort controller to prevent race conditions
    const abortController = new AbortController();
    let creationAborted = false;

    // Add small delay to ensure cleanup completed and avoid race conditions
    const timer = setTimeout(() => {
      // Check if this creation was aborted while waiting
      if (abortController.signal.aborted || creationAborted) {
        console.log('[WavesurferAnnotator] Creation aborted during delay');
        return;
      }

      console.log('[WavesurferAnnotator] Creating new AnnotationManager');

      const mgr = new AnnotationManager({
        onCreate: (a) => onCreate?.(a),
        onUpdate: (a) => onUpdate?.(a),
        onDelete: (id) => onDelete?.(id),
        onClick: (a) => onClick?.(a),
        onViolation: (issues) => onViolation?.(issues.map(i => i.message)),
      });
      mgrRef.current = mgr;
      if (managerRef) managerRef.current = mgr;

      mgr.create({
        container: waveRef.current!,
        timelineContainer: timelineRef.current!,
        audio: src,
        withMinimap: showMinimap,
      });
      mgr.setConstraints(constraints);
      if (controlledKind) mgr.setSelectedKind(controlledKind);

      const ws = (mgr as any).ws as import('wavesurfer.js').default;
      const applyZoom = () => ws.zoom(zoom);
      ws.on('ready', applyZoom);
    }, 50); // Small delay to ensure previous instance is fully cleaned up

    return () => {
      // FIX #1: Signal abort and cancel pending creation
      creationAborted = true;
      abortController.abort();
      clearTimeout(timer);
      console.log('[WavesurferAnnotator] Cleanup - destroying manager');
      if (mgrRef.current) {
        mgrRef.current.destroy();
        mgrRef.current = null;
      }
      if (managerRef) managerRef.current = null;
    };
  }, [src]);

  useEffect(() => { if (controlledKind) mgrRef.current?.setSelectedKind(controlledKind); }, [controlledKind]);
  useEffect(() => {
    const ws = (mgrRef.current as any)?.ws as import('wavesurfer.js').default | undefined;
    if (!ws) return;
    try { ws.zoom(zoom); } catch {}
  }, [zoom]);

  // Setup playback state tracking
  useEffect(() => {
    if (!mgrRef.current) return;

    const ws = (mgrRef.current as any).ws as import('wavesurfer.js').default;
    if (!ws) return;

    const handlePlayEvent = () => setIsPlaying(true);
    const handlePauseEvent = () => setIsPlaying(false);
    const handleTimeUpdate = (time: number) => setCurrentTime(time);
    const handleReady = () => setDuration(ws.getDuration());

    ws.on('play', handlePlayEvent);
    ws.on('pause', handlePauseEvent);
    ws.on('finish', handlePauseEvent);
    ws.on('audioprocess', handleTimeUpdate);
    ws.on('ready', handleReady);

    return () => {
      ws.un('play', handlePlayEvent);
      ws.un('pause', handlePauseEvent);
      ws.un('finish', handlePauseEvent);
      ws.un('audioprocess', handleTimeUpdate);
      ws.un('ready', handleReady);
    };
  }, [mgrRef.current]);

  // Keyboard shortcuts for playback control
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      // Ignore if typing in input field
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      const ws = (mgrRef.current as any)?.ws as import('wavesurfer.js').default | undefined;
      if (!ws) return;

      switch(e.code) {
        case 'Space':
          e.preventDefault();
          ws.playPause();
          break;
        case 'ArrowLeft':
          if (e.shiftKey) {
            // Shift+Left: -1 second
            ws.skip(-1);
          } else {
            // Left: -0.1 second
            ws.skip(-0.1);
          }
          break;
        case 'ArrowRight':
          if (e.shiftKey) {
            // Shift+Right: +1 second
            ws.skip(1);
          } else {
            // Right: +0.1 second
            ws.skip(0.1);
          }
          break;
        case 'Home':
          ws.seekTo(0);
          break;
        case 'End':
          ws.seekTo(1);
          break;
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [mgrRef.current]);

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = (seconds % 60).toFixed(3); // Show milliseconds
    return `${mins}:${secs.padStart(6, '0')}`;
  };

  const handlePlayPause = () => {
    const ws = (mgrRef.current as any)?.ws as import('wavesurfer.js').default | undefined;
    if (!ws) return;
    ws.playPause();
  };

  const handleStop = () => {
    const ws = (mgrRef.current as any)?.ws as import('wavesurfer.js').default | undefined;
    if (!ws) return;
    ws.stop();
    setIsPlaying(false);
  };

  const handlePlaybackRateChange = (rate: number) => {
    const ws = (mgrRef.current as any)?.ws as import('wavesurfer.js').default | undefined;
    if (!ws) return;

    const mediaElement = (ws as any).media;
    if (mediaElement) {
      mediaElement.playbackRate = rate;
      setPlaybackRate(rate);
    }
  };

  const handleAddPoint = () => {
    const ws = (mgrRef.current as any)?.ws as import('wavesurfer.js').default | undefined;
    if (!ws) return;
    mgrRef.current?.createPoint(ws.getCurrentTime(), controlledKind);
  };

  const handleCopyJson = async () => {
    const payload: ExportPayload = mgrRef.current!.export();
    await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
  };

  return (
    <Box sx={{ width: '100%', display: 'flex', flexDirection: 'column', gap: 1 }}>
      {/* Playback Controls Row */}
      <Stack direction="row" alignItems="center" spacing={2} sx={{ p: 1, bgcolor: 'grey.50', borderRadius: 1 }}>
        <Tooltip title={isPlaying ? "Pause (Space)" : "Play (Space)"}>
          <IconButton onClick={handlePlayPause} color="primary" size="large">
            {isPlaying ? <Pause /> : <PlayArrow />}
          </IconButton>
        </Tooltip>

        <Tooltip title="Stop">
          <IconButton onClick={handleStop} color="secondary">
            <Stop />
          </IconButton>
        </Tooltip>

        <Box sx={{ minWidth: 150, textAlign: 'center', fontFamily: 'monospace', fontSize: 14 }}>
          {formatTime(currentTime)} / {formatTime(duration)}
        </Box>

        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Typography variant="caption">Speed:</Typography>
          <Tooltip title="Slower speed (adaptive: 0.025x at ultra-slow, 0.05x at slow, 0.1x at normal)">
            <IconButton
              size="small"
              onClick={() => {
                // Adaptive decrements: smaller steps for slow speeds
                const decrement = playbackRate <= 0.2 ? 0.025 : playbackRate <= 0.5 ? 0.05 : 0.1;
                handlePlaybackRateChange(Math.max(0.05, playbackRate - decrement));
              }}
            >
              <Remove />
            </IconButton>
          </Tooltip>
          <Chip label={`${playbackRate.toFixed(3)}x`} size="small" sx={{ minWidth: 70 }} />
          <Tooltip title="Faster speed (adaptive increments)">
            <IconButton
              size="small"
              onClick={() => {
                // Adaptive increments: smaller steps for slow speeds
                const increment = playbackRate < 0.2 ? 0.025 : playbackRate < 0.5 ? 0.05 : 0.1;
                handlePlaybackRateChange(Math.min(4.0, playbackRate + increment));
              }}
            >
              <Add />
            </IconButton>
          </Tooltip>
          <Button size="small" onClick={() => handlePlaybackRateChange(1)} variant="outlined">
            1x
          </Button>
          <Button size="small" onClick={() => handlePlaybackRateChange(0.25)} variant="outlined">
            0.25x
          </Button>
          <Button size="small" onClick={() => handlePlaybackRateChange(0.1)} variant="outlined">
            0.1x
          </Button>
        </Box>

        <Box sx={{ flexGrow: 1 }} />

        <Tooltip title={`Insert point annotation at playhead (zoom-dependent: 10-20ms)`}>
          <IconButton onClick={handleAddPoint}><Add /></IconButton>
        </Tooltip>
      </Stack>

      {/* Zoom Controls Row */}
      <Stack direction="row" alignItems="center" spacing={1}>
        <Stack direction="row" alignItems="center" sx={{ ml: 'auto' }}>
          <IconButton onClick={() => setZoom((z) => {
            const decrement = z > 500 ? 200 : z > 200 ? 100 : 50;
            return Math.max(50, z - decrement);
          })}><ZoomOut /></IconButton>
          <Chip size="small" label={`${zoom}px/s`} />
          <IconButton onClick={() => setZoom((z) => {
            const increment = z < 200 ? 50 : z < 500 ? 100 : 200;
            return Math.min(3000, z + increment);
          })}><ZoomIn /></IconButton>
          <Tooltip title="Copy annotations JSON">
            <IconButton onClick={handleCopyJson}><ContentCopy /></IconButton>
          </Tooltip>
        </Stack>
      </Stack>

      {/* Waveform */}
      <Box
        id="waveform"
        ref={waveRef}
        sx={{
          border: '1px solid #ddd',
          borderRadius: 1,
          background: '#fafafa',
          minHeight: 140,
          outline: 'none'
        }}
        tabIndex={0}
      />
      <Box ref={timelineRef} sx={{ mt: -0.5 }} />
      <Box sx={{ color: 'text.secondary', fontSize: 12 }}>
        <strong>Keyboard:</strong> Space = play/pause | ←/→ = seek ±100ms | Shift+←/→ = ±1s | Home/End = start/end
        <br />
        <strong>Mouse:</strong> <b>Ctrl/⌘ + Click</b> = drop point | Drag = create region
        <br />
        <strong>Speed:</strong> Supports 0.05x–4.0x (ultra-slow to fast) with adaptive increments | Use preset buttons for quick access
      </Box>
    </Box>
  );
};

export default WavesurferAnnotator;
