#!/bin/bash
# Simple dev script to run backend + frontend together
# Usage: ./dev.sh
# Kill with Ctrl+C (kills both processes)

set -e

# Trap Ctrl+C and kill all background jobs
trap 'echo "Stopping..."; kill $(jobs -p) 2>/dev/null; exit' INT TERM

echo "🚀 Starting Iqrah Annotator (dev mode)..."
echo ""

# Start backend
echo "📡 Starting backend (uvicorn)..."
cd backend
conda run -n iqrah uvicorn app.main:app --reload --port 8000 &
BACKEND_PID=$!
cd ..

# Wait a bit for backend to start
sleep 2

# Start frontend
echo "🎨 Starting frontend (vite)..."
cd frontend
npm run dev &
FRONTEND_PID=$!
cd ..

echo ""
echo "✅ Services running:"
echo "   Backend:  http://localhost:8000 (PID $BACKEND_PID)"
echo "   Frontend: http://localhost:5173+ (PID $FRONTEND_PID)"
echo ""
echo "Press Ctrl+C to stop both services"

# Wait for all background jobs
wait
