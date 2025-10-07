"""
Qari Phoneme Visualization - FINAL PROPER VERSION
==================================================

This implements AI Report 2's approach CORRECTLY:
✅ Word-level segments from segments.json
✅ MMS-FA with windowing
✅ Proper phoneme alignment
✅ All 6,236 ayahs supported
✅ Audio downloading
✅ Real-time cursor (moving dot on pitch)
✅ Arabic words with Tajweed
✅ RTL X-axis
"""

from fastapi import FastAPI, HTTPException
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from pathlib import Path

# Import analysis modules
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.phoneme_mms_proper import extract_phonemes_mms_proper
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.tajweed_loader import get_ayah_words, parse_tajweed_html, get_tajweed_color
from src.iqrah_audio.analysis.segments_loader import get_ayah_segments, download_audio, get_word_segments_with_text

app = FastAPI(title="Qari Tajweed Analysis - Final")

# Mount static files
static_dir = Path(__file__).parent / "static"
static_dir.mkdir(exist_ok=True)
app.mount("/static", StaticFiles(directory=str(static_dir)), name="static")


@app.get("/", response_class=HTMLResponse)
async def home():
    """Serve the main page."""
    return FileResponse("static/qari_final.html")


@app.get("/api/analyze/{surah}/{ayah}")
async def analyze_qari(surah: int, ayah: int):
    """
    Analyze Qari recitation using PROPER AI Report 2 approach.

    Returns:
        - pitch: Pitch contour
        - phonemes: Phoneme segments (MMS-FA aligned)
        - arabic_words: Arabic words with Tajweed
        - audio_url: URL to cached audio
    """
    try:
        print(f"\n{'='*70}")
        print(f"📊 Analyzing: Surah {surah}, Ayah {ayah}")
        print(f"{'='*70}")

        # 1. Load segment data
        print(f"\n1️⃣ Loading segment data...")
        seg_data = get_ayah_segments(surah, ayah)
        if not seg_data:
            raise HTTPException(status_code=404, detail=f"No data for {surah}:{ayah}")

        audio_url = seg_data['audio_url']
        print(f"   ✓ Audio URL: {audio_url}")

        # 2. Download audio
        print(f"\n2️⃣ Downloading audio...")
        audio_path = download_audio(audio_url)

        # 3. Extract pitch
        print(f"\n3️⃣ Extracting pitch...")
        pitch_data = extract_pitch_swiftf0(audio_path)
        print(f"   ✓ Duration: {pitch_data['duration']:.2f}s")

        # 4. Get word segments with Arabic text
        print(f"\n4️⃣ Loading word segments...")
        word_segments = get_word_segments_with_text(surah, ayah)
        print(f"   ✓ {len(word_segments)} word segments")

        # 5. Load transliteration
        print(f"\n5️⃣ Loading transliteration...")
        trans_data = load_transliteration_data()
        transliteration = trans_data.get(f"{surah}:{ayah}", "")
        print(f"   ✓ Transliteration: {transliteration}")

        # 6. Extract phonemes using PROPER MMS-FA approach
        print(f"\n6️⃣ Extracting phonemes with MMS-FA...")
        phonemes = extract_phonemes_mms_proper(
            audio_path=audio_path,
            word_segments=word_segments,
            transliteration=transliteration,
            pitch_data=pitch_data,
            surah=surah,
            ayah=ayah,
            device='cpu'
        )
        print(f"   ✓ {len(phonemes)} phoneme segments")

        # 7. Load Arabic words with Tajweed
        print(f"\n7️⃣ Loading Tajweed markup...")
        arabic_words = get_ayah_words(surah, ayah)
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

        print(f"   ✓ {len(words_with_tajweed)} Arabic words")

        print(f"\n{'='*70}")
        print(f"✅ Analysis complete!")
        print(f"{'='*70}\n")

        # Return results
        return {
            "success": True,
            "surah": surah,
            "ayah": ayah,
            "audio_url": f"/audio/{surah}/{ayah}",  # Use proxy endpoint
            "pitch": pitch_data,
            "phonemes": phonemes,
            "arabic_words": words_with_tajweed,
            "word_segments": word_segments,  # Add word segments for visualization
            "transliteration": transliteration,
            "duration": pitch_data['duration']
        }

    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/audio/{surah}/{ayah}")
async def get_audio(surah: int, ayah: int):
    """Serve cached audio file."""
    seg_data = get_ayah_segments(surah, ayah)
    if not seg_data:
        raise HTTPException(status_code=404, detail="Audio not found")

    audio_path = download_audio(seg_data['audio_url'])

    return FileResponse(
        audio_path,
        media_type="audio/mpeg",
        headers={"Accept-Ranges": "bytes"}
    )


if __name__ == "__main__":
    import uvicorn
    print("\n" + "="*70)
    print("🎯 Qari Tajweed Analysis - FINAL PROPER VERSION")
    print("="*70)
    print("\nImplementing AI Report 2's approach:")
    print("  ✅ Word-level segments (6,236 ayahs)")
    print("  ✅ MMS-FA with windowing")
    print("  ✅ Proper phoneme alignment")
    print("  ✅ Audio downloading")
    print("  ✅ Real-time cursor (dot on pitch)")
    print("  ✅ Arabic words + Tajweed colors")
    print("  ✅ RTL X-axis")
    print("\nStarting on http://0.0.0.0:8004")
    print("="*70 + "\n")

    uvicorn.run(app, host="0.0.0.0", port=8004)
