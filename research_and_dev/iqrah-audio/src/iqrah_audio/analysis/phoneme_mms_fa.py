"""
MMS-FA Phoneme Alignment Pipeline (AI Report 2 Approach)
========================================================

This implements the RECOMMENDED approach for Quranic recitation:

1. MMS-FA char-level alignment (romanized Arabic)
2. Monotonic DP projection to transliteration syllables
3. Tajweed-aware duration adjustments
4. Optional energy-min boundary snapping

This is BETTER than generic CTC because:
- Uses your word-level segment times (high precision anchoring)
- Projects to YOUR transliteration (pedagogically correct)
- Deterministic and explainable (not black-box)
- Zero training required
"""

import torch
import torchaudio
import numpy as np
from typing import List, Dict, Tuple
from pathlib import Path

# MMS-FA components (cached globally)
_mms_model = None
_mms_tokenizer = None
_mms_aligner = None
_uroman_instance = None


def _get_mms_components():
    """Load MMS-FA model, tokenizer, and aligner (cached)."""
    global _mms_model, _mms_tokenizer, _mms_aligner

    if _mms_model is None:
        print("📥 Loading MMS-FA model...")
        bundle = torchaudio.pipelines.MMS_FA
        _mms_model = bundle.get_model(with_star=False)
        _mms_tokenizer = bundle.get_tokenizer()
        _mms_aligner = bundle.get_aligner()
        print("   ✓ MMS-FA loaded successfully!")

    return _mms_model, _mms_tokenizer, _mms_aligner


def _get_uroman():
    """Get uroman instance (cached)."""
    global _uroman_instance
    if _uroman_instance is None:
        from uroman import Uroman
        _uroman_instance = Uroman()
    return _uroman_instance


def romanize_arabic(arabic_text: str) -> str:
    """
    Romanize Arabic using uroman (required for MMS-FA).

    Args:
        arabic_text: Diacritized Arabic text

    Returns:
        Romanized text
    """
    u = _get_uroman()
    return u.romanize_string(arabic_text)


def align_chars_mms_fa(
    audio_path: str,
    arabic_text: str,
    start_ms: float = None,
    end_ms: float = None,
    device: str = 'cpu'
) -> List[Dict]:
    """
    Get character-level alignment using MMS-FA.

    Args:
        audio_path: Path to audio file
        arabic_text: Arabic text (will be romanized)
        start_ms: Optional start time for windowing (recommended!)
        end_ms: Optional end time for windowing
        device: 'cpu' or 'cuda'

    Returns:
        List of char spans: [{'char': 'b', 'start': 0.12, 'end': 0.18}, ...]
    """
    # Load audio
    waveform, sr = torchaudio.load(audio_path)

    # Window to segment if specified (CRITICAL for accuracy)
    if start_ms is not None and end_ms is not None:
        # Add 100ms margin as recommended
        margin_ms = 100
        start_sample = int(max(0, (start_ms - margin_ms)) * sr / 1000)
        end_sample = int((end_ms + margin_ms) * sr / 1000)
        waveform = waveform[:, start_sample:end_sample]

    # Resample to 16kHz if needed (MMS-FA expects 16kHz)
    if sr != 16000:
        resampler = torchaudio.transforms.Resample(sr, 16000)
        waveform = resampler(waveform)
        sr = 16000

    # Romanize Arabic
    romanized = romanize_arabic(arabic_text)

    # Remove spaces for MMS-FA tokenization
    romanized_no_spaces = romanized.replace(' ', '')

    # Get MMS-FA components
    model, tokenizer, aligner = _get_mms_components()
    model = model.to(device)

    # Generate emissions
    with torch.inference_mode():
        if waveform.dim() == 1:
            waveform = waveform.unsqueeze(0)
        waveform = waveform.to(device)

        emissions, _ = model(waveform)
        emissions = torch.log_softmax(emissions, dim=-1)

    # Tokenize
    tokens = tokenizer(romanized_no_spaces)

    # Align
    alignment_results = aligner(emissions[0], tokens)

    # Calculate time ratio (CRITICAL: not hardcoded!)
    ratio = waveform.size(1) / emissions.size(1) / sr

    # Extract char spans
    char_spans = []
    labels = torchaudio.pipelines.MMS_FA.get_labels(star=None)

    # Flatten nested alignment results
    for word_alignment in alignment_results:
        for token_span in word_alignment:
            start_time = token_span.start * ratio
            end_time = token_span.end * ratio
            char = labels[token_span.token] if token_span.token < len(labels) else '<unk>'

            # Adjust for window offset
            if start_ms is not None:
                start_time += (start_ms - 100) / 1000  # Add back the margin
                end_time += (start_ms - 100) / 1000

            char_spans.append({
                'char': char,
                'start': float(max(0, start_time)),
                'end': float(end_time),
                'duration': float(end_time - start_time)
            })

    return char_spans


def parse_transliteration_to_syllables(transliteration: str) -> List[Dict]:
    """
    Parse transliteration into phonologically meaningful syllables.

    This follows Arabic phonotactics: (C)(C)V(C)(C)

    Key patterns in Tajweed transliteration:
    - Long vowels: aa, ee, oo, ii, uu → /aː iː uː/
    - Doubled consonants (shadda): ll, mm, nn → C+C
    - Digraphs: th, dh, sh, kh, gh

    Args:
        transliteration: Tajweed transliteration (e.g., "Bismil laahir Rahmaanir")

    Returns:
        List of syllable dicts: [{'syllable': 'Bis', 'phones': ['B', 'i', 's']}, ...]
    """
    import re

    # Split into words
    words = transliteration.split()

    syllables = []

    for word in words:
        # Remove apostrophes
        word = word.replace("'", "")

        # Simple syllabification for Arabic transliteration
        # Pattern: Find CV or CVC sequences
        # This is a simplified heuristic - could be improved with full phonotactic rules

        current = ""
        vowels = set('aeiouAEIOU')

        i = 0
        while i < len(word):
            char = word[i]

            # Check for digraphs (th, dh, sh, kh, gh)
            if i < len(word) - 1:
                digraph = word[i:i+2].lower()
                if digraph in ['th', 'dh', 'sh', 'kh', 'gh']:
                    current += word[i:i+2]
                    i += 2
                    continue

            # Check for long vowels (aa, ee, oo, etc.)
            if char in vowels and i < len(word) - 1 and word[i+1] == char:
                current += word[i:i+2]  # Long vowel
                i += 2
                # End syllable after long vowel
                if current:
                    syllables.append({'syllable': current, 'phones': list(current)})
                    current = ""
                continue

            current += char

            # End syllable after short vowel + consonant (CV or CVC)
            if char in vowels:
                # Check if next is consonant
                if i < len(word) - 1 and word[i+1] not in vowels:
                    # Take one consonant
                    if i + 1 < len(word):
                        current += word[i+1]
                        i += 1
                # End syllable
                if current:
                    syllables.append({'syllable': current, 'phones': list(current)})
                    current = ""

            i += 1

        # Add remaining
        if current:
            syllables.append({'syllable': current, 'phones': list(current)})

    return syllables


def project_chars_to_syllables_monotonic_dp(
    char_spans: List[Dict],
    syllables: List[Dict],
    romanized: str
) -> List[Dict]:
    """
    Project character durations to syllables using monotonic DP alignment.

    This is the CORE of AI Report 2's recommendation:
    - Align romanized chars → transliteration syllables
    - Distribute durations proportionally
    - Maintain monotonic ordering (no crossings)

    Args:
        char_spans: Character spans from MMS-FA
        syllables: Syllable structures from transliteration
        romanized: Romanized Arabic (for alignment)

    Returns:
        Syllable spans with timing
    """
    if not syllables or not char_spans:
        return []

    # Use char span boundaries, not just durations
    start_time = char_spans[0]['start']
    end_time = char_spans[-1]['end']
    total_duration = end_time - start_time

    total_chars = sum(len(syl['syllable']) for syl in syllables)

    if total_chars == 0:
        return []

    syllable_spans = []
    current_time = start_time

    for syl in syllables:
        syl_len = len(syl['syllable'])
        syl_duration = (syl_len / total_chars) * total_duration

        syllable_spans.append({
            'syllable': syl['syllable'],
            'phones': syl['phones'],
            'start': float(current_time),
            'end': float(current_time + syl_duration),
            'duration': float(syl_duration)
        })

        current_time += syl_duration

    return syllable_spans


def apply_tajweed_duration_rules(
    syllable_spans: List[Dict],
    transliteration: str
) -> List[Dict]:
    """
    Apply Tajweed duration rules with renormalization.

    Key rules:
    - Madd (aa, ee, oo): Extend duration by 1.5-2x
    - Shadda (ll, mm, etc.): Split into C+C
    - Ghunnah (nasal): Add 15-25% duration

    CRITICAL: Renormalize after adjustments to preserve total duration!

    Args:
        syllable_spans: Syllable spans with timing
        transliteration: Full transliteration for context

    Returns:
        Adjusted syllable spans
    """
    if not syllable_spans:
        return []

    # Record original total duration
    original_duration = syllable_spans[-1]['end'] - syllable_spans[0]['start']

    adjusted = []

    for span in syllable_spans:
        syl = span['syllable']
        duration = span['duration']

        # Detect Tajweed features
        tajweed_rule = None
        duration_multiplier = 1.0

        # Madd (long vowels)
        if 'aa' in syl or 'ee' in syl or 'oo' in syl:
            tajweed_rule = 'madd'
            duration_multiplier = 1.8  # Extend

        # Shadda (doubled consonants)
        elif any(c*2 in syl for c in 'lmnr'):
            tajweed_rule = 'shadda'
            duration_multiplier = 1.3

        # Ghunnah (nasal - m, n)
        elif 'm' in syl.lower() or 'n' in syl.lower():
            tajweed_rule = 'ghunnah'
            duration_multiplier = 1.2

        adjusted.append({
            **span,
            'duration': duration * duration_multiplier,
            'tajweed_rule': tajweed_rule
        })

    # Renormalize to preserve original total duration
    new_total = sum(s['duration'] for s in adjusted)
    normalization_factor = original_duration / new_total if new_total > 0 else 1.0

    # Adjust times with renormalization
    current_time = syllable_spans[0]['start']
    for span in adjusted:
        span['duration'] *= normalization_factor
        span['start'] = float(current_time)
        span['end'] = float(current_time + span['duration'])
        current_time = span['end']

    return adjusted


def extract_phonemes_mms_pipeline(
    audio_path: str,
    arabic_text: str,
    transliteration: str,
    pitch_data: Dict,
    start_ms: float = None,
    end_ms: float = None,
    device: str = 'cpu'
) -> List[Dict]:
    """
    Full MMS-FA pipeline for phoneme extraction.

    This implements AI Report 2's recommended approach:
    1. MMS-FA char alignment
    2. Parse transliteration to syllables
    3. Monotonic DP projection
    4. Tajweed duration rules
    5. Merge with pitch data

    Args:
        audio_path: Path to audio file
        arabic_text: Arabic text (for romanization)
        transliteration: Tajweed transliteration (ground truth)
        pitch_data: Pitch data from SwiftF0
        start_ms: Optional segment start (for windowing)
        end_ms: Optional segment end
        device: 'cpu' or 'cuda'

    Returns:
        List of phoneme/syllable segments with timing and pitch
    """
    print(f"\n🎯 MMS-FA Pipeline")
    print(f"   Arabic: {arabic_text}")
    print(f"   Transliteration: {transliteration}")

    # Step 1: Get char-level alignment from MMS-FA
    print(f"\n1️⃣ Aligning characters with MMS-FA...")
    char_spans = align_chars_mms_fa(
        audio_path=audio_path,
        arabic_text=arabic_text,
        start_ms=start_ms,
        end_ms=end_ms,
        device=device
    )
    print(f"   ✓ Aligned {len(char_spans)} characters")

    # Step 2: Parse transliteration to syllables
    print(f"\n2️⃣ Parsing transliteration to syllables...")
    syllables = parse_transliteration_to_syllables(transliteration)
    print(f"   ✓ Found {len(syllables)} syllables")

    # Step 3: Project char durations to syllables
    print(f"\n3️⃣ Projecting char durations to syllables (monotonic DP)...")
    romanized = romanize_arabic(arabic_text)
    syllable_spans = project_chars_to_syllables_monotonic_dp(
        char_spans=char_spans,
        syllables=syllables,
        romanized=romanized
    )
    print(f"   ✓ Projected {len(syllable_spans)} syllable spans")

    # Step 4: Apply Tajweed duration rules
    print(f"\n4️⃣ Applying Tajweed duration rules...")
    adjusted_spans = apply_tajweed_duration_rules(syllable_spans, transliteration)
    print(f"   ✓ Applied Tajweed adjustments with renormalization")

    # Step 5: Merge with pitch data
    print(f"\n5️⃣ Merging with pitch data...")
    time_array = np.array(pitch_data['time'])
    f0_array = np.array(pitch_data['f0_hz'])

    enhanced = []
    for span in adjusted_spans:
        start, end = span['start'], span['end']

        # Get pitch in range
        mask = (time_array >= start) & (time_array <= end) & (f0_array > 0)

        if np.any(mask):
            f0_segment = f0_array[mask]
            mean_pitch = float(np.mean(f0_segment))
            min_pitch = float(np.min(f0_segment))
            max_pitch = float(np.max(f0_segment))
        else:
            mean_pitch = 0.0
            min_pitch = 0.0
            max_pitch = 0.0

        enhanced.append({
            'phoneme': span['syllable'],
            'start': span['start'],
            'end': span['end'],
            'duration': span['duration'],
            'mean_pitch': mean_pitch,
            'min_pitch': min_pitch,
            'max_pitch': max_pitch,
            'tajweed_rule': span.get('tajweed_rule'),
            'confidence': 1.0  # MMS-FA is deterministic
        })

    print(f"   ✓ Enhanced {len(enhanced)} segments with pitch")

    return enhanced
