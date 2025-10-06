"""
Offline Recitation Analysis API
================================

FastAPI backend for offline analysis only.
Much simpler than real-time!
"""

from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse, JSONResponse
from pathlib import Path
import tempfile
import shutil
from typing import Dict

from src.iqrah_audio.core.segments_loader import SegmentsLoader
from src.iqrah_audio.analysis.offline import analyze_recitation
from src.iqrah_audio.analysis.pitch_extractor import extract_pitch_from_url
from src.iqrah_audio.recording import detect_silence_from_file, trim_silence_from_end
import librosa

app = FastAPI(title="Iqrah Audio - Offline Analysis")

# Mount static files
app.mount("/static", StaticFiles(directory="static"), name="static")

# Initialize segments loader
loader = SegmentsLoader()


@app.get("/")
async def root():
    """Serve offline UI."""
    return FileResponse("static/offline.html")


@app.get("/api/reference/{surah}/{ayah}")
async def get_reference_data(surah: int, ayah: int) -> Dict:
    """
    Get reference audio pitch data and metadata for visualization.

    Args:
        surah: Surah number (1-114)
        ayah: Ayah number

    Returns:
        Dictionary with:
            - audio_url: URL to reference audio
            - words: List of Arabic words
            - segments: Word timing segments
            - pitch_data: Extracted pitch for visualization
    """

    try:
        # Load ayah data
        ayah_data = loader.get_ayah(surah, ayah)

        if not ayah_data:
            raise HTTPException(status_code=404, detail=f"Ayah {surah}:{ayah} not found")

        print(f"\n{'='*60}")
        print(f"Loading reference: Surah {surah}, Ayah {ayah}")
        print(f"{'='*60}")

        # Extract pitch from reference audio
        print("Extracting reference pitch...")
        pitch_data = extract_pitch_from_url(ayah_data.audio_url)

        print(f"✓ Reference pitch extracted:")
        print(f"  Duration: {pitch_data['duration']:.2f}s")
        print(f"  Frames: {len(pitch_data['time'])}")

        # Prepare segments
        segments = [
            {
                'start_ms': seg.start_ms,
                'end_ms': seg.end_ms,
                'word': ayah_data.words[i] if i < len(ayah_data.words) else ''
            }
            for i, seg in enumerate(ayah_data.segments)
        ]

        return {
            'audio_url': ayah_data.audio_url,
            'words': ayah_data.words,
            'segments': segments,
            'pitch_data': {
                'time': pitch_data['time'],
                'f0_hz': pitch_data['f0_hz'],
                'confidence': pitch_data['confidence'],
                'voiced': pitch_data['voiced'],
                'duration': pitch_data['duration']
            },
            'metadata': {
                'surah': surah,
                'ayah': ayah,
                'num_words': len(ayah_data.words),
                'num_segments': len(segments)
            }
        }

    except Exception as e:
        print(f"❌ Error loading reference: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/analyze/{surah}/{ayah}")
async def analyze_user_recitation(
    surah: int,
    ayah: int,
    audio: UploadFile = File(...)
) -> Dict:
    """
    Analyze user's recitation against reference.

    Args:
        surah: Surah number (1-114)
        ayah: Ayah number
        audio: User's audio file (MP3, WAV, etc.)

    Returns:
        Complete analysis results:
            - user_pitch: User pitch data
            - ref_pitch: Reference pitch data
            - alignment: DTW alignment results
            - word_scores: Per-word accuracy scores
            - metrics: All 5 comprehensive metrics
    """

    try:
        # Load ayah data
        ayah_data = loader.get_ayah(surah, ayah)

        if not ayah_data:
            raise HTTPException(status_code=404, detail=f"Ayah {surah}:{ayah} not found")

        print(f"\n{'='*60}")
        print(f"ANALYZING RECITATION: Surah {surah}, Ayah {ayah}")
        print(f"{'='*60}")

        # Save uploaded audio to temporary file
        temp_dir = Path(tempfile.gettempdir()) / "iqrah_uploads"
        temp_dir.mkdir(exist_ok=True)

        temp_path = temp_dir / f"user_{surah}_{ayah}_{audio.filename}"

        print(f"Saving uploaded audio to: {temp_path}")
        with open(temp_path, "wb") as buffer:
            shutil.copyfileobj(audio.file, buffer)

        print(f"✓ Audio saved ({temp_path.stat().st_size} bytes)")

        # Convert webm to wav for processing
        print("\nConverting audio to WAV format...")
        wav_path = temp_path.with_suffix('.wav')
        audio_data, sr = librosa.load(str(temp_path), sr=16000, mono=True)

        # Check for silence at end and trim if needed
        print("Checking for trailing silence...")
        has_silence, silence_duration = detect_silence_from_file(
            str(temp_path),
            silence_threshold_db=-40.0,
            silence_duration_ms=2000.0
        )

        if has_silence:
            print(f"✓ Found {silence_duration:.0f}ms trailing silence - trimming...")
            trimmed_audio = trim_silence_from_end(
                audio_data,
                sample_rate=sr,
                silence_threshold_db=-40.0,
                silence_duration_ms=500.0  # Keep 500ms silence
            )
            audio_data = trimmed_audio
            print(f"✓ Audio trimmed to {len(audio_data)/sr:.2f}s")
        else:
            print(f"✓ No significant trailing silence detected ({silence_duration:.0f}ms)")

        # Save as WAV for CREPE
        import soundfile as sf
        sf.write(str(wav_path), audio_data, sr)
        print(f"✓ Converted to WAV: {wav_path}")

        # Use WAV file for analysis
        temp_path = wav_path

        # Prepare segments for analysis
        ref_segments = [
            {'start_ms': seg.start_ms, 'end_ms': seg.end_ms}
            for seg in ayah_data.segments
        ]

        # Run complete offline analysis
        print("\nStarting offline analysis...")
        result = analyze_recitation(
            user_audio_path=str(temp_path),
            ref_audio_url=ayah_data.audio_url,
            ref_segments=ref_segments,
            ref_words=ayah_data.words
        )

        print(f"\n{'='*60}")
        print(f"✅ ANALYSIS COMPLETE")
        print(f"   Overall Score: {result['metrics']['overall_score']}/100")
        print(f"   Good Words: {result['metrics']['pitch_accuracy']['good_words']}/{len(ayah_data.words)}")
        print(f"   Tempo: {result['metrics']['tempo']['mean_ratio']:.2f}x")
        print(f"   Stability: {result['metrics']['stability']['status']}")
        print(f"{'='*60}")

        # Add metadata
        result['metadata'] = {
            'surah': surah,
            'ayah': ayah,
            'words': ayah_data.words,
            'audio_url': ayah_data.audio_url
        }

        # Clean up temp file
        try:
            temp_path.unlink()
        except:
            pass

        return result

    except Exception as e:
        print(f"❌ Error analyzing recitation: {e}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/surahs")
async def get_surahs():
    """Get list of all surahs with metadata."""
    return {
        'surahs': [
            {'number': i, 'name': f'Surah {i}', 'ayahs': 7 if i == 1 else 286}
            for i in range(1, 115)
        ]
    }


if __name__ == "__main__":
    import uvicorn

    print("\n" + "="*60)
    print("🎙️  IQRAH AUDIO - OFFLINE ANALYSIS MODE")
    print("="*60)
    print("\nStarting server on http://localhost:8001")
    print("Frontend: http://localhost:8001")
    print("API docs: http://localhost:8001/docs")
    print("\n" + "="*60 + "\n")

    uvicorn.run(app, host="0.0.0.0", port=8001)
