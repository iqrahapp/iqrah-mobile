# iqrah/__init__.py
"""Iqrah - A library for Quranic analysis and knowledge graph generation."""

import iqrah.morphology
import iqrah.graph

# Make quran_api optional (requires httpx)
try:
    import iqrah.quran_api
    __all__ = ["morphology", "quran_api", "graph", "quran_offline"]
except ImportError:
    __all__ = ["morphology", "graph", "quran_offline"]

# Import offline loader (no external dependencies)
import iqrah.quran_offline

__version__ = "0.1.1"
