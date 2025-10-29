# Tajweed Annotation Tool v0.1

Web-based tool for collecting Tajweed violation (anti-pattern) annotations with frame-level timestamps for ML training data.

## Status: v0.1 MVP ✅ COMPLETE

Backend + Frontend fully implemented and running!

## Features

### Backend
- **Recording Management**: Create, list, update, delete recordings
- **Audio Upload**: Support for WAV/WebM files
- **Frame-Level Annotations**: Region-based labels with timestamps (seconds)
- **JSON Export**: Filter by rule, anti-pattern, date range
- **Data Validation**: Automatic validation of region boundaries
- **Cascade Deletes**: Deleting recording removes all regions and audio file

### Frontend
- **Interactive Waveform**: WaveSurfer.js visualization with zoom
- **Drag-to-Annotate**: Click and drag on waveform to create regions
- **Region Editing**: Edit labels, confidence scores, and notes
- **Playback Controls**: Play, pause, stop with visual feedback
- **Recordings List**: View, filter, and manage all recordings
- **JSON Export**: Download annotations for ML training

## Quick Start

### Prerequisites

- Conda environment `iqrah` (Python 3.13)
- Available at: `/home/samiisd/miniconda3/envs/iqrah`

### Installation

```bash
# Activate environment
conda activate iqrah

# Navigate to backend
cd backend

# Install dependencies
pip install -r requirements.txt

# Initialize database
python -m app.db.init_db
```

### Run Backend Server

```bash
# From backend directory
uvicorn app.main:app --reload
```

API available at: `http://localhost:8000`
API docs at: `http://localhost:8000/docs`

### Run Frontend

```bash
# From frontend directory
npm install
npm run dev
```

Frontend available at: `http://localhost:5173`

### Run Backend Tests

```bash
python test_api.py
```

## API Endpoints

### Recordings

- `POST /api/recordings` - Create recording metadata
- `POST /api/recordings/{id}/upload` - Upload audio file
- `GET /api/recordings` - List recordings (filters: rule, anti_pattern, qpc_location)
- `GET /api/recordings/{id}` - Get specific recording
- `PATCH /api/recordings/{id}` - Update recording metadata
- `DELETE /api/recordings/{id}` - Delete recording (cascade)

### Regions (Annotations)

- `GET /api/recordings/{id}/regions` - Get regions for recording
- `POST /api/regions` - Create annotation region
- `PATCH /api/regions/{id}` - Update region
- `DELETE /api/regions/{id}` - Delete region

### Export

- `GET /api/export/json` - Export as JSON (filters: rule, anti_pattern, from, to)

## Database Schema

### recordings
- `id`: Primary key
- `rule`: "ghunnah" | "qalqalah"
- `anti_pattern`: e.g., "weak-ghunnah", "no-qalqalah"
- `qpc_location`: Optional reference to QPC database (e.g., "89:27:3")
- `sample_rate`: Audio sample rate (16000, 22050, 44100)
- `duration_sec`: Audio duration in seconds
- `audio_path`: Relative path to audio file
- `created_at`: Timestamp

### regions
- `id`: Primary key
- `recording_id`: Foreign key to recordings (CASCADE DELETE)
- `start_sec`: Region start time (seconds)
- `end_sec`: Region end time (seconds)
- `label`: Label (e.g., "weak-ghunnah-onset")
- `confidence`: Optional confidence score (0-1)
- `notes`: Optional text notes
- `created_at`: Timestamp

## Data Sources

- **QPC Database**: `data/qpc-hafs-tajweed.db` (read-only)
  - 83,668 Quranic words with locations (surah:ayah:word)
  - Contains rule annotations in HTML tags
- **Annotation Database**: `backend/data/annotation.db` (created by tool)
  - Recordings and regions for training data

## Project Structure

```
iqrah-annotator/
├── CLAUDE.md                    # AI agent context (concise)
├── instructions.md              # Versioned implementation spec
├── ANNOTATION_TOOL_SPEC.md      # Full specification
├── README.md                    # This file
├── backend/
│   ├── app/
│   │   ├── main.py              # FastAPI application
│   │   ├── db/
│   │   │   ├── __init__.py      # Database connection
│   │   │   ├── models.py        # SQLAlchemy models
│   │   │   └── init_db.py       # DB initialization script
│   │   ├── api/
│   │   │   └── routes/
│   │   │       ├── recordings.py # Recording endpoints
│   │   │       ├── regions.py    # Region endpoints
│   │   │       └── export.py     # Export endpoints
│   │   └── core/
│   │       ├── schemas.py        # Pydantic models
│   │       └── utils.py          # Utility functions
│   ├── data/
│   │   ├── annotation.db         # SQLite database
│   │   └── audio/                # Audio files (organized by date)
│   ├── requirements.txt          # Python dependencies
│   ├── .env                      # Environment variables
│   ├── .env.example              # Environment template
│   ├── test_api.py               # API test suite
│   └── README.md                 # Backend README
└── data/
    └── qpc-hafs-tajweed.db       # QPC reference database
```

## Example Usage

### 1. Create Recording

```bash
curl -X POST http://localhost:8000/api/recordings \
  -H "Content-Type: application/json" \
  -d '{
    "rule": "ghunnah",
    "anti_pattern": "weak-ghunnah",
    "qpc_location": "89:27:3",
    "sample_rate": 16000,
    "duration_sec": 2.5
  }'
```

### 2. Upload Audio

```bash
curl -X POST http://localhost:8000/api/recordings/1/upload \
  -F "file=@audio.wav"
```

### 3. Add Annotation

```bash
curl -X POST http://localhost:8000/api/regions \
  -H "Content-Type: application/json" \
  -d '{
    "recording_id": 1,
    "start_sec": 0.5,
    "end_sec": 1.2,
    "label": "weak-ghunnah-onset",
    "confidence": 0.9,
    "notes": "Clear weak nasal resonance"
  }'
```

### 4. Export Data

```bash
curl http://localhost:8000/api/export/json?rule=ghunnah > export.json
```

## Labels Taxonomy

### Rules
- `ghunnah`: Nasal resonance
- `qalqalah`: Echoing/bouncing

### Anti-Patterns
- Ghunnah: `weak-ghunnah`, `no-ghunnah`
- Qalqalah: `no-qalqalah`, `weak-qalqalah`

### Region Labels
- Ghunnah: `weak-ghunnah-onset`, `weak-ghunnah-sustain`
- Qalqalah: `no-qalqalah`, `burst-misaligned`

## Validation

- Region boundaries: `0 <= start_sec < end_sec <= duration_sec`
- Sample rate: Must be one of `[16000, 22050, 44100]`
- Duration: Must be > 0
- File size: Max 50MB (configurable via `MAX_FILE_MB` env var)

## Testing

All core functionality has been tested:

- ✅ Health check
- ✅ Create recording
- ✅ Upload audio
- ✅ Create regions (multiple)
- ✅ List recordings
- ✅ Get recording
- ✅ Update region
- ✅ List with filters
- ✅ JSON export
- ✅ Delete region
- ✅ Delete recording (cascade)

Run tests with: `python test_api.py`

## Future Versions

### v0.2 (Quality & Workflow)
- Spectrogram overlay
- Simple search
- Import functionality
- Alembic migrations

### v0.3 (Data at Scale)
- Parquet export
- Surah/ayah/word helpers
- Stats caching

### v0.4 (Phoneme-Aware)
- M3 phoneme overlay
- Auto-suggest regions via validators

### v1.0 (Multi-User)
- User roles (student/expert/admin)
- Review queue
- Priority matrix
- Tag system

## Documentation

- [CLAUDE.md](CLAUDE.md) - Concise context for AI agents
- [instructions.md](instructions.md) - Versioned implementation guide
- [ANNOTATION_TOOL_SPEC.md](ANNOTATION_TOOL_SPEC.md) - Full system specification
- [backend/README.md](backend/README.md) - Backend-specific docs

## License

Part of the Iqrah Tajweed validation system.
