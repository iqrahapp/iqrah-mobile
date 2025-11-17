# iqrah/morphology/corpus.py
from typing import Iterator, Union
from abc import ABC, abstractmethod
import csv
from .models import QuranWordSegment
from .enums import SegmentType, PartOfSpeech, GrammaticalFeature


class QuranMorphologyCorpus(ABC):
    """Abstract base class for Quranic morphological corpus implementations."""

    @abstractmethod
    def load_data(self, file_path: str) -> None:
        """Load corpus data from file."""
        pass

    @abstractmethod
    def get_word(
        self, chapter: int, verse: int, word_index: int
    ) -> list[QuranWordSegment]:
        """Get all segments for a specific word."""
        pass

    @abstractmethod
    def get_segment(
        self, chapter: int, verse: int, word_index: int, segment_index: int
    ) -> QuranWordSegment:
        """Get a specific segment."""
        pass

    @abstractmethod
    def get_number_of_segments(self) -> int:
        """Get total number of segments in corpus."""
        pass

    @abstractmethod
    def iterate_corpus(self) -> Iterator[QuranWordSegment]:
        """Iterate through all segments in corpus."""
        pass

    def iter_roots(self) -> Iterator[QuranWordSegment]:
        """Iterate through all root segments."""
        return (segment for segment in self if segment.segment_type == SegmentType.ROOT)

    def __iter__(self) -> Iterator[QuranWordSegment]:
        return self.iterate_corpus()

    def __getitem__(
        self,
        key: Union[tuple[int, int, int, int], slice, tuple[Union[int, slice], ...]],
    ) -> Union[QuranWordSegment, list[QuranWordSegment]]:
        match key:
            case (int(), int(), int(), int()):
                return self.get_segment(*key)
            case _:
                return self._multi_dimensional_slice(key)

    def __len__(self) -> int:
        return self.get_number_of_segments()

    def _multi_dimensional_slice(
        self, key: Union[slice, tuple[Union[int, slice], ...]]
    ) -> list[QuranWordSegment]:
        match key:
            case slice():
                return list(self.iterate_corpus())[key]
            case tuple():
                return self._process_tuple_slice(key)
            case _:
                raise IndexError("Invalid index type")

    def _process_tuple_slice(
        self, key: tuple[Union[int, slice], ...]
    ) -> list[QuranWordSegment]:
        result = list(self.iterate_corpus())
        num_dimensions = len(result[0].location)

        for dim, slice_or_index in enumerate(key):
            if dim >= num_dimensions:
                break

            match slice_or_index:
                case int():
                    result = [
                        seg for seg in result if seg.location[dim] == slice_or_index
                    ]
                    # If result is empty after filtering, return early
                    if not result:
                        return []
                case slice():
                    # If result is already empty, no need to process further
                    if not result:
                        return []

                    start = slice_or_index.start or 1
                    stop = slice_or_index.stop or max(
                        seg.location[dim] for seg in result
                    )
                    result = [
                        seg for seg in result if start <= seg.location[dim] <= stop
                    ]
                case _:
                    raise IndexError(f"Invalid index type for dimension {dim}")

        return result


class QuranicArabicCorpus(QuranMorphologyCorpus):
    """Implementation of QuranMorphologyCorpus for the Quranic Arabic Corpus."""

    def __init__(self):
        self.corpus: list[QuranWordSegment] = []
        self.segments = []  # Alias for compatibility with builder

    @classmethod
    def from_csv(cls, file_path: str) -> "QuranicArabicCorpus":
        """Create corpus from CSV file."""
        corpus = cls()
        corpus.load_data(file_path)
        return corpus

    def load_data(self, file_path: str) -> None:
        with open(file_path, "r", encoding="utf-8") as file:
            reader = csv.reader(file, delimiter="\t")
            next(reader)  # Skip header
            self.corpus = [self._create_segment(row) for row in reader]
            self.segments = self.corpus  # Alias for compatibility

    def _create_segment(self, row: list[str]) -> QuranWordSegment:
        location = tuple(map(int, row[0].split(":")))
        features = self._parse_features(row[3])
        return QuranWordSegment(
            location=location,
            text=row[1],
            segment_type=self._determine_segment_type(features),
            pos=self._determine_pos(row[2]),
            root=features.get("ROOT", ""),
            lemma=features.get("LEM", ""),
            grammatical_features=self._determine_grammatical_features(features),
        )

    @staticmethod
    def _parse_features(feature_string: str) -> dict[str, str]:
        return {
            item.split(":")[0]: item.split(":")[1] if ":" in item else None
            for item in feature_string.split("|")
        }

    def get_word(
        self, chapter: int, verse: int, word_index: int
    ) -> list[QuranWordSegment]:
        return [
            seg
            for seg in self.corpus
            if seg.location[:3] == (chapter, verse, word_index)
        ]

    def get_segment(
        self, chapter: int, verse: int, word_index: int, segment_index: int
    ) -> QuranWordSegment:
        return next(
            (
                seg
                for seg in self.corpus
                if seg.location == (chapter, verse, word_index, segment_index)
            ),
            None,
        )

    def get_number_of_segments(self) -> int:
        return len(self.corpus)

    def iterate_corpus(self) -> Iterator[QuranWordSegment]:
        yield from self.corpus

    @staticmethod
    def _determine_segment_type(features: dict[str, str]) -> SegmentType:
        if "PREF" in features:
            return SegmentType.PREFIX
        elif "SUFF" in features:
            return SegmentType.SUFFIX
        elif "ROOT" in features:
            return SegmentType.ROOT
        elif "PRON" in features:
            return SegmentType.PRONOUN
        elif "LEM" in features:
            return SegmentType.LEMMA
        elif "INL" in features:
            return SegmentType.INLAID
        return SegmentType.UNKNOWN

    @staticmethod
    def _determine_pos(pos_string: str) -> PartOfSpeech:
        try:
            return PartOfSpeech(pos_string)
        except ValueError:
            return PartOfSpeech.UNKNOWN

    @staticmethod
    def _determine_grammatical_features(
        features: dict[str, str]
    ) -> set[GrammaticalFeature]:
        grammatical_features = set()

        feature_mapping = {
            "1": GrammaticalFeature.FIRST_PERSON,
            "2": GrammaticalFeature.SECOND_PERSON,
            "3": GrammaticalFeature.THIRD_PERSON,
            "S": GrammaticalFeature.SINGULAR,
            "D": GrammaticalFeature.DUAL,
            "P": GrammaticalFeature.PLURAL,
            "M": GrammaticalFeature.MASCULINE,
            "F": GrammaticalFeature.FEMININE,
            "NOM": GrammaticalFeature.NOMINATIVE,
            "ACC": GrammaticalFeature.ACCUSATIVE,
            "GEN": GrammaticalFeature.GENITIVE,
            "IND": GrammaticalFeature.INDICATIVE,
            "SUBJ": GrammaticalFeature.SUBJUNCTIVE,
            "JUS": GrammaticalFeature.JUSSIVE,
            "IMP": GrammaticalFeature.IMPERATIVE,
            "PERF": GrammaticalFeature.PERFECT,
            "IMPF": GrammaticalFeature.IMPERFECT,
            "PASS": GrammaticalFeature.PASSIVE,
            "DEF": GrammaticalFeature.DEFINITE,
            "INDEF": GrammaticalFeature.INDEFINITE,
            "EMPH": GrammaticalFeature.EMPHATIC,
            "COND": GrammaticalFeature.CONDITIONAL,
            "INTG": GrammaticalFeature.INTERROGATIVE,
            "NEG": GrammaticalFeature.NEGATIVE,
        }

        for feature, gram_feature in feature_mapping.items():
            if feature in features:
                grammatical_features.add(gram_feature)

        if "PASS" not in features:
            grammatical_features.add(GrammaticalFeature.ACTIVE)

        return grammatical_features
