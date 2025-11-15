"""
Iqrah Content Database Module

This module provides functionality for storing and retrieving Quranic content
in a normalized SQLite database, separate from the graph structure.

Key components:
- schema: Database schema definitions
- builder: Build content database from offline JSON/CSV data
- database: Query interface for fast content lookups
"""

from .schema import ContentDatabaseSchema
from .builder import ContentDatabaseBuilder
from .database import ContentDatabase

__all__ = [
    "ContentDatabaseSchema",
    "ContentDatabaseBuilder",
    "ContentDatabase",
]
