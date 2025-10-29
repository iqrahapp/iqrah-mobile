import { FFmpeg } from '@ffmpeg/ffmpeg';
import { fetchFile, toBlobURL } from '@ffmpeg/util';

type Mode = 'mt' | 'st';
let _ffmpeg: FFmpeg | null = null;
let _mode: Mode | null = null;

const CDN = 'https://cdn.jsdelivr.net/npm';
const CORE_VER = '0.12.10'; // matches @ffmpeg/core(-mt) dist version

async function loadURLs(useMT: boolean) {
  const base = `${CDN}/${useMT ? '@ffmpeg/core-mt' : '@ffmpeg/core'}@${CORE_VER}/dist/esm`;
  const coreURL = await toBlobURL(`${base}/ffmpeg-core.js`, 'text/javascript');
  const wasmURL = await toBlobURL(`${base}/ffmpeg-core.wasm`, 'application/wasm');
  const workerURL = useMT
    ? await toBlobURL(`${base}/ffmpeg-core.worker.js`, 'text/javascript')
    : undefined;
  return { coreURL, wasmURL, workerURL };
}

export async function ensureFFmpegLoaded(timeoutMs = 20000) {
  if (_ffmpeg) return { ffmpeg: _ffmpeg, mode: _mode as Mode };

  const mtOk = typeof SharedArrayBuffer !== 'undefined' && (self as any).crossOriginIsolated === true;
  const preferMT = mtOk;

  const { coreURL, wasmURL, workerURL } = await loadURLs(preferMT);

  const ffmpeg = new FFmpeg();

  const loadPromise = ffmpeg.load({ coreURL, wasmURL, workerURL });

  // Guard: never hang the UI; report failure cleanly
  const loaded = await Promise.race([
    loadPromise.then(() => true),
    new Promise<boolean>((resolve) => setTimeout(() => resolve(false), timeoutMs))
  ]);

  if (!loaded && preferMT) {
    // Retry once in ST mode
    console.warn('[FFmpeg] MT timed out; retrying with ST...');
    const { coreURL: coreURLst, wasmURL: wasmURLst } = await loadURLs(false);
    await ffmpeg.load({ coreURL: coreURLst, wasmURL: wasmURLst });
    _ffmpeg = ffmpeg;
    _mode = 'st';
    console.warn('[FFmpeg] Fell back to ST.');
    return { ffmpeg, mode: 'st' as const };
  }

  if (!loaded && !preferMT) {
    throw new Error('[FFmpeg] Failed to load ST core within timeout');
  }

  _ffmpeg = ffmpeg;
  _mode = preferMT ? 'mt' : 'st';
  console.info('[FFmpeg] Loaded', _mode === 'mt' ? 'multithread core (MT).' : 'single-thread (ST).');
  return { ffmpeg, mode: _mode };
}

/**
 * Trim an audio Blob between [startSec, endSec].
 * Fast path: stream copy. Fallback: re-encode to WAV 16k mono.
 */
export async function trimAudioBlob(
  input: Blob,
  startSec: number,
  endSec: number,
  timeoutMs = 45000,
  onProgress?: (p: number) => void
): Promise<{ blob: Blob; mime: string }> {
  const { ffmpeg } = await ensureFFmpegLoaded();

  if (onProgress) {
    ffmpeg.on('progress', ({ progress }) => onProgress(progress));
  }

  // Input
  await ffmpeg.writeFile('in.webm', await fetchFile(input));

  const dur = Math.max(0, endSec - startSec).toFixed(3);

  // Try stream copy (fast, if container slice is clean)
  try {
    await ffmpeg.exec(
      ['-ss', `${startSec}`, '-t', `${dur}`, '-i', 'in.webm', '-vn', '-c:a', 'copy', '-avoid_negative_ts', 'make_zero', 'out.webm'],
      timeoutMs
    );
    const data = await ffmpeg.readFile('out.webm');
    return { blob: new Blob([new Uint8Array(data as unknown as ArrayBuffer)], { type: 'audio/webm' }), mime: 'audio/webm' };
  } catch {
    // Fallback: deterministic re-encode to WAV 16k mono
    await ffmpeg.exec(
      ['-ss', `${startSec}`, '-t', `${dur}`, '-i', 'in.webm', '-ac', '1', '-ar', '16000', '-c:a', 'pcm_s16le', 'out.wav'],
      timeoutMs
    );
    const data = await ffmpeg.readFile('out.wav');
    return { blob: new Blob([new Uint8Array(data as unknown as ArrayBuffer)], { type: 'audio/wav' }), mime: 'audio/wav' };
  }
}

// Expose for diagnostics
export function getFFmpegMode(): 'mt' | 'st' | null {
  return _mode;
}
