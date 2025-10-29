# Tajweed Annotation Tool - Context for Claude

## Project Goal
Web-based tool for experts to record Tajweed violations (anti-patterns) with frame-level annotations for ML training data.

## Architecture
- **Backend**: FastAPI + SQLite (local-first)
- **Data**: Audio on disk (WAV 16kHz mono), metadata in DB
- **Frontend**: Will use WaveSurfer.js + React (not implemented yet in v0.1)
- **Environment**: Conda `iqrah` env (Python 3.13)

## Quick Start

```bash
# Activate environment
conda activate iqrah

# Install dependencies
cd backend
pip install -r requirements.txt

# Initialize database
python -m app.db.init_db

# Run server
uvicorn app.main:app --reload

# Run tests
python test_api.py
```

## Current Status: v0.1 MVP ✅ COMPLETE
Focus: Quick data collection for **ghunnah** and **qalqalah** violations.

### v0.1 Features
- Create/list/delete recordings
- Upload audio (webm/wav) → stored on disk
- Regions/labels CRUD (frame-level: seconds as float)
- JSON export (filter by rule/anti_pattern/date)
- CORS for localhost frontends

### Data Sources
- `data/qpc-hafs-tajweed.db` (read-only): 83,668 Quranic words with locations (surah:ayah:word)
- `data/annotation.db` (created by tool): recordings + regions

## Database Schema (v0.1)

### recordings
- rule: "ghunnah" | "qalqalah"
- anti_pattern: e.g., "weak-ghunnah" | "no-qalqalah"
- qpc_location: "89:27:3" (optional, links to QPC DB)
- audio_path: relative path (e.g., "audio/2025-10-28/123.wav")
- sample_rate, duration_sec, created_at

### regions
- recording_id (FK)
- start_sec, end_sec (frame-level timestamps)
- label: e.g., "weak-ghunnah-onset"
- confidence (0-1, optional)
- notes (optional)

## API Surface (v0.1 Stable)

### Recordings
- `POST /api/recordings` → create metadata
- `POST /api/recordings/{id}/upload` → saves audio file
- `GET /api/recordings?rule=&anti_pattern=&qpc_location=`
- `GET /api/recordings/{id}`
- `DELETE /api/recordings/{id}` → cascade + delete file

### Regions
- `GET /api/recordings/{id}/regions`
- `POST /api/regions`
- `PATCH /api/regions/{id}`
- `DELETE /api/regions/{id}`

### Export
- `GET /api/export/json?rule=&anti_pattern=&from=&to=`

## Validation Rules
- `0 <= start_sec < end_sec <= duration_sec`
- `sample_rate ∈ {16000, 22050, 44100}`
- `duration_sec > 0`

## Labels Taxonomy (v0.1)

### Rules
- ghunnah, qalqalah (madd optional)

### Anti-patterns
- ghunnah: weak-ghunnah, no-ghunnah
- qalqalah: no-qalqalah, weak-qalqalah

### Region labels
- ghunnah: weak-ghunnah-onset, weak-ghunnah-sustain
- qalqalah: no-qalqalah, burst-misaligned

## File Structure
```
backend/
├── app/
│   ├── main.py              # FastAPI app
│   ├── db/
│   │   ├── models.py        # SQLAlchemy models
│   │   └── init_db.py       # DB initialization
│   ├── api/
│   │   └── routes/          # Endpoint handlers
│   └── core/                # Utilities
├── requirements.txt
└── .env

data/
├── qpc-hafs-tajweed.db      # Read-only reference
├── annotation.db            # Created by init_db
└── audio/                   # Audio files (gitignored)
```

## Feature Flags (env)
```
MOD_CORE=true               # v0.1
MOD_EXPORT_JSON=true        # v0.1
MOD_SEARCH=true             # v0.2
MOD_SPECTRO=false           # v0.2
MOD_PARQUET=false           # v0.3
MOD_M3=false                # v0.4 (phoneme overlay)
MOD_ROLES=false             # v1.0 (multi-user)
```

## Implementation Notes
- Use Pydantic v2 models for all I/O
- Store relative paths in DB, resolve with AUDIO_DIR env var
- Delete cascades: DB delete must remove audio file
- Atomic writes: temp → fsync → rename
- Reject files > 50MB (configurable)
- UTC timestamps
- Handle missing files gracefully

## Testing Strategy
- Test region validation (boundaries)
- Test export includes regions + correct paths
- Test delete cascade removes file
- Test upload/transcode if webm→wav needed

## Future Versions (postponed)
- v0.2: Spectrogram, simple search, Alembic migrations
- v0.3: Parquet export, stats cache
- v0.4: M3 phoneme overlay, auto-suggest
- v1.0: Roles, review queue, tags, priority matrix

## Reference Docs
- Full spec: `ANNOTATION_TOOL_SPEC.md` (comprehensive, multi-user, tags, priority)
- Versioned spec: `instructions.md` (MVP-focused, feature flags)
