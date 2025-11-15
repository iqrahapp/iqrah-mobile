"""Offline Quran data loader - replaces API client with local JSON data."""

import json
from pathlib import Path
from typing import Optional, Dict, List, Any
from loguru import logger

from ..quran_api.models import (
    Quran,
    Chapter,
    Verse,
    Word,
    TranslationByWord,
    TranslationByVerse,
    TransliterationByWord,
    TranslatedChapterName,
)


class OfflineQuranDataLoader:
    """Loads Quran data from local JSON files instead of API."""

    def __init__(self, data_dir: Optional[str] = None):
        """
        Initialize the offline data loader.

        Args:
            data_dir: Path to the data directory. Defaults to research_and_dev/data
        """
        if data_dir is None:
            # Default to research_and_dev/data relative to this file
            # File is at: research_and_dev/iqrah-knowledge-graph2/src/iqrah/quran_offline/loader.py
            # Go up to iqrah-knowledge-graph2, then to research_and_dev, then to data
            current_file = Path(__file__)
            iqrah_kg_dir = current_file.parent.parent.parent.parent
            research_dev_dir = iqrah_kg_dir.parent
            data_dir = research_dev_dir / "data"

        self.data_dir = Path(data_dir)
        if not self.data_dir.exists():
            raise ValueError(f"Data directory not found: {self.data_dir}")

        logger.debug(f"OfflineQuranDataLoader initialized with data_dir: {self.data_dir}")

        # Cache for loaded data
        self._cache: Dict[str, Any] = {}

    def _load_json(self, relative_path: str) -> Dict:
        """Load a JSON file and cache it."""
        if relative_path in self._cache:
            return self._cache[relative_path]

        file_path = self.data_dir / relative_path
        if not file_path.exists():
            raise FileNotFoundError(f"Data file not found: {file_path}")

        logger.debug(f"Loading JSON file: {file_path}")
        with open(file_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        self._cache[relative_path] = data
        return data

    def load_chapter_metadata(self, language: str = "en") -> List[Chapter]:
        """
        Load chapter metadata from offline data.

        Args:
            language: Language code (default: "en")

        Returns:
            List of Chapter objects with metadata
        """
        logger.debug(f"Loading chapter metadata for language: {language}")

        # Load surah info
        surah_info = self._load_json("structural-metadata/surah-info-en.json")

        # Load verse metadata to get verse counts per chapter
        verse_metadata = self._load_json("structural-metadata/quran-metadata-ayah.json")

        chapters = []
        for surah_num in range(1, 115):  # 114 chapters
            info = surah_info[str(surah_num)]

            # Count verses for this chapter
            verse_count = sum(
                1
                for verse_id, verse_data in verse_metadata.items()
                if verse_data["surah_number"] == surah_num
            )

            # Determine revelation place (Meccan or Medinan)
            # This is a simplified mapping - you may want to add this to your data
            revelation_place = "makkah"  # Default, would need proper data

            chapter = Chapter(
                id=surah_num,
                revelation_place=revelation_place,
                revelation_order=surah_num,  # Simplified
                bismillah_pre=(surah_num != 9),  # All except At-Tawbah
                name_simple=info["surah_name"],
                name_complex=info["surah_name"],
                name_arabic=info.get("name_arabic", info["surah_name"]),
                verses_count=verse_count,
                pages=[],  # Would need page data
                translated_name=TranslatedChapterName(
                    language_name=language, name=info["surah_name"]
                ),
                verses=None,  # Will be loaded separately
            )
            chapters.append(chapter)

        logger.debug(f"Loaded {len(chapters)} chapters")
        return chapters

    def load_verses_for_chapter(
        self,
        chapter_number: int,
        language: str = "en",
        words: bool = True,
        text_fields: Optional[List[str]] = None,
        word_fields: Optional[List[str]] = None,
        include_translation: bool = True,
        translation_key: str = "en-sahih-international-inline-footnotes",
    ) -> List[Verse]:
        """
        Load verses for a specific chapter from offline data.

        Args:
            chapter_number: Chapter number (1-114)
            language: Language code for transliteration
            words: Whether to include word-by-word data
            text_fields: Which text fields to include
            word_fields: Which word text fields to include
            include_translation: Whether to include translations
            translation_key: Translation identifier

        Returns:
            List of Verse objects
        """
        logger.debug(f"Loading verses for chapter {chapter_number}")

        # Default text fields
        if text_fields is None:
            text_fields = ["text_uthmani", "text_uthmani_simple"]
        if word_fields is None:
            word_fields = ["text_uthmani"]

        # Load verse metadata
        verse_metadata = self._load_json("structural-metadata/quran-metadata-ayah.json")

        # Load text data
        text_data = {}
        for field in text_fields:
            # Map field names to file paths
            field_map = {
                "text_uthmani": "text/wbw/uthmani.json",
                "text_uthmani_simple": "text/wbw/uthmani-simple.json",
                "text_imlaei": "text/wbw/imlaei.json",
            }
            if field in field_map:
                text_data[field] = self._load_json(field_map[field])

        # Load word text data
        word_text_data = {}
        for field in word_fields:
            field_map = {
                "text_uthmani": "text/wbw/uthmani.json",
                "text_uthmani_simple": "text/wbw/uthmani-simple.json",
            }
            if field in field_map:
                word_text_data[field] = self._load_json(field_map[field])

        # Load translations
        verse_translations = None
        if include_translation:
            translation_path = f"translations/{language[:2]}/{translation_key}.json"
            try:
                verse_translations = self._load_json(translation_path)
            except FileNotFoundError:
                logger.warning(f"Translation file not found: {translation_path}")

        # Load word transliterations
        word_transliterations = None
        if words:
            try:
                word_transliterations = self._load_json(
                    "transliterations/english-wbw-transliteration.json"
                )
            except FileNotFoundError:
                logger.warning("Transliteration file not found")

        verses = []
        # Find all verses for this chapter
        for verse_id, verse_meta in verse_metadata.items():
            if verse_meta["surah_number"] != chapter_number:
                continue

            verse_key = verse_meta["verse_key"]
            verse_number = verse_meta["ayah_number"]

            # Build verse text from words
            verse_text_fields = {}
            for field in text_fields:
                if field in text_data:
                    # Collect all words for this verse
                    word_texts = []
                    word_position = 1
                    while True:
                        word_key = f"{verse_key}:{word_position}"
                        if word_key in text_data[field]:
                            word_texts.append(text_data[field][word_key]["text"])
                            word_position += 1
                        else:
                            break
                    verse_text_fields[field] = " ".join(word_texts) if word_texts else None

            # Load words if requested
            verse_words = []
            if words:
                word_position = 1
                while True:
                    word_key = f"{verse_key}:{word_position}"

                    # Check if word exists in primary text data
                    primary_field = word_fields[0] if word_fields else "text_uthmani"
                    if primary_field not in word_text_data:
                        break

                    if word_key not in word_text_data[primary_field]:
                        break

                    # Build word object
                    word_data = word_text_data[primary_field][word_key]

                    # Get text in different formats
                    word_texts = {}
                    for field in word_fields:
                        if field in word_text_data and word_key in word_text_data[field]:
                            word_texts[field] = word_text_data[field][word_key]["text"]

                    # Get transliteration
                    transliteration_text = None
                    if word_transliterations and word_key in word_transliterations:
                        transliteration_text = word_transliterations[word_key]

                    # Determine char type (simplified - would need better logic)
                    # Check if this is the last word and might be an end marker
                    char_type = "word"  # Default

                    word = Word(
                        id=word_data.get("id"),
                        position=word_position,
                        text=word_texts.get("text_uthmani", word_data["text"]),
                        text_uthmani=word_texts.get("text_uthmani"),
                        text_imlaei=word_texts.get("text_imlaei"),
                        verse_key=verse_key,
                        page_number=1,  # Would need page data
                        line_number=1,  # Would need line data
                        audio_url=f"https://verses.quran.com/{verse_key}_{word_position}.mp3",
                        location=word_key,
                        char_type_name=char_type,
                        code_v1=None,
                        code_v2=None,
                        translation=TranslationByWord(
                            text="",  # Would need word-level translations
                            language_name=language,
                        ),
                        transliteration=TransliterationByWord(
                            text=transliteration_text, language_name="en"
                        ),
                    )
                    verse_words.append(word)
                    word_position += 1

            # Get verse translation
            verse_translation_list = None
            if verse_translations and verse_key in verse_translations:
                verse_translation_list = [
                    TranslationByVerse(
                        id=1, resource_id=1, text=verse_translations[verse_key]["t"]
                    )
                ]

            verse = Verse(
                id=int(verse_id),
                chapter_id=chapter_number,
                verse_number=verse_number,
                verse_key=verse_key,
                verse_index=verse_number - 1,
                text_uthmani=verse_text_fields.get("text_uthmani"),
                text_uthmani_simple=verse_text_fields.get("text_uthmani_simple"),
                text_imlaei=verse_text_fields.get("text_imlaei"),
                text_imlaei_simple=None,
                text_indopak=None,
                text_uthmani_tajweed=None,
                juz_number=1,  # Would need juz data
                hizb_number=1,  # Would need hizb data
                rub_el_hizb_number=1,  # Would need rub data
                manzil_number=1,  # Would need manzil data
                sajdah_type=None,
                sajdah_number=None,
                page_number=1,  # Would need page data
                words=verse_words,
                translations=verse_translation_list,
            )
            verses.append(verse)

        logger.debug(f"Loaded {len(verses)} verses for chapter {chapter_number}")
        return verses

    def load_full_quran(
        self,
        language: str = "en",
        words: bool = True,
        text_fields: Optional[List[str]] = None,
        word_fields: Optional[List[str]] = None,
        include_translation: bool = True,
        translation_key: str = "en-sahih-international-inline-footnotes",
    ) -> Quran:
        """
        Load the complete Quran from offline data.

        Args:
            language: Language code
            words: Whether to include word-by-word data
            text_fields: Which text fields to include
            word_fields: Which word text fields to include
            include_translation: Whether to include translations
            translation_key: Translation identifier

        Returns:
            Quran object with all chapters and verses
        """
        logger.info("Loading full Quran from offline data")

        # Load chapter metadata
        chapters = self.load_chapter_metadata(language=language)

        # Load verses for each chapter
        for chapter in chapters:
            logger.debug(f"Loading verses for chapter {chapter.id}")
            chapter.verses = self.load_verses_for_chapter(
                chapter_number=chapter.id,
                language=language,
                words=words,
                text_fields=text_fields,
                word_fields=word_fields,
                include_translation=include_translation,
                translation_key=translation_key,
            )

        logger.info(
            f"Loaded full Quran: {len(chapters)} chapters, "
            f"{sum(len(c.verses) for c in chapters)} verses"
        )

        return Quran(chapters=chapters)


def load_quran_offline(
    data_dir: Optional[str] = None,
    language: str = "en",
    words: bool = True,
    text_fields: Optional[List[str]] = None,
    word_fields: Optional[List[str]] = None,
    translation_key: str = "en-sahih-international-inline-footnotes",
) -> Quran:
    """
    Convenience function to load Quran data offline.

    This function replaces the async fetch_quran() from the API client.

    Args:
        data_dir: Path to data directory
        language: Language code
        words: Whether to include word-by-word data
        text_fields: Which text fields to include
        word_fields: Which word text fields to include
        translation_key: Translation identifier

    Returns:
        Quran object
    """
    loader = OfflineQuranDataLoader(data_dir=data_dir)
    return loader.load_full_quran(
        language=language,
        words=words,
        text_fields=text_fields,
        word_fields=word_fields,
        include_translation=True,
        translation_key=translation_key,
    )
