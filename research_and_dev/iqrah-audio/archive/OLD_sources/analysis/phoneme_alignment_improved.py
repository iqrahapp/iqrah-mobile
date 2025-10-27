"""
Improved Phoneme Alignment with Windowed CTC + Post-Processing
================================================================

Implements recommendations to fix CTC forced alignment issues:
1. Windowed per-word alignment (prevents blank bleed)
2. Energy+flux boundary snapping (acoustic minima)
3. VAD-based trailing silence trimming
4. Duration mass-balancing per word

Author: Based on AI recommendations for Quranic recitation
"""

import torch
import torchaudio
import numpy as np
import librosa
from typing import List, Dict, Tuple
from pathlib import Path


_model = None
_tokenizer = None
_aligner = None


def _get_wav2vec2():
    """Load Wav2Vec2 FA model (cached)."""
    global _model, _tokenizer, _aligner
    if _model is None:
        print("📥 Loading Wav2Vec2 CTC Aligner...")
        bundle = torchaudio.pipelines.MMS_FA
        _model = bundle.get_model(with_star=False)
        _tokenizer = bundle.get_tokenizer()
        _aligner = bundle.get_aligner()
        print("   ✓ Wav2Vec2 loaded")
    return _model, _tokenizer, _aligner


def romanize_arabic(arabic_text: str) -> str:
    """Romanize Arabic using uroman."""
    from uroman import Uroman
    uroman = Uroman()
    return uroman.romanize_string(arabic_text)


def windowed_ctc_align(
    emissions: torch.Tensor,
    word_segments: List[Dict],
    romanized_words: List[str],
    tokenizer,
    aligner,
    sr: int = 16000,
    pad_ms: int = 20
) -> List[Dict]:
    """
    Run CTC forced alignment per word with padding.

    This prevents blank tokens from bleeding across word boundaries.

    Args:
        emissions: CTC emissions [T, vocab]
        word_segments: List of word dicts with 'start_ms', 'end_ms'
        romanized_words: Romanized text for each word
        tokenizer: MMS tokenizer
        aligner: MMS aligner
        sr: Sample rate
        pad_ms: Padding around word boundaries (ms)

    Returns:
        List of character spans: [{'char': 'a', 'start': 0.1, 'end': 0.15}, ...]
    """
    # Calculate emission frame rate
    # MMS-FA typically has 50Hz frame rate (20ms hop)
    emission_rate = 50.0  # Hz (frames per second)

    char_spans = []
    labels = torchaudio.pipelines.MMS_FA.get_labels(star=None)

    for word_idx, (word_seg, word_roman) in enumerate(zip(word_segments, romanized_words)):
        if not word_roman.strip():
            continue

        # Word time boundaries
        w_start_ms = word_seg['start_ms']
        w_end_ms = word_seg['end_ms']

        # Convert to emission frames with padding
        pad_samples = int((pad_ms / 1000) * sr)
        start_sample = max(0, int((w_start_ms / 1000) * sr) - pad_samples)
        end_sample = int((w_end_ms / 1000) * sr) + pad_samples

        # Map samples to emission frames
        # Assume emissions cover full waveform at emission_rate Hz
        i0 = int(start_sample / sr * emission_rate)
        i1 = int(end_sample / sr * emission_rate)
        i0 = max(0, i0)
        i1 = min(emissions.size(0), i1)

        # Slice emissions for this word
        E_word = emissions[i0:i1]

        if E_word.size(0) == 0:
            continue

        # Tokenize word (remove spaces, lowercase)
        word_clean = word_roman.replace(' ', '').replace('-', '').replace("'", '').lower()
        if not word_clean:
            continue

        try:
            tokens = tokenizer(word_clean)
        except Exception as e:
            print(f"   ⚠️ Tokenization failed for '{word_clean}': {e}")
            continue

        # Align within window
        try:
            # Aligner expects emissions WITHOUT batch dimension
            alignment = aligner(E_word, tokens)
        except Exception as e:
            print(f"   ⚠️ Alignment failed for '{word_clean}': {e}")
            continue

        # Convert frame indices to absolute times
        for word_align in alignment:
            for token_span in word_align:
                # Frame indices relative to E_word
                frame_start = token_span.start
                frame_end = token_span.end

                # Convert to absolute emission frame indices
                abs_frame_start = i0 + frame_start
                abs_frame_end = i0 + frame_end

                # Convert to absolute time
                t_start = abs_frame_start / emission_rate
                t_end = abs_frame_end / emission_rate

                char = labels[token_span.token] if token_span.token < len(labels) else '<unk>'

                char_spans.append({
                    'char': char,
                    'start': float(t_start),
                    'end': float(t_end),
                    'duration': float(t_end - t_start),
                    'word_index': word_idx
                })

    return char_spans


def compute_salience(
    rms: np.ndarray,
    spec_flux: np.ndarray,
    voicing: np.ndarray,
    alpha: float = 0.6,
    beta: float = 0.3,
    gamma: float = 0.1
) -> np.ndarray:
    """
    Compute acoustic salience for boundary detection.

    salience(t) = α·RMS_z(t) + β·SpecFlux_z(t) + γ·(1-voicing(t))

    Lower salience = better boundary location.
    """
    def z_score(a):
        """Z-score normalization."""
        return (a - np.mean(a)) / (np.std(a) + 1e-8)

    rms_z = z_score(rms)
    flux_z = z_score(spec_flux)

    salience = alpha * rms_z + beta * flux_z + gamma * (1 - voicing)
    return salience


def snap_boundaries_to_minima(
    spans: List[Dict],
    rms: np.ndarray,
    spec_flux: np.ndarray,
    voicing: np.ndarray,
    times: np.ndarray,
    win_ms: int = 50,
    alpha: float = 0.6,
    beta: float = 0.3,
    gamma: float = 0.1
) -> List[Dict]:
    """
    Snap phoneme boundaries to local acoustic minima.

    Uses combination of energy, spectral flux, and voicing.
    """
    if len(spans) < 2:
        return spans

    # Compute salience
    salience = compute_salience(rms, spec_flux, voicing, alpha, beta, gamma)

    hop = np.median(np.diff(times)) if len(times) > 1 else 0.01
    W = int(round(win_ms / 1000 / hop))

    # Snap interior boundaries
    for i in range(1, len(spans)):
        t = spans[i]['start']  # Boundary time
        k = np.argmin(np.abs(times - t))

        # Window around boundary
        s = max(0, k - W)
        e = min(len(times), k + W + 1)

        if e - s < 2:
            continue

        # Find local minimum in salience
        sal_window = salience[s:e]
        kk = s + np.argmin(sal_window)
        t_new = times[kk]

        # Update boundary
        spans[i-1]['end'] = t_new
        spans[i-1]['duration'] = t_new - spans[i-1]['start']
        spans[i]['start'] = t_new
        spans[i]['duration'] = spans[i]['end'] - t_new

    return spans


def trim_trailing_silence(
    spans: List[Dict],
    rms: np.ndarray,
    voicing: np.ndarray,
    times: np.ndarray,
    rms_thr: float = 0.02,
    min_sil_ms: int = 200
) -> List[Dict]:
    """
    Trim last phoneme if it overlaps trailing silence.

    Uses VAD: silence = low RMS + no voicing for ≥ min_sil_ms.
    """
    if len(spans) == 0:
        return spans

    hop = np.median(np.diff(times)) if len(times) > 1 else 0.01
    K = int(round(min_sil_ms / 1000 / hop))

    # Detect silence: low RMS AND no voicing
    sil = (rms < rms_thr) & (voicing < 0.5)

    # Find trailing silence run
    run = 0
    for i in range(len(sil) - 1, -1, -1):
        if sil[i]:
            run += 1
        else:
            break

    if run >= K:
        # Found trailing silence
        t_cut = times[len(sil) - run]

        # Trim last span if it overlaps
        if spans[-1]['end'] > t_cut:
            spans[-1]['end'] = t_cut
            spans[-1]['duration'] = t_cut - spans[-1]['start']

    return spans


def balance_word_durations(
    spans: List[Dict],
    word_segments: List[Dict],
    min_duration_ms: float = 30.0
) -> List[Dict]:
    """
    Ensure phoneme durations sum to word duration (soft scaling).

    Scales phonemes proportionally while maintaining min duration.
    IMPORTANT: Don't force exact match - allow small gaps/overlaps.
    """
    out = []

    # Group spans by word
    spans_by_word = {}
    for span in spans:
        word_idx = span.get('word_index', 0)
        if word_idx not in spans_by_word:
            spans_by_word[word_idx] = []
        spans_by_word[word_idx].append(span)

    # Balance each word
    for word_idx, word_seg in enumerate(word_segments):
        if word_idx not in spans_by_word:
            continue

        word_spans = spans_by_word[word_idx]
        w_start = word_seg['start_ms'] / 1000
        w_end = word_seg['end_ms'] / 1000
        W = w_end - w_start

        # Current total duration
        D = sum(s['duration'] for s in word_spans)

        if D > 0 and W > 0:
            # Only scale if mismatch is significant (>10%)
            mismatch = abs(D - W) / W
            if mismatch > 0.10:
                scale = W / D
                # Apply softer scaling to avoid over-correction
                scale = 1.0 + 0.5 * (scale - 1.0)  # Halfway between 1.0 and target
            else:
                scale = 1.0  # Good enough, don't scale

            # Apply scaling
            for span in word_spans:
                d = max(min_duration_ms / 1000, span['duration'] * scale)
                span['duration'] = d
                out.append(span)

            # Adjust start/end times to be continuous within word
            if len(out) > 0:
                first_in_word = len(out) - len(word_spans)
                out[first_in_word]['start'] = w_start

                for i in range(first_in_word, len(out)):
                    if i > first_in_word:
                        out[i]['start'] = out[i-1]['end']
                    out[i]['end'] = out[i]['start'] + out[i]['duration']
        else:
            out.extend(word_spans)

    return out


def extract_phonemes_improved(
    audio_path: str,
    word_segments: List[Dict],
    transliteration: str,
    pitch_data: Dict,
    surah: int,
    ayah: int,
    device: str = 'cpu'
) -> List[Dict]:
    """
    Extract phonemes with improved alignment.

    Pipeline:
    1. Windowed CTC alignment per word
    2. Energy+flux boundary snapping
    3. Trailing silence trimming
    4. Duration mass-balancing
    5. Syllable grouping
    6. Tajweed annotation
    """
    print(f"\n🎯 Improved Phoneme Alignment Pipeline")
    print(f"   Transliteration: {transliteration}")

    # Load audio
    waveform, sr = torchaudio.load(audio_path)
    if waveform.size(0) > 1:
        waveform = waveform.mean(dim=0, keepdim=True)
    if sr != 16000:
        resampler = torchaudio.transforms.Resample(sr, 16000)
        waveform = resampler(waveform)
        sr = 16000

    # Get model
    model, tokenizer, aligner = _get_wav2vec2()
    model = model.to(device)

    # Generate emissions
    with torch.inference_mode():
        emissions, _ = model(waveform.to(device))
        emissions = torch.log_softmax(emissions, dim=-1)
    emissions = emissions.cpu().squeeze(0)  # [T, vocab]

    # Romanize words
    words = transliteration.split()
    romanized_words = [romanize_arabic(w) for w in words]

    print(f"\n1️⃣ Windowed CTC alignment ({len(word_segments)} words)...")
    char_spans = windowed_ctc_align(
        emissions, word_segments, romanized_words,
        tokenizer, aligner, sr
    )
    print(f"   ✓ Aligned {len(char_spans)} characters")

    # Extract acoustic features
    print(f"\n2️⃣ Computing acoustic features...")
    y = waveform.squeeze().numpy()

    # RMS energy
    hop_length = 160  # 10ms at 16kHz
    rms = librosa.feature.rms(y=y, hop_length=hop_length)[0]

    # Spectral flux
    spec = np.abs(librosa.stft(y, hop_length=hop_length))
    spec_flux = np.sum(np.diff(spec, axis=1)**2, axis=0)
    spec_flux = np.concatenate([[0], spec_flux])  # Prepend 0 for same length

    # Voicing from pitch
    times_rms = librosa.frames_to_time(np.arange(len(rms)), sr=sr, hop_length=hop_length)
    voicing = np.interp(times_rms, pitch_data['time'], (np.array(pitch_data['f0_hz']) > 0).astype(float))

    print(f"   ✓ RMS, spectral flux, voicing computed")

    # Group characters into syllables
    print(f"\n3️⃣ Grouping characters into syllables...")
    syllable_spans = group_chars_to_syllables(char_spans, transliteration)
    print(f"   ✓ Grouped into {len(syllable_spans)} syllables")

    # Snap boundaries
    print(f"\n4️⃣ Snapping boundaries to acoustic minima...")
    syllable_spans = snap_boundaries_to_minima(
        syllable_spans, rms, spec_flux, voicing, times_rms
    )
    print(f"   ✓ Boundaries snapped")

    # Trim trailing silence
    print(f"\n5️⃣ Trimming trailing silence...")
    syllable_spans = trim_trailing_silence(syllable_spans, rms, voicing, times_rms)
    print(f"   ✓ Trailing silence trimmed")

    # CRITICAL: Clip syllable boundaries to word boundaries (AFTER snapping)
    print(f"\n5️⃣b Clipping to word boundaries...")
    syllable_spans = clip_to_word_boundaries(syllable_spans, word_segments)
    print(f"   ✓ Boundaries clipped to word limits")

    # Balance durations (DISABLED - causes over-extension)
    # The word segments include silence gaps, we shouldn't force phonemes to fill them
    print(f"\n6️⃣ Duration balancing...")
    print(f"   ⚠️  SKIP - word boundaries include silence gaps")

    # Add Tajweed and pitch
    print(f"\n7️⃣ Adding Tajweed rules and pitch...")
    phonemes = annotate_tajweed_and_pitch(
        syllable_spans, pitch_data, word_segments, surah, ayah
    )
    print(f"   ✓ {len(phonemes)} phonemes ready")

    return phonemes


def clip_to_word_boundaries(syllable_spans: List[Dict], word_segments: List[Dict]) -> List[Dict]:
    """
    Clip syllable boundaries to their corresponding word boundaries.

    This prevents phonemes from bleeding into adjacent words due to CTC padding.
    Phonemes that end up with negative duration are removed.
    """
    valid_spans = []

    for span in syllable_spans:
        w_idx = span.get('word_index', 0)
        if w_idx < 0 or w_idx >= len(word_segments):
            valid_spans.append(span)
            continue

        ws = word_segments[w_idx]
        w_start = ws['start_ms'] / 1000
        w_end = ws['end_ms'] / 1000

        # Clip to word boundaries
        new_start = max(span['start'], w_start)
        new_end = min(span['end'], w_end)

        # Skip phonemes that would have negative/zero duration after clipping
        if new_end <= new_start:
            continue

        span['start'] = new_start
        span['end'] = new_end
        span['duration'] = new_end - new_start
        valid_spans.append(span)

    return valid_spans


def group_chars_to_syllables(char_spans: List[Dict], transliteration: str) -> List[Dict]:
    """
    Group character spans into syllables.

    CRITICAL: Respects word boundaries - characters from different words are never grouped.
    """
    # Group character spans by word_index
    words_chars = {}
    for cs in char_spans:
        w_idx = cs.get('word_index', 0)
        if w_idx not in words_chars:
            words_chars[w_idx] = []
        words_chars[w_idx].append(cs)

    # Process each word independently
    all_syl_spans = []

    for w_idx in sorted(words_chars.keys()):
        word_char_spans = words_chars[w_idx]

        # Reconstruct romanized string for this word
        word_roman = ''.join(cs['char'] for cs in word_char_spans)

        # Parse into syllables
        syllables = parse_syllables(word_roman)

        # Map character spans to syllables
        char_idx = 0

        for syl in syllables:
            if char_idx >= len(word_char_spans):
                break

            # Get chars for this syllable
            syl_chars = word_char_spans[char_idx:char_idx + len(syl)]

            if len(syl_chars) == 0:
                char_idx += len(syl)
                continue

            start_time = syl_chars[0]['start']
            end_time = syl_chars[-1]['end']

            all_syl_spans.append({
                'syllable': syl,
                'start': start_time,
                'end': end_time,
                'duration': end_time - start_time,
                'word_index': w_idx
            })

            char_idx += len(syl)

    return all_syl_spans


def parse_syllables(romanized: str) -> List[str]:
    """Parse romanized text into syllables (same logic as before)."""
    syllables = []
    vowels = set('aeiouAEIOU')
    current = ""
    i = 0

    while i < len(romanized):
        char = romanized[i]

        # Long vowels (aa, ee, oo)
        if char in vowels and i < len(romanized) - 1 and romanized[i+1] == char:
            current += romanized[i:i+2]
            i += 2

            # Take following consonant
            if i < len(romanized) and romanized[i] not in vowels:
                current += romanized[i]
                i += 1

            if current:
                syllables.append(current)
                current = ""
            continue

        current += char

        # End syllable after short vowel + consonant
        if char in vowels:
            if i < len(romanized) - 1 and romanized[i+1] not in vowels:
                current += romanized[i+1]
                i += 1
            if current:
                syllables.append(current)
                current = ""

        i += 1

    if current:
        syllables.append(current)

    return syllables


def annotate_tajweed_and_pitch(
    syllable_spans: List[Dict],
    pitch_data: Dict,
    word_segments: List[Dict],
    surah: int,
    ayah: int
) -> List[Dict]:
    """Add Tajweed rules and pitch statistics to phonemes."""
    from .tajweed_mapper import TajweedMapper

    tajweed_mapper = TajweedMapper()
    time_array = np.array(pitch_data['time'])
    f0_array = np.array(pitch_data['f0_hz'])

    phonemes = []
    for span in syllable_spans:
        start, end = span['start'], span['end']
        word_idx = span.get('word_index', 0)

        # Get Tajweed rule
        tajweed_rule = tajweed_mapper.map_phoneme_to_tajweed(
            phoneme_start=start,
            phoneme_end=end,
            word_idx=word_idx,
            word_segments=word_segments,
            surah=surah,
            ayah=ayah,
            phoneme_text=span['syllable']
        )

        # Get pitch
        mask = (time_array >= start) & (time_array <= end) & (f0_array > 0)
        if np.any(mask):
            f0_seg = f0_array[mask]
            mean_pitch = float(np.mean(f0_seg))
            min_pitch = float(np.min(f0_seg))
            max_pitch = float(np.max(f0_seg))
        else:
            mean_pitch = min_pitch = max_pitch = 0.0

        phonemes.append({
            'phoneme': span['syllable'],
            'start': span['start'],
            'end': span['end'],
            'duration': span['duration'],
            'mean_pitch': mean_pitch,
            'min_pitch': min_pitch,
            'max_pitch': max_pitch,
            'tajweed_rule': tajweed_rule,
            'word_index': word_idx,
            'confidence': 1.0
        })

    return phonemes
