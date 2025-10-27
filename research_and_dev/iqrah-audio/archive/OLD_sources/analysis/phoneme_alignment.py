"""
Phoneme-Level Forced Alignment for Arabic Quran
================================================

Uses Torchaudio FA + MMS-FA CTC + Epitran G2P for phoneme segmentation.

Pipeline:
1. Convert diacritized Arabic text to IPA phonemes (Epitran)
2. Load Arabic MMS-FA CTC model from HuggingFace
3. Run forced alignment to get phoneme timestamps
4. Apply Tajweed post-processing rules
5. Map phonemes to pitch frames for visualization
"""

import torch
import torchaudio
from torchaudio.pipelines import MMS_FA as bundle
import numpy as np
from typing import List, Dict, Tuple
import epitran

# Initialize Epitran for Arabic G2P (Grapheme-to-Phoneme)
_epi = None

def _get_epitran():
    """Lazy load Epitran"""
    global _epi
    if _epi is None:
        _epi = epitran.Epitran('ara-Arab')  # Arabic script
    return _epi


def text_to_phonemes(arabic_text: str) -> List[str]:
    """
    Convert diacritized Arabic text to IPA phonemes.

    Args:
        arabic_text: Fully diacritized Arabic text (with harakat)

    Returns:
        List of IPA phoneme strings
    """
    epi = _get_epitran()
    ipa = epi.transliterate(arabic_text)

    # Split into phonemes (simple split by characters for now)
    # TODO: Proper phoneme segmentation
    phonemes = [p for p in ipa if p.strip()]

    return phonemes


def align_phonemes_to_audio(
    audio_path: str,
    transcript: str,
    sr: int = 16000
) -> List[Dict]:
    """
    Perform forced alignment to get phoneme-level timestamps.

    Uses MMS-FA (Massively Multilingual Speech Forced Aligner).

    Args:
        audio_path: Path to audio file
        transcript: Diacritized Arabic text
        sr: Sample rate

    Returns:
        List of phoneme segments with start/end times
    """
    # Load audio
    waveform, sample_rate = torchaudio.load(audio_path)

    # Resample if needed
    if sample_rate != bundle.sample_rate:
        resampler = torchaudio.transforms.Resample(sample_rate, bundle.sample_rate)
        waveform = resampler(waveform)

    # Get model and tokenizer
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    model = bundle.get_model()
    model.to(device)
    model.eval()

    tokenizer = bundle.get_tokenizer()

    with torch.inference_mode():
        # Move audio to device
        waveform = waveform.to(device)

        # Get emissions (CTC logits)
        emissions, _ = model(waveform)
        emissions = torch.log_softmax(emissions, dim=-1)

    # Tokenize transcript
    tokens = tokenizer(transcript)

    # Get aligner
    aligner = bundle.get_aligner()

    # Perform alignment
    alignment = aligner(emissions[0], tokens)

    # Convert alignment to phoneme segments
    segments = []
    hop_length = 320  # MMS-FA hop length

    for token_span in alignment:
        start_frame = token_span.start
        end_frame = token_span.end
        token_idx = token_span.token

        # Convert frames to time
        start_time = start_frame * hop_length / bundle.sample_rate
        end_time = end_frame * hop_length / bundle.sample_rate

        # Get phoneme from token
        phoneme = tokenizer.tokens[token_idx] if token_idx < len(tokenizer.tokens) else '<unk>'

        segments.append({
            'phoneme': phoneme,
            'start': float(start_time),
            'end': float(end_time),
            'duration': float(end_time - start_time)
        })

    return segments


def apply_tajweed_rules(segments: List[Dict], words: List[Dict]) -> List[Dict]:
    """
    Apply Tajweed-specific post-processing to phoneme segments.

    Handles:
    - Madd (elongation): extend long vowel duration
    - Shadda (gemination): split into two identical consonants
    - Ghunnah (nasalization): adjust nasal duration

    Args:
        segments: Phoneme segments from forced alignment
        words: Word-level segments with Tajweed annotations

    Returns:
        Refined phoneme segments with Tajweed rules applied
    """
    refined = []

    for seg in segments:
        phoneme = seg['phoneme']

        # Madd rule: long vowels should be extended
        if phoneme in ['aː', 'iː', 'uː', 'a:', 'i:', 'u:']:
            # Extend duration by 1.5x for Madd
            seg['duration'] *= 1.5
            seg['end'] = seg['start'] + seg['duration']
            seg['tajweed_rule'] = 'madd'

        # Shadda rule: geminated consonants
        elif 'ː' in phoneme or ':' in phoneme:
            # Split into two segments
            mid_time = (seg['start'] + seg['end']) / 2
            base_phoneme = phoneme.replace('ː', '').replace(':', '')

            refined.append({
                'phoneme': base_phoneme,
                'start': seg['start'],
                'end': mid_time,
                'duration': mid_time - seg['start'],
                'tajweed_rule': 'shadda_1'
            })

            refined.append({
                'phoneme': base_phoneme,
                'start': mid_time,
                'end': seg['end'],
                'duration': seg['end'] - mid_time,
                'tajweed_rule': 'shadda_2'
            })
            continue

        # Ghunnah rule: nasal consonants (م، ن)
        elif phoneme in ['m', 'n', 'ŋ']:
            seg['duration'] *= 1.2
            seg['end'] = seg['start'] + seg['duration']
            seg['tajweed_rule'] = 'ghunnah'

        refined.append(seg)

    return refined


def apply_tajweed_rules_advanced(segments: List[Dict], tajweed_rules: List[Dict]) -> List[Dict]:
    """
    Apply Tajweed rules from qpc-hafs-tajweed.json to phoneme segments.

    Args:
        segments: Phoneme segments from forced alignment
        tajweed_rules: Parsed Tajweed rules with position and class information

    Returns:
        Refined phoneme segments with Tajweed rules applied
    """
    # First apply basic rules
    refined = apply_tajweed_rules(segments, [])

    # Then apply specific rules from qpc-hafs-tajweed.json
    for rule in tajweed_rules:
        rule_class = rule.get('class', '')
        position = rule.get('position', 0)
        char = rule.get('char', '')

        # Map rule class to duration multiplier
        duration_multiplier = 1.0
        tajweed_label = None

        if 'madda_normal' in rule_class:
            duration_multiplier = 2.0
            tajweed_label = 'madda_normal'
        elif 'madda_permissible' in rule_class:
            duration_multiplier = 2.5
            tajweed_label = 'madda_permissible'
        elif 'madda_obligatory' in rule_class:
            duration_multiplier = 5.0
            tajweed_label = 'madda_obligatory'
        elif 'ghunnah' in rule_class:
            duration_multiplier = 1.5
            tajweed_label = 'ghunnah'
        elif 'laam_shamsiyah' in rule_class:
            tajweed_label = 'laam_shamsiyah'
        elif 'ham_wasl' in rule_class:
            tajweed_label = 'ham_wasl'
        elif 'qalqalah' in rule_class:
            tajweed_label = 'qalqalah'
        elif 'idghaam' in rule_class:
            tajweed_label = 'idghaam'
        elif 'ikhfa' in rule_class:
            tajweed_label = 'ikhfa'

        # Apply to corresponding phoneme segment
        # Heuristic: map character position to phoneme segment index
        if position < len(refined):
            if duration_multiplier > 1.0:
                refined[position]['duration'] *= duration_multiplier
                refined[position]['end'] = refined[position]['start'] + refined[position]['duration']

            if tajweed_label:
                refined[position]['tajweed_rule'] = tajweed_label
                refined[position]['tajweed_char'] = char

    return refined


def map_phonemes_to_pitch(
    phoneme_segments: List[Dict],
    pitch_data: Dict
) -> List[Dict]:
    """
    Map phoneme segments to pitch frames for visualization.

    Args:
        phoneme_segments: List of phoneme segments with timestamps
        pitch_data: Pitch data dict with 'time', 'f0_hz', etc.

    Returns:
        Phoneme segments enriched with pitch frame indices
    """
    time = np.array(pitch_data['time'])
    f0_hz = np.array(pitch_data['f0_hz'])

    enriched = []

    for seg in phoneme_segments:
        # Find pitch frames within phoneme time range
        mask = (time >= seg['start']) & (time <= seg['end'])
        frame_indices = np.where(mask)[0]

        # Get pitch values for this phoneme
        phoneme_pitches = f0_hz[mask]

        seg['pitch_frames'] = frame_indices.tolist()
        seg['pitch_values'] = phoneme_pitches.tolist()
        seg['mean_pitch'] = float(np.mean(phoneme_pitches[phoneme_pitches > 0])) if np.any(phoneme_pitches > 0) else 0.0

        enriched.append(seg)

    return enriched


def analyze_word_phonemes(
    audio_path: str,
    word_text: str,
    word_start_ms: float,
    word_end_ms: float,
    pitch_data: Dict,
    tajweed_rules: List[Dict] = None
) -> List[Dict]:
    """
    Complete phoneme analysis pipeline for a single word.

    Args:
        audio_path: Path to audio file
        word_text: Diacritized Arabic word
        word_start_ms: Word start time (milliseconds)
        word_end_ms: Word end time (milliseconds)
        pitch_data: Pitch extraction data
        tajweed_rules: List of Tajweed rules from qpc-hafs-tajweed.json (optional)

    Returns:
        List of phoneme segments with pitch mapping and Tajweed annotations
    """
    # TODO: Extract word audio segment for better alignment
    # For now, align full audio and filter by word boundaries

    # Get phonemes
    phonemes = text_to_phonemes(word_text)

    # Run forced alignment (on full audio for now)
    segments = align_phonemes_to_audio(audio_path, word_text)

    # Filter to word boundaries
    word_start_s = word_start_ms / 1000
    word_end_s = word_end_ms / 1000

    word_segments = [
        seg for seg in segments
        if seg['start'] >= word_start_s and seg['end'] <= word_end_s
    ]

    # Apply Tajweed rules (with parsed rules if available)
    if tajweed_rules:
        word_segments = apply_tajweed_rules_advanced(word_segments, tajweed_rules)
    else:
        word_segments = apply_tajweed_rules(word_segments, [])

    # Map to pitch frames
    word_segments = map_phonemes_to_pitch(word_segments, pitch_data)

    return word_segments


if __name__ == "__main__":
    # Quick test
    print("Testing phoneme alignment...")

    # Test Epitran
    arabic_text = "بِسْمِ اللَّهِ"
    phonemes = text_to_phonemes(arabic_text)
    print(f"Text: {arabic_text}")
    print(f"Phonemes: {phonemes}")
    print("✓ Epitran working")
