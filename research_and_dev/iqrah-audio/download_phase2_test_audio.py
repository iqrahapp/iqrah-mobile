"""
Download specific ayahs for Phase 2 validator testing.
Targets verses with qalqalah and ghunnah annotations.
"""

import json
import requests
from pathlib import Path
import time

# Load segments
with open('data/husary/segments/segments.json', 'r', encoding='utf-8') as f:
    husary_segments = json.load(f)

# Target verses with both qalqalah and ghunnah
target_verses = [
    '89:27',  # User's example - has both qalqalah (ط) and ghunnah (ن)
    '35:6',   # Has both
    '60:4',   # Has both
    '3:91',   # Has both
    '4:58',   # Has both
    '5:17',   # Has both
]

print("=" * 60)
print("Downloading Phase 2 Test Audio (Qalqalah + Ghunnah)")
print("=" * 60)
print()

# Create output directory
output_dir = Path('data/phase2_test_audio')
output_dir.mkdir(exist_ok=True, parents=True)

downloaded = []
failed = []

for verse_id in target_verses:
    if verse_id not in husary_segments:
        print(f"❌ {verse_id}: Not in segments database")
        failed.append(verse_id)
        continue

    entry = husary_segments[verse_id]
    audio_url = entry.get('audio_url')

    if not audio_url:
        print(f"❌ {verse_id}: No audio URL")
        failed.append(verse_id)
        continue

    # Generate filename
    surah, ayah = verse_id.split(':')
    output_file = output_dir / f"surah_{surah}_ayah_{ayah}.mp3"

    # Skip if already downloaded
    if output_file.exists():
        print(f"✅ {verse_id}: Already downloaded ({output_file.stat().st_size / 1024:.1f} KB)")
        downloaded.append((verse_id, output_file))
        continue

    # Download
    try:
        print(f"⏳ {verse_id}: Downloading from {audio_url}...")
        response = requests.get(audio_url, timeout=10)
        response.raise_for_status()

        # Save
        with open(output_file, 'wb') as f:
            f.write(response.content)

        size_kb = len(response.content) / 1024
        print(f"✅ {verse_id}: Downloaded ({size_kb:.1f} KB) → {output_file}")
        downloaded.append((verse_id, output_file))

        # Be nice to the server
        time.sleep(0.5)

    except Exception as e:
        print(f"❌ {verse_id}: Download failed - {e}")
        failed.append(verse_id)

print()
print("=" * 60)
print("Summary")
print("=" * 60)
print(f"Downloaded: {len(downloaded)}")
print(f"Failed: {len(failed)}")

if downloaded:
    print("\n✅ Successfully downloaded:")
    for verse_id, path in downloaded:
        print(f"   {verse_id:10} → {path}")

if failed:
    print("\n❌ Failed:")
    for verse_id in failed:
        print(f"   {verse_id}")

print()
print(f"Output directory: {output_dir.absolute()}")
