"""Main FastAPI application."""

import os
from pathlib import Path
from dotenv import load_dotenv
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address
from slowapi.errors import RateLimitExceeded

from app.api.routes import recordings, regions, export, qpc, taxonomy

# Load environment variables from .env file
env_path = Path(__file__).parent.parent / ".env"
load_dotenv(env_path)

# Feature flags from environment
MOD_CORE = os.getenv("MOD_CORE", "true").lower() == "true"
MOD_EXPORT_JSON = os.getenv("MOD_EXPORT_JSON", "true").lower() == "true"

# Rate limiter
limiter = Limiter(key_func=get_remote_address)

# Create FastAPI app
app = FastAPI(
    title="Tajweed Annotation Tool API",
    description="API for collecting Tajweed violation annotations",
    version="0.1.0"
)

# Add rate limiter state
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# Add gzip compression middleware (compress responses > 1KB)
app.add_middleware(GZipMiddleware, minimum_size=1000)

# CORS configuration
allowed_origins = os.getenv("ALLOWED_ORIGINS", "http://localhost:5173").split(",")
app.add_middleware(
    CORSMiddleware,
    allow_origins=allowed_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Register routes (feature-flagged)
if MOD_CORE:
    app.include_router(recordings.router)
    app.include_router(regions.router)
    app.include_router(qpc.router)  # QPC database queries
    app.include_router(taxonomy.router)  # Taxonomy data

if MOD_EXPORT_JSON:
    app.include_router(export.router)


@app.get("/")
def root():
    """Health check endpoint."""
    return {
        "name": "Tajweed Annotation Tool API",
        "version": "0.1.0",
        "status": "ok",
        "features": {
            "core": MOD_CORE,
            "export_json": MOD_EXPORT_JSON,
        }
    }


@app.get("/health")
def health():
    """Health check endpoint with database status."""
    from app.db import SessionLocal

    health_status = {
        "status": "ok",
        "version": "0.1.0",
        "database": "unknown"
    }

    # Check database connection
    try:
        db = SessionLocal()
        db.execute("SELECT 1")
        db.close()
        health_status["database"] = "ok"
    except Exception as e:
        health_status["database"] = "error"
        health_status["database_error"] = str(e)
        health_status["status"] = "degraded"

    return health_status
