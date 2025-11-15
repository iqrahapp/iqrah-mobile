# iqrah/quran_api/__init__.py
# Models are always available (no external deps)
from .models import (
    Chapter,
    Verse,
    Word,
    TranslationInfo,
    VersesResponse,
    Reciter,
    Quran,
    TranslationByWord,
    TranslationByVerse,
    TransliterationByWord,
)

# Client and utils require httpx (optional)
try:
    from .client import QuranAPIClient
    from .utils import fetch_quran
    __all__ = [
        "QuranAPIClient",
        "Chapter",
        "Verse",
        "Word",
        "TranslationInfo",
        "VersesResponse",
        "Reciter",
        "Quran",
        "fetch_quran",
        "TranslationByWord",
        "TranslationByVerse",
        "TransliterationByWord",
    ]
except ImportError:
    # httpx not available, only export models
    __all__ = [
        "Chapter",
        "Verse",
        "Word",
        "TranslationInfo",
        "VersesResponse",
        "Reciter",
        "Quran",
        "TranslationByWord",
        "TranslationByVerse",
        "TransliterationByWord",
    ]
