"""
Content Database Builder - Production Version

Builds a production-ready SQLite content database from offline Quranic data sources.
Populates all inflexible data tables with optimized loading and proper normalization.

Version: 2.0.0
"""

import sqlite3
import json
from pathlib import Path
from typing import Optional, List, Dict, Any, Set
import logging
from tqdm import tqdm
from collections import defaultdict

from ..quran_offline import load_quran_offline
from ..morphology import MorphologyCorpus
from ..graph.identifiers import NodeIdentifierGenerator as NIG
from .schema import ContentDatabaseSchema


logger = logging.getLogger(__name__)


class ContentDatabaseBuilder:
    """
    Builds a production-ready content database from offline JSON/CSV data.

    This separates content (text, translations, morphology) from graph structure,
    allowing independent updates and smaller graph files.

    Features:
    - Complete inflexible data loading (chapters, verses, words, morphology, lemmas, roots, stems)
    - Structural metadata integration (juz, hizb, page, sajdah, etc.)
    - Optimized for mobile (<40MB target)
    - Proper normalization and foreign keys
    - Progress reporting
    - Error handling and validation
    """

    def __init__(self, data_dir: Optional[Path] = None):
        """
        Initialize builder with data directory.

        Args:
            data_dir: Path to offline data directory. If None, uses default.
        """
        if data_dir is None:
            # Default to research_and_dev/data
            current_file = Path(__file__)
            kg_dir = current_file.parent.parent.parent.parent
            research_dev_dir = kg_dir.parent
            data_dir = research_dev_dir / "data"

        self.data_dir = Path(data_dir)
        if not self.data_dir.exists():
            raise ValueError(f"Data directory not found: {self.data_dir}")

        self.quran = None
        self.morphology = None
        self.metadata_cache: Dict[str, Any] = {}

        logger.info(f"ContentDatabaseBuilder initialized with data_dir: {self.data_dir}")

    def build(
        self,
        output_path: str,
        morphology_corpus_path: Optional[str] = None,
        include_default_packages: bool = True,
        show_progress: bool = True,
    ) -> None:
        """
        Build the complete content database with all inflexible data.

        Args:
            output_path: Path to output SQLite database file
            morphology_corpus_path: Path to morphology CSV file
            include_default_packages: Whether to include default flexible data packages
            show_progress: Whether to show progress bars

        Raises:
            ValueError: If data cannot be loaded
            sqlite3.Error: If database operations fail
        """
        logger.info(f"Building content database at {output_path}")
        logger.info(f"  Schema version: {ContentDatabaseSchema.get_schema_version()}")
        logger.info(f"  Data directory: {self.data_dir}")

        # Load data sources
        logger.info("Loading offline Quran data...")
        self.quran = load_quran_offline()

        if morphology_corpus_path:
            logger.info(f"Loading morphology corpus from {morphology_corpus_path}")
            self.morphology = MorphologyCorpus.from_csv(morphology_corpus_path)
        else:
            # Try default path
            default_morph_path = self.data_dir / "morphology" / "quran-morphology-v0.5.csv"
            if default_morph_path.exists():
                logger.info(f"Loading morphology corpus from {default_morph_path}")
                self.morphology = MorphologyCorpus.from_csv(str(default_morph_path))
            else:
                logger.warning("No morphology corpus provided - morphology data will be skipped")

        # Load metadata files
        logger.info("Loading structural metadata...")
        self._load_metadata()

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
            # Enable foreign keys
            conn.execute("PRAGMA foreign_keys = ON")

            # Set optimizations for faster writes
            conn.execute("PRAGMA journal_mode = WAL")
            conn.execute("PRAGMA synchronous = NORMAL")
            conn.execute("PRAGMA cache_size = 10000")
            conn.execute("PRAGMA temp_store = MEMORY")

            self._create_schema(conn)

            # Populate inflexible data
            logger.info("Populating inflexible data tables...")
            self._populate_chapters(conn, show_progress)
            self._populate_verses(conn, show_progress)
            self._populate_words(conn, show_progress)

            if self.morphology:
                # Extract and populate morphological entities
                lemmas, roots, stems = self._extract_morphological_entities(show_progress)
                self._populate_lemmas(conn, lemmas, show_progress)
                self._populate_roots(conn, roots, show_progress)
                self._populate_stems(conn, stems, show_progress)
                self._populate_morphology_segments(conn, lemmas, roots, stems, show_progress)

            # Populate default flexible data packages if requested
            if include_default_packages:
                logger.info("Populating default content packages...")
                self._populate_default_packages(conn, show_progress)

            # Commit and optimize
            conn.commit()
            logger.info("Optimizing database...")
            self._optimize_database(conn)

            # Get final stats
            stats = self._get_database_stats(conn)
            logger.info("Database build complete!")
            logger.info(f"  Schema version: {stats['schema_version']}")
            logger.info(f"  Chapters: {stats['chapters']}")
            logger.info(f"  Verses: {stats['verses']}")
            logger.info(f"  Words: {stats['words']}")
            logger.info(f"  Morphology segments: {stats['morphology_segments']}")
            logger.info(f"  Lemmas: {stats['lemmas']}")
            logger.info(f"  Roots: {stats['roots']}")
            logger.info(f"  Stems: {stats['stems']}")

            # Get file size
            db_size_mb = db_path.stat().st_size / (1024 * 1024)
            logger.info(f"  Database size: {db_size_mb:.2f} MB")

        except Exception as e:
            conn.rollback()
            logger.error(f"Error building content database: {e}")
            raise
        finally:
            conn.close()

    def _load_metadata(self) -> None:
        """Load all metadata JSON files."""
        metadata_files = {
            "ayah": "structural-metadata/quran-metadata-ayah.json",
            "juz": "structural-metadata/quran-metadata-juz.json",
            "hizb": "structural-metadata/quran-metadata-hizb.json",
            "rub": "structural-metadata/quran-metadata-rub.json",
            "sajda": "structural-metadata/quran-metadata-sajda.json",
            "manzil": "structural-metadata/quran-metadata-manzil.json",
            "ruku": "structural-metadata/quran-metadata-ruku.json",
            "surah_info": "structural-metadata/surah-info-en.json",
        }

        for key, relative_path in metadata_files.items():
            file_path = self.data_dir / relative_path
            if file_path.exists():
                with open(file_path, "r", encoding="utf-8") as f:
                    self.metadata_cache[key] = json.load(f)
                logger.debug(f"Loaded metadata: {key}")
            else:
                logger.warning(f"Metadata file not found: {file_path}")
                self.metadata_cache[key] = {}

    def _create_schema(self, conn: sqlite3.Connection) -> None:
        """Create database schema with all tables and indexes."""
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
        """Populate chapters table with complete metadata."""
        logger.info("Populating chapters...")

        cursor = conn.cursor()
        chapters = self.quran.chapters
        surah_info = self.metadata_cache.get("surah_info", {})

        iterator = tqdm(chapters, desc="Chapters", disable=not show_progress)

        for chapter in iterator:
            node_id = NIG.chapter(chapter.id)

            # Get additional metadata from surah_info
            info = surah_info.get(str(chapter.id), {})

            cursor.execute(
                """
                INSERT INTO chapters (
                    node_id, chapter_number, name_arabic, name_simple, name_complex,
                    name_transliterated, revelation_place, revelation_order,
                    bismillah_pre, verses_count, pages
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    node_id,
                    chapter.id,
                    chapter.name_arabic,
                    chapter.name_simple,
                    chapter.name_complex,
                    info.get("transliteration", {}).get("en"),
                    chapter.revelation_place,
                    chapter.revelation_order if hasattr(chapter, "revelation_order") else None,
                    chapter.bismillah_pre if hasattr(chapter, "bismillah_pre") else True,
                    chapter.verses_count,
                    str(chapter.pages) if hasattr(chapter, "pages") and chapter.pages else None,
                ),
            )

        conn.commit()
        logger.info(f"Populated {len(chapters)} chapters")

    def _populate_verses(self, conn: sqlite3.Connection, show_progress: bool) -> None:
        """Populate verses table with complete structural metadata."""
        logger.info("Populating verses...")

        cursor = conn.cursor()
        total_verses = 0

        ayah_metadata = self.metadata_cache.get("ayah", {})
        chapters = self.quran.chapters
        iterator = tqdm(chapters, desc="Verses", disable=not show_progress)

        for chapter in iterator:
            verses = chapter.verses

            for verse in verses:
                node_id = NIG.verse(verse.verse_key)

                # Get additional metadata
                verse_id = str(verse.id) if hasattr(verse, "id") else None
                metadata = ayah_metadata.get(verse_id, {}) if verse_id else {}

                cursor.execute(
                    """
                    INSERT INTO verses (
                        node_id, verse_key, chapter_number, verse_number,
                        text_uthmani, juz_number, hizb_number, rub_number,
                        manzil_number, ruku_number, page_number,
                        sajdah_type, sajdah_number, words_count
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        node_id,
                        verse.verse_key,
                        verse.chapter_number,
                        verse.verse_number,
                        verse.text_uthmani,
                        verse.juz_number if hasattr(verse, "juz_number") else None,
                        verse.hizb_number if hasattr(verse, "hizb_number") else None,
                        verse.rub_el_hizb_number if hasattr(verse, "rub_el_hizb_number") else None,
                        verse.manzil_number if hasattr(verse, "manzil_number") else None,
                        metadata.get("ruku_number"),
                        verse.page_number if hasattr(verse, "page_number") else None,
                        verse.sajdah_type if hasattr(verse, "sajdah_type") else None,
                        verse.sajdah_number if hasattr(verse, "sajdah_number") else None,
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
                            node_id, verse_key, position, text_uthmani,
                            char_type_name, page_number, line_number
                        ) VALUES (?, ?, ?, ?, ?, ?, ?)
                        """,
                        (
                            node_id,
                            verse.verse_key,
                            position,
                            word.text_uthmani,
                            word.char_type_name if hasattr(word, "char_type_name") else None,
                            word.page_number if hasattr(word, "page_number") else None,
                            word.line_number if hasattr(word, "line_number") else None,
                        ),
                    )

                    total_words += 1

        conn.commit()
        logger.info(f"Populated {total_words} words")

    def _extract_morphological_entities(
        self, show_progress: bool
    ) -> tuple[Dict[str, Dict], Dict[str, Dict], Dict[str, Dict]]:
        """
        Extract unique lemmas, roots, and stems from morphology corpus.

        Returns:
            Tuple of (lemmas_dict, roots_dict, stems_dict)
        """
        logger.info("Extracting morphological entities from corpus...")

        lemmas_dict: Dict[str, Dict] = {}
        roots_dict: Dict[str, Dict] = {}
        stems_dict: Dict[str, Dict] = {}

        segments = self.morphology.segments
        iterator = tqdm(segments, desc="Extracting entities", disable=not show_progress)

        for segment in iterator:
            # Extract lemma
            if segment.lemma:
                lemma = segment.lemma.strip()
                if lemma and lemma not in lemmas_dict:
                    lemmas_dict[lemma] = {
                        "node_id": NIG.lemma(lemma),
                        "arabic": lemma,
                        "occurrences": 1,
                    }
                elif lemma:
                    lemmas_dict[lemma]["occurrences"] += 1

            # Extract root
            if segment.root:
                root = segment.root.strip()
                if root and root not in roots_dict:
                    # Determine root type based on length
                    root_type = "triliteral" if len(root) == 3 else "quadriliteral" if len(root) == 4 else None

                    roots_dict[root] = {
                        "node_id": NIG.root(root),
                        "arabic": root,
                        "root_type": root_type,
                        "occurrences": 1,
                    }
                elif root:
                    roots_dict[root]["occurrences"] += 1

            # Extract stem (from ROOT segment types)
            # A stem is the segment text when it's a root-type segment
            if hasattr(segment, "segment_type") and segment.segment_type and "ROOT" in str(segment.segment_type):
                stem = segment.segment.strip()
                if stem and stem not in stems_dict:
                    stems_dict[stem] = {
                        "node_id": f"STEM:{stem}",
                        "arabic": stem,
                        "occurrences": 1,
                    }
                elif stem:
                    stems_dict[stem]["occurrences"] += 1

        logger.info(f"Extracted {len(lemmas_dict)} unique lemmas")
        logger.info(f"Extracted {len(roots_dict)} unique roots")
        logger.info(f"Extracted {len(stems_dict)} unique stems")

        return lemmas_dict, roots_dict, stems_dict

    def _populate_lemmas(
        self, conn: sqlite3.Connection, lemmas_dict: Dict[str, Dict], show_progress: bool
    ) -> None:
        """Populate lemmas table."""
        logger.info("Populating lemmas...")

        cursor = conn.cursor()
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

    def _populate_roots(
        self, conn: sqlite3.Connection, roots_dict: Dict[str, Dict], show_progress: bool
    ) -> None:
        """Populate roots table."""
        logger.info("Populating roots...")

        cursor = conn.cursor()
        iterator = tqdm(roots_dict.values(), desc="Roots", disable=not show_progress)

        for root_data in iterator:
            cursor.execute(
                """
                INSERT INTO roots (
                    node_id, arabic, root_type, occurrences_count
                ) VALUES (?, ?, ?, ?)
                """,
                (
                    root_data["node_id"],
                    root_data["arabic"],
                    root_data["root_type"],
                    root_data["occurrences"],
                ),
            )

        conn.commit()
        logger.info(f"Populated {len(roots_dict)} roots")

    def _populate_stems(
        self, conn: sqlite3.Connection, stems_dict: Dict[str, Dict], show_progress: bool
    ) -> None:
        """Populate stems table."""
        logger.info("Populating stems...")

        cursor = conn.cursor()
        iterator = tqdm(stems_dict.values(), desc="Stems", disable=not show_progress)

        for stem_data in iterator:
            cursor.execute(
                """
                INSERT INTO stems (
                    node_id, arabic, occurrences_count
                ) VALUES (?, ?, ?)
                """,
                (
                    stem_data["node_id"],
                    stem_data["arabic"],
                    stem_data["occurrences"],
                ),
            )

        conn.commit()
        logger.info(f"Populated {len(stems_dict)} stems")

    def _populate_morphology_segments(
        self,
        conn: sqlite3.Connection,
        lemmas_dict: Dict[str, Dict],
        roots_dict: Dict[str, Dict],
        stems_dict: Dict[str, Dict],
        show_progress: bool,
    ) -> None:
        """Populate morphology_segments table with full morphological analysis."""
        logger.info("Populating morphology segments...")

        cursor = conn.cursor()
        total_segments = 0

        segments = self.morphology.segments
        iterator = tqdm(segments, desc="Morphology", disable=not show_progress)

        for segment in iterator:
            # Get verse key from location (chapter:verse:word:segment)
            location = segment.location
            verse_key = f"{location[0]}:{location[1]}"
            word_position = location[2]
            segment_index = location[3]

            # Get foreign key IDs
            lemma_id = lemmas_dict[segment.lemma]["node_id"] if segment.lemma and segment.lemma in lemmas_dict else None
            root_id = roots_dict[segment.root]["node_id"] if segment.root and segment.root in roots_dict else None

            # Determine stem_id
            stem_id = None
            if hasattr(segment, "segment_type") and segment.segment_type and "ROOT" in str(segment.segment_type):
                stem_text = segment.segment.strip()
                if stem_text in stems_dict:
                    stem_id = stems_dict[stem_text]["node_id"]

            # Get POS tag
            pos_tag = segment.pos if hasattr(segment, "pos") else None
            if pos_tag:
                pos_tag = str(pos_tag).replace("PartOfSpeech.", "")

            # Get segment type
            segment_type = segment.segment_type if hasattr(segment, "segment_type") else None
            if segment_type:
                segment_type = str(segment_type).replace("SegmentType.", "")

            # Convert grammatical features to JSON
            features_json = None
            if hasattr(segment, "grammatical_features") and segment.grammatical_features:
                features_list = [str(f).replace("GrammaticalFeature.", "") for f in segment.grammatical_features]
                features_json = json.dumps(features_list)

            cursor.execute(
                """
                INSERT INTO morphology_segments (
                    verse_key, word_position, segment_index, segment_text,
                    segment_type, lemma_id, root_id, stem_id,
                    pos_tag, features_json
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    verse_key,
                    word_position,
                    segment_index,
                    segment.segment,
                    segment_type,
                    lemma_id,
                    root_id,
                    stem_id,
                    pos_tag,
                    features_json,
                ),
            )

            total_segments += 1

        conn.commit()
        logger.info(f"Populated {total_segments} morphology segments")

    def _populate_default_packages(self, conn: sqlite3.Connection, show_progress: bool) -> None:
        """
        Populate default content packages (if data is available).

        Default packages:
        - English transliteration (word-by-word)
        - Word audio (if available)
        """
        logger.info("Populating default content packages...")

        # This is a placeholder - full implementation would load:
        # 1. Word-by-word transliterations from transliterations/english-wbw-transliteration.json
        # 2. Word audio URLs from recitation/wbw/word_audio_map.json
        # For now, we'll skip this as it's flexible data

        logger.info("Default packages population skipped (flexible data)")

    def _optimize_database(self, conn: sqlite3.Connection) -> None:
        """Optimize database for production use."""
        logger.info("Running VACUUM and ANALYZE...")

        conn.execute("VACUUM")
        conn.execute("ANALYZE")

        logger.info("Database optimization complete")

    def _get_database_stats(self, conn: sqlite3.Connection) -> Dict[str, Any]:
        """Get database statistics."""
        cursor = conn.cursor()

        stats = {}

        # Get schema version
        cursor.execute("SELECT version FROM schema_version LIMIT 1")
        stats["schema_version"] = cursor.fetchone()[0]

        # Count tables
        tables = [
            "chapters",
            "verses",
            "words",
            "morphology_segments",
            "lemmas",
            "roots",
            "stems",
        ]

        for table in tables:
            cursor.execute(f"SELECT COUNT(*) FROM {table}")
            stats[table] = cursor.fetchone()[0]

        return stats
