"""API routes for annotation taxonomy (rules, anti-patterns, labels)."""

from typing import List, Dict
from functools import lru_cache
from fastapi import APIRouter
from pydantic import BaseModel

router = APIRouter(prefix="/api/taxonomy", tags=["taxonomy"])


class RuleInfo(BaseModel):
    """Tajweed rule information."""
    name: str
    display_name: str
    description: str


class AntiPattern(BaseModel):
    """Anti-pattern definition."""
    name: str
    display_name: str
    description: str


class RegionLabel(BaseModel):
    """Region label definition."""
    name: str
    display_name: str
    description: str


class TaxonomyData(BaseModel):
    """Complete taxonomy data."""
    rules: List[RuleInfo]
    anti_patterns: Dict[str, List[AntiPattern]]  # Keyed by rule name
    region_labels: Dict[str, List[RegionLabel]]  # Keyed by rule name


# Taxonomy data based on v0.1 spec
TAXONOMY = TaxonomyData(
    rules=[
        RuleInfo(
            name="ghunnah",
            display_name="Ghunnah (غُنّة)",
            description="Nasal sound produced for 2 counts"
        ),
        RuleInfo(
            name="qalaqah",
            display_name="Qalqalah (قلقلة)",
            description="Echo/vibration sound on certain letters"
        ),
        RuleInfo(
            name="general",
            display_name="General Practice",
            description="General recitation practice without specific rule focus"
        ),
    ],
    anti_patterns={
        # Ghunnah (غنة) - Nasalization (2 counts)
        "ghunnah": [
            AntiPattern(
                name="weak-ghunnah",
                display_name="Weak Ghunnah",
                description="Ghunnah sound is too weak or less than 2 counts"
            ),
            AntiPattern(
                name="no-ghunnah",
                display_name="No Ghunnah",
                description="Ghunnah sound is completely missing"
            ),
            AntiPattern(
                name="excessive-ghunnah",
                display_name="Excessive Ghunnah",
                description="Ghunnah sound is too long (more than 2 counts)"
            ),
        ],

        # Qalqalah (قلقلة) - Echo/Vibration
        "qalaqah": [
            AntiPattern(
                name="no-qalaqah",
                display_name="No Qalqalah",
                description="Qalqalah sound is missing"
            ),
            AntiPattern(
                name="weak-qalaqah",
                display_name="Weak Qalqalah",
                description="Qalqalah sound is too weak or unclear"
            ),
            AntiPattern(
                name="excessive-qalaqah",
                display_name="Excessive Qalqalah",
                description="Qalqalah sound is exaggerated"
            ),
        ],

        # Madda Normal (مد طبيعي) - Natural elongation (2 counts)
        "madda_normal": [
            AntiPattern(
                name="short-madda-normal",
                display_name="Short Normal Madda",
                description="Madda duration is too short (less than 2 counts)"
            ),
            AntiPattern(
                name="long-madda-normal",
                display_name="Long Normal Madda",
                description="Madda duration is too long (more than 2 counts)"
            ),
            AntiPattern(
                name="no-madda-normal",
                display_name="Missing Normal Madda",
                description="Madda elongation is completely missing"
            ),
        ],

        # Madda Permissible (مد جائز منفصل) - Permissible elongation (2-5 counts)
        "madda_permissible": [
            AntiPattern(
                name="short-madda-permissible",
                display_name="Short Permissible Madda",
                description="Permissible madda less than 2 counts"
            ),
            AntiPattern(
                name="excessive-madda-permissible",
                display_name="Excessive Permissible Madda",
                description="Permissible madda more than 5 counts"
            ),
        ],

        # Madda Obligatory Connected (مد واجب متصل) - 4-5 counts
        "madda_obligatory_mottasel": [
            AntiPattern(
                name="short-madda-mottasel",
                display_name="Short Connected Obligatory Madda",
                description="Connected obligatory madda less than 4 counts"
            ),
            AntiPattern(
                name="excessive-madda-mottasel",
                display_name="Excessive Connected Obligatory Madda",
                description="Connected obligatory madda more than 5 counts"
            ),
        ],

        # Madda Obligatory Separated (مد واجب منفصل) - 4-5 counts
        "madda_obligatory_monfasel": [
            AntiPattern(
                name="short-madda-monfasel",
                display_name="Short Separated Obligatory Madda",
                description="Separated obligatory madda less than 4 counts"
            ),
            AntiPattern(
                name="excessive-madda-monfasel",
                display_name="Excessive Separated Obligatory Madda",
                description="Separated obligatory madda more than 5 counts"
            ),
        ],

        # Madda Necessary (مد لازم) - 6 counts
        "madda_necessary": [
            AntiPattern(
                name="short-madda-necessary",
                display_name="Short Necessary Madda",
                description="Necessary madda less than 6 counts"
            ),
            AntiPattern(
                name="excessive-madda-necessary",
                display_name="Excessive Necessary Madda",
                description="Necessary madda more than 6 counts"
            ),
        ],

        # Idghaam with Ghunnah (إدغام بغنة) - Merging with nasalization
        "idgham_ghunnah": [
            AntiPattern(
                name="incomplete-idgham-ghunnah",
                display_name="Incomplete Merging with Ghunnah",
                description="Letters not fully merged or ghunnah missing"
            ),
            AntiPattern(
                name="weak-ghunnah-in-idgham",
                display_name="Weak Ghunnah in Merging",
                description="Ghunnah sound too weak during merging"
            ),
            AntiPattern(
                name="no-merging",
                display_name="No Merging",
                description="Letters pronounced separately instead of merged"
            ),
        ],

        # Idghaam without Ghunnah (إدغام بغير غنة) - Merging without nasalization
        "idgham_wo_ghunnah": [
            AntiPattern(
                name="incomplete-idgham",
                display_name="Incomplete Merging",
                description="Letters not fully merged"
            ),
            AntiPattern(
                name="added-ghunnah",
                display_name="Incorrect Ghunnah Added",
                description="Ghunnah incorrectly added when not required"
            ),
        ],

        # Idghaam Mutajanisayn (إدغام متجانسين) - Merging similar letters
        "idgham_mutajanisayn": [
            AntiPattern(
                name="incomplete-mutajanisayn",
                display_name="Incomplete Similar Letter Merging",
                description="Similar letters not properly merged"
            ),
            AntiPattern(
                name="separation",
                display_name="Letter Separation",
                description="Letters pronounced separately"
            ),
        ],

        # Idghaam Shafawi (إدغام شفوي) - Labial merging
        "idgham_shafawi": [
            AntiPattern(
                name="incomplete-shafawi",
                display_name="Incomplete Labial Merging",
                description="Labial letters not properly merged"
            ),
            AntiPattern(
                name="weak-labial-ghunnah",
                display_name="Weak Labial Ghunnah",
                description="Ghunnah too weak in labial merging"
            ),
        ],

        # Ikhfa (إخفاء) - Concealment with ghunnah
        "ikhafa": [
            AntiPattern(
                name="no-ikhafa",
                display_name="No Concealment",
                description="Letter pronounced clearly instead of concealed"
            ),
            AntiPattern(
                name="complete-merging",
                display_name="Complete Merging Instead of Concealment",
                description="Letter completely merged instead of concealed"
            ),
            AntiPattern(
                name="weak-ikhafa-ghunnah",
                display_name="Weak Ghunnah in Concealment",
                description="Ghunnah too weak during concealment"
            ),
        ],

        # Ikhfa Shafawi (إخفاء شفوي) - Labial concealment
        "ikhafa_shafawi": [
            AntiPattern(
                name="no-labial-ikhafa",
                display_name="No Labial Concealment",
                description="Labial letter not concealed"
            ),
            AntiPattern(
                name="weak-labial-ikhafa-ghunnah",
                display_name="Weak Labial Concealment Ghunnah",
                description="Ghunnah too weak in labial concealment"
            ),
        ],

        # Iqlab (إقلاب) - Conversion to meem with ghunnah
        "iqlab": [
            AntiPattern(
                name="no-conversion",
                display_name="No Conversion",
                description="Noon not converted to meem"
            ),
            AntiPattern(
                name="weak-iqlab-ghunnah",
                display_name="Weak Ghunnah in Conversion",
                description="Ghunnah too weak after conversion"
            ),
            AntiPattern(
                name="incomplete-iqlab",
                display_name="Incomplete Conversion",
                description="Conversion not fully pronounced as meem"
            ),
        ],

        # Laam Shamsiyah (لام شمسية) - Sun letter (laam assimilated)
        "laam_shamsiyah": [
            AntiPattern(
                name="pronounced-laam",
                display_name="Laam Pronounced",
                description="Laam incorrectly pronounced with sun letter"
            ),
            AntiPattern(
                name="incomplete-assimilation",
                display_name="Incomplete Assimilation",
                description="Laam not fully assimilated into sun letter"
            ),
        ],

        # Ham Wasl (همزة وصل) - Connecting hamza (silent when continuing)
        "ham_wasl": [
            AntiPattern(
                name="pronounced-hamza",
                display_name="Hamza Pronounced",
                description="Connecting hamza incorrectly pronounced when continuing"
            ),
            AntiPattern(
                name="missing-hamza-start",
                display_name="Missing Hamza at Start",
                description="Hamza not pronounced when starting"
            ),
        ],

        # Silent letters (حروف ساكنة)
        "slnt": [
            AntiPattern(
                name="pronounced-silent",
                display_name="Silent Letter Pronounced",
                description="Silent letter incorrectly pronounced"
            ),
        ],

        # General violations
        "general": [
            AntiPattern(
                name="general-violation",
                display_name="General Violation",
                description="General tajweed violation"
            ),
        ],
    },
    region_labels={
        "ghunnah": [
            RegionLabel(
                name="weak-ghunnah-onset",
                display_name="Weak Ghunnah Onset",
                description="Beginning portion of weak ghunnah"
            ),
            RegionLabel(
                name="weak-ghunnah-sustain",
                display_name="Weak Ghunnah Sustain",
                description="Sustain portion of weak ghunnah"
            ),
            RegionLabel(
                name="no-ghunnah",
                display_name="Missing Ghunnah",
                description="Region where ghunnah is completely absent"
            ),
        ],
        "qalaqah": [
            RegionLabel(
                name="no-qalaqah",
                display_name="Missing Qalqalah",
                description="Region where qalqalah is missing"
            ),
            RegionLabel(
                name="burst-misaligned",
                display_name="Burst Misaligned",
                description="Qalqalah burst timing is incorrect"
            ),
            RegionLabel(
                name="weak-qalaqah",
                display_name="Weak Qalqalah",
                description="Qalqalah sound is too weak"
            ),
        ],
        "general": [
            RegionLabel(
                name="violation",
                display_name="Violation",
                description="General tajweed violation region"
            ),
        ],
    },
)


@router.get("/", response_model=TaxonomyData)
@lru_cache(maxsize=1)
def get_taxonomy():
    """Get complete annotation taxonomy (rules, anti-patterns, region labels) - cached."""
    return TAXONOMY


@router.get("/rules", response_model=List[RuleInfo])
@lru_cache(maxsize=1)
def get_rules():
    """Get list of available tajweed rules for annotation - cached."""
    return TAXONOMY.rules


@router.get("/anti-patterns/{rule}", response_model=List[AntiPattern])
@lru_cache(maxsize=128)
def get_anti_patterns(rule: str):
    """Get anti-patterns for a specific rule - cached."""
    return TAXONOMY.anti_patterns.get(rule, [])


@router.get("/region-labels/{rule}", response_model=List[RegionLabel])
@lru_cache(maxsize=128)
def get_region_labels(rule: str):
    """Get region labels for a specific rule - cached."""
    return TAXONOMY.region_labels.get(rule, [])
