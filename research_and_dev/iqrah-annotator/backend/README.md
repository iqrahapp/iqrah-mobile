# Tajweed Annotation Tool - Backend v0.1

FastAPI backend for Tajweed violation annotation tool.

## Quick Start

### 1. Install Dependencies

```bash
cd backend
pip install -r requirements.txt
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env if needed
```

### 3. Initialize Database

```bash
python -m app.db.init_db
```

### 4. Run Server

```bash
uvicorn app.main:app --reload --port 8000
```

API available at: http://localhost:8000
Docs available at: http://localhost:8000/docs

## API Endpoints

### Recordings
- `POST /api/recordings` - Create recording metadata
- `POST /api/recordings/{id}/upload` - Upload audio file
- `GET /api/recordings` - List recordings (with filters)
- `GET /api/recordings/{id}` - Get specific recording
- `PATCH /api/recordings/{id}` - Update recording
- `DELETE /api/recordings/{id}` - Delete recording (cascade)

### Regions (Annotations)
- `GET /api/recordings/{id}/regions` - Get regions for recording
- `POST /api/regions` - Create annotation region
- `PATCH /api/regions/{id}` - Update region
- `DELETE /api/regions/{id}` - Delete region

### Export
- `GET /api/export/json` - Export as JSON (with filters)

## Testing

See test examples in `../tests/` directory.

## Directory Structure

```
backend/
├── app/
│   ├── main.py              # FastAPI app
│   ├── db/
│   │   ├── __init__.py      # Database connection
│   │   ├── models.py        # SQLAlchemy models
│   │   └── init_db.py       # DB initialization
│   ├── api/
│   │   └── routes/          # API endpoints
│   │       ├── recordings.py
│   │       ├── regions.py
│   │       └── export.py
│   └── core/
│       ├── schemas.py       # Pydantic models
│       └── utils.py         # Utilities
├── requirements.txt
└── .env
```
