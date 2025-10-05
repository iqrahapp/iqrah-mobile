"""
Segments and Quran Text Loader

Loads annotated word-level segments and Arabic text for Quran recitation.
"""

import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass


@dataclass
class WordSegment:
    """Word segment with timing information."""
    word_id: int
    start_ms: int
    end_ms: int
    duration_ms: int

    @property
    def start_s(self) -> float:
        return self.start_ms / 1000.0

    @property
    def end_s(self) -> float:
        return self.end_ms / 1000.0


@dataclass
class AyahData:
    """Complete ayah data with text and segments."""
    surah: int
    ayah: int
    verse_key: str
    text: str  # Arabic text
    words: List[str]  # Split by whitespace
    audio_url: str
    segments: List[WordSegment]

    def get_word_at_time(self, time_ms: int) -> Optional[Tuple[int, WordSegment]]:
        """
        Get the word segment that should be active at given time.

        Returns:
            (word_index, WordSegment) or None if no word active at this time
        """
        for idx, seg in enumerate(self.segments):
            if seg.start_ms <= time_ms <= seg.end_ms:
                return (idx, seg)
        return None

    def get_expected_word_index(self, time_ms: int) -> int:
        """
        Get the index of the word that should be recited at this time.
        If between words, returns next word to recite.
        """
        for idx, seg in enumerate(self.segments):
            if time_ms < seg.start_ms:
                return max(0, idx - 1)
            if seg.start_ms <= time_ms <= seg.end_ms:
                return idx
        return len(self.segments) - 1


class SegmentsLoader:
    """Load and manage Quran segments and text data."""

    def __init__(self, data_dir: Optional[Path] = None):
        """
        Initialize loader.

        Args:
            data_dir: Path to data directory (defaults to project data/)
        """
        if data_dir is None:
            # Auto-detect data directory
            current = Path(__file__).resolve()
            data_dir = current.parent.parent.parent.parent / "data"

        self.data_dir = Path(data_dir)
        self.segments_path = self.data_dir / "husary" / "segments" / "segments.json"
        self.quran_path = self.data_dir / "indopak.json"

        self._segments_cache: Optional[Dict] = None
        self._quran_cache: Optional[Dict] = None

    def _load_segments(self) -> Dict:
        """Load segments.json (cached)."""
        if self._segments_cache is None:
            with open(self.segments_path) as f:
                self._segments_cache = json.load(f)
        return self._segments_cache

    def _load_quran_text(self) -> Dict:
        """Load indopak.json (cached)."""
        if self._quran_cache is None:
            with open(self.quran_path) as f:
                self._quran_cache = json.load(f)
        return self._quran_cache

    def get_ayah(self, surah: int, ayah: int) -> AyahData:
        """
        Get complete ayah data with segments and text.

        Args:
            surah: Surah number (1-114)
            ayah: Ayah number within surah

        Returns:
            AyahData with all information

        Raises:
            KeyError: If ayah not found in data
        """
        key = f"{surah}:{ayah}"

        segments_data = self._load_segments()
        quran_data = self._load_quran_text()

        if key not in segments_data:
            raise KeyError(f"Segments not found for {key}")
        if key not in quran_data:
            raise KeyError(f"Quran text not found for {key}")

        seg_info = segments_data[key]
        text_info = quran_data[key]

        # Parse segments
        segments = []
        for word_id, start_ms, end_ms in seg_info["segments"]:
            segments.append(WordSegment(
                word_id=word_id,
                start_ms=start_ms,
                end_ms=end_ms,
                duration_ms=end_ms - start_ms
            ))

        # Parse text (split by whitespace)
        text = text_info["text"]
        words = text.split()

        return AyahData(
            surah=surah,
            ayah=ayah,
            verse_key=key,
            text=text,
            words=words,
            audio_url=seg_info["audio_url"],
            segments=segments
        )

    def get_surah(self, surah: int) -> List[AyahData]:
        """
        Get all ayahs in a surah.

        Args:
            surah: Surah number (1-114)

        Returns:
            List of AyahData for all ayahs in surah
        """
        segments_data = self._load_segments()

        # Find all ayahs in this surah
        ayahs = []
        for key in segments_data.keys():
            s, a = map(int, key.split(':'))
            if s == surah:
                ayahs.append((s, a))

        # Sort by ayah number
        ayahs.sort(key=lambda x: x[1])

        return [self.get_ayah(s, a) for s, a in ayahs]

    def get_coverage_stats(self) -> Dict[str, int]:
        """
        Get statistics about data coverage.

        Returns:
            Dict with coverage statistics
        """
        segments = self._load_segments()
        quran = self._load_quran_text()

        total_words = sum(len(data["segments"]) for data in segments.values())

        return {
            "total_ayahs": len(segments),
            "total_verses": len(quran),
            "coverage_percent": 100 * len(segments) / len(quran),
            "total_words": total_words,
            "avg_words_per_ayah": total_words / len(segments)
        }


if __name__ == "__main__":
    # Demo usage
    loader = SegmentsLoader()

    print("=== Coverage Stats ===")
    stats = loader.get_coverage_stats()
    for key, value in stats.items():
        print(f"  {key}: {value:.1f}" if isinstance(value, float) else f"  {key}: {value}")

    print("\n=== Al-Fatiha 1:1 ===")
    ayah = loader.get_ayah(1, 1)
    print(f"Text: {ayah.text}")
    print(f"Words: {ayah.words}")
    print(f"Audio: {ayah.audio_url}")
    print(f"Segments:")
    for idx, seg in enumerate(ayah.segments):
        word = ayah.words[idx] if idx < len(ayah.words) else "?"
        print(f"  [{idx}] {word}: {seg.start_ms}-{seg.end_ms}ms ({seg.duration_ms}ms)")

    print("\n=== Word at time 250ms ===")
    result = ayah.get_word_at_time(250)
    if result:
        idx, seg = result
        print(f"Word {idx}: {ayah.words[idx]} ({seg.start_ms}-{seg.end_ms}ms)")
    else:
        print("No word active at 250ms")
