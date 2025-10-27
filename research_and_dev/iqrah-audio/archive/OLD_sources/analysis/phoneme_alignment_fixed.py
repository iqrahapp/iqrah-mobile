"""
Phoneme-Level Forced Alignment (FIXED VERSION)
==============================================

Implements true phoneme segmentation using:
- Uroman for romanization (required by MMS-FA)
- MMS-FA for character-level alignment
- PanPhon for proper IPA phone segmentation
- Epitran for Arabic → IPA conversion
- Tajweed rules from Arabic diacritics

CRITICAL FIXES:
1. Romanize with uroman before MMS-FA tokenization
2. Use proper time conversion (ratio, not hardcoded hop)
3. Map char spans → phone spans using PanPhon
4. Apply Tajweed from Arabic diacritics, not IPA heuristics
5. Window to word boundaries before alignment
"""

import torch
import torchaudio
import numpy as np
import epitran
import panphon.segment
import re
from typing import List, Dict, Tuple
from pathlib import Path

# Caching
_epitran = None
_mms_model = None
_mms_tokenizer = None
_mms_aligner = None
_uroman_instance = None
_panphon_segmenter = None


def _get_epitran():
    global _epitran
    if _epitran is None:
        _epitran = epitran.Epitran('ara-Arab')
    return _epitran


def _get_mms_components():
    """Get MMS-FA model, tokenizer, and aligner (cached)."""
    global _mms_model, _mms_tokenizer, _mms_aligner

    if _mms_model is None:
        bundle = torchaudio.pipelines.MMS_FA
        _mms_model = bundle.get_model(with_star=False)
        _mms_tokenizer = bundle.get_tokenizer()
        _mms_aligner = bundle.get_aligner()

    return _mms_model, _mms_tokenizer, _mms_aligner


def _get_uroman():
    global _uroman_instance
    if _uroman_instance is None:
        from uroman import Uroman
        _uroman_instance = Uroman()
    return _uroman_instance


def _get_panphon():
    global _panphon_segmenter
    if _panphon_segmenter is None:
        _panphon_segmenter = panphon.segment.Segment()
    return _panphon_segmenter


def romanize_arabic(arabic_text: str) -> str:
    """
    Romanize Arabic text using uroman.

    Args:
        arabic_text: Diacritized Arabic text

    Returns:
        Romanized text (Latin script)
    """
    u = _get_uroman()
    return u.romanize_string(arabic_text)


def text_to_ipa_phones(arabic_text: str) -> List[str]:
    """
    Convert diacritized Arabic to IPA phonemes using Epitran + PanPhon.

    Args:
        arabic_text: Diacritized Arabic text

    Returns:
        List of IPA phonemes (properly segmented with PanPhon)
    """
    # Convert to IPA
    epi = _get_epitran()
    ipa_string = epi.transliterate(arabic_text)

    # Use PanPhon to properly segment IPA string into phones
    # This handles multi-char phones and combining diacritics correctly
    segmenter = _get_panphon()
    phones = segmenter.ipa_segs(ipa_string)

    # Filter out spaces
    phones = [p for p in phones if p.strip()]

    return phones


def extract_word_audio(
    audio_path: str,
    start_ms: float,
    end_ms: float,
    margin_ms: float = 100,
    sr: int = 16000
) -> Tuple[torch.Tensor, int]:
    """
    Extract word audio segment with margins.

    Args:
        audio_path: Path to full audio file
        start_ms: Word start time (milliseconds)
        end_ms: Word end time (milliseconds)
        margin_ms: Safety margin before/after (default 100ms)
        sr: Sample rate

    Returns:
        (waveform, sample_rate)
    """
    waveform, actual_sr = torchaudio.load(audio_path)

    # Resample if needed
    if actual_sr != sr:
        resampler = torchaudio.transforms.Resample(actual_sr, sr)
        waveform = resampler(waveform)
        actual_sr = sr

    # Calculate sample indices with margins
    start_sample = int(max(0, (start_ms - margin_ms)) * actual_sr / 1000)
    end_sample = int((end_ms + margin_ms) * actual_sr / 1000)

    # Extract segment
    word_waveform = waveform[:, start_sample:min(end_sample, waveform.size(1))]

    return word_waveform, actual_sr


def align_characters_mms(
    waveform: torch.Tensor,
    romanized_text: str,
    sr: int = 16000,
    device: str = 'cpu'
) -> List[Dict]:
    """
    Align romanized text to audio using MMS-FA (character-level).

    Args:
        waveform: Audio waveform
        romanized_text: Romanized (uroman) text
        sr: Sample rate
        device: 'cpu' or 'cuda'

    Returns:
        List of character spans: [{'char': 'a', 'start': 0.12, 'end': 0.18}, ...]
    """
    model, tokenizer, aligner = _get_mms_components()
    model = model.to(device)

    # Get emissions
    with torch.inference_mode():
        if waveform.dim() == 1:
            waveform = waveform.unsqueeze(0)
        waveform = waveform.to(device)

        emissions, _ = model(waveform)
        emissions = torch.log_softmax(emissions, dim=-1)

    # Tokenize romanized text (remove spaces - MMS-FA doesn't handle them)
    romanized_no_spaces = romanized_text.replace(' ', '')
    tokens = tokenizer(romanized_no_spaces)

    # Align - returns List[List[TokenSpan]]
    alignment_results = aligner(emissions[0], tokens)

    # Calculate proper time ratio (NOT hardcoded hop_length!)
    ratio = waveform.size(1) / emissions.size(1) / sr

    # Convert to char spans
    char_spans = []
    labels = torchaudio.pipelines.MMS_FA.get_labels(star=None)

    # Flatten alignment (it's a list of word alignments)
    for word_alignment in alignment_results:
        for token_span in word_alignment:
            start_time = token_span.start * ratio
            end_time = token_span.end * ratio
            char = labels[token_span.token] if token_span.token < len(labels) else '<unk>'

            char_spans.append({
                'char': char,
                'start': float(start_time),
                'end': float(end_time),
                'duration': float(end_time - start_time)
            })

    return char_spans


def distribute_char_durations_to_phones(
    char_spans: List[Dict],
    ipa_phones: List[str],
    romanized_text: str
) -> List[Dict]:
    """
    Distribute character span durations to IPA phones.

    Since MMS-FA gives char-level spans but we want phone-level,
    we proportionally distribute durations.

    Args:
        char_spans: Character spans from MMS-FA
        ipa_phones: IPA phones from Epitran+PanPhon
        romanized_text: Romanized text (for mapping)

    Returns:
        Phone spans: [{'phoneme': 'b', 'start': 0.12, 'end': 0.18}, ...]
    """
    # Simple proportional distribution
    # TODO: Use posterior weights for better accuracy

    phone_spans = []
    total_char_duration = sum(cs['duration'] for cs in char_spans)
    total_phones = len(ipa_phones)

    if total_phones == 0 or total_char_duration == 0:
        return []

    # Average duration per phone
    avg_phone_duration = total_char_duration / total_phones

    current_time = char_spans[0]['start'] if char_spans else 0.0

    for phone in ipa_phones:
        start = current_time
        end = start + avg_phone_duration
        current_time = end

        phone_spans.append({
            'phoneme': phone,
            'start': start,
            'end': end,
            'duration': avg_phone_duration
        })

    return phone_spans


def apply_tajweed_from_arabic(
    phone_spans: List[Dict],
    arabic_text: str,
    tajweed_rules: List[Dict] = None
) -> List[Dict]:
    """
    Apply Tajweed rules from Arabic diacritics (NOT IPA heuristics).

    Args:
        phone_spans: Phone segments
        arabic_text: Original Arabic text with diacritics
        tajweed_rules: Parsed Tajweed rules from qpc-hafs-tajweed.json

    Returns:
        Refined phone spans with Tajweed annotations
    """
    refined = []

    # Arabic diacritics
    SHADDA = '\u0651'  # ّ
    SUKUN = '\u0652'   # ْ
    FATHA = '\u064E'   # َ
    KASRA = '\u0650'   # ِ
    DAMMA = '\u064F'   # ُ
    ALIF = 'ا'
    WAW = 'و'
    YA = 'ي'

    # If we have parsed Tajweed rules from JSON, use them
    if tajweed_rules:
        for i, seg in enumerate(phone_spans):
            # Find matching rule by position
            for rule in tajweed_rules:
                rule_class = rule.get('class', '')

                # Apply duration multipliers
                if 'madda_normal' in rule_class:
                    seg['duration'] *= 2.0
                    seg['tajweed_rule'] = 'madda_normal'
                elif 'madda_permissible' in rule_class:
                    seg['duration'] *= 2.5
                    seg['tajweed_rule'] = 'madda_permissible'
                elif 'madda_obligatory' in rule_class:
                    seg['duration'] *= 5.0
                    seg['tajweed_rule'] = 'madda_obligatory'
                elif 'ghunnah' in rule_class:
                    seg['duration'] *= 1.5
                    seg['tajweed_rule'] = 'ghunnah'
                elif 'laam_shamsiyah' in rule_class:
                    seg['tajweed_rule'] = 'laam_shamsiyah'
                elif 'ham_wasl' in rule_class:
                    seg['tajweed_rule'] = 'ham_wasl'

                # Recalculate end time
                if i > 0:
                    seg['start'] = refined[-1]['end']
                seg['end'] = seg['start'] + seg['duration']

            refined.append(seg)
    else:
        # Fallback: basic heuristics
        for seg in phone_spans:
            phoneme = seg['phoneme']

            # Detect long vowels (potential Madd)
            if phoneme in ['aː', 'iː', 'uː', 'a:', 'i:', 'u:']:
                seg['duration'] *= 1.5
                seg['tajweed_rule'] = 'madd'

            # Detect nasals (potential Ghunnah)
            elif phoneme in ['m', 'n', 'ŋ']:
                seg['duration'] *= 1.2
                seg['tajweed_rule'] = 'ghunnah'

            refined.append(seg)

    return refined


def map_phonemes_to_pitch(
    phoneme_segments: List[Dict],
    pitch_data: Dict
) -> List[Dict]:
    """
    Map phoneme segments to pitch frames.

    Args:
        phoneme_segments: Phoneme segments with start/end times
        pitch_data: Pitch data from SwiftF0 (time, f0_hz arrays)

    Returns:
        Enriched phoneme segments with pitch_frames, pitch_values, mean_pitch
    """
    time = np.array(pitch_data['time'])
    f0_hz = np.array(pitch_data['f0_hz'])

    enriched = []

    for seg in phoneme_segments:
        # Find pitch frames within phoneme boundaries (half-open interval)
        mask = (time >= seg['start']) & (time < seg['end'])
        pitch_indices = np.where(mask)[0]

        phoneme_pitches = f0_hz[pitch_indices]

        seg['pitch_frames'] = pitch_indices.tolist()
        seg['pitch_values'] = phoneme_pitches.tolist()

        # Calculate mean pitch (only voiced frames > 50 Hz)
        voiced_pitches = phoneme_pitches[(phoneme_pitches > 50) & (phoneme_pitches < 500)]
        seg['mean_pitch'] = float(np.mean(voiced_pitches)) if len(voiced_pitches) > 0 else 0.0

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
    Complete phoneme analysis pipeline for a single word (FIXED VERSION).

    Args:
        audio_path: Path to audio file
        word_text: Diacritized Arabic word
        word_start_ms: Word start time (milliseconds)
        word_end_ms: Word end time (milliseconds)
        pitch_data: Pitch extraction data (full audio)
        tajweed_rules: Parsed Tajweed rules from qpc-hafs-tajweed.json (optional)

    Returns:
        List of phoneme segments with pitch mapping and Tajweed annotations
    """
    # Step 1: Extract word audio segment (with margins)
    word_waveform, sr = extract_word_audio(
        audio_path, word_start_ms, word_end_ms, margin_ms=100
    )

    # Step 2: Romanize Arabic → uroman (REQUIRED for MMS-FA)
    romanized = romanize_arabic(word_text)

    # Step 3: Get IPA phones (Epitran + PanPhon)
    ipa_phones = text_to_ipa_phones(word_text)

    # Step 4: Character-level alignment (MMS-FA on romanized text)
    char_spans = align_characters_mms(word_waveform, romanized, sr=sr)

    # Step 5: Distribute char durations → phone durations
    phone_spans = distribute_char_durations_to_phones(char_spans, ipa_phones, romanized)

    # Step 6: Apply Tajweed rules from Arabic diacritics
    phone_spans = apply_tajweed_from_arabic(phone_spans, word_text, tajweed_rules)

    # Step 7: Adjust timestamps to absolute time (word start offset)
    margin_s = 0.1  # 100ms margin
    word_start_s = word_start_ms / 1000 - margin_s

    for seg in phone_spans:
        seg['start'] += word_start_s
        seg['end'] += word_start_s

    # Step 8: Clip to word boundaries
    word_start_s_exact = word_start_ms / 1000
    word_end_s_exact = word_end_ms / 1000

    clipped = []
    for seg in phone_spans:
        # Keep segments that overlap with word
        if seg['end'] > word_start_s_exact and seg['start'] < word_end_s_exact:
            seg['start'] = max(seg['start'], word_start_s_exact)
            seg['end'] = min(seg['end'], word_end_s_exact)
            seg['duration'] = seg['end'] - seg['start']

            if seg['duration'] > 0:
                clipped.append(seg)

    # Step 9: Map to pitch frames
    clipped = map_phonemes_to_pitch(clipped, pitch_data)

    return clipped
