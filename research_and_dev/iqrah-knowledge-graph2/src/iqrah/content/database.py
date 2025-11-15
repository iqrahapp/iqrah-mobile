"""
Content Database Query Interface

Provides fast, indexed lookups for Quranic content data.
Used at runtime to augment graph nodes with content.
"""

import sqlite3
from pathlib import Path
from typing import Optional, Dict, List, Any
import logging


logger = logging.getLogger(__name__)


class ContentDatabase:
    """
    Query interface for content database.

    Provides methods to retrieve content by node ID or verse key.
    All queries use indexed lookups for performance.
    """

    def __init__(self, db_path: str):
        """
        Initialize content database connection.

        Args:
            db_path: Path to SQLite content database

        Raises:
            FileNotFoundError: If database doesn't exist
            sqlite3.Error: If connection fails
        """
        db_file = Path(db_path)
        if not db_file.exists():
            raise FileNotFoundError(f"Content database not found: {db_path}")

        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row  # Enable dict-like access
        logger.info(f"Connected to content database: {db_path}")

    def close(self) -> None:
        """Close database connection."""
        if self.conn:
            self.conn.close()
            logger.debug("Content database connection closed")

    def __enter__(self):
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()

    # Chapter queries

    def get_chapter(self, node_id: str) -> Optional[Dict[str, Any]]:
        """
        Get chapter data by node ID.

        Args:
            node_id: Chapter node ID (e.g., "CHAPTER:1")

        Returns:
            Dict with chapter data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM chapters WHERE node_id = ?",
            (node_id,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    def get_chapter_by_number(self, chapter_number: int) -> Optional[Dict[str, Any]]:
        """
        Get chapter data by chapter number.

        Args:
            chapter_number: Chapter number (1-114)

        Returns:
            Dict with chapter data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM chapters WHERE chapter_number = ?",
            (chapter_number,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    # Verse queries

    def get_verse(self, node_id: str) -> Optional[Dict[str, Any]]:
        """
        Get verse data by node ID.

        Args:
            node_id: Verse node ID (e.g., "VERSE:1:1")

        Returns:
            Dict with verse data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM verses WHERE node_id = ?",
            (node_id,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    def get_verse_by_key(self, verse_key: str) -> Optional[Dict[str, Any]]:
        """
        Get verse data by verse key.

        Args:
            verse_key: Verse key (e.g., "1:1")

        Returns:
            Dict with verse data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM verses WHERE verse_key = ?",
            (verse_key,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    def get_verses_for_chapter(self, chapter_number: int) -> List[Dict[str, Any]]:
        """
        Get all verses for a chapter.

        Args:
            chapter_number: Chapter number (1-114)

        Returns:
            List of verse dicts
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM verses WHERE chapter_number = ? ORDER BY verse_number",
            (chapter_number,)
        )
        return [dict(row) for row in cursor.fetchall()]

    def get_verse_translations(
        self, verse_key: str, translation_id: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """
        Get translations for a verse.

        Args:
            verse_key: Verse key (e.g., "1:1")
            translation_id: Optional translation ID filter

        Returns:
            List of translation dicts
        """
        cursor = self.conn.cursor()

        if translation_id:
            cursor.execute(
                "SELECT * FROM verse_translations WHERE verse_key = ? AND translation_id = ?",
                (verse_key, translation_id)
            )
        else:
            cursor.execute(
                "SELECT * FROM verse_translations WHERE verse_key = ?",
                (verse_key,)
            )

        return [dict(row) for row in cursor.fetchall()]

    # Word queries

    def get_word(self, node_id: str) -> Optional[Dict[str, Any]]:
        """
        Get word instance data by node ID.

        Args:
            node_id: Word instance node ID (e.g., "WORD_INSTANCE:1:1:1")

        Returns:
            Dict with word data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM words WHERE node_id = ?",
            (node_id,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    def get_words_for_verse(self, verse_key: str) -> List[Dict[str, Any]]:
        """
        Get all words for a verse.

        Args:
            verse_key: Verse key (e.g., "1:1")

        Returns:
            List of word dicts, ordered by position
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM words WHERE verse_key = ? ORDER BY position",
            (verse_key,)
        )
        return [dict(row) for row in cursor.fetchall()]

    def get_word_translation(
        self, node_id: str, translation_id: str = "default"
    ) -> Optional[str]:
        """
        Get translation for a word instance.

        Args:
            node_id: Word instance node ID
            translation_id: Translation ID

        Returns:
            Translation text or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT text FROM word_translations WHERE node_id = ? AND translation_id = ?",
            (node_id, translation_id)
        )
        row = cursor.fetchone()
        return row["text"] if row else None

    def get_word_transliteration(
        self, node_id: str, language_code: str = "en"
    ) -> Optional[str]:
        """
        Get transliteration for a word instance.

        Args:
            node_id: Word instance node ID
            language_code: Language code (default: "en")

        Returns:
            Transliteration text or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT text FROM word_transliterations WHERE node_id = ? AND language_code = ?",
            (node_id, language_code)
        )
        row = cursor.fetchone()
        return row["text"] if row else None

    def get_word_with_translations(self, node_id: str) -> Optional[Dict[str, Any]]:
        """
        Get word with all translations and transliterations.

        Args:
            node_id: Word instance node ID

        Returns:
            Dict with word data including translations/transliterations
        """
        word = self.get_word(node_id)
        if not word:
            return None

        # Add translations
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT translation_id, text FROM word_translations WHERE node_id = ?",
            (node_id,)
        )
        word["translations"] = {row["translation_id"]: row["text"] for row in cursor.fetchall()}

        # Add transliterations
        cursor.execute(
            "SELECT language_code, text FROM word_transliterations WHERE node_id = ?",
            (node_id,)
        )
        word["transliterations"] = {row["language_code"]: row["text"] for row in cursor.fetchall()}

        return word

    # Morphology queries

    def get_morphology_for_word(
        self, verse_key: str, word_position: int
    ) -> List[Dict[str, Any]]:
        """
        Get morphology segments for a word.

        Args:
            verse_key: Verse key
            word_position: Word position in verse (1-indexed)

        Returns:
            List of morphology segment dicts
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM morphology WHERE verse_key = ? AND word_position = ? ORDER BY segment_index",
            (verse_key, word_position)
        )
        return [dict(row) for row in cursor.fetchall()]

    def get_morphology_for_verse(self, verse_key: str) -> List[Dict[str, Any]]:
        """
        Get all morphology segments for a verse.

        Args:
            verse_key: Verse key

        Returns:
            List of morphology segment dicts
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM morphology WHERE verse_key = ? ORDER BY word_position, segment_index",
            (verse_key,)
        )
        return [dict(row) for row in cursor.fetchall()]

    # Lemma and root queries

    def get_lemma(self, node_id: str) -> Optional[Dict[str, Any]]:
        """
        Get lemma data by node ID.

        Args:
            node_id: Lemma node ID (e.g., "LEMMA:حمد")

        Returns:
            Dict with lemma data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM lemmas WHERE node_id = ?",
            (node_id,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    def get_lemma_by_arabic(self, arabic: str) -> Optional[Dict[str, Any]]:
        """
        Get lemma data by Arabic text.

        Args:
            arabic: Arabic lemma text

        Returns:
            Dict with lemma data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM lemmas WHERE arabic = ?",
            (arabic,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    def get_root(self, node_id: str) -> Optional[Dict[str, Any]]:
        """
        Get root data by node ID.

        Args:
            node_id: Root node ID (e.g., "ROOT:حمد")

        Returns:
            Dict with root data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM roots WHERE node_id = ?",
            (node_id,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    def get_root_by_arabic(self, arabic: str) -> Optional[Dict[str, Any]]:
        """
        Get root data by Arabic text.

        Args:
            arabic: Arabic root text

        Returns:
            Dict with root data or None if not found
        """
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT * FROM roots WHERE arabic = ?",
            (arabic,)
        )
        row = cursor.fetchone()
        return dict(row) if row else None

    # Bulk queries

    def get_content_for_nodes(self, node_ids: List[str]) -> Dict[str, Dict[str, Any]]:
        """
        Get content for multiple nodes efficiently.

        Args:
            node_ids: List of node IDs

        Returns:
            Dict mapping node_id -> content dict
        """
        result = {}

        # Group by node type for efficient queries
        chapters = [nid for nid in node_ids if nid.startswith("CHAPTER:")]
        verses = [nid for nid in node_ids if nid.startswith("VERSE:")]
        words = [nid for nid in node_ids if nid.startswith("WORD_INSTANCE:")]
        lemmas = [nid for nid in node_ids if nid.startswith("LEMMA:")]
        roots = [nid for nid in node_ids if nid.startswith("ROOT:")]

        cursor = self.conn.cursor()

        # Query each type
        if chapters:
            placeholders = ",".join("?" * len(chapters))
            cursor.execute(
                f"SELECT * FROM chapters WHERE node_id IN ({placeholders})",
                chapters
            )
            for row in cursor.fetchall():
                result[row["node_id"]] = dict(row)

        if verses:
            placeholders = ",".join("?" * len(verses))
            cursor.execute(
                f"SELECT * FROM verses WHERE node_id IN ({placeholders})",
                verses
            )
            for row in cursor.fetchall():
                result[row["node_id"]] = dict(row)

        if words:
            placeholders = ",".join("?" * len(words))
            cursor.execute(
                f"SELECT * FROM words WHERE node_id IN ({placeholders})",
                words
            )
            for row in cursor.fetchall():
                result[row["node_id"]] = dict(row)

        if lemmas:
            placeholders = ",".join("?" * len(lemmas))
            cursor.execute(
                f"SELECT * FROM lemmas WHERE node_id IN ({placeholders})",
                lemmas
            )
            for row in cursor.fetchall():
                result[row["node_id"]] = dict(row)

        if roots:
            placeholders = ",".join("?" * len(roots))
            cursor.execute(
                f"SELECT * FROM roots WHERE node_id IN ({placeholders})",
                roots
            )
            for row in cursor.fetchall():
                result[row["node_id"]] = dict(row)

        return result

    # Statistics

    def get_statistics(self) -> Dict[str, int]:
        """
        Get database statistics.

        Returns:
            Dict with counts of various entities
        """
        cursor = self.conn.cursor()

        stats = {}

        # Count tables
        tables = ["chapters", "verses", "words", "word_translations",
                  "word_transliterations", "verse_translations",
                  "morphology", "lemmas", "roots"]

        for table in tables:
            cursor.execute(f"SELECT COUNT(*) as count FROM {table}")
            stats[table] = cursor.fetchone()["count"]

        return stats
