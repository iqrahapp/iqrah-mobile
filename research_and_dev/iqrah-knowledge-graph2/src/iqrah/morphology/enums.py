# iqrah/morphology/enums.py
from enum import StrEnum, auto


class SegmentType(StrEnum):
    PREFIX = auto()
    SUFFIX = auto()
    ROOT = auto()
    LEMMA = auto()
    PRONOUN = auto()
    INLAID = auto()
    UNKNOWN = auto()


class PartOfSpeech(StrEnum):
    NOUN = "N"
    VERB = "V"
    ADJECTIVE = "ADJ"
    ADVERB = "ADV"
    PARTICLE = auto()
    PRONOUN = "PRON"
    PREPOSITION = "PREP"
    CONJUNCTION = "CONJ"
    INTERJECTION = "INTERJ"
    UNKNOWN = "UNKNOWN"


class GrammaticalFeature(StrEnum):
    # Person
    FIRST_PERSON = auto()
    SECOND_PERSON = auto()
    THIRD_PERSON = auto()

    # Number
    SINGULAR = auto()
    DUAL = auto()
    PLURAL = auto()

    # Gender
    MASCULINE = auto()
    FEMININE = auto()

    # Case
    NOMINATIVE = auto()
    ACCUSATIVE = auto()
    GENITIVE = auto()

    # Mood
    INDICATIVE = auto()
    SUBJUNCTIVE = auto()
    JUSSIVE = auto()
    IMPERATIVE = auto()

    # Aspect
    PERFECT = auto()
    IMPERFECT = auto()

    # Voice
    ACTIVE = auto()
    PASSIVE = auto()

    # State
    DEFINITE = auto()
    INDEFINITE = auto()

    # Other
    EMPHATIC = auto()
    CONDITIONAL = auto()
    INTERROGATIVE = auto()
    NEGATIVE = auto()
