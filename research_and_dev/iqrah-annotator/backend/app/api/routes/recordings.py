"""API routes for recordings."""

import os
from typing import List, Optional
from fastapi import APIRouter, Depends, HTTPException, UploadFile, File, Query, Request
from sqlalchemy.orm import Session
from sqlalchemy import func
from slowapi import Limiter
from slowapi.util import get_remote_address

from app.db import get_db
from app.db.models import Recording, Region
from app.core.schemas import (
    RecordingCreate,
    RecordingResponse,
    RecordingUpdate,
    PaginatedResponse,
    ErrorResponse,
)
from app.core.utils import (
    generate_audio_path,
    save_audio_file,
    delete_audio_file,
    validate_audio_file,
)

router = APIRouter(prefix="/api/recordings", tags=["recordings"])

# Rate limiter for upload endpoint
limiter = Limiter(key_func=get_remote_address)


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
@limiter.limit("10/minute")
async def upload_audio(
    request: Request,
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
        raise HTTPException(
            status_code=404,
            detail="Recording not found"
        )

    # Validate file type
    if not file.filename:
        raise HTTPException(
            status_code=400,
            detail="No filename provided"
        )

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

    # Validate audio file
    try:
        audio_info = await validate_audio_file(file_data, recording.sample_rate)
    except ValueError as e:
        raise HTTPException(
            status_code=400,
            detail=str(e)
        )

    # Save file using async I/O
    try:
        await save_audio_file(file_data, recording.audio_path)
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Failed to save audio file: {str(e)}"
        )

    return {
        "message": "Audio uploaded successfully",
        "recording_id": recording_id,
        "audio_path": recording.audio_path,
        "audio_info": audio_info
    }


@router.get("", response_model=PaginatedResponse)
def list_recordings(
    rule: Optional[str] = Query(None, description="Filter by rule"),
    anti_pattern: Optional[str] = Query(None, description="Filter by anti_pattern"),
    qpc_location: Optional[str] = Query(None, description="Filter by QPC location"),
    limit: int = Query(100, ge=1, le=1000, description="Max results"),
    offset: int = Query(0, ge=0, description="Pagination offset"),
    db: Session = Depends(get_db)
):
    """List recordings with optional filters and pagination metadata."""
    query = db.query(Recording).filter(Recording.deleted_at.is_(None))

    # Apply filters
    if rule:
        query = query.filter(Recording.rule == rule)
    if anti_pattern:
        query = query.filter(Recording.anti_pattern == anti_pattern)
    if qpc_location:
        query = query.filter(Recording.qpc_location == qpc_location)

    # Get total count before pagination
    total = query.count()

    # Order by newest first
    query = query.order_by(Recording.created_at.desc())

    # Pagination
    recordings = query.offset(offset).limit(limit).all()

    return PaginatedResponse(
        items=recordings,
        total=total,
        limit=limit,
        offset=offset,
        has_more=(offset + limit) < total
    )


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
async def delete_recording(
    recording_id: int,
    hard_delete: bool = Query(False, description="Permanently delete (default: soft delete)"),
    db: Session = Depends(get_db)
):
    """Delete recording (soft delete by default, cascade deletes regions and audio file on hard delete)."""
    recording = db.query(Recording).filter(Recording.id == recording_id).first()

    if not recording:
        raise HTTPException(status_code=404, detail="Recording not found")

    audio_path = recording.audio_path

    if hard_delete:
        # Hard delete: Remove from database (cascade deletes regions)
        db.delete(recording)
        db.commit()

        # Delete audio file asynchronously
        await delete_audio_file(audio_path)

        return {
            "message": "Recording permanently deleted",
            "recording_id": recording_id,
            "deleted_type": "hard"
        }
    else:
        # Soft delete: Mark as deleted
        from datetime import datetime, timezone
        recording.deleted_at = datetime.now(timezone.utc)
        db.commit()

        return {
            "message": "Recording soft deleted successfully",
            "recording_id": recording_id,
            "deleted_type": "soft"
        }
