# Tajweed Annotation Tool — Versioned Reference Spec

## 0) Guiding Principles

* **Local-first, FastAPI + SQLite** baseline; add services only when needed.
* **Feature-flagged modules** so advanced features can land early without breaking MVP.
* **Append-only schema evolution** (no breaking changes); Alembic from v0.1.1+.
* **Single source of truth for labels/rules**; keep taxonomy small at first, extensible later.
* **Exports are contracts** (JSON v0.1 → Parquet v0.3) with version fields.

---

## 1) Targets by Version

### v0.1 (MVP, solo/local-first)

Core goal: quickly collect annotated data for **ghunnah** and **qalqalah**.

**Features**

* Create/list/delete recordings
* Upload audio (webm/wav) → stored on disk
* Regions/labels CRUD (frame-ish: seconds float)
* JSON export (filter by rule/anti_pattern/date)
* CORS for localhost frontends

**Non-goals** (postpone): roles, dashboards, tags, Celery/Redis, Parquet, priority matrix.

---

### v0.2 (Quality & light workflow)

* **Spectrogram overlay** stored (optional; generated client-side or on upload)
* **Simple search** (`qpc_location`, label contains)
* **“Library” filters** (rule / anti_pattern)
* **Basic import** (re-attach regions to an audio file)
* **Alembic migrations** enabled

---

### v0.3 (Data at scale)

* **Parquet export** (PyArrow), JSON remains
* **Surah/ayah/word** helper endpoints
* **Light stats** (counts per label/rule), cached table
* **Optional**: background tasks via Celery/Redis (for heavy exports)

---

### v0.4 (Phoneme-aware)

* **M3 phoneme overlay** storage (start/end/phoneme/sifat)
* **Auto-suggest regions** via validators (ghunnah/qaqlqalah/madd)
* **Confidence/rationale** fields for suggestions

---

### v1.0 (Multi-user + Ops)

* Roles: student/expert/admin
* Review queue & verification states
* Priority matrix + surah coverage
* Tag system (AND logic)
* Label-Studio/Praat/Audacity import/export bridges

---

## 2) Module Map & Feature Flags

| Module          | Key Entities        | Flag (env)             | First Version |
| --------------- | ------------------- | ---------------------- | ------------- |
| Core Recordings | recordings, regions | `MOD_CORE=true`        | v0.1          |
| Export JSON     | export/json         | `MOD_EXPORT_JSON=true` | v0.1          |
| Search          | simple filters      | `MOD_SEARCH=true`      | v0.2          |
| Spectrogram     | spectro blobs/paths | `MOD_SPECTRO=false`    | v0.2          |
| Parquet         | export/parquet      | `MOD_PARQUET=false`    | v0.3          |
| Light Stats     | stats cache         | `MOD_STATS=false`      | v0.3          |
| M3 Overlay      | phonemes table      | `MOD_M3=false`         | v0.4          |
| Roles/Review    | users, verification | `MOD_ROLES=false`      | v1.0          |
| Tags/Priority   | tags, surah_stats   | `MOD_TAGS=false`       | v1.0          |

> AI Agent: respect flags; don’t ship code paths for disabled modules.

---

## 3) Data Model (evolutionary)

### v0.1 — Minimal schema (SQLite)

```sql
CREATE TABLE IF NOT EXISTS recordings (
  id INTEGER PRIMARY KEY,
  rule TEXT NOT NULL,            -- "ghunnah" | "qalqalah"
  anti_pattern TEXT NOT NULL,    -- e.g. "weak-ghunnah" | "no-qalqalah"
  qpc_location TEXT,             -- "89:27:3" (optional)
  sample_rate INTEGER NOT NULL,
  duration_sec REAL NOT NULL,
  audio_path TEXT NOT NULL,      -- relative path
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS regions (
  id INTEGER PRIMARY KEY,
  recording_id INTEGER NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
  start_sec REAL NOT NULL,
  end_sec REAL NOT NULL,
  label TEXT NOT NULL,           -- e.g. "weak-ghunnah-onset"
  confidence REAL,               -- 0..1
  notes TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_regions_recording_id ON regions(recording_id);
CREATE INDEX IF NOT EXISTS idx_recordings_rule_ap ON recordings(rule, anti_pattern);
```

#### v0.2 additions

```sql
-- Optional spectrogram path (PNG/NPY) if generated server-side
ALTER TABLE recordings ADD COLUMN spectrogram_path TEXT;
-- Simple search helpers (optional indexes)
CREATE INDEX IF NOT EXISTS idx_recordings_qpc ON recordings(qpc_location);
```

#### v0.3 additions

```sql
-- Light stats cache
CREATE TABLE IF NOT EXISTS stats (
  rule TEXT,
  anti_pattern TEXT,
  count_total INTEGER DEFAULT 0,
  last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (rule, anti_pattern)
);
```

#### v0.4 additions (phoneme overlay)

```sql
CREATE TABLE IF NOT EXISTS phonemes (
  id INTEGER PRIMARY KEY,
  recording_id INTEGER NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
  start_sec REAL NOT NULL,
  end_sec REAL NOT NULL,
  symbol TEXT NOT NULL,          -- e.g. "ن"
  sifat_json TEXT                -- JSON blob for sifat/features
);
CREATE INDEX IF NOT EXISTS idx_phonemes_recording_id ON phonemes(recording_id);
```

#### v1.0 additions (roles/tags/priority)

*(kept concise—enable when needed; mirrors your original spec)*

```sql
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  email TEXT UNIQUE,
  role TEXT NOT NULL,            -- "student" | "expert" | "admin"
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE recordings ADD COLUMN status TEXT DEFAULT 'draft';     -- 'draft'|'submitted'|'verified'|'rejected'
ALTER TABLE recordings ADD COLUMN verified_by INTEGER REFERENCES users(id);

CREATE TABLE IF NOT EXISTS tags (
  id INTEGER PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  category TEXT,                 -- 'surah'|'pattern'|'priority'|'custom'
  description TEXT
);

CREATE TABLE IF NOT EXISTS recording_tags (
  recording_id INTEGER NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
  tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY (recording_id, tag_id)
);

-- Priority/coverage (surah-level)
CREATE TABLE IF NOT EXISTS surah_stats (
  surah INTEGER,
  rule TEXT,
  count_words INTEGER DEFAULT 0,
  count_recorded INTEGER DEFAULT 0,
  count_verified INTEGER DEFAULT 0,
  coverage REAL DEFAULT 0.0,
  priority_score REAL DEFAULT 0.0,
  last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (surah, rule)
);
```

---

## 4) API Surface (by stability)

### Stable (v0.1)

**Recordings**

* `POST   /api/recordings` → create metadata
* `POST   /api/recordings/{id}/upload` (multipart) → saves file; optional webm→wav
* `GET    /api/recordings?rule=&anti_pattern=&qpc_location=`
* `GET    /api/recordings/{id}`
* `DELETE /api/recordings/{id}` (cascade + delete audio file)

**Regions**

* `GET    /api/recordings/{id}/regions`
* `POST   /api/regions`
* `PATCH  /api/regions/{id}`
* `DELETE /api/regions/{id}`

**Export**

* `GET    /api/export/json?rule=&anti_pattern=&from=&to=`

**Validation**

* `0 <= start < end <= duration`, `duration > 0`, `sample_rate ∈ {16k, 22.05k, 44.1k}`

### Experimental (behind flags)

* v0.2: `GET /api/search?q=...` (or extend list filters)
* v0.3: `GET /api/export/parquet?...`
* v0.4: `GET /api/recordings/{id}/phonemes`
* v1.0: `GET/POST /api/tags`, `GET /api/stats`, `GET /api/review`, roles endpoints

---

## 5) Export Contracts

### JSON (v0.1+)

```json
{
  "version": "0.1",
  "export_date": "2025-10-28T12:00:00Z",
  "recordings": [
    {
      "id": 123,
      "rule": "ghunnah",
      "anti_pattern": "weak-ghunnah",
      "qpc_location": "89:27:3",
      "sample_rate": 16000,
      "duration_sec": 2.8,
      "audio_path": "audio/2025-10-28/123.wav",
      "regions": [
        { "start_sec": 1.12, "end_sec": 1.46, "label": "weak-ghunnah-onset", "confidence": 0.9, "notes": "" }
      ]
    }
  ]
}
```

### Parquet (v0.3+)

Columns (flattened):

```
recording_id:int64 | audio_path:str | duration_sec:float64 | sample_rate:int32 |
rule:str | anti_pattern:str | qpc_location:str |
region_id:int64 | ann_start_sec:float64 | ann_end_sec:float64 |
ann_label:str | ann_confidence:float64 | ann_notes:str
```

---

## 6) Labels & Taxonomy (start tiny; expandable)

**Rules**

* `ghunnah`, `qalqalah`, `madd` (optional in v0.1)

**Anti-patterns (keep ≤2 per rule in v0.1)**

* ghunnah: `weak-ghunnah`, `no-ghunnah` *(add `over-ghunnah` later)*
* qalqalah: `no-qalqalah`, `weak-qalqalah`
* madd (later): `short-madd`, `over-madd`

**Region labels**

* ghunnah: `weak-ghunnah-onset`, `weak-ghunnah-sustain`
* qalqalah: `no-qalqalah`, `burst-misaligned`

> Store as strings in v0.1; normalize into tables when v1.0 roles/tags arrive.

---

## 7) Frontend (reference notes; not required to build now)

* **WaveSurfer v7** + **Regions** + **Spectrogram** + (optional) **Timeline**
* **MediaRecorder** for capture (webm); server can transcode to wav
* One page:

  * selects `rule`, `anti_pattern`, optional `qpc_location`
  * **Record / Stop** → visualize waveform + spectrogram
  * drag to create **region**, pick **label**, add note
  * **Save** calls API; **Export JSON** downloads manifest

**Spectrogram params (good defaults)**

* `fftSamples=2048`, `frequencyScale='log'`
* Show 0–4 kHz to highlight ghunnah & qalqalah areas

---

## 8) Integration Hooks (future)

* **M3 overlay (v0.4)**: `POST /api/recordings/{id}/phonemes` to attach alignment results.
* **Auto-suggest (v0.4)**: `POST /api/recordings/{id}/suggest` returns candidate regions with confidences.
* **Tool bridges (v1.0)**:

  * Praat TextGrid import/export
  * Audacity label track export
  * Label-Studio JSON import/export

---

## 9) Priority/Stats (when multi-user arrives)

**Pattern priority score** (keep from your spec)

```py
def priority(verified:int, target:int, severity:str)->float:
    w = {"critical":1.0,"moderate":0.7,"minor":0.4}.get(severity,0.5)
    cov = verified/target if target else 0.0
    gap = 1.0 - cov
    return round((gap ** 2) * w, 2)
```

**Surah priority** (coverage × length factor) — unchanged.

---

## 10) Operational Notes

* **Paths**: store **relative** `audio_path`; base dir from `.env` (`AUDIO_DIR=./data/audio`)
* **Atomic writes**: save to temp, fsync, then rename
* **Delete**: DB delete must remove file safely; ignore if missing
* **Size limits**: reject > 50MB uploads (configurable)
* **Time**: UTC timestamps
* **Testing (v0.1)**

  * region validation
  * export contains regions & correct paths
  * delete cascade removes file
* **Migrations**: enable Alembic in v0.1.1

---

## 11) AI Agent Implementation Rules

* **Respect feature flags**; ship guarded code paths.
* **Keep handlers small (<100 LOC)**; move utilities to `core/`.
* **Use Pydantic v2 models** for all I/O; never return raw ORM.
* **SQLite threading**: use one session per request; avoid async DB for now.
* **Exports are contracts**: version field **must** be set and preserved.
* **Commit messages**: `feat(core):`, `feat(export):`, `chore(db):`, `fix(api):`
* **Add unit tests** for each new route or schema change.

---

## 12) Quickstart (backend)

```
pip install -r backend/requirements.txt
cp backend/.env.example backend/.env
python -m app.db.init_db
uvicorn app.main:app --reload
```

**Env**

```
DATABASE_URL=sqlite:///./data/annotation.db
AUDIO_DIR=./data/audio
ALLOWED_ORIGINS=http://localhost:5173
MAX_FILE_MB=50
MOD_CORE=true
MOD_EXPORT_JSON=true
MOD_SEARCH=true
MOD_SPECTRO=false
MOD_PARQUET=false
MOD_STATS=false
MOD_M3=false
MOD_ROLES=false
MOD_TAGS=false
```