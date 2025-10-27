"""
Tajweed Data Loader
===================

Load Arabic words with Tajweed markup from qpc-hafs-tajweed.json
"""

import json
from pathlib import Path
from typing import List, Dict

_tajweed_data = None


def load_tajweed_words(data_path: str = None) -> Dict:
    """
    Load Tajweed data from JSON.

    Args:
        data_path: Path to qpc-hafs-tajweed.json

    Returns:
        Dictionary mapping location (e.g., "1:1:1") to word data
    """
    global _tajweed_data

    if _tajweed_data is not None:
        return _tajweed_data

    if data_path is None:
        data_path = Path(__file__).parent.parent.parent.parent / "data" / "qpc-hafs-tajweed.json"

    with open(data_path, 'r', encoding='utf-8') as f:
        _tajweed_data = json.load(f)

    return _tajweed_data


def get_ayah_words(surah: int, ayah: int) -> List[Dict]:
    """
    Get all words for an ayah with Tajweed markup.

    Args:
        surah: Surah number
        ayah: Ayah number

    Returns:
        List of word dictionaries with Tajweed markup
    """
    data = load_tajweed_words()

    words = []
    word_num = 1

    while True:
        key = f"{surah}:{ayah}:{word_num}"
        if key not in data:
            break

        words.append(data[key])
        word_num += 1

    return words


def parse_tajweed_html(text: str) -> List[Dict]:
    """
    Parse Tajweed HTML markup into segments.

    Example:
        Input: "<rule class=ham_wasl>ٱ</rule>للَّهِ"
        Output: [
            {"text": "ٱ", "class": "ham_wasl"},
            {"text": "للَّهِ", "class": None}
        ]

    Args:
        text: Text with Tajweed HTML markup

    Returns:
        List of text segments with their Tajweed class
    """
    import re

    segments = []
    pattern = r'<rule class=([^>]+)>([^<]+)</rule>'

    last_end = 0

    for match in re.finditer(pattern, text):
        # Add text before the match
        if match.start() > last_end:
            plain_text = text[last_end:match.start()]
            if plain_text:
                segments.append({"text": plain_text, "class": None})

        # Add the matched Tajweed segment
        tajweed_class = match.group(1)
        tajweed_text = match.group(2)
        segments.append({"text": tajweed_text, "class": tajweed_class})

        last_end = match.end()

    # Add remaining text
    if last_end < len(text):
        plain_text = text[last_end:]
        if plain_text:
            segments.append({"text": plain_text, "class": None})

    # If no markup found, return whole text
    if not segments:
        segments.append({"text": text, "class": None})

    return segments


def get_tajweed_color(tajweed_class: str) -> str:
    """
    Get color for a Tajweed rule class.

    Args:
        tajweed_class: Tajweed class name (e.g., "ham_wasl", "madda_normal")

    Returns:
        CSS color string
    """
    if not tajweed_class:
        return "#000000"  # Black for normal text

    # Tajweed color mapping (from qpc-hafs-tajweed specification)
    colors = {
        # Madd (elongation) - Orange/Yellow tones
        "madda_normal": "#FFC87C",
        "madda_permissible": "#FFB84D",
        "madda_necessary": "#FFA500",
        "madda_obligatory_mottasel": "#FF8C00",
        "madda_obligatory_monfasel": "#FF9500",

        # Ghunnah (nasal) - Blue tones
        "ghunnah": "#64C8FF",
        "idgham_ghunnah": "#64C8FF",

        # Qalqalah - Light Green
        "qalaqah": "#90EE90",

        # Idghaam variants - Green tones
        "idgham_wo_ghunnah": "#7FFF7F",
        "idgham_mutajanisayn": "#96FF96",
        "idgham_mutaqaribayn": "#A0FFA0",
        "idgham_shafawi": "#B0FFB0",

        # Hamza wasl - Purple
        "ham_wasl": "#D8BFD8",

        # Silent - Gray
        "slnt": "#CCCCCC",

        # Ikhfa variants - Light Blue/Cyan
        "ikhafa": "#B0E0E6",
        "ikhafa_shafawi": "#AFEEEE",

        # Iqlab - Pink
        "iqlab": "#FFB6C1",

        # Laam shamsiyah - Moccasin (FIXED spelling!)
        "laam_shamsiyah": "#FFE4B5",
    }

    return colors.get(tajweed_class, "#000000")
