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

from fastapi import FastAPI, HTTPException, File, UploadFile, Form
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel
from pathlib import Path
import tempfile
import shutil

# Import analysis modules
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.pitch_extractor_crepe import extract_pitch_crepe_fast, extract_pitch_crepe_accurate
from src.iqrah_audio.analysis.phoneme_mms_proper import extract_phonemes_mms_proper
from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.tajweed_loader import get_ayah_words, parse_tajweed_html, get_tajweed_color
from src.iqrah_audio.analysis.segments_loader import get_ayah_segments, download_audio, get_word_segments_with_text
from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics
from src.iqrah_audio.comparison import compare_recitations
from src.iqrah_audio.comparison.visualization import generate_comparison_visualizations

app = FastAPI(title="Qari Tajweed Analysis - Final")

# Mount static files
static_dir = Path(__file__).parent / "static"
static_dir.mkdir(exist_ok=True)
app.mount("/static", StaticFiles(directory=str(static_dir)), name="static")


@app.get("/", response_class=HTMLResponse)
async def home():
    """Serve the main page."""
    return FileResponse("static/qari_final.html")


@app.get("/comparison", response_class=HTMLResponse)
async def comparison_page():
    """Serve the comparison visualization page (old version)."""
    return FileResponse("static/comparison_visualize.html")


@app.get("/compare", response_class=HTMLResponse)
async def compare_user_page():
    """Serve the user comparison page."""
    return FileResponse("static/compare_user.html")


@app.get("/api/analyze/{surah}/{ayah}")
async def analyze_qari(surah: int, ayah: int, pitch_extractor: str = "swiftf0"):
    """
    Analyze Qari recitation using PROPER AI Report 2 approach.

    Args:
        surah: Surah number
        ayah: Ayah number
        pitch_extractor: 'swiftf0', 'crepe-fast', or 'crepe-accurate'

    Returns:
        - pitch: Pitch contour
        - phonemes: Phoneme segments (MMS-FA aligned)
        - arabic_words: Arabic words with Tajweed
        - audio_url: URL to cached audio
    """
    try:
        print(f"\n{'='*70}")
        print(f"📊 Analyzing: Surah {surah}, Ayah {ayah}")
        print(f"   Pitch Extractor: {pitch_extractor}")
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
        print(f"\n3️⃣ Extracting pitch with {pitch_extractor}...")
        if pitch_extractor == "crepe-fast":
            pitch_data = extract_pitch_crepe_fast(audio_path)
        elif pitch_extractor == "crepe-accurate":
            pitch_data = extract_pitch_crepe_accurate(audio_path)
        else:  # swiftf0 (default)
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

        # 6. Extract phonemes - use Wav2Vec2 CTC (better alignment!)
        print(f"\n6️⃣ Extracting phonemes with Wav2Vec2 CTC...")
        phonemes = extract_phonemes_wav2vec2_ctc(
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

        # 8. Compute statistics
        print(f"\n8️⃣ Computing statistics...")
        statistics = compute_full_statistics(phonemes, pitch_data)
        print(f"   ✓ Tempo: {statistics['tempo']['syllables_per_second']} syl/s (stability: {statistics['tempo']['stability_score']})")
        print(f"   ✓ Pitch: {statistics['pitch']['mean_pitch']} Hz (±{statistics['pitch']['std_pitch']} Hz)")
        print(f"   ✓ Count: {statistics['count']['mean_count']}s (precision: {statistics['count']['precision_score']})")
        if statistics['madd']['total_madds'] > 0:
            print(f"   ✓ Madd accuracy: {statistics['madd']['overall_accuracy']}% ({statistics['madd']['total_madds']} total)")

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
            "duration": pitch_data['duration'],
            "statistics": statistics  # NEW: Add statistics
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


@app.post("/api/compare")
async def compare_api(
    student_surah: int,
    student_ayah: int,
    reference_surah: int,
    reference_ayah: int,
    pitch_extractor: str = "swiftf0"
):
    """
    Compare student recitation against reference (Qari).

    Args:
        student_surah: Student's surah number
        student_ayah: Student's ayah number
        reference_surah: Reference surah number
        reference_ayah: Reference ayah number
        pitch_extractor: Pitch extraction method

    Returns:
        Comprehensive comparison with overall score and component breakdowns
    """
    try:
        print(f"\n{'='*70}")
        print(f"🔄 Comparing: {student_surah}:{student_ayah} vs {reference_surah}:{reference_ayah}")
        print(f"{'='*70}\n")

        # Analyze both recordings (reuse existing analysis pipeline)
        print("1️⃣ Analyzing student...")
        student_result = await analyze_qari(student_surah, student_ayah, pitch_extractor)

        print("\n2️⃣ Analyzing reference...")
        reference_result = await analyze_qari(reference_surah, reference_ayah, pitch_extractor)

        # Get audio paths
        student_seg = get_ayah_segments(student_surah, student_ayah)
        reference_seg = get_ayah_segments(reference_surah, reference_ayah)

        student_audio = download_audio(student_seg['audio_url'])
        reference_audio = download_audio(reference_seg['audio_url'])

        # Run comparison engine
        print("\n3️⃣ Running comparison engine...")
        comparison = compare_recitations(
            student_audio_path=student_audio,
            reference_audio_path=reference_audio,
            student_phonemes=student_result['phonemes'],
            reference_phonemes=reference_result['phonemes'],
            student_pitch=student_result['pitch'],
            reference_pitch=reference_result['pitch'],
            student_stats=student_result['statistics'],
            reference_stats=reference_result['statistics']
        )

        print(f"\n{'='*70}")
        print(f"✅ Comparison complete!")
        print(f"   Overall: {comparison['overall']}/100")
        print(f"   Rhythm: {comparison['rhythm']['score']}/100")
        print(f"   Melody: {comparison['melody']['score']}/100")
        print(f"   Duration: {comparison['durations']['overall']}/100")
        print(f"{'='*70}\n")

        return {
            "success": True,
            "comparison": comparison,
            "student_analysis": student_result,
            "reference_analysis": reference_result
        }

    except Exception as e:
        print(f"\n❌ Comparison error: {e}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=str(e))


class ComparisonRequest(BaseModel):
    student_surah: int
    student_ayah: int
    reference_surah: int
    reference_ayah: int
    pitch_extractor: str = "swiftf0"


@app.post("/api/compare/visualize")
async def compare_visualize(request: ComparisonRequest):
    """
    Compare two recitations and generate visualizations.

    Args:
        request: ComparisonRequest with surah/ayah numbers

    Returns:
        Comparison with base64-encoded visualization images
    """
    try:
        from src.iqrah_audio.comparison.features import extract_features

        student_surah = request.student_surah
        student_ayah = request.student_ayah
        reference_surah = request.reference_surah
        reference_ayah = request.reference_ayah
        pitch_extractor = request.pitch_extractor

        print(f"\n{'='*70}")
        print(f"📊 Comparing with visualizations: {student_surah}:{student_ayah} vs {reference_surah}:{reference_ayah}")
        print(f"{'='*70}\n")

        # Analyze both recordings
        print("1️⃣ Analyzing student...")
        student_result = await analyze_qari(student_surah, student_ayah, pitch_extractor)

        print("\n2️⃣ Analyzing reference...")
        reference_result = await analyze_qari(reference_surah, reference_ayah, pitch_extractor)

        # Get audio paths
        student_seg = get_ayah_segments(student_surah, student_ayah)
        reference_seg = get_ayah_segments(reference_surah, reference_ayah)

        student_audio = download_audio(student_seg['audio_url'])
        reference_audio = download_audio(reference_seg['audio_url'])

        # Extract features
        print("\n3️⃣ Extracting features...")
        student_features = extract_features(
            student_audio,
            student_result['phonemes'],
            student_result['pitch'],
            student_result['statistics']
        )
        reference_features = extract_features(
            reference_audio,
            reference_result['phonemes'],
            reference_result['pitch'],
            reference_result['statistics']
        )

        # Run comparison
        print("\n4️⃣ Running comparison...")
        comparison = compare_recitations(
            student_audio_path=student_audio,
            reference_audio_path=reference_audio,
            student_phonemes=student_result['phonemes'],
            reference_phonemes=reference_result['phonemes'],
            student_pitch=student_result['pitch'],
            reference_pitch=reference_result['pitch'],
            student_stats=student_result['statistics'],
            reference_stats=reference_result['statistics']
        )

        # Generate visualizations
        print("\n5️⃣ Generating visualizations...")
        visualizations = generate_comparison_visualizations(
            comparison_result=comparison,
            student_audio_path=student_audio,
            reference_audio_path=reference_audio,
            student_features=student_features,
            reference_features=reference_features,
            student_phonemes=student_result['phonemes'],
            reference_phonemes=reference_result['phonemes'],
            student_pitch=student_result['pitch'],
            reference_pitch=reference_result['pitch']
        )

        print(f"\n✅ Generated {len(visualizations)} visualizations")
        print(f"{'='*70}\n")

        return {
            "success": True,
            "comparison": comparison,
            "visualizations": visualizations,
            "student_analysis": student_result,
            "reference_analysis": reference_result
        }

    except Exception as e:
        print(f"\n❌ Visualization error: {e}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/compare/user")
async def compare_user_audio(
    audio: UploadFile = File(...),
    surah: int = Form(...),
    ayah: int = Form(...),
    pitch_extractor: str = Form("swiftf0")
):
    """
    Compare user's audio recording against Husary reference for the same ayah.

    Args:
        audio: User's audio file (MP3, WAV, WebM, etc.)
        surah: Surah number (1-114)
        ayah: Ayah number
        pitch_extractor: Pitch extraction method

    Returns:
        Comparison with visualizations against Husary reference
    """
    try:
        from src.iqrah_audio.comparison.features import extract_features

        print(f"\n{'='*70}")
        print(f"🎤 User Recitation vs Husary Reference: {surah}:{ayah}")
        print(f"{'='*70}\n")

        # Save uploaded audio to temporary file
        with tempfile.NamedTemporaryFile(delete=False, suffix='.webm') as tmp_file:
            shutil.copyfileobj(audio.file, tmp_file)
            user_audio_path = tmp_file.name

        print(f"📁 Saved user audio: {user_audio_path}")

        # Analyze user's recitation
        print("\n1️⃣ Analyzing your recitation...")
        word_segments = get_word_segments_with_text(surah, ayah)
        trans_data = load_transliteration_data()
        transliteration = trans_data.get(f"{surah}:{ayah}", "")

        # Extract pitch from user audio
        if pitch_extractor == "crepe":
            user_pitch = extract_pitch_crepe_fast(user_audio_path)
        else:
            user_pitch = extract_pitch_swiftf0(user_audio_path)

        # Extract phonemes from user audio
        user_phonemes = extract_phonemes_wav2vec2_ctc(
            audio_path=user_audio_path,
            word_segments=word_segments,
            transliteration=transliteration,
            pitch_data=user_pitch,
            surah=surah,
            ayah=ayah
        )

        # Compute statistics for user
        user_stats = compute_full_statistics(user_phonemes, user_pitch)

        # Analyze Husary reference (same ayah)
        print("\n2️⃣ Loading Husary reference...")
        husary_result = await analyze_qari(surah, ayah, pitch_extractor)

        # Get Husary audio path
        husary_seg = get_ayah_segments(surah, ayah)
        husary_audio = download_audio(husary_seg['audio_url'])

        # Extract features for comparison
        print("\n3️⃣ Extracting features...")
        user_features = extract_features(
            user_audio_path,
            user_phonemes,
            user_pitch,
            user_stats
        )
        husary_features = extract_features(
            husary_audio,
            husary_result['phonemes'],
            husary_result['pitch'],
            husary_result['statistics']
        )

        # Run comparison
        print("\n4️⃣ Running comparison...")
        comparison = compare_recitations(
            student_audio_path=user_audio_path,
            reference_audio_path=husary_audio,
            student_phonemes=user_phonemes,
            reference_phonemes=husary_result['phonemes'],
            student_pitch=user_pitch,
            reference_pitch=husary_result['pitch'],
            student_stats=user_stats,
            reference_stats=husary_result['statistics']
        )

        # Generate visualizations
        print("\n5️⃣ Generating visualizations...")
        visualizations = generate_comparison_visualizations(
            comparison_result=comparison,
            student_audio_path=user_audio_path,
            reference_audio_path=husary_audio,
            student_features=user_features,
            reference_features=husary_features,
            student_phonemes=user_phonemes,
            reference_phonemes=husary_result['phonemes'],
            student_pitch=user_pitch,
            reference_pitch=husary_result['pitch']
        )

        print(f"\n✅ Comparison complete!")
        print(f"   Overall: {comparison['overall']}/100")
        print(f"   Rhythm: {comparison['rhythm']['score']}/100")
        print(f"   Melody: {comparison['melody']['score']}/100")
        print(f"   Duration: {comparison['durations']['overall']}/100")
        print(f"{'='*70}\n")

        # Clean up temp file
        Path(user_audio_path).unlink(missing_ok=True)

        return {
            "success": True,
            "comparison": comparison,
            "visualizations": visualizations,
            "user_analysis": {
                'phonemes': user_phonemes,
                'pitch': user_pitch,
                'statistics': user_stats
            },
            "reference_analysis": husary_result
        }

    except Exception as e:
        print(f"\n❌ User comparison error: {e}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=str(e))


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
