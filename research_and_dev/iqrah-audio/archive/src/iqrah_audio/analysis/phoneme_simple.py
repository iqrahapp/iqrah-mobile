"""
Simple Phoneme Extraction Using Transliteration
================================================

This is a pragmatic approach that works NOW:
- Use gold English transliteration (already phonetically meaningful)
- Parse into syllables following Arabic phonotactics
- Distribute time evenly across syllables
- Apply Tajweed duration adjustments
- Merge with pitch data

This gives us IMMEDIATE results while we debug MMS-FA alignment.
Once MMS-FA is working properly, we can switch to it for higher precision.
"""

import numpy as np
from typing import List, Dict


def parse_transliteration_to_syllables(transliteration: str) -> List[str]:
    """
    Parse English transliteration into syllables.

    Follows Arabic phonotactic patterns:
    - Long vowels: aa, ee, oo, ii, uu
    - Doubled consonants: ll, mm, nn (shadda)
    - Digraphs: th, dh, sh, kh, gh

    Args:
        transliteration: Tajweed transliteration (e.g., "Bismil laahir Rahmaanir Raheem")

    Returns:
        List of syllables
    """
    words = transliteration.split()
    syllables = []

    for word in words:
        # Remove punctuation
        word = word.replace("'", "").replace("-", "")

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

            # Check for long vowels (aa, ee, oo, ii, uu)
            if char in vowels and i < len(word) - 1 and word[i+1] == char:
                current += word[i:i+2]
                i += 2
                # End syllable after long vowel
                if current:
                    syllables.append(current)
                    current = ""
                continue

            current += char

            # End syllable after short vowel + consonant
            if char in vowels:
                # Check if next is consonant
                if i < len(word) - 1 and word[i+1] not in vowels:
                    # Take one consonant
                    current += word[i+1]
                    i += 1
                # End syllable
                if current:
                    syllables.append(current)
                    current = ""

            i += 1

        # Add remaining
        if current:
            syllables.append(current)

    return syllables


def detect_tajweed_rule(syllable: str) -> str:
    """
    Detect Tajweed rule from syllable.

    Args:
        syllable: Syllable string

    Returns:
        Tajweed rule name or None
    """
    syl_lower = syllable.lower()

    # Madd (long vowels)
    if any(long in syl_lower for long in ['aa', 'ee', 'oo', 'ii', 'uu']):
        return 'madd'

    # Shadda (doubled consonants)
    if any(c*2 in syl_lower for c in 'lmnrtdbszkhg'):
        return 'shadda'

    # Ghunnah (nasal sounds)
    if 'm' in syl_lower or 'n' in syl_lower:
        return 'ghunnah'

    return None


def get_tajweed_duration_multiplier(tajweed_rule: str) -> float:
    """
    Get duration multiplier for Tajweed rule.

    Args:
        tajweed_rule: Tajweed rule name

    Returns:
        Duration multiplier (1.0 = no change)
    """
    multipliers = {
        'madd': 1.8,      # Long vowels extended
        'shadda': 1.3,    # Doubled consonants
        'ghunnah': 1.2,   # Nasal sounds
    }

    return multipliers.get(tajweed_rule, 1.0)


def extract_phonemes_simple(
    transliteration: str,
    pitch_data: Dict,
    audio_duration: float = None
) -> List[Dict]:
    """
    Extract phonemes from transliteration with even time distribution.

    This is the SIMPLE, WORKING approach:
    1. Parse transliteration to syllables
    2. Detect Tajweed rules
    3. Apply duration multipliers
    4. Renormalize to preserve total duration
    5. Merge with pitch data

    Args:
        transliteration: Tajweed transliteration
        pitch_data: Pitch data from SwiftF0
        audio_duration: Total audio duration (uses pitch_data duration if None)

    Returns:
        List of phoneme segments with timing and pitch
    """
    if audio_duration is None:
        audio_duration = pitch_data['duration']

    # Parse to syllables
    syllables = parse_transliteration_to_syllables(transliteration)

    if not syllables:
        return []

    # Detect Tajweed rules and calculate raw durations
    syllable_data = []
    total_weight = 0.0

    for syl in syllables:
        tajweed_rule = detect_tajweed_rule(syl)
        multiplier = get_tajweed_duration_multiplier(tajweed_rule)
        weight = len(syl) * multiplier
        total_weight += weight

        syllable_data.append({
            'syllable': syl,
            'tajweed_rule': tajweed_rule,
            'weight': weight
        })

    # Distribute time proportionally to weights
    time_array = np.array(pitch_data['time'])
    f0_array = np.array(pitch_data['f0_hz'])

    phonemes = []
    current_time = 0.0

    for data in syllable_data:
        # Calculate duration based on weight
        duration = (data['weight'] / total_weight) * audio_duration
        start = current_time
        end = current_time + duration

        # Get pitch in this range
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

        phonemes.append({
            'phoneme': data['syllable'],
            'start': float(start),
            'end': float(end),
            'duration': float(duration),
            'mean_pitch': mean_pitch,
            'min_pitch': min_pitch,
            'max_pitch': max_pitch,
            'tajweed_rule': data['tajweed_rule'],
            'confidence': 1.0
        })

        current_time = end

    return phonemes
