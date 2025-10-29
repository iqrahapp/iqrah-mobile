"""Main FastAPI application."""

import os
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.api.routes import recordings, regions, export, qpc, taxonomy

# Feature flags from environment
MOD_CORE = os.getenv("MOD_CORE", "true").lower() == "true"
MOD_EXPORT_JSON = os.getenv("MOD_EXPORT_JSON", "true").lower() == "true"

# Create FastAPI app
app = FastAPI(
    title="Tajweed Annotation Tool API",
    description="API for collecting Tajweed violation annotations",
    version="0.1.0"
)

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
    """Health check endpoint."""
    return {"status": "ok"}
