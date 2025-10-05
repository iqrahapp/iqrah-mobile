#!/bin/bash
# Test Word-Level UI Implementation

echo "==================================="
echo "WORD-LEVEL UI TEST"
echo "==================================="

# Test 1: API Endpoints
echo -e "\n1. Testing Segments API..."
response=$(curl -s http://localhost:8000/api/segments/1/1 2>/dev/null)
if [ -n "$response" ]; then
    echo "✓ API /api/segments/1/1 is working"
    echo "$response" | python3 -c "import sys, json; data = json.load(sys.stdin); print(f\"  Surah: {data['surah']}, Ayah: {data['ayah']}\"); print(f\"  Words: {len(data['words'])}\"); print(f\"  Text: {data['text'][:50]}...\")" 2>/dev/null || echo "  Response received (JSON parsing issue)"
else
    echo "✗ API not responding"
fi

# Test 2: Coverage Stats
echo -e "\n2. Testing Coverage Stats..."
stats=$(curl -s http://localhost:8000/api/segments/stats 2>/dev/null)
if [ -n "$stats" ]; then
    echo "✓ API /api/segments/stats is working"
    echo "$stats" | python3 -m json.tool 2>/dev/null | head -6
else
    echo "✗ Stats API not responding"
fi

# Test 3: UI Files
echo -e "\n3. Checking UI Files..."
if [ -f "static/index.html" ]; then
    if grep -q "quran-display" static/index.html; then
        echo "✓ Word display section added to HTML"
    else
        echo "✗ Word display section missing"
    fi

    if grep -q "word.current" static/index.html; then
        echo "✓ Word CSS styles added"
    else
        echo "✗ Word CSS styles missing"
    fi
fi

if [ -f "static/app.js" ]; then
    if grep -q "WordLevelTracker" static/app.js; then
        echo "✓ WordLevelTracker class added to JS"
    else
        echo "✗ WordLevelTracker class missing"
    fi

    if grep -q "wordTracker.updateCurrentWord" static/app.js; then
        echo "✓ Word tracking integrated with WebSocket"
    else
        echo "✗ Word tracking integration missing"
    fi
fi

# Test 4: Server Status
echo -e "\n4. Server Status..."
if pgrep -f "python app.py" > /dev/null; then
    echo "✓ Server is running"
    echo "  URL: http://localhost:8000"
    echo ""
    echo "  Open in browser to test:"
    echo "  - Load Al-Fatiha 1:1"
    echo "  - Verify Arabic text displays word-by-word"
    echo "  - Click words to see their timing info"
else
    echo "✗ Server not running"
    echo "  Start with: source activate iqrah && python app.py"
fi

echo -e "\n==================================="
echo "TEST COMPLETE"
echo "==================================="
