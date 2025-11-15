# iqrah/quran_api/client.py
# src/quran_api/client.py
import httpx
from typing import List, Optional, Any, Dict, Union
from functools import wraps
from diskcache import Cache
from loguru import logger
from tenacity import (
    retry,
    stop_after_attempt,
    wait_exponential,
    retry_if_exception_type,
)

from .models import Chapter, VersesResponse, Reciter, TranslationInfo

HTTP_TIMEOUT_EXCEPTIONS = Union[httpx.ConnectTimeout, httpx.ReadTimeout]

retry_decorator = retry(
    retry=retry_if_exception_type(HTTP_TIMEOUT_EXCEPTIONS),
    stop=stop_after_attempt(10),
    wait=wait_exponential(multiplier=1, min=4, max=30),
    reraise=True,
)


class QuranAPIClient:
    BASE_URL = "https://api.quran.com/api/v4"

    def __init__(self, cache_dir: Optional[str] = None):
        cache_dir = cache_dir or "./.cache"
        self.client = httpx.AsyncClient()
        self.cache = Cache(cache_dir)
        logger.debug("QuranAPIClient initialized with persistent caching")

    async def close(self):
        await self.client.aclose()
        self.cache.close()
        logger.debug("QuranAPIClient closed")

    @staticmethod
    def cache_key(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            # Convert args to a string representation
            args_str = ",".join(map(str, args[1:]))  # Skip self
            # Convert kwargs to a sorted string representation
            kwargs_str = ",".join(f"{k}={v}" for k, v in sorted(kwargs.items()))
            # Combine function name, args, and kwargs into a single string
            return f"{func.__name__}:{args_str}:{kwargs_str}"

        return wrapper

    def cached(func):
        @wraps(func)
        async def wrapper(self, *args, **kwargs):
            key = self.cache_key(func)(*args, **kwargs)

            # Check if data is in cache
            if key in self.cache:
                logger.debug(f"Cache hit: {key}")
                return self.cache[key]

            # If not in cache, call the function
            logger.debug(f"Cache miss: {key}")
            result = await func(self, *args, **kwargs)

            # Store result in cache
            self.cache[key] = result

            return result

        return wrapper

    @retry_decorator
    async def _get(
        self, url: str, params: Optional[Dict[str, Any]] = None, **kwargs
    ) -> httpx.Response:
        full_url = f"{self.BASE_URL}/{url}"
        log_params = {k: v for k, v in (params or {}).items() if v is not None}
        logger.debug(f"GET request to {full_url}")
        logger.debug(f"Query parameters: {log_params}")

        response = await self.client.get(full_url, params=params, **kwargs)
        response.raise_for_status()
        logger.debug(f"Response status code: {response.status_code}")
        return response

    @cached
    async def get_chapter(self, id: int, language: str = "en") -> Chapter:
        logger.debug(f"Fetching {id} chapter with language: {language}")
        response = await self._get(
            f"chapters/{id}",
            params={"language": language},
        )
        data = response.json()
        chapter = Chapter.model_validate(data["chapter"])
        logger.trace(f"Successfully fetched chapter {id}: {chapter.name_simple}")
        return chapter

    @cached
    async def get_chapters(self, language: str = "en") -> List[Chapter]:
        logger.debug(f"Fetching 'all' chapters if with language: {language}")
        response = await self._get(
            "chapters",
            params={"language": language},
        )
        data = response.json()
        chapters = [Chapter.model_validate(chapter) for chapter in data["chapters"]]
        logger.trace(f"Successfully fetched {len(chapters)} chapters")
        return chapters

    @cached
    async def get_verses_by_chapter(
        self,
        chapter_number: int,
        language: str = "en",
        words: bool = True,
        translations: Optional[str] = None,
        audio: Optional[int] = None,
        tafsirs: list[str] | None = None,
        word_fields: list[str] | None = None,
        translation_fields: list[str] | None = None,
        fields: Optional[list[str]] = None,
        page: int = 1,
        per_page: int = 10,
    ) -> VersesResponse:
        logger.debug(f"Fetching verses for chapter {chapter_number}")
        params = {
            "language": language,
            "words": str(words).lower(),
            "page": str(page),
            "per_page": str(per_page),
        }
        if tafsirs:
            params["tafsirs"] = ",".join(tafsirs)
        if translations:
            params["translations"] = ",".join(translations)
        if word_fields:
            params["word_fields"] = ",".join(word_fields)
        if translation_fields:
            params["translation_fields"] = ",".join(translation_fields)
        if fields:
            params["fields"] = ",".join(fields)
        if audio:
            params["audio"] = str(audio)
        response = await self._get(f"verses/by_chapter/{chapter_number}", params=params)
        verses_response = VersesResponse.model_validate(response.json())
        logger.trace(
            f"Successfully fetched {len(verses_response.verses)} verses for chapter {chapter_number}"
        )
        return verses_response

    @cached
    async def get_reciters(self, language: str = "en") -> List[Reciter]:
        logger.debug(f"Fetching reciters with language: {language}")
        response = await self._get(
            "resources/recitations", params={"language": language}
        )
        data = response.json()
        reciters = [Reciter.model_validate(reciter) for reciter in data["reciters"]]
        logger.trace(f"Successfully fetched {len(reciters)} reciters")
        return reciters

    @cached
    async def get_translations(
        self, language: Optional[str] = None
    ) -> List[TranslationInfo]:
        logger.debug(f"Fetching translations with language: {language}")
        params = {"language": language} if language else None
        response = await self._get("resources/translations", params=params)
        data = response.json()
        translations = [
            TranslationInfo.model_validate(translation)
            for translation in data["translations"]
        ]
        logger.trace(f"Successfully fetched {len(translations)} translations")
        return translations

    def clear_cache(self):
        self.cache.clear()
        logger.trace("Cache cleared")
