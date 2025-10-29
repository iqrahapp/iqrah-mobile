"""API routes for recordings."""

from typing import List, Optional
from fastapi import APIRouter, Depends, HTTPException, UploadFile, File, Query
from sqlalchemy.orm import Session

from app.db import get_db
from app.db.models import Recording, Region
from app.core.schemas import RecordingCreate, RecordingResponse, RecordingUpdate
from app.core.utils import (
    generate_audio_path,
    save_audio_file,
    delete_audio_file,
)

router = APIRouter(prefix="/api/recordings", tags=["recordings"])


@router.post("", response_model=RecordingResponse, status_code=201)
def create_recording(
    data: RecordingCreate,
    db: Session = Depends(get_db)
):
    """Create a new recording metadata (without audio file yet)."""
    # Generate audio path (will be uploaded later)
    audio_path = generate_audio_path()

    recording = Recording(
        rule=data.rule,
        anti_pattern=data.anti_pattern,
        qpc_location=data.qpc_location,
        sample_rate=data.sample_rate,
        duration_sec=data.duration_sec,
        audio_path=audio_path,
    )

    db.add(recording)
    db.commit()
    db.refresh(recording)

    return recording


@router.post("/{recording_id}/upload", status_code=200)
async def upload_audio(
    recording_id: int,
    file: UploadFile = File(...),
    db: Session = Depends(get_db)
):
    """
    Upload audio file for a recording.

    Accepts: .wav, .webm
    Stores as: .wav (16kHz mono preferred)
    """
    # Get recording
    recording = db.query(Recording).filter(Recording.id == recording_id).first()
    if not recording:
        raise HTTPException(status_code=404, detail="Recording not found")

    # Validate file type
    if not file.filename:
        raise HTTPException(status_code=400, detail="No filename provided")

    ext = file.filename.split(".")[-1].lower()
    if ext not in ["wav", "webm"]:
        raise HTTPException(
            status_code=400,
            detail=f"Unsupported file type: .{ext}. Use .wav or .webm"
        )

    # Read file data
    file_data = await file.read()

    # Check size (50MB limit)
    max_size = int(os.getenv("MAX_FILE_MB", "50")) * 1024 * 1024
    if len(file_data) > max_size:
        raise HTTPException(
            status_code=413,
            detail=f"File too large. Max size: {max_size / 1024 / 1024}MB"
        )

    # Save file
    try:
        save_audio_file(file_data, recording.audio_path)
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Failed to save audio file: {str(e)}"
        )

    return {
        "message": "Audio uploaded successfully",
        "recording_id": recording_id,
        "audio_path": recording.audio_path
    }


@router.get("", response_model=List[RecordingResponse])
def list_recordings(
    rule: Optional[str] = Query(None, description="Filter by rule"),
    anti_pattern: Optional[str] = Query(None, description="Filter by anti_pattern"),
    qpc_location: Optional[str] = Query(None, description="Filter by QPC location"),
    limit: int = Query(100, ge=1, le=1000, description="Max results"),
    offset: int = Query(0, ge=0, description="Pagination offset"),
    db: Session = Depends(get_db)
):
    """List recordings with optional filters."""
    query = db.query(Recording)

    # Apply filters
    if rule:
        query = query.filter(Recording.rule == rule)
    if anti_pattern:
        query = query.filter(Recording.anti_pattern == anti_pattern)
    if qpc_location:
        query = query.filter(Recording.qpc_location == qpc_location)

    # Order by newest first
    query = query.order_by(Recording.created_at.desc())

    # Pagination
    recordings = query.offset(offset).limit(limit).all()

    return recordings


@router.get("/{recording_id}", response_model=RecordingResponse)
def get_recording(
    recording_id: int,
    db: Session = Depends(get_db)
):
    """Get a specific recording by ID."""
    recording = db.query(Recording).filter(Recording.id == recording_id).first()

    if not recording:
        raise HTTPException(status_code=404, detail="Recording not found")

    return recording


@router.patch("/{recording_id}", response_model=RecordingResponse)
def update_recording(
    recording_id: int,
    data: RecordingUpdate,
    db: Session = Depends(get_db)
):
    """Update recording metadata."""
    recording = db.query(Recording).filter(Recording.id == recording_id).first()

    if not recording:
        raise HTTPException(status_code=404, detail="Recording not found")

    # Update fields
    if data.rule is not None:
        recording.rule = data.rule
    if data.anti_pattern is not None:
        recording.anti_pattern = data.anti_pattern
    if data.qpc_location is not None:
        recording.qpc_location = data.qpc_location

    db.commit()
    db.refresh(recording)

    return recording


@router.delete("/{recording_id}", status_code=200)
def delete_recording(
    recording_id: int,
    db: Session = Depends(get_db)
):
    """Delete recording (cascade deletes regions and audio file)."""
    recording = db.query(Recording).filter(Recording.id == recording_id).first()

    if not recording:
        raise HTTPException(status_code=404, detail="Recording not found")

    audio_path = recording.audio_path

    # Delete from database (cascade deletes regions)
    db.delete(recording)
    db.commit()

    # Delete audio file
    delete_audio_file(audio_path)

    return {
        "message": "Recording deleted successfully",
        "recording_id": recording_id
    }


# Import os for env var
import os
