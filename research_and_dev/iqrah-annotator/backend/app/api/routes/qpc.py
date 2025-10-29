"""API routes for querying QPC database."""

import sqlite3
from typing import List, Optional
from fastapi import APIRouter, Query, HTTPException
from pydantic import BaseModel
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
            # Filter by rule in HTML tags
            query += f" AND text LIKE ?"
            params.append(f'%<rule class={rule}>%')

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
            query = """
                SELECT surah,
                       COUNT(DISTINCT ayah) as ayah_count,
                       COUNT(*) as word_count
                FROM words
                WHERE text LIKE ?
                GROUP BY surah
                ORDER BY surah
            """
            qpc_cursor.execute(query, (f'%<rule class={rule}>%',))
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
def get_available_rules():
    """Get list of all available tajweed rules in the database."""
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
