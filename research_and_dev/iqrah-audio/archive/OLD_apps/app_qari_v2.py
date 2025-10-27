"""
Qari Phoneme Visualization App V2
==================================

Features:
- Real-time playback cursor
- Arabic words with Tajweed colors
- RTL X-axis (right-to-left)
- Phoneme segmentation
- Pitch visualization
"""

from fastapi import FastAPI, HTTPException
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from pathlib import Path
import json

# Import analysis modules
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.phoneme_simple import extract_phonemes_simple
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.tajweed_loader import get_ayah_words, parse_tajweed_html, get_tajweed_color

app = FastAPI(title="Qari Phoneme Visualization V2")

# Mount static files
static_dir = Path(__file__).parent / "static"
static_dir.mkdir(exist_ok=True)
app.mount("/static", StaticFiles(directory=str(static_dir)), name="static")


@app.get("/", response_class=HTMLResponse)
async def home():
    """Serve the main page."""
    return FileResponse("static/qari_v2.html")


@app.get("/api/analyze/{surah}/{ayah}")
async def analyze_qari(surah: int, ayah: int):
    """
    Analyze Qari recitation with all features.

    Returns:
        - pitch: Pitch contour
        - phonemes: Phoneme segments
        - arabic_words: Arabic words with Tajweed markup
        - audio_url: URL to audio file
    """
    try:
        # Use local Husary files
        local_audio = Path(f"data/husary/surahs/{surah:03d}/{ayah:02d}.mp3")

        print(f"\n📊 Analyzing Qari recitation: {surah}:{ayah}")
        print(f"   Audio: {local_audio}")

        if not local_audio.exists():
            raise HTTPException(status_code=404, detail="Audio file not found")

        # 1. Extract pitch with SwiftF0
        print(f"\n1️⃣ Extracting pitch...")
        pitch_data = extract_pitch_swiftf0(str(local_audio))
        print(f"   ✓ Duration: {pitch_data['duration']:.2f}s")

        # 2. Load transliteration
        print(f"\n2️⃣ Loading transliteration...")
        trans_data = load_transliteration_data()
        transliteration = trans_data.get(f"{surah}:{ayah}", "")
        print(f"   ✓ Transliteration: {transliteration}")

        # 3. Extract phonemes
        print(f"\n3️⃣ Extracting phonemes...")
        phonemes = extract_phonemes_simple(
            transliteration=transliteration,
            pitch_data=pitch_data
        )
        print(f"   ✓ Found {len(phonemes)} phoneme segments")

        # 4. Load Arabic words with Tajweed markup
        print(f"\n4️⃣ Loading Arabic words with Tajweed...")
        arabic_words = get_ayah_words(surah, ayah)

        # Parse Tajweed HTML for each word
        words_with_tajweed = []
        for word in arabic_words:
            segments = parse_tajweed_html(word['text'])
            words_with_tajweed.append({
                'word_num': int(word['word']),
                'segments': [
                    {
                        'text': seg['text'],
                        'tajweed_class': seg['class'],
                        'color': get_tajweed_color(seg['class'])
                    }
                    for seg in segments
                ]
            })

        print(f"   ✓ Loaded {len(words_with_tajweed)} Arabic words")

        # Return results
        return {
            "success": True,
            "surah": surah,
            "ayah": ayah,
            "audio_url": f"/audio/{surah:03d}/{ayah:02d}.mp3",
            "pitch": pitch_data,
            "phonemes": phonemes,
            "arabic_words": words_with_tajweed,
            "transliteration": transliteration,
            "duration": pitch_data['duration']
        }

    except Exception as e:
        print(f"   ❌ Error: {e}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/audio/{surah}/{ayah}")
async def get_audio(surah: str, ayah: str):
    """Serve audio file."""
    audio_path = Path(f"data/husary/surahs/{surah}/{ayah}")

    if not audio_path.exists():
        raise HTTPException(status_code=404, detail="Audio not found")

    return FileResponse(
        audio_path,
        media_type="audio/mpeg",
        headers={"Accept-Ranges": "bytes"}
    )


if __name__ == "__main__":
    import uvicorn
    print("\n" + "="*70)
    print("🎯 Qari Phoneme Visualization V2")
    print("="*70)
    print("\nStarting server on http://0.0.0.0:8003")
    print("\nFeatures:")
    print("  ✓ Real-time playback cursor")
    print("  ✓ Arabic words with Tajweed colors")
    print("  ✓ RTL X-axis (right-to-left)")
    print("  ✓ Phoneme segmentation")
    print("  ✓ Pitch visualization")
    print("\n" + "="*70 + "\n")

    uvicorn.run(app, host="0.0.0.0", port=8003)
