# iqrah/quran_api/utils.py
# src/quran_api/utils.py

import asyncio
from typing import List, Optional, Any
from tqdm.asyncio import tqdm
from .client import QuranAPIClient
from .models import Quran, Chapter


async def fetch_chapter_verses(
    client: QuranAPIClient,
    chapter: Chapter,
    language: str = "en",
    words: bool = True,
    translations: Optional[str] = None,
    audio: Optional[int] = None,
    tafsirs: list[str] | None = None,
    word_fields: list[str] | None = None,
    translation_fields: list[str] | None = None,
    fields: Optional[list[str]] = None,
    per_page: int = 50,
    progress_bar: Optional[tqdm] = None,
    **kwargs: Any,
) -> Chapter:

    all_verses = []
    current_page = 1
    while True:
        response = await client.get_verses_by_chapter(
            chapter_number=chapter.id,
            language=language,
            words=words,
            translations=translations,
            audio=audio,
            tafsirs=tafsirs,
            word_fields=word_fields,
            translation_fields=translation_fields,
            fields=fields,
            page=current_page,
            per_page=per_page,
            **kwargs,
        )
        all_verses.extend(response.verses)
        if progress_bar:
            progress_bar.update(len(response.verses))
        if response.pagination.next_page is None:
            break
        current_page = response.pagination.next_page

    chapter.verses = all_verses
    return chapter


async def fetch_chapter_by_id(
    client: QuranAPIClient,
    chapter_number: int,
    language: str = "en",
    words: bool = True,
    translations: Optional[str] = None,
    audio: Optional[int] = None,
    tafsirs: Optional[str] = None,
    per_page: int = 50,
    progress_bar: Optional[tqdm] = None,
    **kwargs: Any,
) -> Chapter:
    chapter_metadata = await client.get_chapter(id=chapter_number, language=language)

    if not chapter_metadata:
        raise ValueError(f"Chapter {chapter_number} not found")

    return await fetch_chapter_verses(
        client,
        chapter_metadata,
        language=language,
        words=words,
        translations=translations,
        audio=audio,
        tafsirs=tafsirs,
        per_page=per_page,
        progress_bar=progress_bar,
        **kwargs,
    )


async def fetch_quran(
    client: QuranAPIClient,
    language: str = "en",
    words: bool = True,
    translations: Optional[str] = None,
    audio: Optional[int] = None,
    tafsirs: Optional[str] = None,
    word_fields: list[str] | None = None,
    translation_fields: list[str] | None = None,
    fields: list[str] | None = None,
    per_page: int = 50,
    show_progress: bool = False,
    **kwargs: Any,
) -> Quran:
    # Fetch metadata to get total number of chapters and their verse counts
    metadata = await client.get_chapters(language=language)

    # Calculate total pages across all chapters
    total_verses = sum(chapter.verses_count for chapter in metadata)

    async def helper_fetch_verses(chapter: Chapter) -> Chapter:
        chapter_progress_bar = (
            tqdm(total=chapter.verses_count, desc=f"Chapter {chapter.id}", leave=False)
            if show_progress
            else None
        )

        result = await fetch_chapter_verses(
            client,
            chapter,
            language=language,
            words=words,
            translations=translations,
            audio=audio,
            tafsirs=tafsirs,
            word_fields=word_fields,
            translation_fields=translation_fields,
            fields=fields,
            per_page=per_page,
            progress_bar=chapter_progress_bar,
            **kwargs,
        )

        if chapter_progress_bar:
            chapter_progress_bar.close()

        return result

    # Create a list of tasks for all chapters
    tasks = [helper_fetch_verses(chapter) for chapter in metadata]

    # Use tqdm if show_progress is True
    if show_progress:
        overall_progress_bar = tqdm(total=total_verses, desc="Overall Progress")
        results = []
        for task in asyncio.as_completed(tasks):
            result = await task
            overall_progress_bar.update((result.verses_count))
            results.append(result)
        overall_progress_bar.close()
    else:
        results = await asyncio.gather(*tasks)

    # Sort results by chapter id
    results = sorted(results, key=lambda chapter: chapter.id)

    return Quran(chapters=results)
