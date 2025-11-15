"""API routes for data export."""

import json
from datetime import datetime, timezone
from typing import Optional
from fastapi import APIRouter, Depends, Query, HTTPException
from fastapi.responses import StreamingResponse
from sqlalchemy.orm import Session, joinedload

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
    # Build query with eager loading to avoid N+1 problem
    query = db.query(Recording).options(joinedload(Recording.regions)).filter(
        Recording.deleted_at.is_(None)
    )

    # Apply filters
    if rule:
        query = query.filter(Recording.rule == rule)
    if anti_pattern:
        query = query.filter(Recording.anti_pattern == anti_pattern)
    if from_date:
        try:
            from_dt = datetime.fromisoformat(from_date)
            query = query.filter(Recording.created_at >= from_dt)
        except ValueError as e:
            raise HTTPException(
                status_code=400,
                detail=f"Invalid 'from' date format. Expected YYYY-MM-DD, got '{from_date}'. Error: {str(e)}"
            )
    if to_date:
        try:
            to_dt = datetime.fromisoformat(to_date)
            query = query.filter(Recording.created_at <= to_dt)
        except ValueError as e:
            raise HTTPException(
                status_code=400,
                detail=f"Invalid 'to' date format. Expected YYYY-MM-DD, got '{to_date}'. Error: {str(e)}"
            )

    # Get recordings (regions are already loaded via joinedload)
    recordings = query.all()

    # Build export data
    export_recordings = []
    for rec in recordings:
        # Sort regions by start time
        sorted_regions = sorted(rec.regions, key=lambda r: r.start_sec)

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
            for r in sorted_regions
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
        export_date=datetime.now(timezone.utc),
        recordings=export_recordings
    )

    return export_data


@router.get("/json/stream")
def export_json_stream(
    rule: Optional[str] = Query(None, description="Filter by rule"),
    anti_pattern: Optional[str] = Query(None, description="Filter by anti_pattern"),
    from_date: Optional[str] = Query(None, alias="from", description="From date (YYYY-MM-DD)"),
    to_date: Optional[str] = Query(None, alias="to", description="To date (YYYY-MM-DD)"),
    db: Session = Depends(get_db)
):
    """
    Export recordings and annotations as streaming JSON (for large datasets).
    """
    # Build query
    query = db.query(Recording).options(joinedload(Recording.regions)).filter(
        Recording.deleted_at.is_(None)
    )

    # Apply filters
    if rule:
        query = query.filter(Recording.rule == rule)
    if anti_pattern:
        query = query.filter(Recording.anti_pattern == anti_pattern)
    if from_date:
        try:
            from_dt = datetime.fromisoformat(from_date)
            query = query.filter(Recording.created_at >= from_dt)
        except ValueError as e:
            raise HTTPException(
                status_code=400,
                detail=f"Invalid 'from' date format. Expected YYYY-MM-DD, got '{from_date}'"
            )
    if to_date:
        try:
            to_dt = datetime.fromisoformat(to_date)
            query = query.filter(Recording.created_at <= to_dt)
        except ValueError as e:
            raise HTTPException(
                status_code=400,
                detail=f"Invalid 'to' date format. Expected YYYY-MM-DD, got '{to_date}'"
            )

    def generate():
        """Generate JSON stream."""
        # Start of JSON object
        yield '{"version":"0.1","export_date":"' + datetime.now(timezone.utc).isoformat() + '","recordings":['

        first = True
        for rec in query.yield_per(100):  # Stream 100 records at a time
            if not first:
                yield ','
            first = False

            # Sort regions
            sorted_regions = sorted(rec.regions, key=lambda r: r.start_sec)

            # Build recording export
            rec_export = RecordingExport(
                id=rec.id,
                rule=rec.rule,
                anti_pattern=rec.anti_pattern,
                qpc_location=rec.qpc_location,
                sample_rate=rec.sample_rate,
                duration_sec=rec.duration_sec,
                audio_path=rec.audio_path,
                regions=[
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
                    for r in sorted_regions
                ]
            )

            # Serialize to JSON
            yield json.dumps(rec_export.model_dump(), default=str)

        # End of JSON array and object
        yield ']}'

    return StreamingResponse(
        generate(),
        media_type="application/json",
        headers={"Content-Disposition": "attachment; filename=export.json"}
    )
