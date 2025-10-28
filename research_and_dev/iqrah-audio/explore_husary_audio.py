"""
Explore Husary audio structure and download test cases for Phase 2 validators.
"""

import json
import os
from pathlib import Path

# Load Husary segments
print("Loading Husary audio segments...")
with open('data/husary/segments/segments.json', 'r', encoding='utf-8') as f:
    husary_segments = json.load(f)

print(f"Total segments: {len(husary_segments)}")
print()

# Show structure
print("=" * 60)
print("Sample Entry Structure")
print("=" * 60)

# Get first entry
first_key = list(husary_segments.keys())[0]
first_entry = husary_segments[first_key]
print(f"Key: {first_key}")
print(json.dumps(first_entry, ensure_ascii=False, indent=2))
print()

# Check if we have the verse from user's example (89:27)
print("=" * 60)
print("Checking Surah 89, Ayah 27 (from user's example)")
print("=" * 60)

target_verses = ['89:27', '35:6', '60:4', '3:91', '4:58']

for verse_id in target_verses:
    if verse_id in husary_segments:
        entry = husary_segments[verse_id]
        print(f"\n{verse_id}:")
        print(f"  Duration: {entry.get('duration', 'N/A')}s")
        print(f"  Audio file: {entry.get('audio_file', 'N/A')}")

        # Check if audio file exists
        audio_path = Path('data/husary/surahs') / entry.get('audio_file', '')
        if audio_path.exists():
            print(f"  Status: ✅ File exists ({audio_path.stat().st_size / 1024:.1f} KB)")
        else:
            print(f"  Status: ❌ File not found")
    else:
        print(f"\n{verse_id}: ❌ Not in segments")

# List available surahs
print()
print("=" * 60)
print("Available Surah Audio Files")
print("=" * 60)

surah_dir = Path('data/husary/surahs')
if surah_dir.exists():
    audio_files = sorted(surah_dir.glob('*.mp3'))
    print(f"Total audio files: {len(audio_files)}")

    if len(audio_files) > 0:
        print(f"\nFirst 5 files:")
        for f in audio_files[:5]:
            size_mb = f.stat().st_size / (1024 * 1024)
            print(f"  {f.name:20} ({size_mb:.2f} MB)")
    else:
        print("\n⚠️ No audio files found in data/husary/surahs/")
        print("   You need to download them first!")
else:
    print(f"❌ Directory not found: {surah_dir}")
