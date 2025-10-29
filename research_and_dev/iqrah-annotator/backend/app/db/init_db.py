"""Database initialization script."""

import sys
import os
from pathlib import Path

# Add parent directory to path so we can import app modules
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../..")))

# Load environment variables from .env file
from dotenv import load_dotenv
env_path = Path(__file__).parent.parent.parent / ".env"
load_dotenv(env_path)

from app.db.models import Base
from app.db import engine


def init_db():
    """Initialize database with tables."""
    # Ensure database directory exists
    db_url = str(engine.url)
    if db_url.startswith("sqlite:///"):
        db_path = Path(db_url.replace("sqlite:///", ""))
        db_path.parent.mkdir(parents=True, exist_ok=True)
        print(f"Database directory: {db_path.parent}")

    print("Creating database tables...")
    Base.metadata.create_all(bind=engine)
    print("✅ Database initialized successfully!")
    print(f"   Location: {engine.url}")
    print(f"   Tables: {', '.join(Base.metadata.tables.keys())}")


if __name__ == "__main__":
    init_db()
