"""
Qari Phoneme Visualization App
===============================

Simple one-page app to visualize Qari recitation with:
- Pitch contour
- Phoneme segmentation (from gold transliteration data)
- Tajweed coloration
- Audio playback

NO user comparison - ONLY Qari visualization for learning.
"""

from fastapi import FastAPI, HTTPException
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from pathlib import Path
import json

# Import our analysis modules
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.phoneme_from_transliteration import get_phonemes_from_transliteration
from src.iqrah_audio.reference import ReferenceProcessor

app = FastAPI(title="Qari Phoneme Visualization")

# Mount static files
static_dir = Path(__file__).parent / "static"
static_dir.mkdir(exist_ok=True)
app.mount("/static", StaticFiles(directory=str(static_dir)), name="static")

# Initialize reference processor
ref_processor = ReferenceProcessor()


@app.get("/", response_class=HTMLResponse)
async def home():
    """Serve the main page."""
    return FileResponse("static/qari.html")


@app.get("/api/surahs")
async def get_surahs():
    """Get list of available surahs."""
    # Return surahs 1-114
    surahs = []
    for i in range(1, 115):
        surahs.append({
            "number": i,
            "name": f"Surah {i}"
        })
    return surahs


@app.get("/api/ayahs/{surah}")
async def get_ayahs(surah: int):
    """Get list of ayahs for a surah."""
    # For now, return a fixed number per surah
    # TODO: Load from quran data
    ayah_counts = {
        1: 7, 2: 286, 3: 200, 4: 176, 5: 120
    }
    count = ayah_counts.get(surah, 10)  # Default to 10

    ayahs = []
    for i in range(1, count + 1):
        ayahs.append({
            "number": i,
            "text": f"Ayah {i}"
        })
    return ayahs


@app.get("/api/analyze/{surah}/{ayah}")
async def analyze_qari(surah: int, ayah: int):
    """
    Analyze Qari recitation with phoneme segmentation.

    Returns:
        - pitch: Pitch contour
        - phonemes: Phoneme segments with timing and Tajweed
        - audio_url: URL to audio file
    """
    try:
        # Get reference audio URL
        audio_url = ref_processor.get_audio_url(surah, ayah)

        # Download and extract pitch
        print(f"\n📊 Analyzing Qari recitation: {surah}:{ayah}")
        print(f"   Audio: {audio_url}")

        # For local testing, use Husary files if available
        local_audio = Path(f"data/husary/surahs/{surah:03d}/{ayah:02d}.mp3")

        if local_audio.exists():
            print(f"   Using local file: {local_audio}")
            pitch_data = extract_pitch_swiftf0(str(local_audio))
        else:
            # Would need to download from URL
            raise HTTPException(status_code=404, detail="Audio file not found locally")

        # Extract phonemes from gold transliteration data
        print(f"\n🎯 Extracting phonemes from transliteration...")
        phonemes = get_phonemes_from_transliteration(
            surah=surah,
            ayah=ayah,
            audio_path=str(local_audio),
            pitch_data=pitch_data
        )

        print(f"   ✓ Found {len(phonemes)} phoneme segments")

        # Return results
        return {
            "success": True,
            "surah": surah,
            "ayah": ayah,
            "audio_url": f"/audio/{surah:03d}/{ayah:02d}.mp3",
            "pitch": pitch_data,
            "phonemes": phonemes,
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
    print("\n" + "="*60)
    print("🎯 Qari Phoneme Visualization App")
    print("="*60)
    print("\nStarting server on http://0.0.0.0:8002")
    print("\nFeatures:")
    print("  ✓ Pitch visualization")
    print("  ✓ Phoneme segmentation (from gold data)")
    print("  ✓ Tajweed coloration")
    print("  ✓ Audio playback")
    print("\n" + "="*60 + "\n")

    uvicorn.run(app, host="0.0.0.0", port=8002)
