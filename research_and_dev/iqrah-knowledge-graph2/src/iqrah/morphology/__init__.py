# iqrah/morphology/__init__.py
from iqrah.morphology.enums import SegmentType, PartOfSpeech, GrammaticalFeature
from iqrah.morphology.models import QuranWordSegment
from iqrah.morphology.corpus import QuranMorphologyCorpus, QuranicArabicCorpus

# Alias for backward compatibility
MorphologyCorpus = QuranicArabicCorpus

__all__ = [
    "SegmentType",
    "PartOfSpeech",
    "GrammaticalFeature",
    "QuranWordSegment",
    "QuranMorphologyCorpus",
    "QuranicArabicCorpus",
    "MorphologyCorpus",
]
