#!/bin/bash
set -e

# Test Memorization MVP - Interactive exercise with word energy updates

echo "=== Test Memorization MVP ==="

# Configuration
SERVER_URL="${SERVER_URL:-http://127.0.0.1:3000}"
CLI_BIN="${CLI_BIN:-./target/debug/iqrah}"
USER_ID="test_user"
VERSE_ID="VERSE:1:1"  # Al-Fatiha, first verse
WORD1_ID="WORD:1:1:1"  # First word
WORD2_ID="WORD:1:1:2"  # Second word

echo "Server: $SERVER_URL"
echo "User: $USER_ID"
echo "Verse: $VERSE_ID"
echo ""

# Step 1: Get initial energy for both words
echo "Step 1: Getting initial energy states..."
WORD1_INITIAL=$($CLI_BIN --server "$SERVER_URL" debug get-state "$USER_ID" "$WORD1_ID" 2>/dev/null || echo '{"energy": 0}')
WORD2_INITIAL=$($CLI_BIN --server "$SERVER_URL" debug get-state "$USER_ID" "$WORD2_ID" 2>/dev/null || echo '{"energy": 0}')

if command -v jq &> /dev/null; then
    WORD1_ENERGY_BEFORE=$(echo "$WORD1_INITIAL" | jq -r '.energy // 0')
    WORD2_ENERGY_BEFORE=$(echo "$WORD2_INITIAL" | jq -r '.energy // 0')
    echo "Word 1 initial energy: $WORD1_ENERGY_BEFORE"
    echo "Word 2 initial energy: $WORD2_ENERGY_BEFORE"
else
    WORD1_ENERGY_BEFORE="0"
    WORD2_ENERGY_BEFORE="0"
    echo "Note: jq not available, skipping detailed verification"
fi
echo ""

# Step 2: Run interactive memorization session
echo "Step 2: Running memorization session..."
echo "  - Tap word 1 twice (+0.10 total)"
echo "  - Tap word 2 once (+0.05 total)"

# Create a temporary file with commands (no session_id needed - auto-tracked by server)
COMMANDS_FILE=$(mktemp)
cat > "$COMMANDS_FILE" << 'COMMANDS'
{"type": "UpdateMemorizationWord", "word_node_id": "WORD:1:1:1", "action": "Tap"}
{"type": "UpdateMemorizationWord", "word_node_id": "WORD:1:1:1", "action": "Tap"}
{"type": "UpdateMemorizationWord", "word_node_id": "WORD:1:1:2", "action": "Tap"}
{"type": "EndSession"}
COMMANDS

# Run the exercise (timeout after 10 seconds if it hangs)
timeout 10 $CLI_BIN --server "$SERVER_URL" exercise run MemorizationAyah "$VERSE_ID" < "$COMMANDS_FILE" > /tmp/exercise_output.txt 2>&1 || true

# Display output
echo "Exercise output:"
cat /tmp/exercise_output.txt

# Clean up
rm "$COMMANDS_FILE"
echo ""

# Step 3: Get final energy for both words
echo "Step 3: Getting final energy states..."
WORD1_FINAL=$($CLI_BIN --server "$SERVER_URL" debug get-state "$USER_ID" "$WORD1_ID")
WORD2_FINAL=$($CLI_BIN --server "$SERVER_URL" debug get-state "$USER_ID" "$WORD2_ID")

if command -v jq &> /dev/null; then
    WORD1_ENERGY_AFTER=$(echo "$WORD1_FINAL" | jq -r '.energy // 0')
    WORD2_ENERGY_AFTER=$(echo "$WORD2_FINAL" | jq -r '.energy // 0')
    
    echo ""
    echo "=== Verification ==="
    echo "Word 1 energy: $WORD1_ENERGY_BEFORE -> $WORD1_ENERGY_AFTER"
    echo "Word 2 energy: $WORD2_ENERGY_BEFORE -> $WORD2_ENERGY_AFTER"
    
    # Calculate deltas
    WORD1_DELTA=$(echo "$WORD1_ENERGY_AFTER - $WORD1_ENERGY_BEFORE" | bc -l)
    WORD2_DELTA=$(echo "$WORD2_ENERGY_AFTER - $WORD2_ENERGY_BEFORE" | bc -l)
    
    echo "Word 1 delta: $WORD1_DELTA (expected: 0.10)"
    echo "Word 2 delta: $WORD2_DELTA (expected: 0.05)"
    
    # Verify word 1 increased by approximately 0.10
    if (( $(echo "$WORD1_DELTA >= 0.09 && $WORD1_DELTA <= 0.11" | bc -l) )); then
        echo "✓ Word 1 energy increased correctly"
    else
        echo "✗ Word 1 energy delta out of range: $WORD1_DELTA"
        exit 1
    fi
    
    # Verify word 2 increased by approximately 0.05
    if (( $(echo "$WORD2_DELTA >= 0.04 && $WORD2_DELTA <= 0.06" | bc -l) )); then
        echo "✓ Word 2 energy increased correctly"
    else
        echo "✗ Word 2 energy delta out of range: $WORD2_DELTA"
        exit 1
    fi
fi

echo ""
echo "=== Test PASSED ==="
