#!/bin/bash

# Simple smoke test for the UI

echo "=== UI Smoke Test ==="
echo ""

# Check if server is running
echo "1. Checking if frontend server is running..."
if curl -s http://localhost:5173/ > /dev/null; then
    echo "✅ Frontend server is responding"
else
    echo "❌ Frontend server is NOT running"
    exit 1
fi

# Check if page loads
echo ""
echo "2. Checking if page loads..."
RESPONSE=$(curl -s http://localhost:5173/)
if echo "$RESPONSE" | grep -q '<div id="root"></div>'; then
    echo "✅ HTML structure is correct"
else
    echo "❌ HTML structure is broken"
    exit 1
fi

# Check if main module loads
echo ""
echo "3. Checking if main.tsx module loads..."
if curl -s http://localhost:5173/src/main.tsx | grep -q "import App"; then
    echo "✅ Main module loads"
else
    echo "❌ Main module failed to load"
    exit 1
fi

# Check if API client loads
echo ""
echo "4. Checking if API client module loads..."
if curl -s http://localhost:5173/src/api/client.ts | grep -q "export const"; then
    echo "✅ API client loads"
else
    echo "❌ API client failed to load"
    exit 1
fi

# Check if WaveformPlayer loads
echo ""
echo "5. Checking if WaveformPlayer loads..."
if curl -s http://localhost:5173/src/components/WaveformPlayer.tsx | grep -q "WaveSurfer"; then
    echo "✅ WaveformPlayer component loads"
else
    echo "❌ WaveformPlayer failed to load"
    exit 1
fi

# Check backend is running
echo ""
echo "6. Checking if backend API is running..."
if curl -s http://localhost:8000/ | grep -q '"status":"ok"'; then
    echo "✅ Backend API is responding"
else
    echo "❌ Backend API is NOT running"
    exit 1
fi

echo ""
echo "=== All Tests Passed! ==="
echo ""
echo "Frontend: http://localhost:5173"
echo "Backend:  http://localhost:8000"
echo ""
echo "You can now:"
echo "1. Open http://localhost:5173 in your browser"
echo "2. Press Ctrl+Shift+R to hard reload (clear cache)"
echo "3. Check the browser console (F12) - should have NO errors"
echo ""
