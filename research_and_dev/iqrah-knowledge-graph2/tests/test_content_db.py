"""
Tests for Content Database Schema and Builder

Tests:
- Schema creation and integrity
- Data loading and foreign key constraints
- Query performance and correctness
- Common app query patterns
"""

import pytest
import sqlite3
import tempfile
from pathlib import Path

from iqrah.content.schema import ContentDatabaseSchema
from iqrah.content.builder import ContentDatabaseBuilder
from iqrah.content.database import ContentDatabase


class TestContentDatabaseSchema:
    """Test schema creation and constraints."""

    def test_schema_version(self):
        """Test schema version is correctly set."""
        version = ContentDatabaseSchema.get_schema_version()
        assert version == "2.0.0"

    def test_all_schemas_list(self):
        """Test that all schema SQL statements are returned."""
        schemas = ContentDatabaseSchema.get_all_schemas()
        assert len(schemas) > 0
        assert isinstance(schemas, list)

        # Check that required tables are present
        schema_str = " ".join(schemas)
        assert "chapters" in schema_str
        assert "verses" in schema_str
        assert "words" in schema_str
        assert "morphology_segments" in schema_str
        assert "lemmas" in schema_str
        assert "roots" in schema_str
        assert "stems" in schema_str
        assert "content_packages" in schema_str

    def test_all_indexes_list(self):
        """Test that all index SQL statements are returned."""
        indexes = ContentDatabaseSchema.get_all_indexes()
        assert len(indexes) > 0
        assert isinstance(indexes, list)

        # Check critical indexes
        index_str = " ".join(indexes)
        assert "idx_verses_verse_key" in index_str
        assert "idx_verses_juz" in index_str
        assert "idx_morphology_lemma" in index_str
        assert "idx_morphology_root" in index_str

    def test_schema_creation_in_memory(self):
        """Test schema can be created in an in-memory database."""
        conn = sqlite3.connect(":memory:")
        cursor = conn.cursor()

        try:
            # Enable foreign keys
            conn.execute("PRAGMA foreign_keys = ON")

            # Create all tables
            for schema_sql in ContentDatabaseSchema.get_all_schemas():
                cursor.execute(schema_sql)

            # Create all indexes
            for index_sql in ContentDatabaseSchema.get_all_indexes():
                cursor.execute(index_sql)

            # Insert schema version
            cursor.execute(
                "INSERT INTO schema_version (version) VALUES (?)",
                (ContentDatabaseSchema.get_schema_version(),),
            )

            # Verify tables exist
            cursor.execute(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
            )
            tables = [row[0] for row in cursor.fetchall()]

            required_tables = [
                "chapters",
                "verses",
                "words",
                "morphology_segments",
                "lemmas",
                "roots",
                "stems",
                "content_packages",
                "installed_packages",
                "text_variants",
                "verse_translations",
                "word_translations",
                "word_transliterations",
                "reciters",
                "verse_recitations",
                "word_audio",
            ]

            for table in required_tables:
                assert table in tables, f"Table {table} not found"

        finally:
            conn.close()

    def test_foreign_key_constraints(self):
        """Test that foreign key constraints are properly defined."""
        conn = sqlite3.connect(":memory:")
        cursor = conn.cursor()

        try:
            # Enable foreign keys
            conn.execute("PRAGMA foreign_keys = ON")

            # Create schema
            for schema_sql in ContentDatabaseSchema.get_all_schemas():
                cursor.execute(schema_sql)

            cursor.execute(
                "INSERT INTO schema_version (version) VALUES (?)",
                (ContentDatabaseSchema.get_schema_version(),),
            )

            # Test FK constraint: verses.chapter_number -> chapters.chapter_number
            # Insert a verse without a chapter should fail
            with pytest.raises(sqlite3.IntegrityError):
                cursor.execute(
                    """
                    INSERT INTO verses (
                        node_id, verse_key, chapter_number, verse_number,
                        text_uthmani, words_count
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    """,
                    ("VERSE:999:1", "999:1", 999, 1, "test", 1),
                )

            # Insert a chapter first
            cursor.execute(
                """
                INSERT INTO chapters (
                    node_id, chapter_number, name_arabic, name_simple,
                    name_complex, verses_count
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                ("CHAPTER:1", 1, "الفاتحة", "Al-Fatihah", "The Opening", 7),
            )

            # Now inserting a verse should work
            cursor.execute(
                """
                INSERT INTO verses (
                    node_id, verse_key, chapter_number, verse_number,
                    text_uthmani, words_count
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                ("VERSE:1:1", "1:1", 1, 1, "بِسۡمِ ٱللَّهِ", 4),
            )

            conn.commit()

        finally:
            conn.close()

    def test_unique_constraints(self):
        """Test that unique constraints prevent duplicates."""
        conn = sqlite3.connect(":memory:")
        cursor = conn.cursor()

        try:
            # Create schema
            for schema_sql in ContentDatabaseSchema.get_all_schemas():
                cursor.execute(schema_sql)

            cursor.execute(
                "INSERT INTO schema_version (version) VALUES (?)",
                (ContentDatabaseSchema.get_schema_version(),),
            )

            # Insert a lemma
            cursor.execute(
                "INSERT INTO lemmas (node_id, arabic) VALUES (?, ?)",
                ("LEMMA:كتب", "كتب"),
            )

            # Trying to insert the same lemma should fail (arabic is UNIQUE)
            with pytest.raises(sqlite3.IntegrityError):
                cursor.execute(
                    "INSERT INTO lemmas (node_id, arabic) VALUES (?, ?)",
                    ("LEMMA:كتب2", "كتب"),
                )

        finally:
            conn.close()

    def test_check_constraints(self):
        """Test that check constraints validate data."""
        conn = sqlite3.connect(":memory:")
        cursor = conn.cursor()

        try:
            # Create schema
            for schema_sql in ContentDatabaseSchema.get_all_schemas():
                cursor.execute(schema_sql)

            cursor.execute(
                "INSERT INTO schema_version (version) VALUES (?)",
                (ContentDatabaseSchema.get_schema_version(),),
            )

            # Test chapter_number check constraint (must be 1-114)
            with pytest.raises(sqlite3.IntegrityError):
                cursor.execute(
                    """
                    INSERT INTO chapters (
                        node_id, chapter_number, name_arabic, name_simple,
                        name_complex, verses_count
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    """,
                    ("CHAPTER:115", 115, "test", "test", "test", 1),
                )

            # Test juz_number check constraint (must be 1-30 or NULL)
            cursor.execute(
                """
                INSERT INTO chapters (
                    node_id, chapter_number, name_arabic, name_simple,
                    name_complex, verses_count
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                ("CHAPTER:1", 1, "الفاتحة", "Al-Fatihah", "The Opening", 7),
            )

            with pytest.raises(sqlite3.IntegrityError):
                cursor.execute(
                    """
                    INSERT INTO verses (
                        node_id, verse_key, chapter_number, verse_number,
                        text_uthmani, juz_number, words_count
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)
                    """,
                    ("VERSE:1:1", "1:1", 1, 1, "test", 31, 1),
                )

        finally:
            conn.close()


@pytest.mark.skipif(
    not Path(__file__).parent.parent.parent.parent.joinpath("data").exists(),
    reason="Data directory not found",
)
class TestContentDatabaseBuilder:
    """Test database building from offline data."""

    @pytest.fixture
    def temp_db_path(self):
        """Create a temporary database file."""
        with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as f:
            yield f.name
        # Cleanup
        Path(f.name).unlink(missing_ok=True)

    def test_builder_initialization(self):
        """Test builder can be initialized."""
        builder = ContentDatabaseBuilder()
        assert builder.data_dir.exists()

    def test_build_database(self, temp_db_path):
        """Test full database build (this is a slow test)."""
        builder = ContentDatabaseBuilder()

        # Build database
        builder.build(
            output_path=temp_db_path,
            include_default_packages=False,
            show_progress=False,
        )

        # Verify database was created
        assert Path(temp_db_path).exists()

        # Check database size (should be reasonable)
        db_size_mb = Path(temp_db_path).stat().st_size / (1024 * 1024)
        assert db_size_mb < 100, f"Database too large: {db_size_mb:.2f} MB"

        # Connect and verify data
        conn = sqlite3.connect(temp_db_path)
        cursor = conn.cursor()

        try:
            # Check schema version
            cursor.execute("SELECT version FROM schema_version")
            version = cursor.fetchone()[0]
            assert version == "2.0.0"

            # Check counts
            cursor.execute("SELECT COUNT(*) FROM chapters")
            chapter_count = cursor.fetchone()[0]
            assert chapter_count == 114

            cursor.execute("SELECT COUNT(*) FROM verses")
            verse_count = cursor.fetchone()[0]
            assert verse_count == 6236

            cursor.execute("SELECT COUNT(*) FROM words")
            word_count = cursor.fetchone()[0]
            assert word_count > 77000  # Should be ~77k+

            # Check morphology data (if available)
            cursor.execute("SELECT COUNT(*) FROM morphology_segments")
            morph_count = cursor.fetchone()[0]
            if morph_count > 0:
                assert morph_count > 130000  # Should be ~130k+

                cursor.execute("SELECT COUNT(*) FROM lemmas")
                lemma_count = cursor.fetchone()[0]
                assert lemma_count > 1000

                cursor.execute("SELECT COUNT(*) FROM roots")
                root_count = cursor.fetchone()[0]
                assert root_count > 1000

        finally:
            conn.close()


class TestContentDatabase:
    """Test database query interface."""

    @pytest.fixture
    def in_memory_db(self):
        """Create an in-memory database with sample data."""
        conn = sqlite3.connect(":memory:")
        cursor = conn.cursor()

        # Create schema
        for schema_sql in ContentDatabaseSchema.get_all_schemas():
            cursor.execute(schema_sql)

        cursor.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            (ContentDatabaseSchema.get_schema_version(),),
        )

        # Insert sample data
        cursor.execute(
            """
            INSERT INTO chapters (
                node_id, chapter_number, name_arabic, name_simple,
                name_complex, verses_count
            ) VALUES (?, ?, ?, ?, ?, ?)
            """,
            ("CHAPTER:1", 1, "الفاتحة", "Al-Fatihah", "The Opening", 7),
        )

        cursor.execute(
            """
            INSERT INTO verses (
                node_id, verse_key, chapter_number, verse_number,
                text_uthmani, juz_number, page_number, words_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("VERSE:1:1", "1:1", 1, 1, "بِسۡمِ ٱللَّهِ", 1, 1, 4),
        )

        cursor.execute(
            """
            INSERT INTO words (
                node_id, verse_key, position, text_uthmani
            ) VALUES (?, ?, ?, ?)
            """,
            ("WORD_INSTANCE:1:1:1", "1:1", 1, "بِسۡمِ"),
        )

        cursor.execute(
            """
            INSERT INTO lemmas (
                node_id, arabic, occurrences_count
            ) VALUES (?, ?, ?)
            """,
            ("LEMMA:سمو", "سمو", 100),
        )

        cursor.execute(
            """
            INSERT INTO roots (
                node_id, arabic, root_type, occurrences_count
            ) VALUES (?, ?, ?, ?)
            """,
            ("ROOT:سمو", "سمو", "triliteral", 100),
        )

        cursor.execute(
            """
            INSERT INTO morphology_segments (
                verse_key, word_position, segment_index, segment_text,
                segment_type, lemma_id, root_id, pos_tag
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("1:1", 1, 1, "بِ", "PREFIX", None, None, "P"),
        )

        cursor.execute(
            """
            INSERT INTO morphology_segments (
                verse_key, word_position, segment_index, segment_text,
                segment_type, lemma_id, root_id, pos_tag
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("1:1", 1, 2, "سْمِ", "ROOT", "LEMMA:سمو", "ROOT:سمو", "N"),
        )

        conn.commit()
        conn.close()

        return ":memory:"

    def test_query_verse_by_key(self, in_memory_db):
        """Test querying verse by verse_key."""
        conn = sqlite3.connect(in_memory_db)
        cursor = conn.cursor()

        # Recreate sample data (in-memory DB is reset)
        for schema_sql in ContentDatabaseSchema.get_all_schemas():
            cursor.execute(schema_sql)

        cursor.execute(
            """
            INSERT INTO chapters (
                node_id, chapter_number, name_arabic, name_simple,
                name_complex, verses_count
            ) VALUES (?, ?, ?, ?, ?, ?)
            """,
            ("CHAPTER:1", 1, "الفاتحة", "Al-Fatihah", "The Opening", 7),
        )

        cursor.execute(
            """
            INSERT INTO verses (
                node_id, verse_key, chapter_number, verse_number,
                text_uthmani, words_count
            ) VALUES (?, ?, ?, ?, ?, ?)
            """,
            ("VERSE:1:1", "1:1", 1, 1, "بِسۡمِ ٱللَّهِ", 4),
        )

        # Query
        cursor.execute("SELECT * FROM verses WHERE verse_key = ?", ("1:1",))
        row = cursor.fetchone()

        assert row is not None
        conn.close()

    def test_query_words_for_verse(self, in_memory_db):
        """Test querying all words for a verse."""
        conn = sqlite3.connect(in_memory_db)
        cursor = conn.cursor()

        # Recreate sample data
        for schema_sql in ContentDatabaseSchema.get_all_schemas():
            cursor.execute(schema_sql)

        cursor.execute(
            """
            INSERT INTO verses (
                node_id, verse_key, chapter_number, verse_number,
                text_uthmani, words_count
            ) VALUES (?, ?, ?, ?, ?, ?)
            """,
            ("VERSE:1:1", "1:1", 1, 1, "بِسۡمِ ٱللَّهِ", 4),
        )

        for i in range(1, 5):
            cursor.execute(
                """
                INSERT INTO words (
                    node_id, verse_key, position, text_uthmani
                ) VALUES (?, ?, ?, ?)
                """,
                (f"WORD_INSTANCE:1:1:{i}", "1:1", i, f"word{i}"),
            )

        # Query
        cursor.execute(
            "SELECT * FROM words WHERE verse_key = ? ORDER BY position", ("1:1",)
        )
        rows = cursor.fetchall()

        assert len(rows) == 4
        conn.close()

    def test_query_filter_by_juz(self, in_memory_db):
        """Test filtering verses by juz."""
        conn = sqlite3.connect(in_memory_db)
        cursor = conn.cursor()

        # Recreate sample data
        for schema_sql in ContentDatabaseSchema.get_all_schemas():
            cursor.execute(schema_sql)

        for i in range(1, 6):
            cursor.execute(
                """
                INSERT INTO verses (
                    node_id, verse_key, chapter_number, verse_number,
                    text_uthmani, juz_number, words_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (f"VERSE:1:{i}", f"1:{i}", 1, i, "test", 1 if i <= 3 else 2, 1),
            )

        # Query juz 1
        cursor.execute("SELECT * FROM verses WHERE juz_number = ?", (1,))
        rows = cursor.fetchall()

        assert len(rows) == 3
        conn.close()


class TestCommonQueryPatterns:
    """Test common app query patterns for performance and correctness."""

    @pytest.fixture
    def sample_db(self):
        """Create a database with comprehensive sample data."""
        conn = sqlite3.connect(":memory:")
        cursor = conn.cursor()

        # Create schema
        for schema_sql in ContentDatabaseSchema.get_all_schemas():
            cursor.execute(schema_sql)

        cursor.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            (ContentDatabaseSchema.get_schema_version(),),
        )

        # Insert sample chapter
        cursor.execute(
            """
            INSERT INTO chapters (
                node_id, chapter_number, name_arabic, name_simple,
                name_complex, verses_count
            ) VALUES (?, ?, ?, ?, ?, ?)
            """,
            ("CHAPTER:1", 1, "الفاتحة", "Al-Fatihah", "The Opening", 2),
        )

        # Insert sample verses
        cursor.execute(
            """
            INSERT INTO verses (
                node_id, verse_key, chapter_number, verse_number,
                text_uthmani, juz_number, page_number, words_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("VERSE:1:1", "1:1", 1, 1, "بِسۡمِ ٱللَّهِ", 1, 1, 4),
        )

        # Insert sample words
        cursor.execute(
            """
            INSERT INTO words (
                node_id, verse_key, position, text_uthmani
            ) VALUES (?, ?, ?, ?)
            """,
            ("WORD_INSTANCE:1:1:1", "1:1", 1, "بِسۡمِ"),
        )

        # Insert sample lemma and root
        cursor.execute(
            "INSERT INTO lemmas (node_id, arabic) VALUES (?, ?)",
            ("LEMMA:سمو", "سمو"),
        )

        cursor.execute(
            "INSERT INTO roots (node_id, arabic, root_type) VALUES (?, ?, ?)",
            ("ROOT:سمو", "سمو", "triliteral"),
        )

        # Insert morphology
        cursor.execute(
            """
            INSERT INTO morphology_segments (
                verse_key, word_position, segment_index, segment_text,
                segment_type, lemma_id, root_id, pos_tag
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("1:1", 1, 2, "سْمِ", "ROOT", "LEMMA:سمو", "ROOT:سمو", "N"),
        )

        conn.commit()
        return conn

    def test_query_verse_with_chapter_name(self, sample_db):
        """Test getting verse with chapter name."""
        cursor = sample_db.cursor()

        cursor.execute(
            """
            SELECT
                v.verse_key,
                v.text_uthmani,
                c.name_simple AS chapter_name
            FROM verses v
            JOIN chapters c ON v.chapter_number = c.chapter_number
            WHERE v.verse_key = ?
            """,
            ("1:1",),
        )

        row = cursor.fetchone()
        assert row is not None
        assert row[0] == "1:1"
        assert row[2] == "Al-Fatihah"

    def test_query_words_with_morphology(self, sample_db):
        """Test getting words with morphology data."""
        cursor = sample_db.cursor()

        cursor.execute(
            """
            SELECT
                w.position,
                w.text_uthmani,
                m.segment_text,
                l.arabic AS lemma,
                r.arabic AS root
            FROM words w
            LEFT JOIN morphology_segments m ON w.verse_key = m.verse_key
                AND w.position = m.word_position
            LEFT JOIN lemmas l ON m.lemma_id = l.node_id
            LEFT JOIN roots r ON m.root_id = r.node_id
            WHERE w.verse_key = ?
            ORDER BY w.position, m.segment_index
            """,
            ("1:1",),
        )

        rows = cursor.fetchall()
        assert len(rows) > 0

    def test_query_find_words_by_root(self, sample_db):
        """Test finding all words derived from a root."""
        cursor = sample_db.cursor()

        cursor.execute(
            """
            SELECT DISTINCT
                w.verse_key,
                w.position,
                w.text_uthmani,
                r.arabic AS root
            FROM morphology_segments m
            JOIN roots r ON m.root_id = r.node_id
            JOIN words w ON m.verse_key = w.verse_key
                AND m.word_position = w.position
            WHERE r.arabic = ?
            """,
            ("سمو",),
        )

        rows = cursor.fetchall()
        assert len(rows) > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
