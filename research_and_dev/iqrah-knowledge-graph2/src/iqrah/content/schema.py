"""
Content Database Schema

Defines the normalized schema for storing Quranic content data.
This data is stored separately from the graph structure to allow:
- Independent updates (e.g., new translations without regenerating graphs)
- Smaller graph files (structure only)
- Fast indexed lookups at runtime
"""

from typing import List


class ContentDatabaseSchema:
    """SQL schema for content database."""

    @staticmethod
    def get_schema_version() -> str:
        """Get the current schema version."""
        return "1.0.0"

    @staticmethod
    def get_all_schemas() -> List[str]:
        """Get all SQL CREATE TABLE statements."""
        return [
            ContentDatabaseSchema.schema_version(),
            ContentDatabaseSchema.chapters(),
            ContentDatabaseSchema.verses(),
            ContentDatabaseSchema.words(),
            ContentDatabaseSchema.word_translations(),
            ContentDatabaseSchema.word_transliterations(),
            ContentDatabaseSchema.verse_translations(),
            ContentDatabaseSchema.morphology(),
            ContentDatabaseSchema.lemmas(),
            ContentDatabaseSchema.roots(),
        ]

    @staticmethod
    def get_all_indexes() -> List[str]:
        """Get all CREATE INDEX statements."""
        return [
            # Verse indexes
            "CREATE INDEX IF NOT EXISTS idx_verses_verse_key ON verses(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_verses_chapter_number ON verses(chapter_number);",

            # Word indexes
            "CREATE INDEX IF NOT EXISTS idx_words_verse_key ON words(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_words_word_text ON words(word_text);",

            # Translation indexes
            "CREATE INDEX IF NOT EXISTS idx_word_translations_node_id ON word_translations(node_id);",
            "CREATE INDEX IF NOT EXISTS idx_word_translations_verse_key ON word_translations(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_word_transliterations_node_id ON word_transliterations(node_id);",
            "CREATE INDEX IF NOT EXISTS idx_verse_translations_verse_key ON verse_translations(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_verse_translations_translation_id ON verse_translations(translation_id);",

            # Morphology indexes
            "CREATE INDEX IF NOT EXISTS idx_morphology_verse_key ON morphology(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_morphology_lemma ON morphology(lemma);",
            "CREATE INDEX IF NOT EXISTS idx_morphology_root ON morphology(root);",

            # Lemma and root indexes
            "CREATE INDEX IF NOT EXISTS idx_lemmas_arabic ON lemmas(arabic);",
            "CREATE INDEX IF NOT EXISTS idx_roots_arabic ON roots(arabic);",
        ]

    @staticmethod
    def schema_version() -> str:
        """Table to track schema version."""
        return """
        CREATE TABLE IF NOT EXISTS schema_version (
            version TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        """

    @staticmethod
    def chapters() -> str:
        """Chapter metadata table."""
        return """
        CREATE TABLE IF NOT EXISTS chapters (
            node_id TEXT PRIMARY KEY,
            chapter_number INTEGER NOT NULL UNIQUE,
            name_arabic TEXT NOT NULL,
            name_simple TEXT NOT NULL,
            name_complex TEXT NOT NULL,
            revelation_place TEXT,
            revelation_order INTEGER,
            bismillah_pre BOOLEAN,
            verses_count INTEGER NOT NULL,
            pages TEXT
        );
        """

    @staticmethod
    def verses() -> str:
        """Verse content table."""
        return """
        CREATE TABLE IF NOT EXISTS verses (
            node_id TEXT PRIMARY KEY,
            verse_key TEXT NOT NULL UNIQUE,
            chapter_number INTEGER NOT NULL,
            verse_number INTEGER NOT NULL,
            text_uthmani TEXT NOT NULL,
            text_uthmani_simple TEXT,
            text_imlaei TEXT,
            text_indopak TEXT,
            juz_number INTEGER,
            hizb_number INTEGER,
            rub_number INTEGER,
            sajdah_type TEXT,
            sajdah_number INTEGER,
            page_number INTEGER,
            words_count INTEGER NOT NULL,
            FOREIGN KEY (chapter_number) REFERENCES chapters(chapter_number)
        );
        """

    @staticmethod
    def words() -> str:
        """Word instance content table."""
        return """
        CREATE TABLE IF NOT EXISTS words (
            node_id TEXT PRIMARY KEY,
            verse_key TEXT NOT NULL,
            position INTEGER NOT NULL,
            word_text TEXT NOT NULL,
            text_uthmani TEXT NOT NULL,
            text_uthmani_simple TEXT,
            text_imlaei TEXT,
            text_indopak TEXT,
            char_type_name TEXT,
            page_number INTEGER,
            line_number INTEGER,
            audio_url TEXT,
            UNIQUE(verse_key, position),
            FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
        );
        """

    @staticmethod
    def word_translations() -> str:
        """Word-by-word translation table."""
        return """
        CREATE TABLE IF NOT EXISTS word_translations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            node_id TEXT NOT NULL,
            verse_key TEXT NOT NULL,
            position INTEGER NOT NULL,
            translation_id TEXT NOT NULL,
            language_code TEXT NOT NULL,
            text TEXT NOT NULL,
            UNIQUE(node_id, translation_id),
            FOREIGN KEY (node_id) REFERENCES words(node_id)
        );
        """

    @staticmethod
    def word_transliterations() -> str:
        """Word-by-word transliteration table."""
        return """
        CREATE TABLE IF NOT EXISTS word_transliterations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            node_id TEXT NOT NULL,
            verse_key TEXT NOT NULL,
            position INTEGER NOT NULL,
            language_code TEXT NOT NULL,
            text TEXT NOT NULL,
            UNIQUE(node_id, language_code),
            FOREIGN KEY (node_id) REFERENCES words(node_id)
        );
        """

    @staticmethod
    def verse_translations() -> str:
        """Verse translation table (multiple translations supported)."""
        return """
        CREATE TABLE IF NOT EXISTS verse_translations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            verse_key TEXT NOT NULL,
            translation_id TEXT NOT NULL,
            language_code TEXT NOT NULL,
            text TEXT NOT NULL,
            resource_name TEXT,
            resource_id INTEGER,
            UNIQUE(verse_key, translation_id),
            FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
        );
        """

    @staticmethod
    def morphology() -> str:
        """Morphology data from Quranic Arabic Corpus."""
        return """
        CREATE TABLE IF NOT EXISTS morphology (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            verse_key TEXT NOT NULL,
            word_position INTEGER NOT NULL,
            segment_index INTEGER NOT NULL,
            segment TEXT NOT NULL,
            lemma TEXT,
            root TEXT,
            pos_tag TEXT,
            features TEXT,
            UNIQUE(verse_key, word_position, segment_index)
        );
        """

    @staticmethod
    def lemmas() -> str:
        """Lemma (dictionary form) table."""
        return """
        CREATE TABLE IF NOT EXISTS lemmas (
            node_id TEXT PRIMARY KEY,
            arabic TEXT NOT NULL UNIQUE,
            transliteration TEXT,
            meaning TEXT,
            occurrences_count INTEGER DEFAULT 0
        );
        """

    @staticmethod
    def roots() -> str:
        """Root (triliteral/quadriliteral) table."""
        return """
        CREATE TABLE IF NOT EXISTS roots (
            node_id TEXT PRIMARY KEY,
            arabic TEXT NOT NULL UNIQUE,
            transliteration TEXT,
            meaning TEXT,
            occurrences_count INTEGER DEFAULT 0
        );
        """
