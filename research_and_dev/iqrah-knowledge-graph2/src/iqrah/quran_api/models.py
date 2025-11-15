# iqrah/quran_api/models.py
from typing import Any
from pydantic import BaseModel, Field
from bs4 import BeautifulSoup


class TranslatedName(BaseModel):
    language_name: str = Field(..., alias="language_name")
    name: str


class Chapter(BaseModel):
    id: int
    revelation_place: str = Field(..., alias="revelation_place")
    revelation_order: int = Field(..., alias="revelation_order")
    bismillah_pre: bool = Field(..., alias="bismillah_pre")
    name_simple: str = Field(..., alias="name_simple")
    name_complex: str = Field(..., alias="name_complex")
    name_arabic: str = Field(..., alias="name_arabic")
    verses_count: int = Field(..., alias="verses_count")
    pages: list[int]
    translated_name: TranslatedName = Field(..., alias="translated_name")


class Reciter(BaseModel):
    id: int
    reciter_name: str = Field(..., alias="reciter_name")
    style: str | None = None
    translated_name: TranslatedName = Field(..., alias="translated_name")


class TranslationInfo(BaseModel):
    id: int
    name: str
    author_name: str = Field(..., alias="author_name")
    slug: str | None = None
    language_name: str = Field(..., alias="language_name")
    translated_name: TranslatedName = Field(..., alias="translated_name")


class Pagination(BaseModel):
    per_page: int = Field(..., alias="per_page")
    current_page: int = Field(..., alias="current_page")
    next_page: int | None = Field(None, alias="next_page")
    total_pages: int = Field(..., alias="total_pages")
    total_records: int = Field(..., alias="total_records")


class TranslationByWord(BaseModel):
    text: str
    language_name: str = Field(..., alias="language_name")


class TranslationByVerse(BaseModel):
    id: int
    resource_id: int
    text: str


class TransliterationByWord(BaseModel):
    text: str | None = None
    language_name: str | None = Field(None, alias="language_name")


class Segment(BaseModel):
    start_word_index: int
    stop_word_index: int
    start_timestamp_ms: int
    stop_timestamp_ms: int


class Audio(BaseModel):
    url: str
    duration: int | None = None
    format: str | None = None
    segments: list[Segment]


class Word(BaseModel):
    id: int | None = None
    position: int
    text: str
    text_uthmani: str | None = Field(None, alias="text_uthmani")
    text_imlaei: str | None = Field(None, alias="text_imlaei")
    verse_key: str | None = Field(None, alias="verse_key")
    page_number: int = Field(..., alias="page_number")
    line_number: int = Field(..., alias="line_number")
    audio_url: str | None = Field(None, alias="audio_url")
    location: str | None = None
    char_type_name: str = Field(..., alias="char_type_name")
    code_v1: str | None = Field(None, alias="code_v1")
    code_v2: str | None = Field(None, alias="code_v2")
    translation: "TranslationByWord"
    transliteration: "TransliterationByWord"
    v1_page: int | None = Field(None, alias="v1_page")
    v2_page: int | None = Field(None, alias="v2_page")

    def _any_text(self):
        text = [self.text, self.text_uthmani, self.text_imlaei]
        return next((t for t in text if t is not None), None)

    def get_letters_count(self) -> int:
        text = self._any_text()
        if text is None:
            raise ValueError("No text found")
        return len(text)

    def is_end_word(self) -> bool:
        return self.char_type_name == "end"


def strip_html_tags(html: str) -> str:
    """
    Removes HTML tags and keeps only the text content from a given HTML string.

    Args:
        html (str): Input string containing HTML tags.

    Returns:
        str: String with HTML tags removed.
    """
    soup = BeautifulSoup(html, "html.parser")
    return soup.get_text(strip=True)


class Verse(BaseModel):
    id: int
    chapter_id: int | None = Field(None, alias="chapter_id")
    verse_number: int = Field(..., alias="verse_number")
    verse_key: str = Field(..., alias="verse_key")
    verse_index: int | None = Field(None, alias="verse_index")
    text_uthmani: str | None = Field(None, alias="text_uthmani")
    text_uthmani_simple: str | None = Field(None, alias="text_uthmani_simple")
    text_imlaei: str | None = Field(None, alias="text_imlaei")
    text_imlaei_simple: str | None = Field(None, alias="text_imlaei_simple")
    text_indopak: str | None = Field(None, alias="text_indopak")
    text_uthmani_tajweed: str | None = Field(None, alias="text_uthmani_tajweed")
    juz_number: int = Field(..., alias="juz_number")
    hizb_number: int = Field(..., alias="hizb_number")
    rub_el_hizb_number: int = Field(..., alias="rub_el_hizb_number")
    manzil_number: int = Field(..., alias="manzil_number")
    sajdah_type: str | None = Field(None, alias="sajdah_type")
    sajdah_number: int | None = Field(None, alias="sajdah_number")
    page_number: int = Field(..., alias="page_number")
    words: list[Word]
    translations: list[TranslationByVerse] | None = None

    def _any_text(self):
        text = [
            self.text_uthmani,
            self.text_uthmani_simple,
            self.text_imlaei,
            self.text_imlaei_simple,
            self.text_indopak,
        ]
        text = [t for t in text if t is not None]
        if not text and self.text_uthmani_tajweed is not None:
            text = [strip_html_tags(self.text_uthmani_tajweed)]

        return next(iter(text), None)

    def get_letters_count(self) -> int:
        text = self._any_text()
        if text is None:
            raise ValueError("No text found")
        return len(text)

    def get_words_count(self) -> int:
        return len(self.words) + (-1 if self.words[-1].is_end_word() else 0)

    @property
    def chapter(self) -> int:
        return self.chapter_id or int(self.verse_key.split(":")[0])


class VersesResponse(BaseModel):
    verses: list[Verse]
    pagination: Pagination


class TranslatedChapterName(BaseModel):
    language_name: str
    name: str


class Chapter(BaseModel):
    id: int
    revelation_place: str
    revelation_order: int
    bismillah_pre: bool
    name_simple: str
    name_complex: str
    name_arabic: str
    verses_count: int
    pages: list[int]
    translated_name: TranslatedChapterName
    verses: list[Verse] | None = None  # to be loaded manually


class WordIndex(BaseModel):
    chapter: int
    verse: int
    word: int

    @staticmethod
    def from_str(index: str) -> "WordIndex":
        indexes = index.split(":")
        if len(indexes) != 3:
            raise ValueError(
                f"Invalid word index: {index}. Should be 3 parts separated by ':'. Eg. 1:2:3:1"
            )
        try:
            indexes = [int(i) for i in indexes]
        except ValueError:
            raise ValueError(
                f"Invalid word index: {index}. Should be composed of 3 integers separated by ':'"
            )
        return WordIndex(*indexes)


class Quran(BaseModel):
    chapters: list[Chapter]

    def __getitem__(self, key) -> Any:
        # Handle string-based access (backward compatibility)
        if isinstance(key, str):
            return self._legacy_getitem(key)

        # Handle new slice-based access
        if not isinstance(key, tuple):
            key = (key,)

        # Ensure we don't have too many dimensions
        if len(key) > 3:
            raise KeyError("Maximum 3 dimensions allowed (chapter:verse:word)")

        # Convert all indices to proper slices and adjust for 1-based indexing
        slices = []
        for k in key:
            if isinstance(k, int):
                # Handle out of bounds for direct integer access
                if k < 1:
                    raise KeyError(f"Index {k} is invalid: indices start at 1")
                if k > len(self.chapters):  # Add bounds checking
                    raise IndexError(
                        f"Chapter index {k} out of range (1 to {len(self.chapters)})"
                    )
                slices.append(slice(k - 1, k))
            elif isinstance(k, slice):
                # Adjust for 1-based indexing
                start = (k.start - 1) if k.start is not None else None
                stop = k.stop if k.stop is not None else None

                # Validate start/stop bounds if provided
                if start is not None and (start < 0):
                    raise KeyError("Negative indices not supported")
                if stop is not None:
                    if stop <= 0:
                        raise KeyError("Invalid stop index")
                    stop = stop - 1  # Make stop exclusive

                slices.append(slice(start, stop, k.step))
            elif k == ...:
                # Handle Ellipsis (...) as full range
                slices.append(slice(None))
            else:
                raise KeyError(f"Invalid index type: {type(k)}")

        # Get chapters
        if not slices:
            return self.chapters

        # Handle potential IndexError from slice
        try:
            chapters = self.chapters[slices[0]]
        except IndexError as e:
            raise IndexError(
                f"Chapter index out of range (1 to {len(self.chapters)})"
            ) from e

        if not isinstance(chapters, list):
            chapters = [chapters]

        # Handle single chapter request
        if len(slices) == 1:
            return chapters

        # Get verses
        result = []
        for chapter in chapters:
            if not chapter.verses:
                continue

            try:
                verses = chapter.verses[slices[1]]
            except IndexError:
                continue  # Skip verses out of range for this chapter

            if not isinstance(verses, list):
                verses = [verses]

            # Handle verse-level request
            if len(slices) == 2:
                result.extend(verses)
                continue

            # Get words
            for verse in verses:
                if not verse.words:
                    continue

                try:
                    words = verse.words[slices[2]]
                except IndexError:
                    continue  # Skip words out of range for this verse

                if not isinstance(words, list):
                    words = [words]
                result.extend(words)

        return result

    def _legacy_getitem(self, indexes: str) -> Any:
        """Original string-based indexing for backward compatibility"""
        indexes = indexes.split(":")
        if len(indexes) > 3:
            raise KeyError(
                f"Invalid index: {indexes}, should be 1 to 3 parts separated by ':'"
            )
        try:
            indexes = [int(i) for i in indexes]
        except ValueError:
            raise KeyError(
                f"Invalid index: {indexes}, should be composed of 1 to 3 integers separated by ':'"
            )

        if not (1 <= indexes[0] <= len(self.chapters)):
            raise KeyError(
                f"Invalid chapter index: {indexes[0]}. Should be between 1 and {len(self.chapters)}"
            )

        chapter = self.chapters[indexes[0] - 1]
        if len(indexes) == 1:
            return chapter

        if not (1 <= indexes[1] <= len(chapter.verses)):
            raise KeyError(
                f"Invalid verse index: {indexes[1]}. For chapter {chapter.id}, Should be between 1 and {len(chapter.verses)}"
            )
        verse = chapter.verses[indexes[1] - 1]
        if len(indexes) == 2:
            return verse

        if not (1 <= indexes[2] <= len(verse.words)):
            raise KeyError(
                f"Invalid word index: {indexes[2]}. For verse {verse.verse_key}, Should be between 1 and {len(verse.words)}"
            )
        word = verse.words[indexes[2] - 1]

        return word

    def get_word(self, chapter: int, verse: int, word_index: int) -> Word:
        return self[f"{chapter}:{verse}:{word_index}"]

    def get_verse(self, chapter: int, verse: int) -> Verse:
        return self[f"{chapter}:{verse}"]

    def get_chapter(self, chapter: int) -> Chapter:
        return self[f"{chapter}"]

    def total_verses(self) -> int:
        return sum(len(chapter.verses) for chapter in self.chapters)

    def chapter_names(self) -> dict[int, str]:
        return {
            i + 1: chapter.verses[0].verse_key.split(":")[0]
            for i, chapter in enumerate(self.chapters)
        }

    def search_text(self, query: str) -> list[Verse]:
        results = []
        for chapter in self.chapters:
            for verse in chapter.verses:
                if query.lower() in verse.text_uthmani.lower():
                    results.append(verse)
        return results
