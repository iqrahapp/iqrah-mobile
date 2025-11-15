#!/bin/bash
# Simpler dev script (assumes conda env already activated)
# Usage: conda activate iqrah && ./dev-simple.sh

trap 'kill $(jobs -p) 2>/dev/null; exit' INT TERM

echo "🚀 Starting services..."

(cd backend && uvicorn app.main:app --reload --port 8000) &
(cd frontend && npm run dev) &

echo "✅ Running at http://localhost:8000 (backend) and http://localhost:5173+ (frontend)"
echo "Press Ctrl+C to stop"

wait
