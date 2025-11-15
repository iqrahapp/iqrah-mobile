# tests/test_quran_models.py
import pytest
from iqrah.quran_api.models import (
    Quran,
    Chapter,
    TranslatedChapterName,
    TranslationByWord,
    TransliterationByWord,
    Verse,
    Word,
)


@pytest.fixture
def sample_quran():
    """Create a sample Quran with 3 chapters, each with 3 verses, each with 3 words"""

    def make_word(num: int) -> Word:
        return Word(
            position=num,
            text=f"word{num}",
            page_number=1,
            line_number=1,
            char_type_name="word",
            translation=TranslationByWord(text=f"translation{num}", language_name="en"),
            transliteration=TransliterationByWord(
                text=f"trans{num}", language_name="en"
            ),
        )

    def make_verse(num: int) -> Verse:
        return Verse(
            id=num,
            verse_number=num,
            verse_key=f"{num}:{num}",
            juz_number=1,
            hizb_number=1,
            rub_el_hizb_number=1,
            manzil_number=1,
            page_number=1,
            words=[make_word(i) for i in range(1, 4)],
        )

    def make_chapter(num: int) -> Chapter:
        return Chapter(
            id=num,
            revelation_place="mecca",
            revelation_order=num,
            bismillah_pre=True,
            name_simple=f"Chapter{num}",
            name_complex=f"Chapter{num}",
            name_arabic=f"Chapter{num}",
            verses_count=3,
            pages=[1],
            translated_name=TranslatedChapterName(
                language_name="en", name=f"Chapter {num}"
            ),
            verses=[make_verse(i) for i in range(1, 4)],
        )

    return Quran(chapters=[make_chapter(i) for i in range(1, 4)])


def test_single_chapter(sample_quran):
    # Get single chapter
    assert len(sample_quran[1]) == 1
    assert sample_quran[1][0].id == 1


def test_chapter_range(sample_quran):
    # Get range of chapters
    chapters = sample_quran[1:3]
    assert len(chapters) == 2
    assert [c.id for c in chapters] == [1, 2]


def test_chapter_verse(sample_quran):
    # Get specific verse from multiple chapters
    verses = sample_quran[1:3, 1]
    assert len(verses) == 2
    assert all(v.verse_number == 1 for v in verses)


def test_full_range(sample_quran):
    # Get all verses from multiple chapters
    verses = sample_quran[1:3, :]
    assert len(verses) == 6  # 2 chapters * 3 verses


def test_specific_words(sample_quran):
    # Get specific words from specific verses
    words = sample_quran[1, 1:3, 2]
    assert len(words) == 2
    assert all(w.position == 2 for w in words)


def test_all_words(sample_quran):
    # Get all words from all verses in specified chapters
    words = sample_quran[1:3, :, :]
    assert len(words) == 18  # 2 chapters * 3 verses * 3 words


def test_ellipsis(sample_quran):
    # Test using ... for full ranges
    words = sample_quran[1, ..., :]
    assert len(words) == 9  # 1 chapter * 3 verses * 3 words


def test_backward_compatibility(sample_quran):
    # Test that old string-based indexing still works
    assert sample_quran["1"].id == 1
    assert sample_quran["1:1"].verse_number == 1
    assert sample_quran["1:1:1"].position == 1


def test_invalid_index(sample_quran):
    with pytest.raises(KeyError):
        sample_quran[1:3, 1:3, 1:3, 1]  # Too many dimensions

    with pytest.raises(KeyError):
        sample_quran[0]  # Invalid chapter number (1-based indexing)

    with pytest.raises(KeyError):
        sample_quran[1, 0]  # Invalid verse number


def test_empty_slice(sample_quran):
    # Test empty slice returns full list
    assert len(sample_quran[:]) == len(sample_quran.chapters)


def test_negative_indices(sample_quran):
    with pytest.raises(KeyError):
        sample_quran[-1]  # Negative indices not supported


def test_zero_index(sample_quran):
    with pytest.raises(KeyError):
        sample_quran[0]  # Zero index not allowed (1-based indexing)


def test_out_of_bounds(sample_quran):
    with pytest.raises(IndexError):
        sample_quran[len(sample_quran.chapters) + 1]


def test_empty_chapter(sample_quran):
    # Add an empty chapter
    empty_chapter = Chapter(
        id=len(sample_quran.chapters) + 1,
        revelation_place="mecca",
        revelation_order=1,
        bismillah_pre=True,
        name_simple="Empty",
        name_complex="Empty",
        name_arabic="Empty",
        verses_count=0,
        pages=[1],
        translated_name=TranslatedChapterName(language_name="en", name="Empty Chapter"),
        verses=[],
    )
    sample_quran.chapters.append(empty_chapter)

    # Should handle empty chapters gracefully
    assert len(sample_quran[4:5, :]) == 0


def test_bounds_checking(sample_quran):
    # Test various out-of-bounds scenarios

    # Direct index access
    with pytest.raises(IndexError):
        sample_quran[len(sample_quran.chapters) + 1]

    with pytest.raises(KeyError):
        sample_quran[0]  # Below minimum (1-based indexing)

    # Slice access
    assert len(sample_quran[1:100]) == len(sample_quran.chapters)  # Overrunning slice
    assert len(sample_quran[100:200]) == 0  # Completely out of bounds slice

    # Nested access
    with pytest.raises(IndexError):
        sample_quran[len(sample_quran.chapters) + 1, 1]

    # Test that overrunning doesn't raise for nested slices
    result = sample_quran[1:3, 1:100, 1:100]
    assert len(result) > 0  # Should return valid results within bounds


def test_invalid_indices(sample_quran):
    # Test various invalid index scenarios

    with pytest.raises(KeyError):
        sample_quran[-1]  # Negative index

    with pytest.raises(KeyError):
        sample_quran[1:-1]  # Negative slice end

    with pytest.raises(KeyError):
        sample_quran[0:5]  # Zero start index

    with pytest.raises(KeyError):
        sample_quran["invalid"]  # Invalid string index

    with pytest.raises(KeyError):
        sample_quran[1.5]  # Float index


def test_empty_slices(sample_quran):
    # Test various empty slice scenarios
    assert len(sample_quran[2:1]) == 0  # Empty due to start > stop
    assert len(sample_quran[1:3, 2:1]) == 0  # Empty verse range
    assert len(sample_quran[1:3, 1:2, 2:1]) == 0  # Empty word range
