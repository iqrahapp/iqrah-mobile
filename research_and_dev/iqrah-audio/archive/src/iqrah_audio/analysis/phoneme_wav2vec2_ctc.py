"""
Phoneme Extraction using Wav2Vec2 CTC Alignment
================================================

This uses Wav2Vec2's CTC alignment to align the full transliteration
with the audio, without relying on word boundaries.

This should fix misalignment issues caused by word boundary errors.
"""

import torch
import torchaudio
import numpy as np
from typing import List, Dict


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


def extract_phonemes_wav2vec2_ctc(
    audio_path: str,
    word_segments: List[Dict],
    transliteration: str,
    pitch_data: Dict,
    surah: int,
    ayah: int,
    device='cpu'
) -> List[Dict]:
    """
    Extract phonemes using Wav2Vec2 CTC alignment on the FULL audio.

    This avoids word boundary issues by aligning the entire transliteration
    at once, then post-processing to assign word indices.

    Args:
        audio_path: Path to audio file
        word_segments: Word segments (used for word_index assignment)
        transliteration: Full transliteration string
        pitch_data: Pitch data for adding pitch info
        surah: Surah number (for Tajweed)
        ayah: Ayah number (for Tajweed)
        device: 'cpu' or 'cuda'

    Returns:
        List of phoneme dictionaries
    """
    try:
        from uroman import Uroman
        UROMAN_AVAILABLE = True
    except ImportError:
        UROMAN_AVAILABLE = False
        print("⚠️  uroman not available, using transliteration as-is")

    from .tajweed_mapper import TajweedMapper

    # Load audio
    waveform, sr = torchaudio.load(audio_path)

    # Convert to mono if stereo
    if waveform.size(0) > 1:
        waveform = waveform.mean(dim=0, keepdim=True)

    # Resample to 16kHz
    if sr != 16000:
        resampler = torchaudio.transforms.Resample(sr, 16000)
        waveform = resampler(waveform)
        sr = 16000

    # Romanize full transliteration (if uroman available)
    if UROMAN_AVAILABLE:
        uroman = Uroman()
        romanized = uroman.romanize_string(transliteration)
    else:
        # Use transliteration as-is (already in romanized form)
        romanized = transliteration
    romanized_clean = romanized.replace(' ', '').replace('-', '').replace("'", "")

    print(f"   Romanized: {romanized_clean}")

    # Get Wav2Vec2 model
    model, tokenizer, aligner = _get_wav2vec2()
    model = model.to(device)

    # Encode text
    with torch.inference_mode():
        emissions, _ = model(waveform.to(device))
        emissions = torch.log_softmax(emissions, dim=-1)

    emissions = emissions.cpu().detach()

    # Remove batch dimension - aligner expects 2D [time, vocab]
    if emissions.dim() == 3:
        emissions = emissions.squeeze(0)

    # Tokenize
    tokens = tokenizer(romanized_clean.lower())

    # Align (returns list of word alignments, each containing token spans)
    alignment = aligner(emissions, tokens)

    # Time conversion ratio
    ratio = waveform.size(1) / emissions.size(0) / sr

    # Get labels
    labels = torchaudio.pipelines.MMS_FA.get_labels(star=None)

    # Extract char spans
    char_spans = []
    for word_align in alignment:
        for token_span in word_align:
            start_time = token_span.start * ratio
            end_time = token_span.end * ratio

            char = labels[token_span.token] if token_span.token < len(labels) else '<unk>'

            char_spans.append({
                'char': char,
                'start': float(start_time),
                'end': float(end_time),
                'duration': float(end_time - start_time)
            })

    # Parse romanized text into syllables
    # CRITICAL FIX: Keep long vowels WITH consonants (C + long_vowel + C pattern)
    syllables = []
    vowels = set('aeiouAEIOU')

    current = ""
    i = 0
    while i < len(romanized_clean):
        char = romanized_clean[i]

        # CRITICAL FIX: Long vowels (aa, ee, oo, etc.)
        if char in vowels and i < len(romanized_clean) - 1 and romanized_clean[i+1] == char:
            # Add the long vowel
            current += romanized_clean[i:i+2]
            i += 2

            # CRITICAL: Also take following consonant if present (heem not he-em)
            if i < len(romanized_clean) and romanized_clean[i] not in vowels:
                current += romanized_clean[i]
                i += 1

            # End syllable
            if current:
                syllables.append(current)
                current = ""
            continue

        current += char

        # End syllable after short vowel + consonant
        if char in vowels:
            if i < len(romanized_clean) - 1 and romanized_clean[i+1] not in vowels:
                current += romanized_clean[i+1]
                i += 1
            if current:
                syllables.append(current)
                current = ""

        i += 1

    if current:
        syllables.append(current)

    print(f"   Syllables: {syllables}")

    # Map character spans to syllables
    syl_spans = []
    char_idx = 0

    for syl in syllables:
        if char_idx >= len(char_spans):
            break

        # Get char spans for this syllable
        syl_char_spans = char_spans[char_idx:char_idx + len(syl)]

        if len(syl_char_spans) == 0:
            char_idx += len(syl)
            continue

        start_time = syl_char_spans[0]['start']
        end_time = syl_char_spans[-1]['end']

        syl_spans.append({
            'syllable': syl,
            'start': float(start_time),
            'end': float(end_time),
            'duration': float(end_time - start_time)
        })

        char_idx += len(syl)

    # Assign word indices based on time
    def get_word_index(time_s):
        """Get word index for a given time."""
        time_ms = time_s * 1000
        for i, seg in enumerate(word_segments):
            if seg['start_ms'] <= time_ms <= seg['end_ms']:
                return i
            # Also check if within margin
            if seg['start_ms'] - 200 <= time_ms <= seg['end_ms'] + 200:
                return i
        # Default: find closest
        closest_idx = 0
        min_dist = abs(time_ms - word_segments[0]['start_ms'])
        for i, seg in enumerate(word_segments):
            dist = min(abs(time_ms - seg['start_ms']), abs(time_ms - seg['end_ms']))
            if dist < min_dist:
                min_dist = dist
                closest_idx = i
        return closest_idx

    # Add pitch and Tajweed info
    tajweed_mapper = TajweedMapper()
    time_array = np.array(pitch_data['time'])
    f0_array = np.array(pitch_data['f0_hz'])

    phonemes = []
    for syl_span in syl_spans:
        start, end = syl_span['start'], syl_span['end']
        word_idx = get_word_index(start)

        # Get Tajweed rule
        tajweed_rule = tajweed_mapper.map_phoneme_to_tajweed(
            phoneme_start=start,
            phoneme_end=end,
            word_idx=word_idx,
            word_segments=word_segments,
            surah=surah,
            ayah=ayah,
            phoneme_text=syl_span['syllable']  # CRITICAL: Pass phoneme text for content matching
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
            'phoneme': syl_span['syllable'],
            'start': syl_span['start'],
            'end': syl_span['end'],
            'duration': syl_span['duration'],
            'mean_pitch': mean_pitch,
            'min_pitch': min_pitch,
            'max_pitch': max_pitch,
            'tajweed_rule': tajweed_rule,
            'word_index': word_idx,
            'confidence': 1.0
        })

    return phonemes
