"""
Content Database Schema - Production Version

Defines the complete, normalized schema for storing Quranic content data
with separation between inflexible (always included) and flexible (downloadable) data.

This data is stored separately from the graph structure to allow:
- Independent updates (e.g., new translations without regenerating graphs)
- Smaller graph files (structure only)
- Fast indexed lookups at runtime
- User-downloadable content packages

Schema Version: 2.0.0
"""

from typing import List


class ContentDatabaseSchema:
    """SQL schema for content database."""

    @staticmethod
    def get_schema_version() -> str:
        """Get the current schema version."""
        return "2.0.0"

    @staticmethod
    def get_all_schemas() -> List[str]:
        """Get all SQL CREATE TABLE statements."""
        return [
            # Metadata tables
            ContentDatabaseSchema.schema_version(),
            ContentDatabaseSchema.content_packages(),
            ContentDatabaseSchema.installed_packages(),

            # Inflexible data tables (always included)
            ContentDatabaseSchema.chapters(),
            ContentDatabaseSchema.verses(),
            ContentDatabaseSchema.words(),
            ContentDatabaseSchema.morphology_segments(),
            ContentDatabaseSchema.lemmas(),
            ContentDatabaseSchema.roots(),
            ContentDatabaseSchema.stems(),

            # Flexible data tables (user-downloadable)
            ContentDatabaseSchema.text_variants(),
            ContentDatabaseSchema.verse_translations(),
            ContentDatabaseSchema.word_translations(),
            ContentDatabaseSchema.word_transliterations(),
            ContentDatabaseSchema.reciters(),
            ContentDatabaseSchema.verse_recitations(),
            ContentDatabaseSchema.word_audio(),
        ]

    @staticmethod
    def get_all_indexes() -> List[str]:
        """Get all CREATE INDEX statements for optimized queries."""
        return [
            # Chapter indexes
            "CREATE INDEX IF NOT EXISTS idx_chapters_number ON chapters(chapter_number);",

            # Verse indexes (critical for filtering)
            "CREATE INDEX IF NOT EXISTS idx_verses_verse_key ON verses(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_verses_chapter ON verses(chapter_number);",
            "CREATE INDEX IF NOT EXISTS idx_verses_juz ON verses(juz_number);",
            "CREATE INDEX IF NOT EXISTS idx_verses_hizb ON verses(hizb_number);",
            "CREATE INDEX IF NOT EXISTS idx_verses_page ON verses(page_number);",
            "CREATE INDEX IF NOT EXISTS idx_verses_rub ON verses(rub_number);",

            # Word indexes
            "CREATE INDEX IF NOT EXISTS idx_words_verse_key ON words(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_words_position ON words(verse_key, position);",

            # Morphology indexes (critical for exercise generation)
            "CREATE INDEX IF NOT EXISTS idx_morphology_verse_key ON morphology_segments(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_morphology_word_pos ON morphology_segments(verse_key, word_position);",
            "CREATE INDEX IF NOT EXISTS idx_morphology_lemma ON morphology_segments(lemma_id);",
            "CREATE INDEX IF NOT EXISTS idx_morphology_root ON morphology_segments(root_id);",
            "CREATE INDEX IF NOT EXISTS idx_morphology_stem ON morphology_segments(stem_id);",
            "CREATE INDEX IF NOT EXISTS idx_morphology_pos_tag ON morphology_segments(pos_tag);",

            # Lemma and root indexes (for lookup queries)
            "CREATE INDEX IF NOT EXISTS idx_lemmas_arabic ON lemmas(arabic);",
            "CREATE INDEX IF NOT EXISTS idx_roots_arabic ON roots(arabic);",
            "CREATE INDEX IF NOT EXISTS idx_stems_arabic ON stems(arabic);",

            # Text variant indexes
            "CREATE INDEX IF NOT EXISTS idx_text_variants_package ON text_variants(package_id);",
            "CREATE INDEX IF NOT EXISTS idx_text_variants_verse ON text_variants(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_text_variants_word ON text_variants(word_id);",

            # Translation indexes
            "CREATE INDEX IF NOT EXISTS idx_verse_trans_package ON verse_translations(package_id);",
            "CREATE INDEX IF NOT EXISTS idx_verse_trans_verse ON verse_translations(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_word_trans_package ON word_translations(package_id);",
            "CREATE INDEX IF NOT EXISTS idx_word_trans_word ON word_translations(word_id);",

            # Transliteration indexes
            "CREATE INDEX IF NOT EXISTS idx_word_translit_package ON word_transliterations(package_id);",
            "CREATE INDEX IF NOT EXISTS idx_word_translit_word ON word_transliterations(word_id);",

            # Recitation indexes
            "CREATE INDEX IF NOT EXISTS idx_verse_recit_reciter ON verse_recitations(reciter_id);",
            "CREATE INDEX IF NOT EXISTS idx_verse_recit_verse ON verse_recitations(verse_key);",
            "CREATE INDEX IF NOT EXISTS idx_word_audio_word ON word_audio(word_id);",
        ]

    # =========================================================================
    # METADATA TABLES
    # =========================================================================

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
    def content_packages() -> str:
        """
        Metadata for downloadable content packages.

        Package types:
        - text_variant: Alternative text scripts (Imlaei, Indopak, etc.)
        - translation: Verse translations (different translators)
        - word_translation: Word-by-word translations
        - transliteration: Transliterations
        - reciter: Audio recitations
        """
        return """
        CREATE TABLE IF NOT EXISTS content_packages (
            package_id TEXT PRIMARY KEY,
            package_type TEXT NOT NULL,
            name TEXT NOT NULL,
            language_code TEXT,
            author TEXT,
            description TEXT,
            version TEXT NOT NULL,
            size_bytes INTEGER,
            is_default BOOLEAN DEFAULT 0,
            metadata_json TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            CHECK (package_type IN ('text_variant', 'translation', 'word_translation',
                                     'transliteration', 'reciter'))
        );
        """

    @staticmethod
    def installed_packages() -> str:
        """Track which content packages are currently installed."""
        return """
        CREATE TABLE IF NOT EXISTS installed_packages (
            package_id TEXT PRIMARY KEY,
            installed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (package_id) REFERENCES content_packages(package_id)
        );
        """

    # =========================================================================
    # INFLEXIBLE DATA TABLES (Always Included)
    # =========================================================================

    @staticmethod
    def chapters() -> str:
        """
        Chapter metadata table.

        Contains essential chapter information from surah-info-en.json.
        This is inflexible data - always included in the database.
        """
        return """
        CREATE TABLE IF NOT EXISTS chapters (
            node_id TEXT PRIMARY KEY,
            chapter_number INTEGER NOT NULL UNIQUE,
            name_arabic TEXT NOT NULL,
            name_simple TEXT NOT NULL,
            name_complex TEXT NOT NULL,
            name_transliterated TEXT,
            revelation_place TEXT,
            revelation_order INTEGER,
            bismillah_pre BOOLEAN DEFAULT 1,
            verses_count INTEGER NOT NULL,
            pages TEXT,
            CHECK (chapter_number BETWEEN 1 AND 114),
            CHECK (revelation_place IN ('makkah', 'madinah', NULL))
        );
        """

    @staticmethod
    def verses() -> str:
        """
        Verse content and metadata table.

        Contains verse text (Uthmani as default), structural metadata
        (juz, hizb, page, sajdah), and verse counts.
        This is inflexible data - always included in the database.
        """
        return """
        CREATE TABLE IF NOT EXISTS verses (
            node_id TEXT PRIMARY KEY,
            verse_key TEXT NOT NULL UNIQUE,
            chapter_number INTEGER NOT NULL,
            verse_number INTEGER NOT NULL,
            text_uthmani TEXT NOT NULL,

            -- Structural metadata for filtering
            juz_number INTEGER,
            hizb_number INTEGER,
            rub_number INTEGER,
            manzil_number INTEGER,
            ruku_number INTEGER,
            page_number INTEGER,

            -- Sajdah metadata
            sajdah_type TEXT,
            sajdah_number INTEGER,

            -- Counts
            words_count INTEGER NOT NULL,

            FOREIGN KEY (chapter_number) REFERENCES chapters(chapter_number),
            CHECK (juz_number BETWEEN 1 AND 30 OR juz_number IS NULL),
            CHECK (hizb_number BETWEEN 1 AND 60 OR hizb_number IS NULL),
            CHECK (page_number BETWEEN 1 AND 604 OR page_number IS NULL),
            CHECK (sajdah_type IN ('recommended', 'obligatory', NULL))
        );
        """

    @staticmethod
    def words() -> str:
        """
        Word instance content table.

        Contains word-level data with Uthmani text as default.
        Each word is uniquely identified by (verse_key, position).
        This is inflexible data - always included in the database.
        """
        return """
        CREATE TABLE IF NOT EXISTS words (
            node_id TEXT PRIMARY KEY,
            verse_key TEXT NOT NULL,
            position INTEGER NOT NULL,
            text_uthmani TEXT NOT NULL,
            char_type_name TEXT,
            page_number INTEGER,
            line_number INTEGER,

            UNIQUE(verse_key, position),
            FOREIGN KEY (verse_key) REFERENCES verses(verse_key),
            CHECK (position > 0)
        );
        """

    @staticmethod
    def morphology_segments() -> str:
        """
        Morphology segments table.

        Stores morphological analysis from Quranic Arabic Corpus.
        Each word can have multiple segments (prefix, stem, suffix).
        This is inflexible data - critical for grammar exercises.

        Links to lemmas, roots, and stems via foreign keys.
        """
        return """
        CREATE TABLE IF NOT EXISTS morphology_segments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            verse_key TEXT NOT NULL,
            word_position INTEGER NOT NULL,
            segment_index INTEGER NOT NULL,

            -- Segment data
            segment_text TEXT NOT NULL,
            segment_type TEXT,

            -- Morphological links
            lemma_id TEXT,
            root_id TEXT,
            stem_id TEXT,

            -- Part of speech
            pos_tag TEXT,

            -- Grammatical features (stored as JSON for flexibility)
            features_json TEXT,

            UNIQUE(verse_key, word_position, segment_index),
            FOREIGN KEY (verse_key) REFERENCES verses(verse_key),
            FOREIGN KEY (lemma_id) REFERENCES lemmas(node_id),
            FOREIGN KEY (root_id) REFERENCES roots(node_id),
            FOREIGN KEY (stem_id) REFERENCES stems(node_id),
            CHECK (word_position > 0),
            CHECK (segment_index > 0)
        );
        """

    @staticmethod
    def lemmas() -> str:
        """
        Lemma (dictionary form) table.

        Contains unique lemmas extracted from morphology corpus.
        Lemmas are the dictionary/canonical forms of words.
        This is inflexible data - critical for vocabulary exercises.
        """
        return """
        CREATE TABLE IF NOT EXISTS lemmas (
            node_id TEXT PRIMARY KEY,
            arabic TEXT NOT NULL UNIQUE,
            transliteration TEXT,
            meaning_en TEXT,
            occurrences_count INTEGER DEFAULT 0,
            CHECK (occurrences_count >= 0)
        );
        """

    @staticmethod
    def roots() -> str:
        """
        Root (triliteral/quadriliteral) table.

        Contains unique roots extracted from morphology corpus.
        Roots are the fundamental 3-4 letter base forms.
        This is inflexible data - critical for etymology exercises.
        """
        return """
        CREATE TABLE IF NOT EXISTS roots (
            node_id TEXT PRIMARY KEY,
            arabic TEXT NOT NULL UNIQUE,
            transliteration TEXT,
            meaning_en TEXT,
            root_type TEXT,
            occurrences_count INTEGER DEFAULT 0,
            CHECK (occurrences_count >= 0),
            CHECK (root_type IN ('triliteral', 'quadriliteral', NULL))
        );
        """

    @staticmethod
    def stems() -> str:
        """
        Stem (word without affixes) table.

        Contains unique stems extracted from morphology corpus.
        Stems are words without prefixes/suffixes.
        This is inflexible data - useful for pattern recognition.
        """
        return """
        CREATE TABLE IF NOT EXISTS stems (
            node_id TEXT PRIMARY KEY,
            arabic TEXT NOT NULL UNIQUE,
            transliteration TEXT,
            pattern TEXT,
            occurrences_count INTEGER DEFAULT 0,
            CHECK (occurrences_count >= 0)
        );
        """

    # =========================================================================
    # FLEXIBLE DATA TABLES (User-Downloadable)
    # =========================================================================

    @staticmethod
    def text_variants() -> str:
        """
        Alternative text variants (Imlaei, Indopak, etc.).

        This is flexible data - users can download different text scripts.
        Can be applied at verse level or word level.
        """
        return """
        CREATE TABLE IF NOT EXISTS text_variants (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_id TEXT NOT NULL,

            -- Can be verse-level or word-level
            verse_key TEXT,
            word_id TEXT,

            -- Text content
            text TEXT NOT NULL,

            FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
            FOREIGN KEY (verse_key) REFERENCES verses(verse_key),
            FOREIGN KEY (word_id) REFERENCES words(node_id),
            CHECK ((verse_key IS NOT NULL AND word_id IS NULL) OR
                   (verse_key IS NULL AND word_id IS NOT NULL))
        );
        """

    @staticmethod
    def verse_translations() -> str:
        """
        Verse translation table (multiple translations supported).

        This is flexible data - users can download different translators.
        Supports footnotes and multiple language translations.
        """
        return """
        CREATE TABLE IF NOT EXISTS verse_translations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_id TEXT NOT NULL,
            verse_key TEXT NOT NULL,
            text TEXT NOT NULL,

            -- Optional footnotes (stored as JSON)
            footnotes_json TEXT,

            UNIQUE(package_id, verse_key),
            FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
            FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
        );
        """

    @staticmethod
    def word_translations() -> str:
        """
        Word-by-word translation table.

        This is flexible data - users can download different word translations.
        Useful for learning and understanding Quranic vocabulary.
        """
        return """
        CREATE TABLE IF NOT EXISTS word_translations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_id TEXT NOT NULL,
            word_id TEXT NOT NULL,
            text TEXT NOT NULL,

            UNIQUE(package_id, word_id),
            FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
            FOREIGN KEY (word_id) REFERENCES words(node_id)
        );
        """

    @staticmethod
    def word_transliterations() -> str:
        """
        Word-by-word transliteration table.

        This is flexible data - users can download different transliteration schemes.
        Helps with pronunciation learning.
        """
        return """
        CREATE TABLE IF NOT EXISTS word_transliterations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_id TEXT NOT NULL,
            word_id TEXT NOT NULL,
            text TEXT NOT NULL,

            UNIQUE(package_id, word_id),
            FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
            FOREIGN KEY (word_id) REFERENCES words(node_id)
        );
        """

    @staticmethod
    def reciters() -> str:
        """
        Reciter metadata table.

        This is flexible data - users can download different reciters.
        Each reciter is a package with associated audio files.
        """
        return """
        CREATE TABLE IF NOT EXISTS reciters (
            reciter_id TEXT PRIMARY KEY,
            package_id TEXT NOT NULL UNIQUE,
            name_arabic TEXT,
            name_english TEXT NOT NULL,
            style TEXT,

            FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
            CHECK (style IN ('murattal', 'mujawwad', 'muallim', NULL))
        );
        """

    @staticmethod
    def verse_recitations() -> str:
        """
        Verse-level audio recitations.

        This is flexible data - associated with reciter packages.
        Contains audio URLs and optional timing segments.
        """
        return """
        CREATE TABLE IF NOT EXISTS verse_recitations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            reciter_id TEXT NOT NULL,
            verse_key TEXT NOT NULL,
            audio_url TEXT NOT NULL,
            duration_ms INTEGER,

            -- Optional word-level timing segments (stored as JSON)
            segments_json TEXT,

            UNIQUE(reciter_id, verse_key),
            FOREIGN KEY (reciter_id) REFERENCES reciters(reciter_id),
            FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
        );
        """

    @staticmethod
    def word_audio() -> str:
        """
        Word-level audio files.

        This is flexible data - shared across reciters (pronunciation is same).
        Contains audio URLs for individual word pronunciations.
        """
        return """
        CREATE TABLE IF NOT EXISTS word_audio (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            word_id TEXT NOT NULL UNIQUE,
            audio_url TEXT NOT NULL,
            duration_ms INTEGER,

            FOREIGN KEY (word_id) REFERENCES words(node_id)
        );
        """
