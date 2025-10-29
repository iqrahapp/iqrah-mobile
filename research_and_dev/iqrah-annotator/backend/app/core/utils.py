"""Utility functions for file handling and validation."""

import os
import tempfile
import shutil
from datetime import datetime
from pathlib import Path
from typing import Optional


def get_audio_dir() -> Path:
    """Get the audio directory from environment or use default."""
    audio_dir = os.getenv("AUDIO_DIR", "./data/audio")
    path = Path(audio_dir)
    path.mkdir(parents=True, exist_ok=True)
    return path


def generate_audio_path(extension: str = "wav") -> str:
    """
    Generate a unique audio file path with date-based organization.

    Returns:
        Relative path like "2025-10-28/123456789.wav"
    """
    today = datetime.utcnow().strftime("%Y-%m-%d")
    timestamp = int(datetime.utcnow().timestamp() * 1000)  # milliseconds
    filename = f"{timestamp}.{extension}"

    # Create date subdirectory
    date_dir = get_audio_dir() / today
    date_dir.mkdir(parents=True, exist_ok=True)

    # Return relative path
    return f"{today}/{filename}"


def save_audio_file(file_data: bytes, relative_path: str) -> Path:
    """
    Save audio file atomically (temp → fsync → rename).

    Args:
        file_data: Audio file bytes
        relative_path: Relative path (e.g., "2025-10-28/123.wav")

    Returns:
        Absolute path to saved file
    """
    audio_dir = get_audio_dir()
    target_path = audio_dir / relative_path
    target_path.parent.mkdir(parents=True, exist_ok=True)

    # Write to temp file first
    with tempfile.NamedTemporaryFile(
        mode="wb",
        delete=False,
        dir=target_path.parent,
        prefix=".tmp_",
        suffix=".wav"
    ) as tmp_file:
        tmp_file.write(file_data)
        tmp_file.flush()
        os.fsync(tmp_file.fileno())
        tmp_path = tmp_file.name

    # Atomic rename
    shutil.move(tmp_path, target_path)

    return target_path


def delete_audio_file(relative_path: str) -> bool:
    """
    Delete audio file, return True if deleted or didn't exist.

    Args:
        relative_path: Relative path to audio file

    Returns:
        True if file was deleted or didn't exist
    """
    audio_dir = get_audio_dir()
    file_path = audio_dir / relative_path

    if file_path.exists():
        try:
            file_path.unlink()
            return True
        except Exception as e:
            print(f"Warning: Failed to delete {file_path}: {e}")
            return False

    # File doesn't exist, consider success
    return True


def get_audio_file_path(relative_path: str) -> Optional[Path]:
    """
    Get absolute path to audio file if it exists.

    Args:
        relative_path: Relative path to audio file

    Returns:
        Absolute Path if exists, None otherwise
    """
    audio_dir = get_audio_dir()
    file_path = audio_dir / relative_path

    return file_path if file_path.exists() else None


def validate_region_boundaries(start_sec: float, end_sec: float, duration_sec: float) -> bool:
    """
    Validate region boundaries are within recording duration.

    Args:
        start_sec: Region start time
        end_sec: Region end time
        duration_sec: Recording duration

    Returns:
        True if valid

    Raises:
        ValueError: If boundaries are invalid
    """
    if start_sec < 0:
        raise ValueError("start_sec must be >= 0")

    if end_sec <= start_sec:
        raise ValueError("end_sec must be > start_sec")

    if end_sec > duration_sec:
        raise ValueError(f"end_sec ({end_sec}) exceeds duration ({duration_sec})")

    return True
