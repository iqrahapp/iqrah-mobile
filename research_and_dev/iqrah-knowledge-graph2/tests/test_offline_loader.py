"""Unit tests for offline Quran data loader."""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

import pytest
from iqrah.quran_offline.loader import OfflineQuranDataLoader, load_quran_offline


class TestOfflineQuranDataLoader:
    """Test the offline Quran data loader."""

    def test_loader_initialization(self):
        """Test that loader initializes correctly."""
        loader = OfflineQuranDataLoader()
        assert loader.data_dir.exists()
        assert loader.data_dir.is_dir()

    def test_load_chapter_metadata(self):
        """Test loading chapter metadata."""
        loader = OfflineQuranDataLoader()
        chapters = loader.load_chapter_metadata()

        assert len(chapters) == 114
        assert chapters[0].name_simple == "Al-Fatihah"
        assert chapters[0].id == 1
        assert chapters[-1].id == 114

    def test_load_verses_for_chapter(self):
        """Test loading verses for a specific chapter."""
        loader = OfflineQuranDataLoader()
        verses = loader.load_verses_for_chapter(1)

        assert len(verses) == 7  # Al-Fatihah has 7 verses
        assert verses[0].verse_key == "1:1"
        assert verses[0].chapter_id == 1
        assert len(verses[0].words) > 0

    def test_verse_word_data(self):
        """Test that word-by-word data is loaded correctly."""
        loader = OfflineQuranDataLoader()
        verses = loader.load_verses_for_chapter(1)

        first_verse = verses[0]
        assert len(first_verse.words) > 0

        first_word = first_verse.words[0]
        assert first_word.text is not None
        assert first_word.transliteration is not None
        assert first_word.transliteration.text is not None
        assert first_word.position == 1

    def test_full_quran_loading(self):
        """Test loading the full Quran."""
        quran = load_quran_offline()

        assert len(quran.chapters) == 114
        assert quran.total_verses() == 6236

    def test_quran_indexing(self):
        """Test accessing verses by key."""
        quran = load_quran_offline()

        # Test verse access
        verse = quran["1:1"]
        assert verse.verse_key == "1:1"
        assert verse.chapter_id == 1
        assert verse.verse_number == 1

        # Test chapter access
        chapter = quran["1"]
        assert chapter.id == 1
        assert chapter.name_simple == "Al-Fatihah"

    def test_verse_text_fields(self):
        """Test that verse text fields are populated."""
        quran = load_quran_offline()
        verse = quran["1:1"]

        assert verse.text_uthmani_simple is not None
        assert len(verse.text_uthmani_simple) > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
