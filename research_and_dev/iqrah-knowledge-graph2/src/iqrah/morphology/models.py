# iqrah/morphology/models.py
from dataclasses import dataclass, field
from typing import Tuple, Set, FrozenSet
from .enums import SegmentType, PartOfSpeech, GrammaticalFeature


@dataclass(frozen=True, slots=True)
class QuranWordSegment:
    location: Tuple[int, int, int, int]  # (chapter, verse, word, segment)
    text: str
    segment_type: SegmentType
    pos: PartOfSpeech
    root: str
    lemma: str
    grammatical_features: FrozenSet[GrammaticalFeature] = field(
        default_factory=frozenset
    )

    def __post_init__(self):
        if not isinstance(self.grammatical_features, frozenset):
            object.__setattr__(
                self, "grammatical_features", frozenset(self.grammatical_features)
            )
