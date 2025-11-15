"""SQLAlchemy models for annotation database."""

from datetime import datetime, timezone
from sqlalchemy import (
    Column,
    Integer,
    String,
    Float,
    Text,
    DateTime,
    ForeignKey,
    Index,
    Boolean,
)
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship

Base = declarative_base()


class Recording(Base):
    """Recording session with metadata."""

    __tablename__ = "recordings"

    id = Column(Integer, primary_key=True)
    rule = Column(String, nullable=False)  # "ghunnah" | "qalqalah"
    anti_pattern = Column(String, nullable=False)  # e.g., "weak-ghunnah"
    qpc_location = Column(String, nullable=True)  # e.g., "89:27:3" (optional)
    sample_rate = Column(Integer, nullable=False)  # e.g., 16000
    duration_sec = Column(Float, nullable=False)  # seconds
    audio_path = Column(String, nullable=False)  # relative path
    created_at = Column(DateTime, default=lambda: datetime.now(timezone.utc))
    deleted_at = Column(DateTime, nullable=True)  # Soft delete support

    # Relationship
    regions = relationship(
        "Region", back_populates="recording", cascade="all, delete-orphan"
    )

    # Indexes
    __table_args__ = (
        Index("idx_recordings_rule_ap", "rule", "anti_pattern"),
        Index("idx_recordings_qpc", "qpc_location"),
        Index("idx_recordings_created_at", "created_at"),
        Index("idx_recordings_deleted_at", "deleted_at"),
    )


class Region(Base):
    """Frame-level annotation region."""

    __tablename__ = "regions"

    id = Column(Integer, primary_key=True)
    recording_id = Column(
        Integer, ForeignKey("recordings.id", ondelete="CASCADE"), nullable=False
    )
    start_sec = Column(Float, nullable=False)
    end_sec = Column(Float, nullable=False)
    label = Column(String, nullable=False)  # e.g., "weak-ghunnah-onset"
    confidence = Column(Float, nullable=True)  # 0..1
    notes = Column(Text, nullable=True)
    created_at = Column(DateTime, default=lambda: datetime.now(timezone.utc))

    # Relationship
    recording = relationship("Recording", back_populates="regions")

    # Indexes
    __table_args__ = (Index("idx_regions_recording_id", "recording_id"),)
