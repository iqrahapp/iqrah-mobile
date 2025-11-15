type Num = number;

export interface TrimResult {
  start: Num;
  end: Num;
  confidence: Num; // 0..1
}

export interface SpeechSegment {
  start: Num;
  end: Num;
  confidence: Num;
}

type TrimParams = {
  sampleRate?: Num;   // target decode rate (best-effort)
  winMs?: Num;        // analysis window
  hopMs?: Num;        // analysis hop
  padMs?: Num;        // pad on each side
  enterDb?: Num;      // how much above noise to "enter" speech
  exitDb?: Num;       // how much above noise to "exit" speech
  minDurMs?: Num;     // minimum segment duration
  noiseProbeSec?: Num;// head/tail probe length for noise
};

const DEFAULTS: Required<TrimParams> = {
  sampleRate: 16000,
  winMs: 25,
  hopMs: 10,
  padMs: 100,
  enterDb: 12,   // enter when > noise + 12 dB
  exitDb: 6,     // exit when < noise + 6 dB
  minDurMs: 300,
  noiseProbeSec: 2,
};

function toDb(x: Num) {
  // amplitude RMS -> dBFS-ish (relative). Add epsilon to avoid -Inf
  return 20 * Math.log10(Math.max(x, 1e-12));
}

function percentile(sortedAsc: Float32Array, p: Num) {
  const idx = Math.max(0, Math.min(sortedAsc.length - 1, Math.floor((p / 100) * (sortedAsc.length - 1))));
  return sortedAsc[idx];
}

function movingAvgInPlace(arr: Float32Array, radius: number = 2) {
  // simple boxcar smoothing
  if (arr.length === 0 || radius <= 0) return arr;
  const out = new Float32Array(arr.length);
  let acc = 0;
  let count = 0;
  for (let i = 0; i < arr.length; i++) {
    const add = arr[i];
    acc += add;
    count++;
    if (i > radius) {
      acc -= arr[i - radius - 1];
      count--;
    }
    out[i] = acc / count;
  }
  // trailing window
  acc = 0; count = 0;
  for (let i = arr.length - 1; i >= 0; i--) {
    acc += out[i];
    count++;
    if (arr.length - 1 - i > radius) {
      acc -= out[i + radius + 1];
      count--;
    }
    out[i] = acc / count;
  }
  return out;
}

async function decodeToMono(blob: Blob, targetSr: number): Promise<AudioBuffer> {
  const ctx = new AudioContext({ sampleRate: targetSr });
  try {
    const ab = await blob.arrayBuffer();
    const buf = await ctx.decodeAudioData(ab);
    // downmix to mono if needed
    if (buf.numberOfChannels === 1) return buf;
    const len = buf.length;
    const sr = buf.sampleRate;
    const mono = new Float32Array(len);
    const ch0 = buf.getChannelData(0);
    for (let i = 0; i < len; i++) mono[i] += ch0[i];
    for (let c = 1; c < buf.numberOfChannels; c++) {
      const ch = buf.getChannelData(c);
      for (let i = 0; i < len; i++) mono[i] += ch[i];
    }
    for (let i = 0; i < len; i++) mono[i] /= buf.numberOfChannels;
    // Put mono back into a buffer
    const monoBuf = new AudioBuffer({ numberOfChannels: 1, length: len, sampleRate: sr });
    monoBuf.copyToChannel(mono, 0);
    return monoBuf;
  } finally {
    await ctx.close().catch(() => {});
  }
}

function computeRmsTrace(
  mono: Float32Array,
  sr: number,
  winMs: number,
  hopMs: number
): { rms: Float32Array; frameSize: number; hopSize: number } {
  const frameSize = Math.max(1, Math.round(sr * (winMs / 1000)));
  const hopSize = Math.max(1, Math.round(sr * (hopMs / 1000)));
  const nFrames = Math.max(0, Math.floor((mono.length - frameSize) / hopSize) + 1);
  const rms = new Float32Array(nFrames);

  // prefix sum of squares for O(1) window sums
  const N = mono.length;
  const prefix = new Float64Array(N + 1);
  for (let i = 0; i < N; i++) prefix[i + 1] = prefix[i] + mono[i] * mono[i];

  for (let k = 0, pos = 0; k < nFrames; k++, pos += hopSize) {
    const end = pos + frameSize;
    const sumSq = prefix[end] - prefix[pos];
    rms[k] = Math.sqrt(sumSq / frameSize);
  }
  return { rms, frameSize, hopSize };
}

function estimateNoiseDb(
  rms: Float32Array,
  sr: number,
  hopSize: number,
  winMs: number,
  probeSec: number
): Num {
  // probe first+last N seconds, collect frames, take 20th percentile in dB
  const framesPerSec = Math.max(1, Math.round(sr / hopSize));
  const probeFrames = Math.min(rms.length, Math.round(probeSec * framesPerSec));
  const pool: number[] = [];
  for (let i = 0; i < probeFrames; i++) pool.push(rms[i]);
  for (let i = Math.max(0, rms.length - probeFrames); i < rms.length; i++) pool.push(rms[i]);
  if (pool.length === 0) return toDb(1e-3);
  const as = Float32Array.from(pool).sort((a, b) => a - b);
  return toDb(percentile(as, 20));
}

export async function autoTrim(blob: Blob, params: TrimParams = {}): Promise<TrimResult> {
  const P = { ...DEFAULTS, ...params };
  try {
    const audioBuffer = await decodeToMono(blob, P.sampleRate);
    const ch = audioBuffer.getChannelData(0);
    const sr = audioBuffer.sampleRate;

    const { rms, frameSize, hopSize } = computeRmsTrace(ch, sr, P.winMs, P.hopMs);
    if (rms.length === 0) {
      return { start: 0, end: audioBuffer.duration, confidence: 0 };
    }

    // smooth & convert to dB for robust thresholds
    const smoothed = movingAvgInPlace(rms, 2);
    const db = new Float32Array(smoothed.length);
    for (let i = 0; i < smoothed.length; i++) db[i] = toDb(smoothed[i]);

    // noise estimate and hysteresis thresholds
    const noiseDb = estimateNoiseDb(smoothed, sr, hopSize, P.winMs, P.noiseProbeSec);
    const enter = noiseDb + P.enterDb;
    const exit = noiseDb + P.exitDb;

    // find first / last speech using hysteresis
    let first = -1;
    let last = -1;
    let inSpeech = false;
    for (let i = 0; i < db.length; i++) {
      if (!inSpeech && db[i] > enter) { inSpeech = true; first = i; }
      if (inSpeech && db[i] < exit) { inSpeech = false; last = Math.max(last, i - 1); }
    }
    if (inSpeech) last = db.length - 1;

    if (first === -1 || last === -1 || last < first) {
      // fallback: no trim
      return { start: 0, end: audioBuffer.duration, confidence: 0.2 };
    }

    // pad in frames
    const padFrames = Math.round((P.padMs / 1000) * sr / hopSize);
    first = Math.max(0, first - padFrames);
    last = Math.min(db.length - 1, last + padFrames);

    // convert to seconds (end uses +1)
    const start = (first * hopSize) / sr;
    const end = ((last + 1) * hopSize + (frameSize - hopSize)) / sr; // include frame tail

    // SNR (inside segment) vs noise
    const segDb = db.subarray(first, last + 1);
    const segDbSorted = Float32Array.from(segDb).sort((a, b) => a - b);
    const speechDb = percentile(segDbSorted, 60); // robust central loudness
    const snrDb = speechDb - noiseDb;

    // confidence: map 6–18 dB -> 0–1, penalize very short
    const raw = (snrDb - 6) / 12;
    const dur = Math.max(0, end - start);
    const durFactor = Math.min(1, dur / 0.8);
    const confidence = Math.max(0, Math.min(1, raw)) * (0.7 + 0.3 * durFactor);

    return { start: Math.max(0, start), end: Math.min(audioBuffer.duration, end), confidence };
  } catch (e) {
    console.error('autoTrim failed:', e);
    // safer fallback: no trim
    return { start: 0, end: 0, confidence: 0 };
  }
}

export async function detectSpeechBounds(
  audioBlob: Blob,
  clickTime: number,
  context?: { beforeSec?: number; afterSec?: number } & TrimParams
): Promise<SpeechSegment> {
  const P = { ...DEFAULTS, ...context };
  const { beforeSec = 2, afterSec = 5 } = context || {};
  const MIN = (P.minDurMs / 1000);

  const audioBuffer = await decodeToMono(audioBlob, P.sampleRate);
  const mono = audioBuffer.getChannelData(0);
  const sr = audioBuffer.sampleRate;

  const startSample = Math.max(0, Math.floor((clickTime - beforeSec) * sr));
  const endSample = Math.min(mono.length, Math.floor((clickTime + afterSec) * sr));
  const view = mono.subarray(startSample, endSample);

  const { rms, frameSize, hopSize } = computeRmsTrace(view, sr, P.winMs, P.hopMs);
  if (rms.length === 0) {
    const s = Math.max(0, clickTime - MIN / 2);
    const e = Math.min(audioBuffer.duration, s + MIN);
    return { start: s, end: e, confidence: 0.2 };
  }

  const smoothed = movingAvgInPlace(rms, 2);
  const db = new Float32Array(smoothed.length);
  for (let i = 0; i < smoothed.length; i++) db[i] = toDb(smoothed[i]);

  const noiseDb = estimateNoiseDb(smoothed, sr, hopSize, P.winMs, Math.min(beforeSec, 1.5));
  const enter = noiseDb + P.exitDb + 4; // slightly stricter locally
  const exit = noiseDb + P.exitDb;

  const clickFrame = Math.max(0, Math.min(db.length - 1, Math.floor((clickTime * sr - startSample - frameSize) / hopSize)));
  // walk backward to last silence then forward to next silence using hysteresis
  let a = clickFrame, below = 0;
  while (a > 0) {
    if (db[a] < exit) below++; else below = 0;
    if (below >= 5) { a += 5; break; }
    a--;
  }
  if (a <= 0) a = 0;

  let b = clickFrame, below2 = 0;
  while (b < db.length - 1) {
    if (db[b] < exit) below2++; else below2 = 0;
    if (below2 >= 5) { b -= 5; break; }
    b++;
  }
  if (b >= db.length - 1) b = db.length - 1;

  // seconds (include frame tail)
  let start = (startSample + a * hopSize) / sr;
  let end = (startSample + (b + 1) * hopSize + (frameSize - hopSize)) / sr;

  // ensure minimum duration
  if (end - start < MIN) {
    const mid = clickTime;
    start = Math.max(0, mid - MIN / 2);
    end = Math.min(audioBuffer.duration, start + MIN);
  }

  // small padding
  const pad = P.padMs / 1000 / 2;
  start = Math.max(0, start - pad);
  end = Math.min(audioBuffer.duration, end + pad);

  // local SNR/confidence
  const aF = Math.max(0, a);
  const bF = Math.max(aF, b);
  const seg = db.subarray(aF, bF + 1);
  const segSorted = Float32Array.from(seg).sort((x, y) => x - y);
  const speechDb = percentile(segSorted, 60);
  const snrDb = speechDb - noiseDb;
  const conf = Math.max(0, Math.min(1, (snrDb - 6) / 12));

  return { start, end, confidence: conf };
}

export function calculateAverageDuration(segments: Array<{ start: number; end: number }>): number {
  if (segments.length === 0) return 0.3; // 300ms is a safer default for syllables/short words
  const durations = segments.map(s => Math.max(0, s.end - s.start));
  return durations.reduce((a, b) => a + b, 0) / durations.length;
}
