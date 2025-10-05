#!/usr/bin/env python3
"""
Real-Time Recitation Analysis Web API
======================================

FastAPI backend with WebSocket support for real-time Quranic recitation analysis.

Features:
- REST API for reference audio upload
- WebSocket streaming for real-time analysis
- JSON feedback with visual cues
- Performance metrics tracking

Usage:
    # Start server
    uvicorn app:app --reload --port 8000

    # Or run directly
    python app.py
"""

import asyncio
import base64
import io
import json
import numpy as np
import soundfile as sf
from pathlib import Path
from typing import Optional, Dict, Any
from dataclasses import asdict

from fastapi import FastAPI, WebSocket, WebSocketDisconnect, UploadFile, File, HTTPException
from fastapi.staticfiles import StaticFiles
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.middleware.cors import CORSMiddleware

from iqrah_audio.streaming import (
    RealtimePipeline,
    PipelineConfig,
    RealtimeHints,
)
from iqrah_audio.core.segments_loader import SegmentsLoader, AyahData

# Initialize FastAPI
app = FastAPI(
    title="Iqrah Audio - Real-Time Recitation Analysis",
    description="Ultra-low latency Quranic recitation analysis API",
    version="1.0.0",
)

# Enable CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Mount static files
static_dir = Path(__file__).parent / "static"
if static_dir.exists():
    app.mount("/static", StaticFiles(directory=str(static_dir)), name="static")

# Global state
pipelines: Dict[str, RealtimePipeline] = {}
default_reference_path = "data/husary/surahs/01.mp3"
segments_loader = SegmentsLoader()


@app.get("/", response_class=HTMLResponse)
async def root():
    """Serve the main web interface."""
    html_path = Path(__file__).parent / "static" / "index.html"
    if html_path.exists():
        return FileResponse(html_path)

    # Fallback simple HTML
    return HTMLResponse(content="""
    <!DOCTYPE html>
    <html>
    <head>
        <title>Iqrah Audio - Real-Time Analysis</title>
        <meta charset="utf-8">
    </head>
    <body>
        <h1>Iqrah Audio - Real-Time Recitation Analysis</h1>
        <p>WebSocket endpoint: <code>ws://localhost:8000/ws/analyze</code></p>
        <p>API documentation: <a href="/docs">/docs</a></p>
    </body>
    </html>
    """)


@app.get("/api/health")
async def health_check():
    """Health check endpoint."""
    return {
        "status": "healthy",
        "pipelines_active": len(pipelines),
        "version": "1.0.0"
    }


@app.get("/api/segments/{surah}/{ayah}")
async def get_ayah_segments(surah: int, ayah: int):
    """
    Get word-level segments and text for an ayah.

    Args:
        surah: Surah number (1-114)
        ayah: Ayah number

    Returns:
        {
            "surah": 1,
            "ayah": 1,
            "verse_key": "1:1",
            "text": "بِسۡمِ اللهِ الرَّحۡمٰنِ الرَّحِيۡمِ",
            "words": ["بِسۡمِ", "اللهِ", "الرَّحۡمٰنِ", "الرَّحِيۡمِ"],
            "audio_url": "https://...",
            "segments": [
                {"word_id": 1, "start_ms": 0, "end_ms": 480, "duration_ms": 480},
                ...
            ]
        }
    """
    try:
        ayah_data = segments_loader.get_ayah(surah, ayah)

        return {
            "surah": ayah_data.surah,
            "ayah": ayah_data.ayah,
            "verse_key": ayah_data.verse_key,
            "text": ayah_data.text,
            "words": ayah_data.words,
            "audio_url": ayah_data.audio_url,
            "segments": [
                {
                    "word_id": seg.word_id,
                    "start_ms": seg.start_ms,
                    "end_ms": seg.end_ms,
                    "duration_ms": seg.duration_ms
                }
                for seg in ayah_data.segments
            ]
        }
    except KeyError as e:
        raise HTTPException(status_code=404, detail=str(e))


@app.get("/api/segments/stats")
async def get_coverage_stats():
    """
    Get statistics about available segment data.

    Returns:
        Coverage statistics including total ayahs, words, etc.
    """
    return segments_loader.get_coverage_stats()


@app.post("/api/reference/upload")
async def upload_reference(
    file: UploadFile = File(...),
    session_id: str = "default"
):
    """
    Upload reference audio for a session.

    Args:
        file: Audio file (mp3, wav, etc.)
        session_id: Session identifier

    Returns:
        Session info with reference details
    """
    try:
        # Read audio file
        audio_bytes = await file.read()
        audio_buffer = io.BytesIO(audio_bytes)

        # Load audio
        audio, sr = sf.read(audio_buffer)
        if len(audio.shape) > 1:
            audio = audio.mean(axis=1)
        audio = audio.astype(np.float32)

        duration = len(audio) / sr

        # Create pipeline
        config = PipelineConfig(
            sample_rate=sr,
            enable_anchors=True,
            update_rate_hz=30.0,  # Increased for more responsive feedback
        )

        pipeline = RealtimePipeline(audio, config)
        pipelines[session_id] = pipeline

        return {
            "session_id": session_id,
            "status": "ready",
            "reference": {
                "filename": file.filename,
                "duration": duration,
                "sample_rate": sr,
                "frames": len(pipeline.reference_pitch.f0_hz),
                "anchors": len(pipeline.reference_anchors),
                "audio_url": f"/api/reference/audio/{session_id}",
            },
            "reference_pitch": [
                {"f0_hz": float(f0)} for f0 in pipeline.reference_pitch.f0_hz
            ]
        }

    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.get("/api/reference/default")
async def use_default_reference(session_id: str = "default"):
    """
    Use default reference (Husary Al-Fatiha) for a session.

    Args:
        session_id: Session identifier

    Returns:
        Session info with reference details
    """
    try:
        # Load default reference
        audio, sr = sf.read(default_reference_path)
        if len(audio.shape) > 1:
            audio = audio.mean(axis=1)
        audio = audio.astype(np.float32)

        duration = len(audio) / sr

        # Create pipeline
        config = PipelineConfig(
            sample_rate=sr,
            enable_anchors=True,
            update_rate_hz=30.0,  # Increased for more responsive feedback
        )

        pipeline = RealtimePipeline(audio, config)
        pipelines[session_id] = pipeline

        return {
            "session_id": session_id,
            "status": "ready",
            "reference": {
                "filename": "Husary Al-Fatiha (default)",
                "duration": duration,
                "sample_rate": sr,
                "frames": len(pipeline.reference_pitch.f0_hz),
                "anchors": len(pipeline.reference_anchors),
                "audio_url": f"/api/reference/audio/{session_id}",
            },
            "reference_pitch": [
                {"f0_hz": float(f0)} for f0 in pipeline.reference_pitch.f0_hz
            ]
        }

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/reference/audio/{session_id}")
async def get_reference_audio(session_id: str = "default"):
    """
    Get reference audio file for playback.
    
    Args:
        session_id: Session identifier (or 'default' for Husary Al-Fatiha)
    
    Returns:
        Audio file
    """
    if session_id == "default":
        # Return default reference audio
        audio_path = Path(__file__).parent / default_reference_path
        if audio_path.exists():
            return FileResponse(audio_path, media_type="audio/mpeg")
    
    raise HTTPException(status_code=404, detail="Reference audio not found")


@app.websocket("/ws/analyze")
async def websocket_analyze(websocket: WebSocket):
    """
    WebSocket endpoint for real-time audio analysis.

    Protocol:
        Client -> Server:
            {
                "type": "config",
                "session_id": "default",
                "sample_rate": 44100
            }

            {
                "type": "audio",
                "data": "<base64-encoded-float32-array>"
            }

        Server -> Client:
            {
                "type": "hints",
                "data": {...},  // RealtimeHints
                "stats": {...}  // PipelineStats
            }

            {
                "type": "error",
                "message": "..."
            }
    """
    await websocket.accept()

    session_id = None
    pipeline = None

    try:
        while True:
            # Receive message
            message = await websocket.receive_json()
            msg_type = message.get("type")

            if msg_type == "config":
                # Configure session
                session_id = message.get("session_id", "default")

                # Get or create pipeline
                if session_id not in pipelines:
                    # Use default reference
                    audio, sr = sf.read(default_reference_path)
                    if len(audio.shape) > 1:
                        audio = audio.mean(axis=1)
                    audio = audio.astype(np.float32)

                    config = PipelineConfig(
                        sample_rate=sr,
                        enable_anchors=True,
                        update_rate_hz=30.0,  # Increased from 15 to 30 Hz for more responsive feedback
                    )

                    pipeline = RealtimePipeline(audio, config)
                    pipelines[session_id] = pipeline
                else:
                    pipeline = pipelines[session_id]

                # Send confirmation
                await websocket.send_json({
                    "type": "config_ok",
                    "session_id": session_id,
                    "reference_frames": len(pipeline.reference_pitch.f0_hz),
                })

            elif msg_type == "audio":
                # Process audio chunk
                if pipeline is None:
                    await websocket.send_json({
                        "type": "error",
                        "message": "No pipeline configured. Send config first."
                    })
                    continue

                # Decode base64 audio data
                audio_b64 = message.get("data")
                audio_bytes = base64.b64decode(audio_b64)
                audio_chunk = np.frombuffer(audio_bytes, dtype=np.float32)

                # Debug: Log audio chunk info (only first time)
                if pipeline.stats.total_frames_processed == 0:
                    print(f"📊 First audio chunk: {len(audio_chunk)} samples, "
                          f"RMS: {np.sqrt(np.mean(audio_chunk**2)):.4f}")

                # Process chunk
                hints = pipeline.process_chunk(audio_chunk)

                # Get stats
                stats = pipeline.get_stats()

                # Send response
                response = {
                    "type": "processed",
                    "has_hints": hints is not None,
                }

                if hints:
                    # Convert hints to dict and ensure all values are JSON serializable
                    hints_dict = asdict(hints)
                    # Convert numpy types to Python native types
                    for key, value in hints_dict.items():
                        if isinstance(value, (np.integer, np.floating)):
                            hints_dict[key] = value.item()
                        elif isinstance(value, np.ndarray):
                            hints_dict[key] = value.tolist()
                    
                    response["hints"] = hints_dict
                    print(f"✓ Hints generated: {hints.message[:50]}... (status={hints.status})")

                response["stats"] = {
                    "total_latency_ms": stats.total_latency_ms,
                    "pitch_latency_ms": stats.pitch_latency_ms,
                    "dtw_latency_ms": stats.dtw_latency_ms,
                    "frames_processed": stats.total_frames_processed,
                }

                await websocket.send_json(response)

            elif msg_type == "reset":
                # Reset pipeline
                if pipeline:
                    pipeline.reset()
                    await websocket.send_json({
                        "type": "reset_ok",
                        "session_id": session_id
                    })

            else:
                await websocket.send_json({
                    "type": "error",
                    "message": f"Unknown message type: {msg_type}"
                })

    except WebSocketDisconnect:
        print(f"Client disconnected: {session_id}")
    except Exception as e:
        await websocket.send_json({
            "type": "error",
            "message": str(e)
        })
        print(f"WebSocket error: {e}")


@app.get("/api/stats/{session_id}")
async def get_stats(session_id: str):
    """Get pipeline statistics for a session."""
    if session_id not in pipelines:
        raise HTTPException(status_code=404, detail="Session not found")

    pipeline = pipelines[session_id]
    stats = pipeline.get_stats()
    state = pipeline.get_alignment_state()

    return {
        "session_id": session_id,
        "stats": {
            "total_latency_ms": stats.total_latency_ms,
            "pitch_latency_ms": stats.pitch_latency_ms,
            "dtw_latency_ms": stats.dtw_latency_ms,
            "feedback_latency_ms": stats.feedback_latency_ms,
            "frames_processed": stats.total_frames_processed,
            "hints_generated": stats.hints_generated,
            "audio_duration_s": stats.total_audio_duration_s,
        },
        "alignment": {
            "reference_position": state.reference_position,
            "lead_lag_ms": state.lead_lag_ms,
            "confidence": state.confidence,
            "status": state.status,
        }
    }


if __name__ == "__main__":
    import uvicorn

    print("=" * 80)
    print("Iqrah Audio - Real-Time Recitation Analysis API")
    print("=" * 80)
    print("\nStarting server...")
    print("  - API docs: http://localhost:8000/docs")
    print("  - WebSocket: ws://localhost:8000/ws/analyze")
    print("  - Health: http://localhost:8000/api/health")
    print("\n" + "=" * 80)

    uvicorn.run(app, host="0.0.0.0", port=8000)
