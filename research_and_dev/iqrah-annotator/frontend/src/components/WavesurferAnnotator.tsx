import React, { useEffect, useRef, useState } from 'react';
import { Box, Stack, Chip, Tooltip, IconButton } from '@mui/material';
import { Add, ZoomIn, ZoomOut, ContentCopy } from '@mui/icons-material';
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
}) => {
  const waveRef = useRef<HTMLDivElement>(null);
  const timelineRef = useRef<HTMLDivElement>(null);
  const mgrRef = useRef<AnnotationManager | null>(null);

  const [zoom, setZoom] = useState(50);

  useEffect(() => {
    if (!waveRef.current || !timelineRef.current || !src) return;

    const mgr = new AnnotationManager({
      onCreate: (a) => onCreate?.(a),
      onUpdate: (a) => onUpdate?.(a),
      onDelete: (id) => onDelete?.(id),
      onClick: (a) => onClick?.(a),
      onViolation: (issues) => onViolation?.(issues.map(i => i.message)),
    });
    mgrRef.current = mgr;

    mgr.create({
      container: waveRef.current,
      timelineContainer: timelineRef.current,
      audio: src,
      withMinimap: showMinimap,
    });
    mgr.setConstraints(constraints);
    if (controlledKind) mgr.setSelectedKind(controlledKind);

    const ws = (mgr as any).ws as import('wavesurfer.js').default;
    const applyZoom = () => ws.zoom(zoom);
    ws.on('ready', applyZoom);

    return () => mgr.destroy();
  }, [src]);

  useEffect(() => { if (controlledKind) mgrRef.current?.setSelectedKind(controlledKind); }, [controlledKind]);
  useEffect(() => {
    const ws = (mgrRef.current as any)?.ws as import('wavesurfer.js').default | undefined;
    if (!ws) return;
    try { ws.zoom(zoom); } catch {}
  }, [zoom]);

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
      {/* Simple top controls (zoom + add point) */}
      <Stack direction="row" alignItems="center" spacing={1}>
        <Tooltip title="Insert point annotation at playhead (20ms)">
          <span><IconButton onClick={handleAddPoint}><Add /></IconButton></span>
        </Tooltip>
        <Stack direction="row" alignItems="center" sx={{ ml: 'auto' }}>
          <IconButton onClick={() => setZoom((z) => Math.max(10, z - 10))}><ZoomOut /></IconButton>
          <Chip size="small" label={`${zoom}px/s`} />
          <IconButton onClick={() => setZoom((z) => Math.min(400, z + 10))}><ZoomIn /></IconButton>
          <Tooltip title="Copy annotations JSON">
            <IconButton onClick={handleCopyJson}><ContentCopy /></IconButton>
          </Tooltip>
        </Stack>
      </Stack>

      {/* Waveform */}
      <Box id="waveform" ref={waveRef} sx={{ border: '1px solid #ddd', borderRadius: 1, background: '#fafafa', minHeight: 140, outline: 'none' }} tabIndex={0} />
      <Box ref={timelineRef} sx={{ mt: -0.5 }} />
      <Box sx={{ color: 'text.secondary', fontSize: 12 }}>
        Tip: <b>Ctrl/⌘ + Click</b> to drop a point; drag to create a region. Invalid placements are prevented with feedback.
      </Box>
    </Box>
  );
};

export default WavesurferAnnotator;
