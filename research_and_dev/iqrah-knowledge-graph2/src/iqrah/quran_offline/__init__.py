"""Offline Quran data loader module."""

from .loader import OfflineQuranDataLoader, load_quran_offline
from ..quran_api.models import Quran

__all__ = [
    "OfflineQuranDataLoader",
    "load_quran_offline",
    "Quran",
]
