"""
Tajweed Rules Parser
====================

Parse Hafs Tajweed annotations from qpc-hafs-tajweed.json
Extract Tajweed rules embedded in text (ham_wasl, laam_shamsiyah, madda, etc.)
"""

import json
import re
from pathlib import Path
from typing import Dict, List, Tuple

# Tajweed rule classes and their properties
TAJWEED_RULES = {
    'ham_wasl': {
        'name': 'Hamzatul Wasl',
        'description': 'Connective hamza - silent when connected',
        'phoneme_effect': 'optional_initial'
    },
    'laam_shamsiyah': {
        'name': 'Lam Shamsiyah',
        'description': 'Solar lam - assimilates to following letter',
        'phoneme_effect': 'assimilation'
    },
    'madda_normal': {
        'name': 'Madd Tabee',
        'description': 'Natural elongation (2 counts)',
        'duration_multiplier': 2.0
    },
    'madda_permissible': {
        'name': 'Madd Jaiz',
        'description': 'Permissible elongation (2-5 counts)',
        'duration_multiplier': 2.5
    },
    'madda_obligatory': {
        'name': 'Madd Wajib',
        'description': 'Obligatory elongation (4-6 counts)',
        'duration_multiplier': 5.0
    },
    'ghunnah': {
        'name': 'Ghunnah',
        'description': 'Nasalization (2 counts)',
        'duration_multiplier': 1.5
    },
    'idgham_wo_ghunnah': {
        'name': 'Idgham without Ghunnah',
        'description': 'Merge without nasalization',
        'phoneme_effect': 'merge'
    },
    'idgham_w_ghunnah': {
        'name': 'Idgham with Ghunnah',
        'description': 'Merge with nasalization',
        'phoneme_effect': 'merge_nasal'
    },
    'iqlab': {
        'name': 'Iqlab',
        'description': 'Convert noon sakin to meem',
        'phoneme_effect': 'n_to_m'
    },
    'ikhafa': {
        'name': 'Ikhfa',
        'description': 'Hide noon sakin',
        'phoneme_effect': 'hide_nasal'
    }
}


def load_tajweed_data(json_path: str = None) -> Dict:
    """
    Load Hafs Tajweed JSON data.

    Args:
        json_path: Path to qpc-hafs-tajweed.json

    Returns:
        Dictionary mapping location (surah:ayah:word) to word data
    """
    if json_path is None:
        # Default location
        json_path = Path(__file__).parent.parent.parent.parent / 'data' / 'qpc-hafs-tajweed.json'

    with open(json_path, 'r', encoding='utf-8') as f:
        return json.load(f)


def parse_tajweed_text(text: str) -> Tuple[str, List[Dict]]:
    """
    Parse text with embedded Tajweed rule tags.

    Args:
        text: Text with <rule class=...>char</rule> tags

    Returns:
        (clean_text, rules_list)
        - clean_text: Text without tags
        - rules_list: List of {char, rule_class, position}
    """
    clean_text = ""
    rules = []
    position = 0

    # Pattern to match <rule class=X>char</rule>
    pattern = r'<rule class=([^>]+)>([^<]+)</rule>'

    last_end = 0
    for match in re.finditer(pattern, text):
        # Add text before match
        before_text = text[last_end:match.start()]
        clean_text += before_text
        position += len(before_text)

        rule_class = match.group(1)
        char = match.group(2)

        # Record rule
        rules.append({
            'char': char,
            'rule_class': rule_class,
            'position': position,
            'rule_info': TAJWEED_RULES.get(rule_class, {})
        })

        clean_text += char
        position += len(char)
        last_end = match.end()

    # Add remaining text
    clean_text += text[last_end:]

    return clean_text, rules


def get_word_tajweed(surah: int, ayah: int, word: int, tajweed_data: Dict = None) -> Dict:
    """
    Get Tajweed-annotated word data.

    Args:
        surah: Surah number
        ayah: Ayah number
        word: Word number
        tajweed_data: Pre-loaded Tajweed data (optional)

    Returns:
        {
            'text': 'Original text with tags',
            'clean_text': 'Text without tags',
            'rules': [...],
            'location': 'surah:ayah:word'
        }
    """
    if tajweed_data is None:
        tajweed_data = load_tajweed_data()

    location = f"{surah}:{ayah}:{word}"
    word_data = tajweed_data.get(location)

    if not word_data:
        return None

    text = word_data['text']
    clean_text, rules = parse_tajweed_text(text)

    return {
        'text': text,
        'clean_text': clean_text,
        'rules': rules,
        'location': location,
        'surah': surah,
        'ayah': ayah,
        'word': word
    }


def get_ayah_tajweed(surah: int, ayah: int, tajweed_data: Dict = None) -> List[Dict]:
    """
    Get all Tajweed-annotated words for an ayah.

    Args:
        surah: Surah number
        ayah: Ayah number
        tajweed_data: Pre-loaded Tajweed data (optional)

    Returns:
        List of word Tajweed data
    """
    if tajweed_data is None:
        tajweed_data = load_tajweed_data()

    words = []
    word_num = 1

    while True:
        word_data = get_word_tajweed(surah, ayah, word_num, tajweed_data)
        if not word_data:
            break
        words.append(word_data)
        word_num += 1

    return words


def apply_tajweed_to_phonemes(
    phoneme_segments: List[Dict],
    tajweed_rules: List[Dict]
) -> List[Dict]:
    """
    Apply Tajweed rules to phoneme segments.

    Args:
        phoneme_segments: List of phoneme segments from forced alignment
        tajweed_rules: List of Tajweed rules from parse_tajweed_text

    Returns:
        Enhanced phoneme segments with Tajweed info
    """
    enhanced = []

    for seg in phoneme_segments:
        # Check if phoneme matches any Tajweed rule
        for rule in tajweed_rules:
            rule_info = rule['rule_info']

            # Apply duration multipliers for Madd
            if 'duration_multiplier' in rule_info:
                seg['duration'] *= rule_info['duration_multiplier']
                seg['end'] = seg['start'] + seg['duration']
                seg['tajweed_rule'] = rule['rule_class']
                seg['tajweed_name'] = rule_info.get('name', '')
                seg['tajweed_desc'] = rule_info.get('description', '')

        enhanced.append(seg)

    return enhanced


if __name__ == "__main__":
    # Test the parser
    print("Testing Tajweed Parser...")

    # Test 1: Parse text with rules
    text = '<rule class=ham_wasl>ٱ</rule><rule class=laam_shamsiyah>ل</rule>رَّحۡمَ<rule class=madda_normal>ـٰ</rule>نِ'
    clean, rules = parse_tajweed_text(text)

    print(f"\nOriginal: {text}")
    print(f"Clean: {clean}")
    print(f"Rules: {len(rules)} found")
    for rule in rules:
        print(f"  - {rule['char']}: {rule['rule_class']} at position {rule['position']}")

    # Test 2: Get word data
    word_data = get_word_tajweed(1, 1, 1)  # Bismillah first word
    if word_data:
        print(f"\nWord 1:1:1:")
        print(f"  Text: {word_data['text']}")
        print(f"  Clean: {word_data['clean_text']}")
        print(f"  Rules: {len(word_data['rules'])}")

    # Test 3: Get full ayah
    ayah_words = get_ayah_tajweed(1, 1)
    print(f"\nAyah 1:1 has {len(ayah_words)} words")

    print("\n✓ Tajweed parser working!")
