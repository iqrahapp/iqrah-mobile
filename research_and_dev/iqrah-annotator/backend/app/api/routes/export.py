"""API routes for data export."""

from datetime import datetime
from typing import Optional
from fastapi import APIRouter, Depends, Query
from fastapi.responses import JSONResponse
from sqlalchemy.orm import Session

from app.db import get_db
from app.db.models import Recording, Region
from app.core.schemas import ExportResponse, RecordingExport, RegionResponse

router = APIRouter(prefix="/api/export", tags=["export"])


@router.get("/json", response_model=ExportResponse)
def export_json(
    rule: Optional[str] = Query(None, description="Filter by rule"),
    anti_pattern: Optional[str] = Query(None, description="Filter by anti_pattern"),
    from_date: Optional[str] = Query(None, alias="from", description="From date (YYYY-MM-DD)"),
    to_date: Optional[str] = Query(None, alias="to", description="To date (YYYY-MM-DD)"),
    db: Session = Depends(get_db)
):
    """
    Export recordings and annotations as JSON.

    Filters:
    - rule: Filter by rule name
    - anti_pattern: Filter by anti-pattern name
    - from: Filter by creation date (from)
    - to: Filter by creation date (to)
    """
    # Build query
    query = db.query(Recording)

    # Apply filters
    if rule:
        query = query.filter(Recording.rule == rule)
    if anti_pattern:
        query = query.filter(Recording.anti_pattern == anti_pattern)
    if from_date:
        try:
            from_dt = datetime.fromisoformat(from_date)
            query = query.filter(Recording.created_at >= from_dt)
        except ValueError:
            pass  # Ignore invalid date format
    if to_date:
        try:
            to_dt = datetime.fromisoformat(to_date)
            query = query.filter(Recording.created_at <= to_dt)
        except ValueError:
            pass  # Ignore invalid date format

    # Get recordings
    recordings = query.all()

    # Build export data
    export_recordings = []
    for rec in recordings:
        # Get regions for this recording
        regions = (
            db.query(Region)
            .filter(Region.recording_id == rec.id)
            .order_by(Region.start_sec)
            .all()
        )

        region_responses = [
            RegionResponse(
                id=r.id,
                recording_id=r.recording_id,
                start_sec=r.start_sec,
                end_sec=r.end_sec,
                label=r.label,
                confidence=r.confidence,
                notes=r.notes,
                created_at=r.created_at
            )
            for r in regions
        ]

        export_recordings.append(
            RecordingExport(
                id=rec.id,
                rule=rec.rule,
                anti_pattern=rec.anti_pattern,
                qpc_location=rec.qpc_location,
                sample_rate=rec.sample_rate,
                duration_sec=rec.duration_sec,
                audio_path=rec.audio_path,
                regions=region_responses
            )
        )

    # Build export response
    export_data = ExportResponse(
        version="0.1",
        export_date=datetime.utcnow(),
        recordings=export_recordings
    )

    return export_data
