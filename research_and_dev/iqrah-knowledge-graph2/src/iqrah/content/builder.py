"""
Content Database Builder

Builds a SQLite content database from offline Quranic data sources.
Populates all tables with normalized content data for fast lookup.
"""

import sqlite3
from pathlib import Path
from typing import Optional, List
import logging
from tqdm import tqdm

from ..quran_offline import load_quran_offline
from ..morphology import MorphologyCorpus
from ..graph.identifiers import NodeIdentifierGenerator as NIG
from .schema import ContentDatabaseSchema


logger = logging.getLogger(__name__)


class ContentDatabaseBuilder:
    """
    Builds a content database from offline JSON/CSV data.

    This separates content (text, translations) from graph structure,
    allowing independent updates and smaller graph files.
    """

    def __init__(self, data_dir: Optional[Path] = None):
        """
        Initialize builder with data directory.

        Args:
            data_dir: Path to offline data directory. If None, uses default.
        """
        self.data_dir = data_dir
        self.quran = None
        self.morphology = None

    def build(
        self,
        output_path: str,
        morphology_corpus_path: Optional[str] = None,
        translation_ids: Optional[List[str]] = None,
        show_progress: bool = True,
    ) -> None:
        """
        Build the complete content database.

        Args:
            output_path: Path to output SQLite database file
            morphology_corpus_path: Path to morphology CSV file
            translation_ids: List of translation IDs to include (e.g., ["en-131"])
            show_progress: Whether to show progress bars

        Raises:
            ValueError: If data cannot be loaded
            sqlite3.Error: If database operations fail
        """
        logger.info(f"Building content database at {output_path}")

        # Load data sources
        logger.info("Loading offline Quran data...")
        self.quran = load_quran_offline()

        if morphology_corpus_path:
            logger.info(f"Loading morphology corpus from {morphology_corpus_path}")
            self.morphology = MorphologyCorpus.from_csv(morphology_corpus_path)

        # Default to English translation if none specified
        if translation_ids is None:
            translation_ids = ["en-131"]  # Dr. Mustafa Khattab

        # Create database
        db_path = Path(output_path)
        db_path.parent.mkdir(parents=True, exist_ok=True)

        # Remove existing database if present
        if db_path.exists():
            logger.warning(f"Removing existing database at {output_path}")
            db_path.unlink()

        # Connect and create schema
        conn = sqlite3.connect(output_path)
        try:
            self._create_schema(conn)
            self._populate_chapters(conn, show_progress)
            self._populate_verses(conn, show_progress)
            self._populate_words(conn, show_progress)
            self._populate_word_translations(conn, translation_ids, show_progress)
            self._populate_word_transliterations(conn, show_progress)
            self._populate_verse_translations(conn, translation_ids, show_progress)

            if self.morphology:
                self._populate_morphology(conn, show_progress)
                self._populate_lemmas(conn, show_progress)
                self._populate_roots(conn, show_progress)

            conn.commit()
            logger.info(f"Content database built successfully at {output_path}")

        except Exception as e:
            conn.rollback()
            logger.error(f"Error building content database: {e}")
            raise
        finally:
            conn.close()

    def _create_schema(self, conn: sqlite3.Connection) -> None:
        """Create database schema."""
        logger.info("Creating database schema...")

        cursor = conn.cursor()

        # Create tables
        for schema_sql in ContentDatabaseSchema.get_all_schemas():
            cursor.execute(schema_sql)

        # Create indexes
        for index_sql in ContentDatabaseSchema.get_all_indexes():
            cursor.execute(index_sql)

        # Insert schema version
        cursor.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            (ContentDatabaseSchema.get_schema_version(),),
        )

        conn.commit()
        logger.info("Schema created successfully")

    def _populate_chapters(self, conn: sqlite3.Connection, show_progress: bool) -> None:
        """Populate chapters table."""
        logger.info("Populating chapters...")

        cursor = conn.cursor()
        chapters = self.quran.chapters

        iterator = tqdm(chapters, desc="Chapters", disable=not show_progress)

        for chapter in iterator:
            node_id = NIG.chapter(chapter.id)

            cursor.execute(
                """
                INSERT INTO chapters (
                    node_id, chapter_number, name_arabic, name_simple, name_complex,
                    revelation_place, revelation_order, bismillah_pre, verses_count, pages
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    node_id,
                    chapter.id,
                    chapter.name_arabic,
                    chapter.name_simple,
                    chapter.name_complex,
                    chapter.revelation_place,
                    chapter.revelation_order,
                    chapter.bismillah_pre,
                    chapter.verses_count,
                    str(chapter.pages) if chapter.pages else None,
                ),
            )

        conn.commit()
        logger.info(f"Populated {len(chapters)} chapters")

    def _populate_verses(self, conn: sqlite3.Connection, show_progress: bool) -> None:
        """Populate verses table."""
        logger.info("Populating verses...")

        cursor = conn.cursor()
        total_verses = 0

        chapters = self.quran.chapters
        iterator = tqdm(chapters, desc="Verses", disable=not show_progress)

        for chapter in iterator:
            verses = chapter.verses

            for verse in verses:
                node_id = NIG.verse(verse.verse_key)

                cursor.execute(
                    """
                    INSERT INTO verses (
                        node_id, verse_key, chapter_number, verse_number,
                        text_uthmani, text_uthmani_simple, text_imlaei,
                        juz_number, hizb_number, rub_number,
                        sajdah_type, sajdah_number, page_number, words_count
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        node_id,
                        verse.verse_key,
                        verse.chapter_number,
                        verse.verse_number,
                        verse.text_uthmani,
                        verse.text_uthmani_simple if hasattr(verse, "text_uthmani_simple") else None,
                        verse.text_imlaei if hasattr(verse, "text_imlaei") else None,
                        verse.juz_number,
                        verse.hizb_number,
                        verse.rub_el_hizb_number if hasattr(verse, "rub_el_hizb_number") else None,
                        verse.sajdah_type if hasattr(verse, "sajdah_type") else None,
                        verse.sajdah_number if hasattr(verse, "sajdah_number") else None,
                        verse.page_number,
                        len(verse.words),
                    ),
                )

                total_verses += 1

        conn.commit()
        logger.info(f"Populated {total_verses} verses")

    def _populate_words(self, conn: sqlite3.Connection, show_progress: bool) -> None:
        """Populate words table."""
        logger.info("Populating words...")

        cursor = conn.cursor()
        total_words = 0

        chapters = self.quran.chapters
        iterator = tqdm(chapters, desc="Words", disable=not show_progress)

        for chapter in iterator:
            for verse in chapter.verses:
                for position, word in enumerate(verse.words, start=1):
                    node_id = NIG.word_instance(verse.verse_key, position)

                    cursor.execute(
                        """
                        INSERT INTO words (
                            node_id, verse_key, position, word_text,
                            text_uthmani, text_uthmani_simple, text_imlaei,
                            char_type_name, page_number, line_number, audio_url
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                        """,
                        (
                            node_id,
                            verse.verse_key,
                            position,
                            word.text_uthmani,  # Use as canonical word_text
                            word.text_uthmani,
                            word.text_uthmani_simple if hasattr(word, "text_uthmani_simple") else None,
                            word.text_imlaei if hasattr(word, "text_imlaei") else None,
                            word.char_type_name if hasattr(word, "char_type_name") else None,
                            word.page_number if hasattr(word, "page_number") else None,
                            word.line_number if hasattr(word, "line_number") else None,
                            word.audio_url if hasattr(word, "audio_url") else None,
                        ),
                    )

                    total_words += 1

        conn.commit()
        logger.info(f"Populated {total_words} words")

    def _populate_word_translations(
        self, conn: sqlite3.Connection, translation_ids: List[str], show_progress: bool
    ) -> None:
        """Populate word_translations table."""
        logger.info(f"Populating word translations for {translation_ids}...")

        cursor = conn.cursor()
        total_translations = 0

        chapters = self.quran.chapters
        iterator = tqdm(chapters, desc="Word Translations", disable=not show_progress)

        for chapter in iterator:
            for verse in chapter.verses:
                # Try to get translations for this verse
                try:
                    # Access translation if available
                    # Note: This depends on how translations are loaded in offline loader
                    for position, word in enumerate(verse.words, start=1):
                        node_id = NIG.word_instance(verse.verse_key, position)

                        # Check if word has translation
                        if hasattr(word, "translation") and word.translation:
                            translation_text = word.translation.text if hasattr(word.translation, "text") else str(word.translation)

                            cursor.execute(
                                """
                                INSERT INTO word_translations (
                                    node_id, verse_key, position, translation_id, language_code, text
                                ) VALUES (?, ?, ?, ?, ?, ?)
                                """,
                                (
                                    node_id,
                                    verse.verse_key,
                                    position,
                                    "default",  # Translation ID
                                    "en",  # Language code
                                    translation_text,
                                ),
                            )

                            total_translations += 1

                except Exception as e:
                    # Translation might not be available for all words
                    logger.debug(f"No translation for {verse.verse_key}: {e}")

        conn.commit()
        logger.info(f"Populated {total_translations} word translations")

    def _populate_word_transliterations(
        self, conn: sqlite3.Connection, show_progress: bool
    ) -> None:
        """Populate word_transliterations table."""
        logger.info("Populating word transliterations...")

        cursor = conn.cursor()
        total_transliterations = 0

        chapters = self.quran.chapters
        iterator = tqdm(chapters, desc="Word Transliterations", disable=not show_progress)

        for chapter in iterator:
            for verse in chapter.verses:
                for position, word in enumerate(verse.words, start=1):
                    node_id = NIG.word_instance(verse.verse_key, position)

                    # Check if word has transliteration
                    if hasattr(word, "transliteration") and word.transliteration:
                        transliteration_text = word.transliteration.text if hasattr(word.transliteration, "text") else str(word.transliteration)

                        cursor.execute(
                            """
                            INSERT INTO word_transliterations (
                                node_id, verse_key, position, language_code, text
                            ) VALUES (?, ?, ?, ?, ?)
                            """,
                            (
                                node_id,
                                verse.verse_key,
                                position,
                                "en",  # Language code
                                transliteration_text,
                            ),
                        )

                        total_transliterations += 1

        conn.commit()
        logger.info(f"Populated {total_transliterations} word transliterations")

    def _populate_verse_translations(
        self, conn: sqlite3.Connection, translation_ids: List[str], show_progress: bool
    ) -> None:
        """Populate verse_translations table."""
        logger.info(f"Populating verse translations for {translation_ids}...")

        # Note: Full implementation would load different translations
        # For now, we'll skip this as it depends on the translation data structure
        logger.warning("Verse translations not yet implemented - skipping")

    def _populate_morphology(self, conn: sqlite3.Connection, show_progress: bool) -> None:
        """Populate morphology table."""
        logger.info("Populating morphology...")

        cursor = conn.cursor()
        total_segments = 0

        segments = self.morphology.segments
        iterator = tqdm(segments, desc="Morphology", disable=not show_progress)

        for segment in iterator:
            cursor.execute(
                """
                INSERT INTO morphology (
                    verse_key, word_position, segment_index, segment,
                    lemma, root, pos_tag, features
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    segment.verse_key,
                    segment.word_position,
                    segment.segment_index,
                    segment.segment,
                    segment.lemma,
                    segment.root,
                    segment.pos_tag if hasattr(segment, "pos_tag") else None,
                    segment.features if hasattr(segment, "features") else None,
                ),
            )

            total_segments += 1

        conn.commit()
        logger.info(f"Populated {total_segments} morphology segments")

    def _populate_lemmas(self, conn: sqlite3.Connection, show_progress: bool) -> None:
        """Populate lemmas table."""
        logger.info("Populating lemmas...")

        cursor = conn.cursor()

        # Extract unique lemmas from morphology
        lemmas_dict = {}
        for segment in self.morphology.segments:
            if segment.lemma and segment.lemma not in lemmas_dict:
                lemmas_dict[segment.lemma] = {
                    "node_id": NIG.lemma(segment.lemma),
                    "arabic": segment.lemma,
                    "occurrences": 1,
                }
            elif segment.lemma:
                lemmas_dict[segment.lemma]["occurrences"] += 1

        iterator = tqdm(lemmas_dict.values(), desc="Lemmas", disable=not show_progress)

        for lemma_data in iterator:
            cursor.execute(
                """
                INSERT INTO lemmas (
                    node_id, arabic, occurrences_count
                ) VALUES (?, ?, ?)
                """,
                (
                    lemma_data["node_id"],
                    lemma_data["arabic"],
                    lemma_data["occurrences"],
                ),
            )

        conn.commit()
        logger.info(f"Populated {len(lemmas_dict)} lemmas")

    def _populate_roots(self, conn: sqlite3.Connection, show_progress: bool) -> None:
        """Populate roots table."""
        logger.info("Populating roots...")

        cursor = conn.cursor()

        # Extract unique roots from morphology
        roots_dict = {}
        for segment in self.morphology.segments:
            if segment.root and segment.root not in roots_dict:
                roots_dict[segment.root] = {
                    "node_id": NIG.root(segment.root),
                    "arabic": segment.root,
                    "occurrences": 1,
                }
            elif segment.root:
                roots_dict[segment.root]["occurrences"] += 1

        iterator = tqdm(roots_dict.values(), desc="Roots", disable=not show_progress)

        for root_data in iterator:
            cursor.execute(
                """
                INSERT INTO roots (
                    node_id, arabic, occurrences_count
                ) VALUES (?, ?, ?)
                """,
                (
                    root_data["node_id"],
                    root_data["arabic"],
                    root_data["occurrences"],
                ),
            )

        conn.commit()
        logger.info(f"Populated {len(roots_dict)} roots")
