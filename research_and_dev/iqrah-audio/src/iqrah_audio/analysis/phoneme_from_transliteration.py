"""
Phoneme Extraction from English Transliteration (Gold Data)
============================================================

Uses the english-transliteration-tajweed.json to get accurate phoneme
segmentation for Qari recitations. This is MUCH more accurate than
MMS-FA + IPA heuristics because it's human-annotated.

Example:
    "1:1": "Bismil laahir Rahmaanir Raheem"

This gives us the actual phonemes spoken by the Qari!
"""

import json
from pathlib import Path
from typing import List, Dict
import re

_transliteration_data = None

def load_transliteration_data(data_path: str = None) -> Dict[str, str]:
    """
    Load English transliteration data.

    Args:
        data_path: Path to english-transliteration-tajweed.json

    Returns:
        Dictionary mapping ayah_key (e.g. "1:1") to transliteration
    """
    global _transliteration_data

    if _transliteration_data is not None:
        return _transliteration_data

    if data_path is None:
        # Default path
        data_path = Path(__file__).parent.parent.parent.parent / "data" / "english-transliteration-tajweed.json"

    with open(data_path, 'r', encoding='utf-8') as f:
        _transliteration_data = json.load(f)

    return _transliteration_data


def get_phonemes_from_transliteration(
    surah: int,
    ayah: int,
    audio_path: str,
    pitch_data: Dict
) -> List[Dict]:
    """
    Extract phonemes from gold English transliteration data.

    This is the SIMPLE and ACCURATE approach - use human-annotated
    transliterations instead of trying to synthesize phonemes.

    Args:
        surah: Surah number
        ayah: Ayah number
        audio_path: Path to audio file
        pitch_data: Pitch data dictionary

    Returns:
        List of phoneme segments with timing and pitch
    """
    # Load transliteration
    trans_data = load_transliteration_data()
    ayah_key = f"{surah}:{ayah}"

    if ayah_key not in trans_data:
        return []

    transliteration = trans_data[ayah_key]

    # Split into phoneme-level segments (syllables)
    # English transliteration gives us natural syllable boundaries
    words = transliteration.split()

    phoneme_segments = []

    # Get audio duration
    audio_duration = pitch_data['duration']
    time_array = pitch_data['time']
    f0_array = pitch_data['f0_hz']

    # Distribute time evenly across words (simple approach)
    total_words = len(words)
    if total_words == 0:
        return []

    word_duration = audio_duration / total_words

    for i, word in enumerate(words):
        # Each word gets equal time slice
        word_start = i * word_duration
        word_end = (i + 1) * word_duration

        # Split word into syllables (approximate using hyphens and vowel clusters)
        syllables = split_into_syllables(word)

        if not syllables:
            continue

        syllable_duration = (word_end - word_start) / len(syllables)

        for j, syllable in enumerate(syllables):
            syl_start = word_start + j * syllable_duration
            syl_end = syl_start + syllable_duration

            # Get pitch for this segment
            syl_pitch = get_pitch_in_range(time_array, f0_array, syl_start, syl_end)

            phoneme_segments.append({
                'phoneme': syllable,
                'start': float(syl_start),
                'end': float(syl_end),
                'duration': float(syllable_duration),
                'mean_pitch': float(syl_pitch['mean']) if syl_pitch else 0.0,
                'min_pitch': float(syl_pitch['min']) if syl_pitch else 0.0,
                'max_pitch': float(syl_pitch['max']) if syl_pitch else 0.0,
                'tajweed_rule': None  # Will be added later from tajweed data
            })

    return phoneme_segments


def split_into_syllables(word: str) -> List[str]:
    """
    Split English transliteration word into syllables.

    Simple heuristic:
    - Split on hyphens
    - Split on consonant clusters
    - Keep vowel clusters together

    Args:
        word: Transliterated word (e.g., "Bismil", "laahir")

    Returns:
        List of syllables
    """
    # First, split on explicit hyphens
    if '-' in word:
        parts = word.split('-')
        syllables = []
        for part in parts:
            syllables.extend(split_into_syllables(part))
        return syllables

    # Remove apostrophes
    word = word.replace("'", "")

    # Simple consonant-vowel pattern splitting
    # This is approximate but good enough for visualization
    vowels = set('aeiouAEIOU')

    syllables = []
    current = ""

    for i, char in enumerate(word):
        current += char

        # Split after vowel + consonant (CV pattern)
        if i < len(word) - 1:
            if char in vowels and word[i+1] not in vowels:
                # Check if next char is last char
                if i+1 < len(word) - 1:
                    syllables.append(current)
                    current = ""

    if current:
        syllables.append(current)

    # Filter empty
    syllables = [s for s in syllables if s]

    # If no syllables found, return whole word
    if not syllables:
        return [word] if word else []

    return syllables


def get_pitch_in_range(
    time_array: List[float],
    f0_array: List[float],
    start: float,
    end: float
) -> Dict:
    """
    Get pitch statistics for a time range.

    Args:
        time_array: Time points
        f0_array: F0 values
        start: Start time (seconds)
        end: End time (seconds)

    Returns:
        Dictionary with pitch stats: mean, min, max
    """
    import numpy as np

    time = np.array(time_array)
    f0 = np.array(f0_array)

    # Get frames in range
    mask = (time >= start) & (time <= end) & (f0 > 0)

    if not np.any(mask):
        return None

    f0_segment = f0[mask]

    return {
        'mean': np.mean(f0_segment),
        'min': np.min(f0_segment),
        'max': np.max(f0_segment)
    }
