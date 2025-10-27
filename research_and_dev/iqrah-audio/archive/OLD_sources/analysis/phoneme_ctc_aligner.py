"""
SOTA Phoneme Alignment with CTC Forced Aligner
===============================================

Uses ctc-forced-aligner with Arabic Wav2Vec2 models for accurate
character-level alignment with the gold transliteration data.

This is the SOTA approach recommended in the research reports:
- 5× less memory than TorchAudio CTC
- >90% phoneme boundary accuracy
- Leverages human-annotated transliterations as ground truth
"""

import torch
import numpy as np
from pathlib import Path
from typing import List, Dict, Tuple
from ctc_forced_aligner import (
    load_audio, load_alignment_model, generate_emissions,
    preprocess_text, get_alignments, get_spans, postprocess_results
)

# Global model cache
_alignment_model = None
_tokenizer = None


def get_ctc_model(device="cpu", dtype=torch.float32):
    """
    Load Arabic Wav2Vec2 CTC model (cached).

    Args:
        device: 'cpu' or 'cuda'
        dtype: torch.float16 for GPU, torch.float32 for CPU

    Returns:
        (model, tokenizer) tuple
    """
    global _alignment_model, _tokenizer

    if _alignment_model is None:
        print("📥 Loading Arabic Wav2Vec2 CTC model...")
        print("   Model: jonatasgrosman/wav2vec2-large-xlsr-53-arabic")

        _alignment_model, _tokenizer = load_alignment_model(
            device=device,
            model_name="jonatasgrosman/wav2vec2-large-xlsr-53-arabic",
            dtype=dtype
        )

        print("   ✓ Model loaded successfully!")

    return _alignment_model, _tokenizer


def align_phonemes_with_ctc(
    audio_path: str,
    transliteration: str,
    device="cpu"
) -> List[Dict]:
    """
    Align phonemes using CTC forced alignment with gold transliteration.

    This uses the SOTA approach:
    1. Load Arabic Wav2Vec2 model
    2. Generate acoustic emissions from audio
    3. Use transliteration as ground truth
    4. Get character-level alignments

    Args:
        audio_path: Path to audio file
        transliteration: English transliteration (e.g., "Bismil laahir Rahmaanir Raheem")
        device: 'cpu' or 'cuda'

    Returns:
        List of phoneme segments with accurate timestamps:
        [
            {"start": 0.000, "end": 0.080, "text": "B", "score": 0.95},
            {"start": 0.080, "end": 0.200, "text": "i", "score": 0.92},
            ...
        ]
    """
    # Determine dtype based on device
    dtype = torch.float16 if device == "cuda" else torch.float32

    # Load model
    model, tokenizer = get_ctc_model(device=device, dtype=dtype)

    # Load audio
    print(f"\n🎵 Loading audio: {audio_path}")
    audio_waveform = load_audio(audio_path, model.dtype, model.device)

    # Generate emissions
    print(f"🔊 Generating acoustic emissions...")
    emissions, stride = generate_emissions(
        model,
        audio_waveform,
        batch_size=16
    )

    # Preprocess transliteration
    print(f"📝 Preprocessing transliteration: '{transliteration}'")
    tokens_starred, text_starred = preprocess_text(
        transliteration,
        romanize=True,  # Already romanized
        language="ara"
    )

    # Get character-level alignments
    print(f"🎯 Computing CTC forced alignment...")
    segments, scores, blank_token = get_alignments(
        emissions,
        tokens_starred,
        tokenizer
    )

    # Get time spans
    spans = get_spans(tokens_starred, segments, blank_token)

    # Post-process to get final timestamps
    phoneme_timestamps = postprocess_results(text_starred, spans, stride, scores)

    print(f"   ✓ Aligned {len(phoneme_timestamps)} phoneme segments")

    return phoneme_timestamps


def merge_phoneme_with_pitch(
    phoneme_segments: List[Dict],
    pitch_data: Dict
) -> List[Dict]:
    """
    Merge phoneme segments with pitch data.

    Args:
        phoneme_segments: CTC-aligned phoneme segments
        pitch_data: Pitch data from SwiftF0

    Returns:
        Enhanced phoneme segments with pitch information
    """
    time_array = np.array(pitch_data['time'])
    f0_array = np.array(pitch_data['f0_hz'])

    enhanced = []

    for seg in phoneme_segments:
        start, end = seg['start'], seg['end']

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

        enhanced.append({
            'phoneme': seg['text'],
            'start': float(start),
            'end': float(end),
            'duration': float(end - start),
            'mean_pitch': mean_pitch,
            'min_pitch': min_pitch,
            'max_pitch': max_pitch,
            'confidence': seg.get('score', 1.0),
            'tajweed_rule': None  # Will be added later
        })

    return enhanced


def extract_phonemes_sota(
    audio_path: str,
    transliteration: str,
    pitch_data: Dict,
    device="cpu"
) -> List[Dict]:
    """
    Extract phonemes using SOTA CTC forced alignment approach.

    This combines:
    - CTC forced alignment (character-level accuracy)
    - Gold transliteration data (ground truth)
    - Pitch data integration (for visualization)

    Args:
        audio_path: Path to audio file
        transliteration: English transliteration
        pitch_data: Pitch data from SwiftF0
        device: 'cpu' or 'cuda'

    Returns:
        List of enhanced phoneme segments
    """
    # Get CTC alignments
    phoneme_segments = align_phonemes_with_ctc(
        audio_path=audio_path,
        transliteration=transliteration,
        device=device
    )

    # Merge with pitch data
    enhanced_segments = merge_phoneme_with_pitch(phoneme_segments, pitch_data)

    return enhanced_segments
