"""
Extract proper Uthmani text from QPC Hafs Tajweed database.
"""

import json
import re

def get_uthmani_text_for_verse(verse_id: str):
    """Get the full Uthmani text for a verse by combining all words."""
    with open('data/qpc-hafs-tajweed.json', 'r', encoding='utf-8') as f:
        tajweed_data = json.load(f)

    surah, ayah = verse_id.split(':')
    verse_words = []

    # Get all words for this verse
    for location, entry in sorted(tajweed_data.items()):
        if location.startswith(f"{surah}:{ayah}:"):
            text = entry.get('text', '')
            # Remove HTML tags to get clean Uthmani text
            clean_text = re.sub(r'<[^>]+>', '', text)
            verse_words.append(clean_text)

    return ' '.join(verse_words)


# Test with our target verses
test_verses = ['89:27', '35:6', '4:58']

for verse_id in test_verses:
    uthmani_text = get_uthmani_text_for_verse(verse_id)
    print(f"{verse_id}: {uthmani_text}")
    print()
