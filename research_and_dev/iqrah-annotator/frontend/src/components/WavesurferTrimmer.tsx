import React, { useEffect, useRef, useState, useCallback } from 'react'
import { Stack, Button, Typography } from '@mui/material'
import WaveSurfer from 'wavesurfer.js'
import Regions from 'wavesurfer.js/dist/plugins/regions.js'
import Timeline from 'wavesurfer.js/dist/plugins/timeline.js'
import Hover from 'wavesurfer.js/dist/plugins/hover.js'
import Minimap from 'wavesurfer.js/dist/plugins/minimap.js'

type Trim = { start: number; end: number }
type Props = { audioUrl: string; value?: Trim | null; onChange?: (trim: Trim) => void }

const WavesurferTrimmer: React.FC<Props> = ({ audioUrl, value, onChange }) => {
  const waveRef = useRef<HTMLDivElement>(null)
  const timelineRef = useRef<HTMLDivElement>(null)
  const wsRef = useRef<WaveSurfer | null>(null)
  const regionsRef = useRef<ReturnType<typeof Regions.create> | null>(null)
  const leftOverlayRef = useRef<HTMLDivElement>(null)
  const rightOverlayRef = useRef<HTMLDivElement>(null)

  const [zoom, setZoom] = useState(100) // pixels per second (minPxPerSec)

  // PERF FIX #3.5: Use proper throttling with trailing call to ensure last update applies
  const updateOverlaysImmediate = useCallback((start: number, end: number) => {
    if (!wsRef.current || !waveRef.current) return;

    const duration = wsRef.current.getDuration();

    // Calculate positions as percentages
    const startPercent = (start / duration) * 100;
    const endPercent = (end / duration) * 100;

    // Update left overlay (covers 0 to start)
    if (leftOverlayRef.current) {
      leftOverlayRef.current.style.width = `${startPercent}%`;
    }

    // Update right overlay (covers end to duration)
    if (rightOverlayRef.current) {
      rightOverlayRef.current.style.width = `${100 - endPercent}%`;
    }
  }, []);

  // Simple throttle: allow at most one call every 16ms, but ensure last call is applied
  const lastCallRef = useRef<{ start: number; end: number } | null>(null);
  const throttleTimerRef = useRef<number | null>(null);

  const updateOverlays = useCallback((start: number, end: number) => {
    lastCallRef.current = { start, end };

    if (throttleTimerRef.current) {
      // Already throttling, last call will be applied when timer completes
      return;
    }

    // Execute immediately
    updateOverlaysImmediate(start, end);

    // Set up throttle timer
    throttleTimerRef.current = window.setTimeout(() => {
      throttleTimerRef.current = null;

      // Apply last queued call if it's different
      if (lastCallRef.current && (lastCallRef.current.start !== start || lastCallRef.current.end !== end)) {
        updateOverlaysImmediate(lastCallRef.current.start, lastCallRef.current.end);
      }
      lastCallRef.current = null;
    }, 16);
  }, [updateOverlaysImmediate]);

  const upsertTrimRegion = (start: number, end: number) => {
    const regions = regionsRef.current!.getRegions()
    const prev = regions.find(r => String(r.id) === 'trim-region')
    // Use a green semi-transparent overlay to show the kept region
    const opts = {
      id: 'trim-region',
      start,
      end,
      color: 'rgba(76, 175, 80, 0.15)', // Light green overlay
      drag: true,
      resize: true,
      content: 'Keep this region'
    }
    if (!prev) {
      const region = regionsRef.current!.addRegion(opts);
      if (region.element) {
        region.element.setAttribute('role', 'trim');
        // Add a more visible border
        region.element.style.border = '2px solid #4CAF50';
        region.element.style.borderRadius = '4px';
      }
    } else {
      prev.setOptions(opts);
      if (prev.element) {
        prev.element.style.border = '2px solid #4CAF50';
        prev.element.style.borderRadius = '4px';
      }
    }

    // Update the overlays
    updateOverlays(start, end);
  }

  useEffect(() => {
    if (!waveRef.current || !timelineRef.current) return

    const ws = WaveSurfer.create({
      container: waveRef.current,
      url: audioUrl,
      waveColor: '#4F4A85',
      progressColor: '#383351',
      minPxPerSec: zoom,
      autoScroll: true,
      autoCenter: true,
      dragToSeek: true,
      barWidth: 1, barGap: 1, barRadius: 1,
      plugins: [],
    })

    const regions = ws.registerPlugin(Regions.create())
    regionsRef.current = regions
    ws.registerPlugin(Timeline.create({ container: timelineRef.current }))
    ws.registerPlugin(Hover.create({
      lineColor: '#ff0000',
      lineWidth: 2,
      labelBackground: '#555',
      labelColor: '#fff',
      labelSize: '11px',
      formatTimeCallback: (seconds: number) => {
        const minutes = Math.floor(seconds / 60);
        const secs = (seconds % 60).toFixed(3);
        return minutes > 0 ? `${minutes}:${secs.padStart(6, '0')}` : `${secs}s`;
      }
    }))
    ws.registerPlugin(Minimap.create({ height: 24 }))

    regions.enableDragSelection({ color: 'rgba(76, 175, 80, 0.15)' })

    ws.on('ready', () => {
      const dur = ws.getDuration()
      const initial: Trim =
        value && value.end > value.start ? value : { start: Math.max(0, dur * 0.05), end: Math.max(0.3, dur * 0.9) }
      upsertTrimRegion(initial.start, initial.end)
      onChange?.(initial)
    })

    regions.on('region-created', r => {
      if (String(r.id) !== 'trim-region') {
        const existed = regions.getRegions().find(x => String(x.id) === 'trim-region')
        if (existed) r.remove()
        else {
          r.setOptions({ id: 'trim-region', color: 'rgba(76, 175, 80, 0.15)', content: 'Keep this region' })
          if (r.element) {
            r.element.style.border = '2px solid #4CAF50';
            r.element.style.borderRadius = '4px';
          }
        }
      }
    })

    regions.on('region-updated', r => {
      r.setOptions({ color: 'rgba(76, 175, 80, 0.15)' });
      if (r.element) {
        r.element.style.border = '2px solid #4CAF50';
        r.element.style.borderRadius = '4px';
      }

      // Commit changes on update
      const start = Math.max(0, Math.min(r.start, r.end))
      const end = Math.max(start, Math.max(r.start, r.end))
      if (Math.abs(r.start - start) > 0.001 || Math.abs(r.end - end) > 0.001) {
        r.setOptions({ start, end })
      }

      // Update overlays
      updateOverlays(start, end);
      onChange?.({ start, end })
    })

    wsRef.current = ws
    return () => {
      console.log('[WavesurferTrimmer] Cleanup - destroying WaveSurfer');
      ws.destroy();

      // Clear throttle timer to prevent memory leak
      if (throttleTimerRef.current) {
        clearTimeout(throttleTimerRef.current);
        throttleTimerRef.current = null;
      }

      // NOTE: Do NOT revoke blob URLs here! Only the parent component that CREATES
      // the blob URL should revoke it. Child components receiving URLs as props
      // should never revoke them, as this causes race conditions in React StrictMode.
    }
  }, [audioUrl])

  useEffect(() => { if (value && regionsRef.current) upsertTrimRegion(value.start, value.end) }, [value?.start, value?.end])

  // Sync zoom changes to WaveSurfer instance (only after audio is ready)
  useEffect(() => {
    if (!wsRef.current) return;

    const ws = wsRef.current;

    // Guard: Only zoom if audio is loaded
    const applyZoom = () => {
      try {
        console.log('[WavesurferTrimmer] Syncing zoom to:', zoom);
        ws.zoom(zoom);
      } catch (err) {
        console.warn('[WavesurferTrimmer] Zoom failed (audio not ready yet):', err);
      }
    };

    // If already ready, apply immediately
    if (ws.getDuration() > 0) {
      applyZoom();
    } else {
      // Otherwise, wait for ready event
      ws.once('ready', applyZoom);
    }

    return () => {
      // Clean up listener if zoom changes before ready
      ws.un('ready', applyZoom);
    };
  }, [zoom]);

  const handleZoomIn = () => {
    // Use larger increments for higher zoom levels (exponential feel)
    const increment = zoom < 200 ? 50 : zoom < 500 ? 100 : 200;
    const newZoom = Math.min(3000, zoom + increment);
    console.log('[WavesurferTrimmer] Zoom in button clicked, new zoom:', newZoom);
    setZoom(newZoom) // Effect will apply to WaveSurfer
  }

  const handleZoomOut = () => {
    // Use larger decrements for higher zoom levels (exponential feel)
    const decrement = zoom > 500 ? 200 : zoom > 200 ? 100 : 50;
    const newZoom = Math.max(50, zoom - decrement);
    console.log('[WavesurferTrimmer] Zoom out button clicked, new zoom:', newZoom);
    setZoom(newZoom) // Effect will apply to WaveSurfer
  }

  return (
    <div style={{ width: '100%' }}>
      <div style={{ position: 'relative' }}>
        {/* Left overlay - trimmed out area */}
        <div
          ref={leftOverlayRef}
          style={{
            position: 'absolute',
            left: 0,
            top: 0,
            bottom: 0,
            width: '0%',
            backgroundColor: 'rgba(0, 0, 0, 0.3)',
            pointerEvents: 'none',
            zIndex: 10,
            borderRight: '2px solid #f44336',
          }}
        />
        {/* Right overlay - trimmed out area */}
        <div
          ref={rightOverlayRef}
          style={{
            position: 'absolute',
            right: 0,
            top: 0,
            bottom: 0,
            width: '0%',
            backgroundColor: 'rgba(0, 0, 0, 0.3)',
            pointerEvents: 'none',
            zIndex: 10,
            borderLeft: '2px solid #f44336',
          }}
        />
        <div id="trimmer" ref={waveRef} style={{ height: 160, border: '1px solid #ddd', borderRadius: 4, background: '#fafafa' }} />
      </div>
      <div ref={timelineRef} />

      <Stack direction="row" spacing={2} alignItems="center" sx={{ mt: 1 }}>
        <Typography variant="body2">Zoom:</Typography>
        <Button size="small" variant="outlined" onClick={handleZoomOut}>
          -
        </Button>
        <Typography variant="body2" sx={{ minWidth: 60, textAlign: 'center' }}>
          {zoom}px/s
        </Typography>
        <Button size="small" variant="outlined" onClick={handleZoomIn}>
          +
        </Button>
        <Typography variant="caption" color="text.secondary" sx={{ ml: 2 }}>
          <strong style={{ color: '#4CAF50' }}>Green:</strong> Keep
          <strong style={{ color: '#f44336', marginLeft: 8 }}>Dark:</strong> Trim out
        </Typography>
      </Stack>
    </div>
  )
}
export default WavesurferTrimmer
