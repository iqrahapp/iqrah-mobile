"""
Explore tajweed annotation database to find verses with qalqalah and ghunnah.
"""

import json
from collections import defaultdict

# Load tajweed annotations
print("Loading tajweed annotations...")
with open('data/qpc-hafs-tajweed.json', 'r', encoding='utf-8') as f:
    tajweed_data = json.load(f)

print(f"Total entries: {len(tajweed_data)}")
print()

# Count rule types
rule_counts = defaultdict(int)
verses_with_rules = defaultdict(set)

for location, entry in tajweed_data.items():
    text = entry.get('text', '')

    # Parse rule classes from HTML-like tags
    if '<rule class=' in text:
        import re
        rules = re.findall(r'<rule class=([^>]+)>', text)

        for rule in rules:
            rule_counts[rule] += 1
            # Extract surah:ayah (without word number)
            surah, ayah, word = location.split(':')
            verse_id = f"{surah}:{ayah}"
            verses_with_rules[rule].add(verse_id)

# Show rule counts
print("=" * 60)
print("Tajweed Rule Counts")
print("=" * 60)
for rule, count in sorted(rule_counts.items(), key=lambda x: x[1], reverse=True):
    print(f"{rule:20} {count:6,} occurrences, {len(verses_with_rules[rule]):5,} verses")

print()
print("=" * 60)
print("Phase 2 Validators - Sample Verses")
print("=" * 60)

# Find verses with qalqalah
qalqalah_verses = list(verses_with_rules.get('qalaqah', []))[:10]
print(f"\nQalqalah verses (first 10): {len(qalqalah_verses)} total")
for verse in qalqalah_verses:
    print(f"  {verse}")

# Find verses with ghunnah
ghunnah_verses = list(verses_with_rules.get('ghunnah', []))[:10]
print(f"\nGhunnah verses (first 10): {len(ghunnah_verses)} total")
for verse in ghunnah_verses:
    print(f"  {verse}")

# Find a verse with BOTH qalqalah and ghunnah
print("\n" + "=" * 60)
print("Verses with BOTH qalqalah and ghunnah (first 5):")
print("=" * 60)

both_rules = verses_with_rules.get('qalaqah', set()) & verses_with_rules.get('ghunnah', set())
for verse in list(both_rules)[:5]:
    print(f"\n{verse}:")

    # Show the words in this verse
    surah, ayah = verse.split(':')
    verse_words = []

    for location, entry in tajweed_data.items():
        if location.startswith(f"{surah}:{ayah}:"):
            text = entry.get('text', '')
            if '<rule class=' in text:
                verse_words.append(f"  {location}: {text}")

    for word_info in verse_words[:3]:  # Show first 3 words
        print(word_info)

# Example: Show full details for 89:27:3 from user's example
print("\n" + "=" * 60)
print("Example: Surah 89, Ayah 27, Word 3 (from user)")
print("=" * 60)
location = '89:27:3'
if location in tajweed_data:
    entry = tajweed_data[location]
    print(json.dumps(entry, ensure_ascii=False, indent=2))
