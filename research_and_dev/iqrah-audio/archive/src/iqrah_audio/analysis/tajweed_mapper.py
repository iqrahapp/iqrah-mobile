"""
Tajweed Mapper - Maps Tajweed rules from qpc-hafs-tajweed.json to phonemes.

Uses the authoritative Tajweed data instead of heuristics.
"""
import json
import re
from pathlib import Path
from typing import Dict, List, Optional

# Tajweed rule to color mapping
TAJWEED_COLORS = {
    'madda_normal': '#FFC87C',      # Madd - Elongation (orange)
    'madda_permissible': '#FFC87C',
    'madda_obligatory': '#FFC87C',
    'madda_necessary': '#FFC87C',
    'ghunnah': '#64C8FF',            # Ghunnah - Nasal (blue)
    'idgham_shaddah': '#FFB6C1',     # Shadda - Doubled (pink)
    'iqlab': '#AAAAAA',              # Iqlab - Conversion (gray)
    'ikhfa': '#999999',              # Ikhfa - Concealment (gray)
    'idgham_wo_ghunnah': '#888888',  # Idgham without ghunnah (dark gray)
    'idgham_w_ghunnah': '#64C8FF',   # Idgham with ghunnah (blue)
    'qalqalah': '#90EE90',           # Qalqalah - Echoing (light green)
    'ham_wasl': '#D8BFD8',           # Hamzat wasl (thistle)
    'laam_shamsiyah': '#FFE4B5',     # Lam shamsiyah (moccasin)
    'silent': '#CCCCCC',             # Silent letters (light gray)
}

# Rule categories for simplified display
RULE_CATEGORIES = {
    'madda_normal': 'madd',
    'madda_permissible': 'madd',
    'madda_obligatory': 'madd',
    'madda_necessary': 'madd',
    'ghunnah': 'ghunnah',
    'idgham_w_ghunnah': 'ghunnah',
    'idgham_shaddah': 'shadda',
    'qalqalah': 'qalqalah',
    'iqlab': 'other',
    'ikhfa': 'other',
    'idgham_wo_ghunnah': 'other',
    'ham_wasl': 'other',
    'laam_shamsiyah': 'other',
    'silent': 'other',
}


class TajweedMapper:
    """Maps phonemes to Tajweed rules from authoritative data."""

    def __init__(self, tajweed_json_path: str = "data/qpc-hafs-tajweed.json"):
        self.tajweed_data = {}
        self.load_tajweed_data(tajweed_json_path)

    def load_tajweed_data(self, path: str):
        """Load Tajweed data from JSON file."""
        json_path = Path(path)
        if not json_path.exists():
            print(f"⚠️ Warning: Tajweed data not found at {path}")
            return

        with open(json_path, 'r', encoding='utf-8') as f:
            self.tajweed_data = json.load(f)
        print(f"✅ Loaded Tajweed data: {len(self.tajweed_data)} words")

    def get_word_tajweed_rules(self, surah: int, ayah: int, word_idx: int) -> List[Dict]:
        """
        Get Tajweed rules for a specific word.

        Returns list of dicts with:
        - text: Arabic text (without tags)
        - rule: Tajweed rule class (or None)
        - category: Simplified category (madd/ghunnah/shadda/other)
        - color: Color for visualization
        """
        key = f"{surah}:{ayah}:{word_idx}"
        if key not in self.tajweed_data:
            return []

        word_data = self.tajweed_data[key]
        text_with_tags = word_data['text']

        # Parse HTML-like tags
        segments = []
        pos = 0
        pattern = r'<rule class=([^>]+)>([^<]+)</rule>'

        for match in re.finditer(pattern, text_with_tags):
            # Add text before this match (if any)
            if match.start() > pos:
                plain_text = text_with_tags[pos:match.start()]
                segments.append({
                    'text': plain_text,
                    'rule': None,
                    'category': None,
                    'color': None
                })

            # Add the rule segment
            rule_class = match.group(1)
            rule_text = match.group(2)
            category = RULE_CATEGORIES.get(rule_class, 'other')
            color = TAJWEED_COLORS.get(rule_class, '#CCCCCC')

            segments.append({
                'text': rule_text,
                'rule': rule_class,
                'category': category,
                'color': color
            })

            pos = match.end()

        # Add any remaining text
        if pos < len(text_with_tags):
            remaining = text_with_tags[pos:]
            # Remove any stray HTML artifacts
            remaining = re.sub(r'<[^>]+>', '', remaining)
            if remaining:
                segments.append({
                    'text': remaining,
                    'rule': None,
                    'category': None,
                    'color': None
                })

        return segments

    def map_phoneme_to_tajweed(
        self,
        phoneme_start: float,
        phoneme_end: float,
        word_idx: int,
        word_segments: List[Dict],
        surah: int,
        ayah: int,
        phoneme_text: str = None
    ) -> Optional[str]:
        """
        Map a phoneme to the most appropriate Tajweed rule.

        CRITICAL FIX: Use content-based matching for Madd rules.
        If phoneme contains long vowel (aa/ee/oo) and word has Madd rule → match!

        Args:
            phoneme_text: The phoneme text (e.g., "eem", "laah", "maan")

        Returns:
            Specific Tajweed rule (e.g., 'madda_normal', 'madda_necessary') or None
        """
        if word_idx >= len(word_segments):
            return None

        # Get Tajweed rules for this word
        tajweed_segments = self.get_word_tajweed_rules(surah, ayah, word_idx + 1)  # 1-indexed

        if not tajweed_segments:
            return None

        # CRITICAL FIX: For Madd rules, use phoneme content matching
        # If phoneme has long vowel (aa/ee/oo/etc.) and word has Madd → assign it!
        if phoneme_text:
            phoneme_lower = phoneme_text.lower()
            has_long_vowel = any(lv in phoneme_lower for lv in ['aa', 'ee', 'oo', 'ii', 'uu'])

            if has_long_vowel:
                # Find Madd rules in this word
                for seg in tajweed_segments:
                    if seg['rule'] and 'madda' in seg['rule']:
                        return seg['rule']

        # For non-Madd rules, use time-based matching
        word_seg = word_segments[word_idx]
        word_start_ms = word_seg['start_ms']
        word_end_ms = word_seg['end_ms']
        word_duration = word_end_ms - word_start_ms

        if word_duration <= 0:
            return None

        # Calculate relative position of phoneme within word
        phoneme_mid = (phoneme_start + phoneme_end) / 2
        phoneme_mid_ms = phoneme_mid * 1000

        if phoneme_mid_ms < word_start_ms or phoneme_mid_ms > word_end_ms:
            return None

        relative_pos = (phoneme_mid_ms - word_start_ms) / word_duration

        # Use proportional character distribution for non-Madd rules
        total_chars = sum(len(seg['text']) for seg in tajweed_segments)
        cumulative = 0

        for seg in tajweed_segments:
            seg_length = len(seg['text'])
            seg_fraction = seg_length / total_chars if total_chars > 0 else 0
            seg_start = cumulative
            seg_end = cumulative + seg_fraction

            if seg_start <= relative_pos <= seg_end:
                return seg['rule']

            cumulative = seg_end

        # Default: return rule of first segment
        return tajweed_segments[0]['rule'] if tajweed_segments else None
