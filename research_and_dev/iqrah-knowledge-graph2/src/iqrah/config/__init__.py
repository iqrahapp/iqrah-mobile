"""
Configuration Module

Provides configuration management for knowledge graph generation.
"""

from .config import (
    KnowledgeGraphConfig,
    load_config,
    load_preset,
    get_available_presets,
)

__all__ = [
    "KnowledgeGraphConfig",
    "load_config",
    "load_preset",
    "get_available_presets",
]
