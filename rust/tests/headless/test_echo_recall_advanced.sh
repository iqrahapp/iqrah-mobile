#!/bin/bash
set -e # Exit on error

echo "--- STARTING ECHO RECALL ADVANCED CONTEXT TEST ---"

# Configuration
SERVER_URL="${SERVER_URL:-http://127.0.0.1:3000}"
CLI_BIN="${CLI_BIN:-./target/debug/iqrah}"
USER_ID="test_user_context"

# SETUP: We need three contiguous words in a verse. Let's use Al-Fatiha: 1:2
# VERSE:1:2 contains the words "ٱلرَّحْمَـٰنِ ٱلرَّحِيمِ" (The Most Gracious, the Most Merciful)
AYAH2_FIRST_WORD="WORD:1:2:1"  # ٱلرَّحْمَـٰنِ
AYAH2_SECOND_WORD="WORD:1:2:2"  # ٱلرَّحِيمِ

echo "Server: $SERVER_URL"
echo "User: $USER_ID"
echo ""

# Ensure a known starting state: Master the first word, forget the second word
echo "Setting up initial energy states..."
$CLI_BIN --server "$SERVER_URL" debug set-state "$USER_ID" "$AYAH2_FIRST_WORD" --energy 0.9 > /dev/null 2>&1
$CLI_BIN --server "$SERVER_URL" debug set-state "$USER_ID" "$AYAH2_SECOND_WORD" --energy 0.0 > /dev/null 2>&1
echo "✓ Initial states set"
echo ""

# 1. START SESSION for the ayah
echo "Step 1: Starting Echo Recall session for Ayah 2..."
SESSION_JSON=$($CLI_BIN --server "$SERVER_URL" exercise start echo-recall VERSE:1:2)
echo "$SESSION_JSON" | head -20

# Extract session ID if jq is available
if command -v jq &> /dev/null; then
    SESSION_ID=$(echo "$SESSION_JSON" | jq -r '.session_id')
    echo "Session ID: $SESSION_ID"

    # ASSERT 1: The first word should be hidden (energy >= 0.85 is actually only 0.9, but should be visible based on logic)
    # Actually, with energy=0.9, it should be Hidden. The second word with energy=0.0 should be Visible
    SECOND_WORD_VIS=$(echo "$SESSION_JSON" | jq -r '.initial_state.words[1].visibility.type')
    echo "Second word visibility type: $SECOND_WORD_VIS"

    if [ "$SECOND_WORD_VIS" = "Visible" ]; then
        echo "✅ PASSED: Second word is Visible (energy=0.0 < 0.15)"
    else
        echo "❌ FAILED: Second word should be Visible, got: $SECOND_WORD_VIS"
        exit 1
    fi

    # Get the initial hint type for words if any are obscured
    # Check if there are any obscured words in the result
    echo ""
    echo "Initial word visibility states:"
    echo "$SESSION_JSON" | jq -r '.initial_state.words[] | "\(.node_id): \(.visibility.type) (energy: \(.energy))"'
else
    # Without jq, just extract session_id with grep
    SESSION_ID=$(echo "$SESSION_JSON" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
    echo "Session ID: $SESSION_ID"
    echo "Note: jq not available, skipping detailed verification"
fi
echo ""

# 2. SUBMIT ACTIONS: Now, let's strengthen the second word to make it an anchor
echo "Step 2: Learning the second word with fast recalls to create an anchor..."
echo "  Submitting 5 fast recalls (500ms each) to increase energy..."

for i in {1..5}; do
    echo "  Recall $i/5..."
    LATEST_STATE=$($CLI_BIN --server "$SERVER_URL" exercise action echo-recall "$SESSION_ID" "$AYAH2_SECOND_WORD" 500)
done

echo "✓ Completed 5 recalls"
echo ""

# ASSERT 2: Check the final energy of the second word
if command -v jq &> /dev/null; then
    # Check the final state
    echo "Step 3: Checking final state..."
    FINAL_SECOND_ENERGY=$(echo "$LATEST_STATE" | jq -r '.new_state.words[1].energy')
    echo "Second word final energy: $FINAL_SECOND_ENERGY"

    # Each fast recall (500ms) should give approximately +0.1 energy (close to optimal 700ms)
    # After 5 recalls: ~0.0 + 5*0.1 = 0.5 energy
    # Check that energy > 0.3 (anchor threshold)
    if (( $(echo "$FINAL_SECOND_ENERGY > 0.3" | bc -l) )); then
        echo "✅ PASSED: Second word energy increased to anchor level (>0.3): $FINAL_SECOND_ENERGY"
    else
        echo "❌ FAILED: Second word energy should be >0.3, got: $FINAL_SECOND_ENERGY"
        exit 1
    fi

    # Show all word states
    echo ""
    echo "Final word states:"
    echo "$LATEST_STATE" | jq -r '.new_state.words[] | "\(.node_id): \(.visibility.type) (energy: \(.energy))"'
fi
echo ""

# 3. END SESSION
echo "Step 4: Ending session..."
END_RESULT=$($CLI_BIN --server "$SERVER_URL" exercise end "$SESSION_ID")
echo "✓ Session ended"
echo ""

# 4. FINAL ASSERTION: Check that the second word's energy was persisted correctly
echo "Step 5: Verifying persistence..."
FINAL_STATE=$($CLI_BIN --server "$SERVER_URL" debug get-state "$USER_ID" "$AYAH2_SECOND_WORD")

if command -v jq &> /dev/null; then
    PERSISTED_ENERGY=$(echo "$FINAL_STATE" | jq -r '.energy')
    echo "Persisted energy for second word: $PERSISTED_ENERGY"

    if (( $(echo "$PERSISTED_ENERGY > 0.4" | bc -l) )); then
        echo "✅ PASSED: Final energy for second word persisted correctly (>0.4): $PERSISTED_ENERGY"
    else
        echo "❌ FAILED: Persisted energy should be >0.4, got: $PERSISTED_ENERGY"
        exit 1
    fi
fi

echo ""
echo "--- ALL TESTS PASSED ---"
