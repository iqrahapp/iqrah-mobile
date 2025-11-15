"""
Knowledge Graph Configuration

Manages configuration for knowledge graph generation.
Supports loading from YAML files and presets.
"""

import yaml
from pathlib import Path
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field, asdict
import logging


logger = logging.getLogger(__name__)


@dataclass
class EdgeConfig:
    """Configuration for a specific edge type."""

    enabled: bool = True
    params: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ScoringConfig:
    """Configuration for PageRank scoring."""

    enabled: bool = True
    alpha: float = 0.85
    max_iter: int = 50000
    personalize_foundational: bool = True
    personalize_influence: bool = False


@dataclass
class ExportConfig:
    """Configuration for graph export."""

    format: str = "cbor"  # cbor, graphml, both
    compression_level: int = 9
    include_scores: bool = True


@dataclass
class KnowledgeGraphConfig:
    """
    Complete configuration for knowledge graph generation.

    Attributes:
        name: Config name
        description: Config description
        chapters: Chapter range (e.g., "1-114", "1", "1-10")
        memorization: Memorization edge config
        tajweed: Tajweed edge config
        translation: Translation edge config
        grammar: Grammar edge config
        deep_understanding: Deep understanding edge config
        scoring: Scoring config
        export: Export config
        metadata: Additional metadata
    """

    name: str = "default"
    description: str = "Default configuration"

    # Chapter selection
    chapters: str = "1-114"  # Full Quran by default

    # Edge configurations
    memorization: EdgeConfig = field(default_factory=lambda: EdgeConfig(enabled=True))
    tajweed: EdgeConfig = field(default_factory=lambda: EdgeConfig(enabled=False))
    translation: EdgeConfig = field(default_factory=lambda: EdgeConfig(enabled=True))
    grammar: EdgeConfig = field(default_factory=lambda: EdgeConfig(enabled=True))
    deep_understanding: EdgeConfig = field(default_factory=lambda: EdgeConfig(enabled=False))

    # Scoring configuration
    scoring: ScoringConfig = field(default_factory=ScoringConfig)

    # Export configuration
    export: ExportConfig = field(default_factory=ExportConfig)

    # Additional metadata
    metadata: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """Convert config to dict."""
        return asdict(self)

    def to_yaml(self, path: str) -> None:
        """Save config to YAML file."""
        output_path = Path(path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_path, 'w') as f:
            yaml.dump(self.to_dict(), f, default_flow_style=False, sort_keys=False)

        logger.info(f"Config saved to {path}")

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "KnowledgeGraphConfig":
        """Create config from dict."""
        # Convert nested dicts to dataclass instances
        if "memorization" in data and isinstance(data["memorization"], dict):
            data["memorization"] = EdgeConfig(**data["memorization"])

        if "tajweed" in data and isinstance(data["tajweed"], dict):
            data["tajweed"] = EdgeConfig(**data["tajweed"])

        if "translation" in data and isinstance(data["translation"], dict):
            data["translation"] = EdgeConfig(**data["translation"])

        if "grammar" in data and isinstance(data["grammar"], dict):
            data["grammar"] = EdgeConfig(**data["grammar"])

        if "deep_understanding" in data and isinstance(data["deep_understanding"], dict):
            data["deep_understanding"] = EdgeConfig(**data["deep_understanding"])

        if "scoring" in data and isinstance(data["scoring"], dict):
            data["scoring"] = ScoringConfig(**data["scoring"])

        if "export" in data and isinstance(data["export"], dict):
            data["export"] = ExportConfig(**data["export"])

        return cls(**data)

    def parse_chapters(self) -> List[int]:
        """
        Parse chapter range string into list of chapter numbers.

        Examples:
            "1" -> [1]
            "1-3" -> [1, 2, 3]
            "1,5,10" -> [1, 5, 10]
            "1-3,5,10-12" -> [1, 2, 3, 5, 10, 11, 12]

        Returns:
            List of chapter numbers

        Raises:
            ValueError: If format is invalid
        """
        chapters = set()

        for part in self.chapters.split(','):
            part = part.strip()

            if '-' in part:
                # Range
                try:
                    start, end = part.split('-')
                    start, end = int(start), int(end)

                    if start < 1 or end > 114 or start > end:
                        raise ValueError(
                            f"Invalid chapter range: {part}. Must be 1-114 and start <= end."
                        )

                    chapters.update(range(start, end + 1))

                except ValueError as e:
                    raise ValueError(f"Invalid chapter range format: {part}") from e

            else:
                # Single chapter
                try:
                    chapter = int(part)

                    if chapter < 1 or chapter > 114:
                        raise ValueError(f"Invalid chapter number: {chapter}. Must be 1-114.")

                    chapters.add(chapter)

                except ValueError as e:
                    raise ValueError(f"Invalid chapter number: {part}") from e

        return sorted(list(chapters))


def load_config(path: str) -> KnowledgeGraphConfig:
    """
    Load configuration from YAML file.

    Args:
        path: Path to YAML config file

    Returns:
        KnowledgeGraphConfig instance

    Raises:
        FileNotFoundError: If file doesn't exist
        ValueError: If YAML is invalid
    """
    config_path = Path(path)

    if not config_path.exists():
        raise FileNotFoundError(f"Config file not found: {path}")

    logger.info(f"Loading config from {path}")

    with open(config_path, 'r') as f:
        data = yaml.safe_load(f)

    if not isinstance(data, dict):
        raise ValueError(f"Invalid config file: {path}")

    return KnowledgeGraphConfig.from_dict(data)


def load_preset(preset_name: str) -> KnowledgeGraphConfig:
    """
    Load a preset configuration.

    Args:
        preset_name: Name of preset (e.g., "basic", "full", "research")

    Returns:
        KnowledgeGraphConfig instance

    Raises:
        FileNotFoundError: If preset doesn't exist
    """
    # Find preset file
    # Try relative to this file first
    module_dir = Path(__file__).parent.parent.parent.parent  # Up to repo root
    preset_path = module_dir / "config" / "presets" / f"{preset_name}.yaml"

    if not preset_path.exists():
        # Try installed location
        import pkg_resources
        try:
            preset_path = Path(
                pkg_resources.resource_filename(
                    'iqrah',
                    f'../config/presets/{preset_name}.yaml'
                )
            )
        except Exception:
            pass

    if not preset_path.exists():
        raise FileNotFoundError(
            f"Preset '{preset_name}' not found. "
            f"Available presets: {', '.join(get_available_presets())}"
        )

    return load_config(str(preset_path))


def get_available_presets() -> List[str]:
    """
    Get list of available preset names.

    Returns:
        List of preset names
    """
    # Find presets directory
    module_dir = Path(__file__).parent.parent.parent.parent
    presets_dir = module_dir / "config" / "presets"

    if not presets_dir.exists():
        return []

    presets = []
    for yaml_file in presets_dir.glob("*.yaml"):
        presets.append(yaml_file.stem)

    return sorted(presets)
