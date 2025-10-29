"""Pydantic schemas for API validation."""

from datetime import datetime
from typing import Optional, List
from pydantic import BaseModel, Field, field_validator


# ============ Recording Schemas ============


class RecordingCreate(BaseModel):
    """Schema for creating a recording."""

    rule: str = Field(..., description="Rule name (e.g., 'ghunnah', 'qalqalah')")
    anti_pattern: str = Field(..., description="Anti-pattern name (e.g., 'weak-ghunnah')")
    qpc_location: Optional[str] = Field(None, description="QPC location (e.g., '89:27:3')")
    sample_rate: int = Field(..., ge=8000, le=48000, description="Sample rate in Hz")
    duration_sec: float = Field(..., gt=0, description="Duration in seconds")

    @field_validator("sample_rate")
    @classmethod
    def validate_sample_rate(cls, v):
        """Validate sample rate is one of the standard values."""
        allowed = [8000, 16000, 22050, 44100, 48000]
        if v not in allowed:
            raise ValueError(f"Sample rate must be one of {allowed}")
        return v


class RecordingUpdate(BaseModel):
    """Schema for updating a recording."""

    rule: Optional[str] = None
    anti_pattern: Optional[str] = None
    qpc_location: Optional[str] = None


class RecordingResponse(BaseModel):
    """Schema for recording response."""

    id: int
    rule: str
    anti_pattern: str
    qpc_location: Optional[str]
    sample_rate: int
    duration_sec: float
    audio_path: str
    created_at: datetime

    class Config:
        from_attributes = True


# ============ Region Schemas ============


class RegionCreate(BaseModel):
    """Schema for creating a region."""

    recording_id: int = Field(..., description="Recording ID")
    start_sec: float = Field(..., ge=0, description="Start time in seconds")
    end_sec: float = Field(..., gt=0, description="End time in seconds")
    label: str = Field(..., description="Label (e.g., 'weak-ghunnah-onset')")
    confidence: Optional[float] = Field(None, ge=0, le=1, description="Confidence (0-1)")
    notes: Optional[str] = Field(None, description="Optional notes")

    @field_validator("end_sec")
    @classmethod
    def validate_time_range(cls, v, info):
        """Validate end_sec > start_sec."""
        if "start_sec" in info.data and v <= info.data["start_sec"]:
            raise ValueError("end_sec must be greater than start_sec")
        return v


class RegionUpdate(BaseModel):
    """Schema for updating a region."""

    start_sec: Optional[float] = Field(None, ge=0)
    end_sec: Optional[float] = Field(None, gt=0)
    label: Optional[str] = None
    confidence: Optional[float] = Field(None, ge=0, le=1)
    notes: Optional[str] = None


class RegionResponse(BaseModel):
    """Schema for region response."""

    id: int
    recording_id: int
    start_sec: float
    end_sec: float
    label: str
    confidence: Optional[float]
    notes: Optional[str]
    created_at: datetime

    class Config:
        from_attributes = True


# ============ Export Schemas ============


class RecordingExport(BaseModel):
    """Schema for recording in export."""

    id: int
    rule: str
    anti_pattern: str
    qpc_location: Optional[str]
    sample_rate: int
    duration_sec: float
    audio_path: str
    regions: List[RegionResponse]


class ExportResponse(BaseModel):
    """Schema for export response."""

    version: str = "0.1"
    export_date: datetime
    recordings: List[RecordingExport]
