"""
Segment Data Loader
===================

Loads word-level segment data from data/husary/segments/segments.json

This contains CRITICAL data for proper phoneme alignment:
- Word-level timestamps (start_ms, end_ms)
- Audio URLs for downloading
- 6,236 ayahs with precise segmentation

Format: segments.json["1:1"] = {
    "surah_number": 1,
    "ayah_number": 1,
    "audio_url": "https://...",
    "segments": [[word_idx, start_ms, end_ms], ...]
}
"""

import json
from pathlib import Path
from typing import Dict, List
import urllib.request
import tempfile

_segments_data = None


def load_segments_data(data_path: str = None) -> Dict:
    """
    Load segment data from JSON.

    Args:
        data_path: Path to segments.json

    Returns:
        Dictionary mapping ayah_key to segment data
    """
    global _segments_data

    if _segments_data is not None:
        return _segments_data

    if data_path is None:
        data_path = Path(__file__).parent.parent.parent.parent / "data" / "husary" / "segments" / "segments.json"

    with open(data_path, 'r') as f:
        _segments_data = json.load(f)

    return _segments_data


def get_ayah_segments(surah: int, ayah: int) -> Dict:
    """
    Get segment data for an ayah.

    Args:
        surah: Surah number
        ayah: Ayah number

    Returns:
        Segment data dict with audio_url and word segments
    """
    data = load_segments_data()
    key = f"{surah}:{ayah}"
    return data.get(key, None)


def download_audio(audio_url: str, cache_dir: str = None) -> str:
    """
    Download audio file from URL (with caching).

    Args:
        audio_url: URL to audio file
        cache_dir: Directory for caching (uses temp if None)

    Returns:
        Path to downloaded audio file
    """
    if cache_dir is None:
        cache_dir = Path(tempfile.gettempdir()) / "iqrah_audio_cache"

    cache_dir = Path(cache_dir)
    cache_dir.mkdir(parents=True, exist_ok=True)

    # Create cache filename from URL
    import hashlib
    url_hash = hashlib.md5(audio_url.encode()).hexdigest()
    cache_file = cache_dir / f"{url_hash}.mp3"

    # Download if not cached
    if not cache_file.exists():
        print(f"📥 Downloading: {audio_url}")
        urllib.request.urlretrieve(audio_url, cache_file)
        print(f"   ✓ Cached: {cache_file}")
    else:
        print(f"   ✓ Using cached: {cache_file}")

    return str(cache_file)


def get_word_segments_with_text(surah: int, ayah: int) -> List[Dict]:
    """
    Get word segments with Arabic text.

    Combines segment timestamps with Arabic words from Tajweed data.

    Args:
        surah: Surah number
        ayah: Ayah number

    Returns:
        List of word segments: [{'text': 'بسم', 'start_ms': 0, 'end_ms': 480}, ...]
    """
    from .tajweed_loader import get_ayah_words

    # Get segment timestamps
    seg_data = get_ayah_segments(surah, ayah)
    if not seg_data:
        return []

    segments = seg_data['segments']  # [[word_idx, start_ms, end_ms], ...]

    # Get Arabic words
    arabic_words = get_ayah_words(surah, ayah)

    # Combine
    word_segments = []
    for seg in segments:
        word_idx, start_ms, end_ms = seg

        # Find matching Arabic word (1-indexed)
        arabic_word = next((w for w in arabic_words if int(w['word']) == word_idx), None)

        if arabic_word:
            # Strip Tajweed HTML tags to get plain text
            import re
            text = arabic_word['text']
            text = re.sub(r'<[^>]+>', '', text)  # Remove HTML tags

            word_segments.append({
                'word_index': word_idx,
                'text': text,
                'start_ms': start_ms,
                'end_ms': end_ms,
                'duration_ms': end_ms - start_ms
            })

    return word_segments
