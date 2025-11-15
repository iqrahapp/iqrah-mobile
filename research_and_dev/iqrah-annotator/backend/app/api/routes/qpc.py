"""API routes for querying QPC database."""

import sqlite3
from typing import List, Optional, Dict
from functools import lru_cache
from fastapi import APIRouter, Query, HTTPException, Body
from pydantic import BaseModel, Field
import os
import re

router = APIRouter(prefix="/api/qpc", tags=["qpc"])

# Data paths
QPC_DB_PATH = os.path.join(
    os.path.dirname(__file__),
    "../../../../data/qpc-hafs-tajweed.db"
)
SURAH_METADATA_DB_PATH = os.path.join(
    os.path.dirname(__file__),
    "../../../../data/quran-metadata-surah-name.db"
)


class QPCWord(BaseModel):
    """QPC word model."""
    id: int
    location: str  # "surah:ayah:word"
    surah: int
    ayah: int
    word: int
    text: str
    rules: List[str]  # Extracted from HTML tags


class AyahText(BaseModel):
    """Ayah text with tajweed markup."""
    surah: int
    ayah: int
    text: str  # HTML with tajweed tags
    rules: List[str]  # All rules present in this ayah


def extract_rules_from_text(text: str) -> List[str]:
    """Extract tajweed rules from HTML-tagged text."""
    # Find all <rule class=RULENAME> tags
    rules = re.findall(r'<rule class=([^>]+)>', text)
    return list(set(rules))  # Unique rules


@router.get("/words", response_model=List[QPCWord])
def get_qpc_words(
    rule: Optional[str] = Query(None, description="Filter by rule (e.g., 'ghunnah', 'qalqalah')"),
    surah: Optional[int] = Query(None, description="Filter by surah number"),
    limit: int = Query(100, ge=1, le=1000, description="Max results"),
    offset: int = Query(0, ge=0, description="Pagination offset")
):
    """
    Get words from QPC database with optional filters.

    Returns words that contain specific tajweed rules.
    """
    if not os.path.exists(QPC_DB_PATH):
        raise HTTPException(status_code=500, detail="QPC database not found")

    try:
        conn = sqlite3.connect(QPC_DB_PATH)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()

        # Build query
        query = "SELECT * FROM words WHERE 1=1"
        params = []

        if surah:
            query += " AND surah = ?"
            params.append(surah)

        if rule:
            # Filter by rule in HTML tags (escape special LIKE characters)
            escaped_rule = rule.replace('%', '\\%').replace('_', '\\_').replace('[', '\\[')
            query += " AND text LIKE ? ESCAPE '\\'"
            params.append(f'%<rule class={escaped_rule}>%')

        query += " ORDER BY id LIMIT ? OFFSET ?"
        params.extend([limit, offset])

        cursor.execute(query, params)
        rows = cursor.fetchall()

        words = []
        for row in rows:
            word_dict = dict(row)
            word_dict['rules'] = extract_rules_from_text(word_dict['text'])
            words.append(QPCWord(**word_dict))

        conn.close()
        return words

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Database error: {str(e)}")


class SurahInfo(BaseModel):
    """Surah metadata."""
    surah: int
    name_arabic: str
    name_english: str
    ayah_count: int
    word_count: int


@router.get("/surahs", response_model=List[SurahInfo])
def get_surahs(
    rule: Optional[str] = Query(None, description="Filter surahs containing this rule")
):
    """Get list of surahs with metadata, ayah and word counts."""
    if not os.path.exists(QPC_DB_PATH):
        raise HTTPException(status_code=500, detail="QPC database not found")
    if not os.path.exists(SURAH_METADATA_DB_PATH):
        raise HTTPException(status_code=500, detail="Surah metadata database not found")

    try:
        # Get word/ayah counts from QPC database
        qpc_conn = sqlite3.connect(QPC_DB_PATH)
        qpc_cursor = qpc_conn.cursor()

        if rule:
            # Escape special LIKE characters to prevent SQL injection
            escaped_rule = rule.replace('%', '\\%').replace('_', '\\_').replace('[', '\\[')
            query = """
                SELECT surah,
                       COUNT(DISTINCT ayah) as ayah_count,
                       COUNT(*) as word_count
                FROM words
                WHERE text LIKE ? ESCAPE '\\'
                GROUP BY surah
                ORDER BY surah
            """
            qpc_cursor.execute(query, (f'%<rule class={escaped_rule}>%',))
        else:
            query = """
                SELECT surah,
                       COUNT(DISTINCT ayah) as ayah_count,
                       COUNT(*) as word_count
                FROM words
                GROUP BY surah
                ORDER BY surah
            """
            qpc_cursor.execute(query)

        qpc_rows = qpc_cursor.fetchall()
        qpc_conn.close()

        # Get metadata from surah metadata database
        meta_conn = sqlite3.connect(SURAH_METADATA_DB_PATH)
        meta_cursor = meta_conn.cursor()
        meta_cursor.execute("SELECT id, name_simple, name_arabic FROM chapters ORDER BY id")
        meta_rows = {row[0]: {"name_english": row[1], "name_arabic": row[2]} for row in meta_cursor.fetchall()}
        meta_conn.close()

        # Combine data
        surahs = []
        for row in qpc_rows:
            surah_id = row[0]
            meta = meta_rows.get(surah_id, {"name_english": f"Surah {surah_id}", "name_arabic": ""})
            surahs.append(
                SurahInfo(
                    surah=surah_id,
                    name_arabic=meta["name_arabic"],
                    name_english=meta["name_english"],
                    ayah_count=row[1],
                    word_count=row[2]
                )
            )

        return surahs

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Database error: {str(e)}")


@router.get("/rules", response_model=List[str])
@lru_cache(maxsize=1)
def get_available_rules():
    """Get list of all available tajweed rules in the database (cached)."""
    if not os.path.exists(QPC_DB_PATH):
        raise HTTPException(status_code=500, detail="QPC database not found")

    try:
        conn = sqlite3.connect(QPC_DB_PATH)
        cursor = conn.cursor()

        # Get sample of texts with rules
        cursor.execute("SELECT text FROM words WHERE text LIKE '%<rule%' LIMIT 1000")
        rows = cursor.fetchall()

        all_rules = set()
        for row in rows:
            rules = extract_rules_from_text(row[0])
            all_rules.update(rules)

        conn.close()
        return sorted(list(all_rules))

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Database error: {str(e)}")


@router.get("/ayahs/{surah}", response_model=List[AyahText])
def get_ayahs(
    surah: int,
    from_ayah: Optional[int] = Query(None, description="Start from this ayah"),
    to_ayah: Optional[int] = Query(None, description="End at this ayah"),
):
    """Get ayah texts with tajweed markup for a surah."""
    if not os.path.exists(QPC_DB_PATH):
        raise HTTPException(status_code=500, detail="QPC database not found")

    try:
        conn = sqlite3.connect(QPC_DB_PATH)
        cursor = conn.cursor()

        # Build query
        query = """
            SELECT surah, ayah, GROUP_CONCAT(text, ' ') as ayah_text
            FROM words
            WHERE surah = ?
        """
        params = [surah]

        if from_ayah is not None:
            query += " AND ayah >= ?"
            params.append(from_ayah)

        if to_ayah is not None:
            query += " AND ayah <= ?"
            params.append(to_ayah)

        query += " GROUP BY surah, ayah ORDER BY ayah"

        cursor.execute(query, params)
        rows = cursor.fetchall()

        ayahs = []
        for row in rows:
            text = row[2]
            rules = extract_rules_from_text(text)
            ayahs.append(
                AyahText(
                    surah=row[0],
                    ayah=row[1],
                    text=text,
                    rules=rules
                )
            )

        conn.close()
        return ayahs

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Database error: {str(e)}")


@router.get("/words/{location}/anti-patterns", response_model=List[dict])
def get_word_anti_patterns(location: str):
    """Get applicable anti-patterns for a specific word based on its rules."""
    if not os.path.exists(QPC_DB_PATH):
        raise HTTPException(status_code=500, detail="QPC database not found")

    try:
        conn = sqlite3.connect(QPC_DB_PATH)
        cursor = conn.cursor()

        # Parse location: "surah:ayah:word"
        parts = location.split(':')
        if len(parts) != 3:
            raise HTTPException(status_code=400, detail="Invalid location format. Expected 'surah:ayah:word'")

        try:
            surah, ayah, word = map(int, parts)
        except ValueError:
            raise HTTPException(status_code=400, detail="Location parts must be integers")

        # Get word and extract rules
        cursor.execute(
            "SELECT text FROM words WHERE surah=? AND ayah=? AND word=?",
            (surah, ayah, word)
        )
        row = cursor.fetchone()
        conn.close()

        if not row:
            raise HTTPException(status_code=404, detail=f"Word not found at location {location}")

        rules = extract_rules_from_text(row[0])

        # Get anti-patterns for these rules
        from app.api.routes.taxonomy import TAXONOMY

        result = []
        for rule in rules:
            if rule in TAXONOMY.anti_patterns:
                result.extend([
                    {
                        "name": ap.name,
                        "display_name": ap.display_name,
                        "description": ap.description,
                        "rule": rule
                    }
                    for ap in TAXONOMY.anti_patterns[rule]
                ])

        return result

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Database error: {str(e)}")


class BatchLocationsRequest(BaseModel):
    """Request model for batch locations."""
    locations: List[str] = Field(..., max_length=100, description="List of locations (max 100)")


@router.post("/words/batch/anti-patterns", response_model=Dict[str, List[dict]])
def get_batch_word_anti_patterns(request: BatchLocationsRequest):
    """Get applicable anti-patterns for multiple words in batch.

    Returns a dictionary mapping word location to list of applicable anti-patterns.
    Words without anti-patterns will have an empty list.

    Limit: Maximum 100 locations per request.
    """
    if not os.path.exists(QPC_DB_PATH):
        raise HTTPException(status_code=500, detail="QPC database not found")

    locations = request.locations

    # Validate batch size
    if len(locations) > 100:
        raise HTTPException(
            status_code=400,
            detail="Batch size exceeds limit. Maximum 100 locations per request."
        )

    try:
        from app.api.routes.taxonomy import TAXONOMY

        conn = sqlite3.connect(QPC_DB_PATH)
        cursor = conn.cursor()

        result = {}

        # Parse all locations first
        parsed_locations = []
        for location in locations:
            parts = location.split(':')
            if len(parts) != 3:
                result[location] = []
                continue

            try:
                surah, ayah, word = map(int, parts)
                parsed_locations.append((location, surah, ayah, word))
            except ValueError:
                result[location] = []
                continue

        if not parsed_locations:
            return result

        # Build single query with IN clause
        placeholders = ','.join(['(?,?,?)'] * len(parsed_locations))
        query = f"SELECT surah, ayah, word, text FROM words WHERE (surah, ayah, word) IN ({placeholders})"

        # Flatten the parameters
        params = []
        for _, surah, ayah, word in parsed_locations:
            params.extend([surah, ayah, word])

        cursor.execute(query, params)
        rows = cursor.fetchall()

        # Create a mapping of (surah, ayah, word) -> text
        word_texts = {(row[0], row[1], row[2]): row[3] for row in rows}

        # Process each location
        for location, surah, ayah, word in parsed_locations:
            text = word_texts.get((surah, ayah, word))

            if not text:
                result[location] = []
                continue

            rules = extract_rules_from_text(text)

            # Get anti-patterns for these rules
            anti_patterns = []
            for rule in rules:
                if rule in TAXONOMY.anti_patterns:
                    anti_patterns.extend([
                        {
                            "name": ap.name,
                            "display_name": ap.display_name,
                            "description": ap.description,
                            "rule": rule
                        }
                        for ap in TAXONOMY.anti_patterns[rule]
                    ])

            result[location] = anti_patterns

        conn.close()
        return result

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Database error: {str(e)}")
