// src/lib/vad/silero.ts
// Self-contained Silero VAD wrapper for the browser (ONNX Runtime Web)

import * as ort from 'onnxruntime-web';
// Fallback to RMS-based VAD if Silero fails
import {
  autoTrim as rmsAutoTrim,
  detectSpeechBounds as rmsDetectSpeechBounds
} from '../../utils/autoTrim';
import { getFriendlyVADError, formatCompactError } from '../../utils/errorMessages';

export type Num = number;

export interface TrimResult {
  start: Num;
  end: Num;
  confidence: Num; // 0..1
}

export interface SpeechSegment {
  start: Num;
  end: Num;
  confidence: Num; // 0..1
}

export type SileroParams = {
  // Silero / post-processing knobs (tuned for Quran recitation)
  threshold?: number;              // speech prob to trigger (default 0.5)
  minSpeechMs?: number;            // >=250ms
  minSilenceMs?: number;           // >=100ms
  padMs?: number;                  // pad added to both sides (30ms)
  maxMergeGapMs?: number;          // merge segments separated by tiny gaps (<=80ms)
  sampleRate?: number;             // 16000
  windowMs?: number;               // 32ms -> 512 samples @16k
  contextSamples?: number;         // 64 (as in Silero examples)
};

const P_DEFAULT: Required<SileroParams> = {
  threshold: 0.45,        // Lower for Quran sustained vowels
  minSpeechMs: 220,       // Slightly shorter for syllables
  minSilenceMs: 120,      // Longer to respect tajwīd breaths
  padMs: 40,              // More padding for natural boundaries
  maxMergeGapMs: 100,     // Larger gap merge for breath-sized pauses
  sampleRate: 16000,
  windowMs: 32,
  contextSamples: 64,
};

// ---- ORT init ----
let session: ort.InferenceSession | null = null;

// ---- Session queue to prevent concurrent inference ----
let inferenceQueue: Promise<any> = Promise.resolve();
function enqueue<T>(task: () => Promise<T>): Promise<T> {
  const next = inferenceQueue.then(() => task(), () => task()); // ensure chain continues even if previous failed
  inferenceQueue = next.catch(() => {}); // swallow to keep chain alive
  return next;
}

async function ensureOrt(modelUrl = '/models/silero/silero_vad.onnx') {
  if (session) return session;

  // Point ORT to local WASM files
  ort.env.wasm.numThreads = typeof navigator !== 'undefined'
    ? Math.max(1, Math.min(4, Math.floor((navigator.hardwareConcurrency || 4) / 2)))
    : 1;
  ort.env.wasm.simd = true;
  // Important: use file overrides or a prefix. We'll set a prefix to our public dir.
  ort.env.wasm.wasmPaths = '/ort-wasm/';

  session = await ort.InferenceSession.create(modelUrl, {
    executionProviders: ['wasm'],
    graphOptimizationLevel: 'all',
  });
  return session;
}

// ---- Audio decoding / resampling to 16k mono ----
async function decodeToMono16k(blob: Blob, targetSr = 16000): Promise<AudioBuffer> {
  const ab = await blob.arrayBuffer();
  const ctx = new AudioContext(); // use native rate first
  try {
    const buf = await ctx.decodeAudioData(ab);
    const channels = buf.numberOfChannels;
    const lengthSec = buf.duration;
    const offline = new OfflineAudioContext(1, Math.ceil(lengthSec * targetSr), targetSr);

    // Downmix to mono (via ChannelMerger not needed; set gain nodes per channel)
    const mono = offline.createGain();
    mono.gain.value = 1 / channels;

    // Create an AudioBuffer with same content then connect per-channel
    const temp = offline.createBuffer(channels, buf.length, buf.sampleRate);
    for (let c = 0; c < channels; c++) {
      temp.copyToChannel(buf.getChannelData(c), c);
    }
    const tmpSrc = offline.createBufferSource();
    tmpSrc.buffer = temp;

    tmpSrc.connect(mono);
    mono.connect(offline.destination);
    tmpSrc.start(0);

    const out = await offline.startRendering();
    return out;
  } finally {
    await ctx.close().catch(() => {});
  }
}

// ---- Silero stateful iterator ----
// Silero ONNX model I/O names and shapes (from official examples):
// inputs:  "input" [1, effective_window], "state" [2, 1, 128], "sr" [1]
// outputs: "output" [1, 1],                "stateN" [2, 1, 128]
// window:  32ms @16k -> 512 samples; plus 64-sample context prepended.

class SileroVAD {
  private params: Required<SileroParams>;
  private state: ort.Tensor; // Float32[2,1,128]
  private srTensor: ort.Tensor; // Int64[1]
  private context: Float32Array; // last N samples
  private ready = false;

  constructor(params?: SileroParams) {
    this.params = { ...P_DEFAULT, ...(params || {}) };
    // init zero state and context
    this.state = new ort.Tensor('float32', new Float32Array(2 * 1 * 128), [2, 1, 128]);
    this.srTensor = new ort.Tensor('int64', BigInt64Array.from([BigInt(this.params.sampleRate)] as any), [1]);
    this.context = new Float32Array(this.params.contextSamples);
  }

  async load() {
    await ensureOrt();
    this.ready = true;
  }

  reset() {
    this.state = new ort.Tensor('float32', new Float32Array(2 * 1 * 128), [2, 1, 128]);
    this.context.fill(0);
  }

  // Process a streaming Float32Array (mono @16k) and return probs per 32ms frame
  async iterate(floatMono16k: Float32Array): Promise<Float32Array> {
    if (!this.ready) throw new Error('SileroVAD not loaded');
    const sr = this.params.sampleRate;
    const win = Math.round((this.params.windowMs / 1000) * sr); // 512
    const ctxN = this.params.contextSamples;                     // 64
    const hop = win; // Silero advances by window size (no overlap) in reference iterator

    const nFrames = Math.max(0, Math.floor((floatMono16k.length - win) / hop) + 1);
    const probs = new Float32Array(nFrames);

    // Reusable buffer (context + chunk)
    const effective = ctxN + win;
    const input = new Float32Array(effective);

    for (let f = 0, pos = 0; f < nFrames; f++, pos += hop) {
      // prepend last context
      input.set(this.context, 0);
      input.set(floatMono16k.subarray(pos, pos + win), ctxN);

      // update context for next call
      this.context.set(input.subarray(effective - ctxN));

      const inputTensor = new ort.Tensor('float32', input, [1, effective]);
      const feeds: Record<string, ort.Tensor> = {
        input: inputTensor,
        state: this.state as ort.Tensor,
        sr: this.srTensor,
      };
      const out = await enqueue(() => ensureOrt().then(s => s.run(feeds, ['output', 'stateN'])));
      const prob = (out['output'].data as Float32Array)[0]; // [1,1]
      probs[f] = prob;
      this.state = out['stateN'] as ort.Tensor;
    }

    return probs;
  }
}

// ---- Morphological operations for gap closing ----
function maskFromProbs(probs: Float32Array, threshold: number): Uint8Array {
  const m = new Uint8Array(probs.length);
  for (let i = 0; i < probs.length; i++) m[i] = probs[i] >= threshold ? 1 : 0;
  return m;
}

function dilate(mask: Uint8Array, r: number): Uint8Array {
  const out = new Uint8Array(mask.length);
  let run = 0; // sliding count of ones
  // forward pass
  for (let i = 0; i < mask.length; i++) {
    if (mask[i]) run = r; else run = Math.max(0, run - 1);
    out[i] = run ? 1 : 0;
  }
  // backward pass to grow both sides
  const out2 = new Uint8Array(mask.length);
  run = 0;
  for (let i = mask.length - 1; i >= 0; i--) {
    if (out[i]) run = r; else run = Math.max(0, run - 1);
    out2[i] = (out[i] || run) ? 1 : 0;
  }
  return out2;
}

function erode(mask: Uint8Array, r: number): Uint8Array {
  const out = new Uint8Array(mask.length);
  let zeros = 0;
  // forward
  for (let i = 0; i < mask.length; i++) {
    if (mask[i] === 0) zeros = r; else zeros = Math.max(0, zeros - 1);
    out[i] = zeros ? 0 : 1;
  }
  // backward
  const out2 = new Uint8Array(mask.length);
  zeros = 0;
  for (let i = mask.length - 1; i >= 0; i--) {
    if (out[i] === 0) zeros = r; else zeros = Math.max(0, zeros - 1);
    out2[i] = (out[i] === 0 || zeros) ? 0 : 1;
  }
  return out2;
}

function morphClose(mask: Uint8Array, r: number): Uint8Array {
  return erode(dilate(mask, r), r);
}

// ---- Edge snapping to local silence minima ----
function snapToLocalMinEnergy(
  mono: Float32Array,
  sr: number,
  approxTimeSec: number,
  searchMs = 80,
  winMs = 12
): number {
  const search = Math.round((searchMs / 1000) * sr);
  const half = Math.round((winMs / 1000) * sr / 2);
  let bestIdx = Math.round(approxTimeSec * sr);
  let bestEnergy = Number.POSITIVE_INFINITY;

  const start = Math.max(0, bestIdx - search);
  const end = Math.min(mono.length - 1, bestIdx + search);

  for (let i = start; i <= end; i += Math.max(1, Math.floor(half / 2))) {
    const a = Math.max(0, i - half);
    const b = Math.min(mono.length, i + half);
    let sum = 0;
    for (let j = a; j < b; j++) sum += mono[j] * mono[j];
    const e = sum / Math.max(1, b - a);
    if (e < bestEnergy) { bestEnergy = e; bestIdx = i; }
  }
  return bestIdx / sr;
}

// ---- Post-processing: timestamps from probabilities ----
function probsToSegments(
  probs: Float32Array,
  windowMs: number,
  p: Required<SileroParams>
): Array<{ startSec: number; endSec: number; conf: number }> {
  const winSec = windowMs / 1000;
  const minSpeech = p.minSpeechMs / 1000;
  const pad = p.padMs / 1000;

  // Apply morphological closing to merge tiny gaps (breath-sized)
  const gapFrames = Math.round((p.maxMergeGapMs / 1000) / winSec);
  let mask = maskFromProbs(probs, p.threshold);
  mask = morphClose(mask, Math.max(1, gapFrames));

  // Build segments from mask runs of 1s
  const raw: Array<{ a: number; b: number; mean: number; p50: number }> = [];

  const getSegStats = (aIdx: number, bIdx: number) => {
    const slice = probs.subarray(aIdx, bIdx + 1);
    let sum = 0;
    for (let i = 0; i < slice.length; i++) sum += slice[i];
    const sorted = Float32Array.from(slice).sort((x, y) => x - y);
    const p50 = sorted[ Math.floor(sorted.length * 0.5) ] || 0;
    return { mean: slice.length ? sum / slice.length : 0, p50 };
  };

  let triggered = false;
  let segStart = -1;

  for (let i = 0; i < mask.length; i++) {
    const isSpeech = mask[i] === 1;

    if (isSpeech) {
      if (!triggered) {
        triggered = true;
        segStart = i;
      }
    } else if (triggered) {
      // End of speech run
      const a = segStart;
      const b = i - 1;
      triggered = false;
      const durSec = (b - a + 1) * winSec;
      if (durSec >= minSpeech) {
        const { mean, p50 } = getSegStats(a, b);
        raw.push({ a, b, mean, p50 });
      }
    }
  }
  // Handle final segment if still triggered
  if (triggered && segStart >= 0) {
    const a = segStart;
    const b = mask.length - 1;
    const durSec = (b - a + 1) * winSec;
    if (durSec >= minSpeech) {
      const { mean, p50 } = getSegStats(a, b);
      raw.push({ a, b, mean, p50 });
    }
  }

  // Morphological closing already merged gaps, so no additional merging needed
  // Convert to seconds with padding
  return raw.map(({ a, b, mean, p50 }) => {
    let startSec = a * winSec;
    let endSec = (b + 1) * winSec; // include last frame
    startSec = Math.max(0, startSec - pad);
    endSec = endSec + pad;
    const conf = Math.max(0, Math.min(1, (mean * 0.7 + p50 * 0.3))); // mixture conf
    return { startSec, endSec, conf };
  });
}

// ---- Public API (drop-in) ----
export async function autoTrim(blob: Blob, params?: SileroParams): Promise<TrimResult> {
  try {
    const p = { ...P_DEFAULT, ...(params || {}) };
    const buf = await decodeToMono16k(blob, p.sampleRate);

    // Read channel data as a contiguous Float32Array copy (safer than direct reference)
    const mono = new Float32Array(buf.length);
    buf.copyFromChannel(mono, 0);

    const vad = new SileroVAD(p);
    await vad.load();
    vad.reset();
    const probs = await vad.iterate(mono);
    const segs = probsToSegments(probs, p.windowMs, p);

    if (segs.length === 0) {
      // no speech: return full duration with low confidence (consistent with your UX)
      console.warn('[Silero VAD] No voice detected');
      return { start: 0, end: buf.duration, confidence: 0.2 };
    }

    // For trim: take the **first and last** segment bounds
    let start = segs[0].startSec;
    let end = segs[segs.length - 1].endSec;
    const conf = Math.max(...segs.map(s => s.conf)); // optimistic trim confidence

    // Apply edge snapping to align with local silence minima
    start = snapToLocalMinEnergy(mono, p.sampleRate, start, 80, 12);
    end = snapToLocalMinEnergy(mono, p.sampleRate, end, 80, 12);

    return {
      start: Math.max(0, start),
      end: Math.min(buf.duration, end),
      confidence: conf,
    };
  } catch (err) {
    console.error('[Silero VAD] autoTrim failed, falling back to RMS:', err);
    return rmsAutoTrim(blob);
  }
}

export async function detectSpeechBounds(
  audioBlob: Blob,
  clickTime: number,
  context?: { beforeSec?: number; afterSec?: number } & SileroParams
): Promise<SpeechSegment> {
  try {
    const p = { ...P_DEFAULT, ...(context || {}) };
    const { beforeSec = 2, afterSec = 5 } = context || {};

    const buf = await decodeToMono16k(audioBlob, p.sampleRate);
    const sr = buf.sampleRate;

    // Get full mono for edge snapping
    const fullMono = new Float32Array(buf.length);
    buf.copyFromChannel(fullMono, 0);

    const startSample = Math.max(0, Math.floor((clickTime - beforeSec) * sr));
    const endSample = Math.min(buf.length, Math.floor((clickTime + afterSec) * sr));
    const view = new Float32Array(endSample - startSample);
    buf.copyFromChannel(view, 0, startSample);

    const vad = new SileroVAD(p);
    await vad.load();
    vad.reset();
    const probs = await vad.iterate(view);
    const segs = probsToSegments(probs, p.windowMs, p);

    // Choose the segment that contains clickTime if any; else nearest
    const click = clickTime - startSample / sr;
    let chosen = segs.find(s => s.startSec <= click && click <= s.endSec);
    if (!chosen) {
      // nearest by center distance
      chosen = segs
        .map(s => ({ s, d: Math.abs((s.startSec + s.endSec) / 2 - click) }))
        .sort((a, b) => a.d - b.d)[0]?.s;
    }

    if (!chosen) {
      // fallback: minimal window around click
      console.warn('[Silero VAD] No speech segment found near click');
      const minDur = Math.max(0.3, p.minSpeechMs / 1000);
      const half = minDur / 2;
      let s = clickTime - half;
      let e = clickTime + half;
      s = Math.max(0, s);
      e = Math.min(buf.duration, e);
      return { start: s, end: e, confidence: 0.2 };
    }

    // Map back to absolute time
    let start = Math.max(0, chosen.startSec + startSample / sr);
    let end = Math.min(buf.duration, chosen.endSec + startSample / sr);

    // Apply edge snapping to align with local silence minima
    start = snapToLocalMinEnergy(fullMono, sr, start, 80, 12);
    end = snapToLocalMinEnergy(fullMono, sr, end, 80, 12);

    return { start, end, confidence: chosen.conf };
  } catch (err) {
    // Convert to friendly error message but still fall back to RMS
    const friendlyError = getFriendlyVADError(err as Error);
    console.warn('[Silero VAD] Speech detection failed:', formatCompactError(friendlyError));
    console.error('[Silero VAD] Original error:', err);

    // Fallback to RMS-based detection (still works if Silero fails)
    return rmsDetectSpeechBounds(audioBlob, clickTime, context);
  }
}

export function calculateAverageDuration(segments: Array<{ start: number; end: number }>): number {
  if (segments.length === 0) return 0.3; // safer default for short words/syllables
  const durations = segments.map(s => Math.max(0, s.end - s.start));
  return durations.reduce((a, b) => a + b, 0) / durations.length;
}

/**
 * Preload the Silero VAD model at app startup to remove first-use lag.
 * Call this during app initialization (e.g., in a useEffect on app mount).
 */
export async function preloadVAD(): Promise<void> {
  try {
    await ensureOrt();
    console.log('[Silero VAD] Model preloaded successfully');
  } catch (err) {
    console.warn('[Silero VAD] Preload failed, will lazy-load on first use:', err);
  }
}
