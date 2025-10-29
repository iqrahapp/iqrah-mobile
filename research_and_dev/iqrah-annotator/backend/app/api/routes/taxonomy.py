"""API routes for annotation taxonomy (rules, anti-patterns, labels)."""

from typing import List, Dict
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
        ],
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
        ],
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
def get_taxonomy():
    """Get complete annotation taxonomy (rules, anti-patterns, region labels)."""
    return TAXONOMY


@router.get("/rules", response_model=List[RuleInfo])
def get_rules():
    """Get list of available tajweed rules for annotation."""
    return TAXONOMY.rules


@router.get("/anti-patterns/{rule}", response_model=List[AntiPattern])
def get_anti_patterns(rule: str):
    """Get anti-patterns for a specific rule."""
    return TAXONOMY.anti_patterns.get(rule, [])


@router.get("/region-labels/{rule}", response_model=List[RegionLabel])
def get_region_labels(rule: str):
    """Get region labels for a specific rule."""
    return TAXONOMY.region_labels.get(rule, [])
