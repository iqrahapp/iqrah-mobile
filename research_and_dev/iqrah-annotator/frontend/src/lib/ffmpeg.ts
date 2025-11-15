import { FFmpeg } from '@ffmpeg/ffmpeg';
import { fetchFile, toBlobURL } from '@ffmpeg/util';
import { getFriendlyFFmpegError, formatCompactError } from '../utils/errorMessages';

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
 *
 * FIX #2: Reduced default timeout from 45s to 15s for word segments
 * Progress callback reports 0-1 (0% to 100%)
 */
export async function trimAudioBlob(
  input: Blob,
  startSec: number,
  endSec: number,
  timeoutMs = 15000, // FIX #2: Reduced from 45000 to 15000 (15s)
  onProgress?: (p: number) => void
): Promise<{ blob: Blob; mime: string }> {
  const { ffmpeg } = await ensureFFmpegLoaded();

  // Create progress handler that we can remove later
  const progressHandler = onProgress
    ? ({ progress }: { progress: number }) => onProgress(progress)
    : undefined;

  try {
    if (progressHandler) {
      ffmpeg.on('progress', progressHandler);
    }

    // Clean up any existing files first to prevent state corruption
    try {
      await ffmpeg.deleteFile('in.webm').catch(() => {});
      await ffmpeg.deleteFile('out.webm').catch(() => {});
      await ffmpeg.deleteFile('out.wav').catch(() => {});
    } catch {
      // Ignore cleanup errors
    }

    // Input
    await ffmpeg.writeFile('in.webm', await fetchFile(input));

    const dur = Math.max(0, endSec - startSec).toFixed(3);

    console.log('[FFmpeg] Trimming command params:', {
      startSec: startSec.toFixed(3),
      endSec: endSec.toFixed(3),
      duration: dur,
      inputSize: input.size,
    });

    // IMPORTANT: Use re-encode for precise trimming instead of stream copy
    // Stream copy with -ss before -i seeks to nearest keyframe (imprecise)
    // For word-level segments, we need frame-accurate trimming
    let result: { blob: Blob; mime: string };

    // Use accurate seeking: -ss AFTER -i for precision (slower but accurate)
    // Re-encode to WAV 16k mono for deterministic output
    const encodeCmd = ['-i', 'in.webm', '-ss', `${startSec}`, '-t', `${dur}`, '-ac', '1', '-ar', '16000', '-c:a', 'pcm_s16le', 'out.wav'];
    console.log('[FFmpeg] Executing precise re-encode:', encodeCmd.join(' '));

    await ffmpeg.exec(encodeCmd, timeoutMs);
    const data = await ffmpeg.readFile('out.wav');
    result = {
      blob: new Blob([new Uint8Array(data as unknown as ArrayBuffer)], { type: 'audio/wav' }),
      mime: 'audio/wav'
    };

    console.log('[FFmpeg] Re-encode succeeded, output size:', result.blob.size);
    console.log('[FFmpeg] Expected WAV size for', dur, 's at 16kHz mono:', parseInt(dur) * 16000 * 2 + 44, 'bytes');

    return result;
  } catch (error) {
    // Convert FFmpeg errors to user-friendly messages
    const friendlyError = getFriendlyFFmpegError(error as Error);
    const errorMessage = formatCompactError(friendlyError);
    console.error('[FFmpeg] Error:', error);
    console.error('[FFmpeg] Friendly message:', errorMessage);

    // Throw a new error with the friendly message
    const enhancedError = new Error(errorMessage);
    (enhancedError as any).originalError = error;
    (enhancedError as any).friendlyError = friendlyError;
    throw enhancedError;
  } finally {
    // CRITICAL: Always clean up, even on error
    if (progressHandler) {
      ffmpeg.off('progress', progressHandler);
    }

    // Clean up virtual filesystem
    try {
      await ffmpeg.deleteFile('in.webm').catch(() => {});
      await ffmpeg.deleteFile('out.webm').catch(() => {});
      await ffmpeg.deleteFile('out.wav').catch(() => {});
    } catch {
      // Ignore cleanup errors
    }
  }
}

// Expose for diagnostics
export function getFFmpegMode(): 'mt' | 'st' | null {
  return _mode;
}
