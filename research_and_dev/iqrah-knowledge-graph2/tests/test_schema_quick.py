"""
Quick validation test for content database schema.
This is a standalone test that doesn't require full iqrah package installation.
"""

import sqlite3
import sys
from pathlib import Path

# Import schema module directly by loading the file
import importlib.util
schema_path = Path(__file__).parent.parent / "src" / "iqrah" / "content" / "schema.py"
spec = importlib.util.spec_from_file_location("schema", schema_path)
schema = importlib.util.module_from_spec(spec)
spec.loader.exec_module(schema)
ContentDatabaseSchema = schema.ContentDatabaseSchema


def test_schema_version():
    """Test schema version is correctly set."""
    version = ContentDatabaseSchema.get_schema_version()
    assert version == "2.0.0", f"Expected version 2.0.0, got {version}"
    print("✓ Schema version correct: 2.0.0")


def test_schema_creation():
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

        print(f"✓ All {len(required_tables)} required tables created successfully")

    finally:
        conn.close()


def test_foreign_keys():
    """Test foreign key constraints work."""
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
        try:
            cursor.execute(
                """
                INSERT INTO verses (
                    node_id, verse_key, chapter_number, verse_number,
                    text_uthmani, words_count
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                ("VERSE:999:1", "999:1", 999, 1, "test", 1),
            )
            assert False, "Expected IntegrityError for missing FK"
        except sqlite3.IntegrityError:
            pass  # Expected

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
        print("✓ Foreign key constraints working correctly")

    finally:
        conn.close()


def test_check_constraints():
    """Test check constraints validate data."""
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
        try:
            cursor.execute(
                """
                INSERT INTO chapters (
                    node_id, chapter_number, name_arabic, name_simple,
                    name_complex, verses_count
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                ("CHAPTER:115", 115, "test", "test", "test", 1),
            )
            assert False, "Expected IntegrityError for invalid chapter_number"
        except sqlite3.IntegrityError:
            pass  # Expected

        print("✓ Check constraints working correctly")

    finally:
        conn.close()


def test_indexes_created():
    """Test that critical indexes are created."""
    conn = sqlite3.connect(":memory:")
    cursor = conn.cursor()

    try:
        # Create schema and indexes
        for schema_sql in ContentDatabaseSchema.get_all_schemas():
            cursor.execute(schema_sql)

        for index_sql in ContentDatabaseSchema.get_all_indexes():
            cursor.execute(index_sql)

        # Get all indexes
        cursor.execute(
            "SELECT name FROM sqlite_master WHERE type='index' ORDER BY name"
        )
        indexes = [row[0] for row in cursor.fetchall()]

        # Check critical indexes exist
        critical_indexes = [
            "idx_verses_verse_key",
            "idx_verses_juz",
            "idx_verses_page",
            "idx_morphology_lemma",
            "idx_morphology_root",
            "idx_words_verse_key",
        ]

        for index in critical_indexes:
            assert index in indexes, f"Critical index {index} not found"

        print(f"✓ All {len(critical_indexes)} critical indexes created successfully")

    finally:
        conn.close()


if __name__ == "__main__":
    print("Running Content Database Schema Validation Tests\n")
    print("=" * 60)

    try:
        test_schema_version()
        test_schema_creation()
        test_foreign_keys()
        test_check_constraints()
        test_indexes_created()

        print("=" * 60)
        print("\n✅ All tests passed!")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
