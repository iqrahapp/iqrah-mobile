// src/annotation/manager.ts
import WaveSurfer from 'wavesurfer.js';
import Regions from 'wavesurfer.js/dist/plugins/regions.js';
import Timeline from 'wavesurfer.js/dist/plugins/timeline.js';
import Hover from 'wavesurfer.js/dist/plugins/hover.js';
import Minimap from 'wavesurfer.js/dist/plugins/minimap.js';

import type {
  Annotation,
  AnnotationId,
  AnnotationKind,
  AnnotationMeta,
  AnnotationSet,
  ConstraintTable,
  ExportPayload,
} from './types';
import { DEFAULT_CONSTRAINTS, validateAnnotation } from './constraints';

function hashColor(id: string, alpha: number) {
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) >>> 0;
  const hue = h % 360;
  return `rgba(${hslToRgb(hue / 360, 0.6, 0.5).join(',')}, ${alpha})`;
}
function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const f = (n: number) => {
    const k = (n + h * 12) % 12;
    const a = s * Math.min(l, 1 - l);
    return Math.round((l - a * Math.max(-1, Math.min(k - 3, Math.min(9 - k, 1)))) * 255);
  };
  return [f(0), f(8), f(4)];
}

export type UiCallbacks = {
  onViolation?: (issues: { message: string }[]) => void;
  onCreate?: (ann: Annotation) => void;
  onUpdate?: (ann: Annotation) => void;
  onDelete?: (id: AnnotationId) => void;
  onClick?: (ann: Annotation) => void;
};

export class AnnotationManager {
  ws!: WaveSurfer;
  regions!: ReturnType<typeof Regions.create>;
  timeline!: ReturnType<typeof Timeline.create>;
  hover!: ReturnType<typeof Hover.create>;
  minimap?: ReturnType<typeof Minimap.create>;

  private annotations: AnnotationSet = {};
  private selectedKind: AnnotationKind = 'word';
  private constraints: ConstraintTable = DEFAULT_CONSTRAINTS;
  private lastGeometry: Map<string, { start: number; end: number }> = new Map();
  private ui: UiCallbacks;

  constructor(ui: UiCallbacks = {}) { this.ui = ui; }

  create({
    container,
    timelineContainer,
    url,
    audio,
    withMinimap = true,
  }: {
    container: HTMLElement; timelineContainer: HTMLElement;
    url?: string; audio?: Blob | File | string;
    withMinimap?: boolean;
  }) {
    this.ws = WaveSurfer.create({
      container,
      url: typeof audio === 'string' ? audio : url,
      waveColor: '#4F4A85',
      progressColor: '#383351',
      dragToSeek: true,
      minPxPerSec: 50,
      autoScroll: true,
      autoCenter: true,
      barWidth: 1, barGap: 1, barRadius: 1,
    });

    this.regions = this.ws.registerPlugin(Regions.create());
    this.timeline = this.ws.registerPlugin(Timeline.create({ container: timelineContainer }));
    this.hover    = this.ws.registerPlugin(Hover.create({
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
    }));
    if (withMinimap)  this.minimap  = this.ws.registerPlugin(Minimap.create({ height: 24 }));

    if (audio && typeof audio !== 'string') {
      const objUrl = URL.createObjectURL(audio);
      this.ws.load(objUrl);
      this.ws.once('destroy', () => URL.revokeObjectURL(objUrl));
    }

    this.regions.enableDragSelection({
      color: this.colorFor('__preview__', this.selectedKind),
    });

    this.regions.on('region-created', (r) => {
      const id = String(r.id);
      const color = this.colorFor(id, this.selectedKind);
      r.setOptions({
        color,
        content: '',
      });
      if (r.element) {
        r.element.setAttribute('data-type', this.selectedKind);
        r.element.setAttribute('data-id', id);
      }
      const ann = this.toAnnotation(r, this.selectedKind, { color, alpha: this.alphaFor(this.selectedKind) });
      const { valid, issues } = validateAnnotation(ann, this.annotations, this.constraints);
      if (!valid) {
        r.remove();
        this.ui.onViolation?.(issues);
        this.flash('#e53935');
        return;
      }
      this.annotations[id] = ann;
      this.lastGeometry.set(id, { start: ann.start, end: ann.end });
      this.ui.onCreate?.(ann);
    });

    this.regions.on('region-updated', (r) => {
      const id = String(r.id);
      const prev = this.annotations[id];
      if (!prev) return;
      const candidate: Annotation = { ...prev, start: r.start, end: r.end };
      const res = validateAnnotation(candidate, this.annotations, this.constraints, id);

      // Visual feedback during drag
      r.setOptions({ color: res.valid ? prev.meta.color : 'rgba(244,67,54,0.45)' });

      // On release, validate and revert if invalid
      if (!res.valid) {
        // Check if this is the end of the drag (rough heuristic)
        setTimeout(() => {
          const currentR = this.regions.getRegions().find(region => String(region.id) === id);
          if (!currentR) return;

          const stillInvalid = !validateAnnotation(
            { ...prev, start: currentR.start, end: currentR.end },
            this.annotations,
            this.constraints,
            id
          ).valid;

          if (stillInvalid) {
            const geo = this.lastGeometry.get(id);
            if (geo) {
              currentR.setOptions({ start: geo.start, end: geo.end, color: prev.meta.color });
            }
            this.ui.onViolation?.(res.issues);
            this.shake(currentR);
          } else {
            // Valid now, commit the change
            this.annotations[id] = { ...prev, start: currentR.start, end: currentR.end };
            this.lastGeometry.set(id, { start: currentR.start, end: currentR.end });
            this.ui.onUpdate?.(this.annotations[id]);
          }
        }, 100);
      } else {
        // Valid update, commit immediately
        this.annotations[id] = candidate;
        this.lastGeometry.set(id, { start: candidate.start, end: candidate.end });
        this.ui.onUpdate?.(candidate);
      }
    });

    this.regions.on('region-removed', (r) => {
      const id = String(r.id);
      if (this.annotations[id]) {
        delete this.annotations[id];
        this.lastGeometry.delete(id);
        this.ui.onDelete?.(id);
      }
    });

    this.regions.on('region-clicked', (r) => {
      const id = String(r.id);
      const ann = this.annotations[id];
      if (ann) this.ui.onClick?.(ann);
    });

    // WaveSurfer click event (may not fire with Ctrl held)
    this.ws.on('click', (relativeX: number) => {
      const duration = this.ws.getDuration();
      const time = relativeX * duration;
      console.log('[AnnotationManager] WS click event:', { relativeX, time });
    });

    // Container-level click handler for Ctrl+Click
    let ctrlClickHandler: ((e: MouseEvent) => void) | null = null;
    ctrlClickHandler = (e: MouseEvent) => {
      if (!e.ctrlKey && !e.metaKey) return;

      // Get click position relative to waveform
      const rect = container.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const relativeX = x / rect.width;
      const duration = this.ws.getDuration();
      const time = relativeX * duration;

      console.log('[AnnotationManager] Ctrl+Click detected:', { x, relativeX, time, duration });

      if (time >= 0 && time <= duration) {
        e.preventDefault();
        e.stopPropagation();
        console.log('[AnnotationManager] Creating point at:', time);
        this.createPoint(time);
      }
    };
    container.addEventListener('click', ctrlClickHandler);

    container.addEventListener('keydown', (ev) => {
      if ((ev.target as HTMLElement)?.tagName === 'INPUT') return;
      if (ev.code === 'Space') { ev.preventDefault(); this.ws.playPause(); }
    });

    // Support play-range custom event for playing selections
    const playRangeHandler = (e: Event) => {
      const { startTime, endTime } = (e as CustomEvent).detail || {};
      if (startTime == null || endTime == null) return;

      this.ws.setTime(startTime);
      this.ws.play();

      const stopAtEnd = () => {
        if (this.ws.getCurrentTime() >= endTime) {
          this.ws.pause();
          this.ws.un('timeupdate', stopAtEnd);
        }
      };
      this.ws.on('timeupdate', stopAtEnd);
    };
    window.addEventListener('play-range', playRangeHandler);

    // Update destroy to clean up both handlers
    const previousDestroy = this.destroy.bind(this);
    this.destroy = () => {
      if (ctrlClickHandler) {
        container.removeEventListener('click', ctrlClickHandler);
      }
      window.removeEventListener('play-range', playRangeHandler);
      previousDestroy();
    };
  }

  setSelectedKind(kind: AnnotationKind) {
    this.selectedKind = kind;
    const color = this.colorFor('__preview__', kind);
    this.regions.enableDragSelection({ color });
  }
  setConstraints(table: ConstraintTable) { this.constraints = table; }

  createPoint(atSec: number, kind: AnnotationKind = this.selectedKind, meta: Partial<AnnotationMeta> = {}) {
    // Calculate zoom-dependent segment size (10-20ms based on zoom level)
    // At low zoom (50-100 px/s): 20ms, at high zoom (1000+ px/s): 10ms
    const getZoomDependentSegmentSize = (): number => {
      try {
        // Get current zoom level (pixels per second)
        const zoom = (this.ws as any).options?.minPxPerSec ?? 100;

        // Interpolate between 20ms (0.020s) at low zoom and 10ms (0.010s) at high zoom
        // zoom = 50-100: 20ms
        // zoom = 100-500: 20ms -> 15ms
        // zoom = 500-1000: 15ms -> 10ms
        // zoom = 1000+: 10ms

        if (zoom <= 100) return 0.020; // 20ms
        if (zoom >= 1000) return 0.010; // 10ms

        // Linear interpolation
        const t = (zoom - 100) / 900; // Normalize 100-1000 to 0-1
        return 0.020 - (t * 0.010); // 20ms -> 10ms
      } catch {
        return 0.015; // Default 15ms if error
      }
    };

    const len = getZoomDependentSegmentSize();
    const start = atSec;
    const end = Math.min(atSec + len, this.ws.getDuration());

    const id = crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
    const color = meta.color ?? this.colorFor(id, kind, meta.alpha);
    const ann: Annotation = {
      id, kind, start, end,
      meta: { label: meta.label ?? (kind === 'word' ? 'word' : kind), alpha: this.alphaFor(kind), ...meta, color },
    };
    const res = validateAnnotation(ann, this.annotations, this.constraints);
    if (!res.valid) {
      console.error('[AnnotationManager] createPoint validation failed:', res.issues);
      this.ui.onViolation?.(res.issues);
      this.flash('#e53935');
      return null;
    }
    this.annotations[id] = ann;
    this.lastGeometry.set(id, { start, end });
    const region = this.regions.addRegion({ id, start, end, color, content: ann.meta.label ?? '', drag: true, resize: true });
    if (region.element) {
      region.element.setAttribute('data-type', kind);
      region.element.setAttribute('data-id', id);
    }
    console.log('[AnnotationManager] createPoint success (segment size:', (end - start).toFixed(4), 's), calling onCreate:', ann);
    this.ui.onCreate?.(ann);
    return ann;
  }

  /** Restore an annotation without triggering onCreate callback */
  restoreAnnotation(id: string, start: number, end: number, kind: AnnotationKind, meta: Partial<AnnotationMeta> = {}) {
    // Check if already exists
    if (this.annotations[id]) return false;

    const color = meta.color ?? this.colorFor(id, kind, meta.alpha);
    const ann: Annotation = {
      id, kind, start, end,
      meta: { label: meta.label ?? (kind === 'word' ? 'word' : kind), alpha: this.alphaFor(kind), ...meta, color },
    };

    // Add to internal state
    this.annotations[id] = ann;
    this.lastGeometry.set(id, { start, end });

    // Add visual region
    const region = this.regions.addRegion({ id, start, end, color, content: ann.meta.label ?? '', drag: true, resize: true });
    if (region.element) {
      region.element.setAttribute('data-type', kind);
      region.element.setAttribute('data-id', id);
    }

    return true;
  }

  updateAnnotation(id: AnnotationId, patch: Partial<Pick<Annotation, 'start'|'end'|'meta'>>) {
    const prev = this.annotations[id]; if (!prev) return false;
    const candidate: Annotation = { ...prev, ...patch, start: patch.start ?? prev.start, end: patch.end ?? prev.end, meta: { ...prev.meta, ...(patch.meta ?? {}) } };
    const res = validateAnnotation(candidate, this.annotations, this.constraints, id);
    if (!res.valid) { this.ui.onViolation?.(res.issues); return false; }
    this.annotations[id] = candidate;
    this.lastGeometry.set(id, { start: candidate.start, end: candidate.end });
    const region = this.regions.getRegions().find(r => String(r.id) === id);
    region?.setOptions({ start: candidate.start, end: candidate.end, color: candidate.meta.color, content: candidate.meta.label ?? '' });
    this.ui.onUpdate?.(candidate); return true;
  }

  removeAnnotation(id: AnnotationId) {
    const region = this.regions.getRegions().find(r => String(r.id) === id);
    region?.remove();
    if (this.annotations[id]) { delete this.annotations[id]; this.lastGeometry.delete(id); this.ui.onDelete?.(id); }
  }

  /** PERF FIX #3.4: Batch remove annotations by kind (more efficient than one-by-one) */
  removeByKind(kind: AnnotationKind): void {
    const toRemove = Object.values(this.annotations).filter(a => a.kind === kind);
    toRemove.forEach(a => {
      const region = this.regions.getRegions().find(r => String(r.id) === a.id);
      region?.remove();
      delete this.annotations[a.id];
      this.lastGeometry.delete(a.id);
      this.ui.onDelete?.(a.id);
    });
  }

  /** FIX #16: Clear all annotations */
  clearAll(): void {
    const allIds = Object.keys(this.annotations);
    allIds.forEach(id => {
      const region = this.regions.getRegions().find(r => String(r.id) === id);
      region?.remove();
      delete this.annotations[id];
      this.lastGeometry.delete(id);
      this.ui.onDelete?.(id);
    });
  }

  /** FIX #16: Batch update annotations (useful for undo/redo or batch edits) */
  batchUpdate(updates: Array<{ id: AnnotationId; start?: number; end?: number; meta?: Partial<AnnotationMeta> }>): void {
    updates.forEach(({ id, start, end, meta }) => {
      this.updateAnnotation(id, { start, end, meta });
    });
  }

  /** FIX #16: Get statistics about annotations */
  getStats(): {
    total: number;
    byKind: Record<AnnotationKind, number>;
    totalDuration: number;
    avgDuration: number;
  } {
    const annotations = this.getAll();
    const byKind: Record<AnnotationKind, number> = { surah: 0, word: 0, other: 0 };
    let totalDuration = 0;

    annotations.forEach(a => {
      byKind[a.kind] = (byKind[a.kind] || 0) + 1;
      totalDuration += a.end - a.start;
    });

    return {
      total: annotations.length,
      byKind,
      totalDuration,
      avgDuration: annotations.length > 0 ? totalDuration / annotations.length : 0,
    };
  }

  getAll(): Annotation[] { return Object.values(this.annotations).sort((a,b)=>a.start-b.start); }
  queryByKind(kind: AnnotationKind) { return this.getAll().filter(a => a.kind === kind); }
  queryInRange(start: number, end: number) { return this.getAll().filter(a => !(a.end <= start || a.start >= end)); }

  import(payload: { annotations: Annotation[] }) {
    payload.annotations.forEach(a => {
      const res = validateAnnotation(a, this.annotations, this.constraints);
      if (!res.valid) return;
      const color = a.meta.color ?? this.colorFor(a.id, a.kind, a.meta.alpha);
      this.annotations[a.id] = { ...a, meta: { ...a.meta, color } };
      this.lastGeometry.set(a.id, { start: a.start, end: a.end });
      const region = this.regions.addRegion({ id: a.id, start: a.start, end: a.end, color, content: a.meta.label ?? '', drag: true, resize: true });
      if (region.element) {
        region.element.setAttribute('data-type', a.kind);
        region.element.setAttribute('data-id', a.id);
      }
    });
  }

  export(): ExportPayload { return { version: 1, createdAt: new Date().toISOString(), annotations: this.getAll() }; }
  destroy() { this.ws?.destroy(); this.annotations = {}; this.lastGeometry.clear(); }

  private alphaFor(kind: AnnotationKind): number { if (kind==='surah') return 0.18; if (kind==='word') return 0.28; return 0.22; }
  private colorFor(id: string, kind: AnnotationKind, alpha?: number) { return hashColor(id, alpha ?? this.alphaFor(kind)); }
  private shake(region: any) { const el = region.element as HTMLElement | undefined; if (!el) return; el.animate([{transform:'translateX(0)'},{transform:'translateX(-3px)'},{transform:'translateX(3px)'},{transform:'translateX(0)'}],{duration:150}); }
  private flash(color: string) { const el = (this.ws as any)?._container as HTMLElement | undefined; if (!el) return; const orig = el.style.boxShadow; el.style.boxShadow=`0 0 0 2px ${color}`; setTimeout(()=>el.style.boxShadow=orig,180); }
  private toAnnotation(r: any, kind: AnnotationKind, meta: Partial<AnnotationMeta>): Annotation {
    const id = String(r.id);
    return { id, kind, start: r.start, end: r.end, meta: { label: r.content ?? '', color: meta.color, alpha: meta.alpha, parentId: undefined, ...meta } };
  }
}
