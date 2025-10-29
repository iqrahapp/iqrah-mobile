"""API routes for regions (annotations)."""

from typing import List
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session

from app.db import get_db
from app.db.models import Recording, Region
from app.core.schemas import RegionCreate, RegionResponse, RegionUpdate
from app.core.utils import validate_region_boundaries

router = APIRouter(prefix="/api", tags=["regions"])


@router.get("/recordings/{recording_id}/regions", response_model=List[RegionResponse])
def get_recording_regions(
    recording_id: int,
    db: Session = Depends(get_db)
):
    """Get all regions for a recording."""
    # Verify recording exists
    recording = db.query(Recording).filter(Recording.id == recording_id).first()
    if not recording:
        raise HTTPException(status_code=404, detail="Recording not found")

    # Get regions, ordered by start time
    regions = (
        db.query(Region)
        .filter(Region.recording_id == recording_id)
        .order_by(Region.start_sec)
        .all()
    )

    return regions


@router.post("/regions", response_model=RegionResponse, status_code=201)
def create_region(
    data: RegionCreate,
    db: Session = Depends(get_db)
):
    """Create a new annotation region."""
    # Verify recording exists
    recording = db.query(Recording).filter(Recording.id == data.recording_id).first()
    if not recording:
        raise HTTPException(status_code=404, detail="Recording not found")

    # Validate region boundaries
    try:
        validate_region_boundaries(data.start_sec, data.end_sec, recording.duration_sec)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

    # Create region
    region = Region(
        recording_id=data.recording_id,
        start_sec=data.start_sec,
        end_sec=data.end_sec,
        label=data.label,
        confidence=data.confidence,
        notes=data.notes,
    )

    db.add(region)
    db.commit()
    db.refresh(region)

    return region


@router.patch("/regions/{region_id}", response_model=RegionResponse)
def update_region(
    region_id: int,
    data: RegionUpdate,
    db: Session = Depends(get_db)
):
    """Update an annotation region."""
    region = db.query(Region).filter(Region.id == region_id).first()

    if not region:
        raise HTTPException(status_code=404, detail="Region not found")

    # Get recording for boundary validation
    recording = db.query(Recording).filter(Recording.id == region.recording_id).first()

    # Update fields
    if data.start_sec is not None:
        region.start_sec = data.start_sec
    if data.end_sec is not None:
        region.end_sec = data.end_sec
    if data.label is not None:
        region.label = data.label
    if data.confidence is not None:
        region.confidence = data.confidence
    if data.notes is not None:
        region.notes = data.notes

    # Validate boundaries after update
    try:
        validate_region_boundaries(region.start_sec, region.end_sec, recording.duration_sec)
    except ValueError as e:
        db.rollback()
        raise HTTPException(status_code=400, detail=str(e))

    db.commit()
    db.refresh(region)

    return region


@router.delete("/regions/{region_id}", status_code=200)
def delete_region(
    region_id: int,
    db: Session = Depends(get_db)
):
    """Delete an annotation region."""
    region = db.query(Region).filter(Region.id == region_id).first()

    if not region:
        raise HTTPException(status_code=404, detail="Region not found")

    db.delete(region)
    db.commit()

    return {
        "message": "Region deleted successfully",
        "region_id": region_id
    }
