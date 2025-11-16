#!/bin/bash
set -e

# Test MCQ Flow - Process a review and verify state change

echo "=== Test MCQ Flow ==="

# Configuration
SERVER_URL="${SERVER_URL:-http://127.0.0.1:3000}"
CLI_BIN="${CLI_BIN:-./target/debug/iqrah}"
USER_ID="test_user"
NODE_ID="VERSE:1:1"  # Al-Fatiha, first verse
GRADE="Good"

echo "Server: $SERVER_URL"
echo "User: $USER_ID"
echo "Node: $NODE_ID"
echo ""

# Step 1: Get initial state
echo "Step 1: Getting initial state..."
INITIAL_STATE=$($CLI_BIN --server "$SERVER_URL" debug get-state "$USER_ID" "$NODE_ID")
echo "$INITIAL_STATE"

# Extract initial energy (using jq if available, otherwise grep)
if command -v jq &> /dev/null; then
    INITIAL_ENERGY=$(echo "$INITIAL_STATE" | jq -r '.energy // 0')
    INITIAL_REVIEW_COUNT=$(echo "$INITIAL_STATE" | jq -r '.review_count // 0')
    echo "Initial energy: $INITIAL_ENERGY"
    echo "Initial review count: $INITIAL_REVIEW_COUNT"
else
    echo "Note: jq not available, skipping detailed energy check"
    INITIAL_ENERGY="0"
    INITIAL_REVIEW_COUNT="0"
fi
echo ""

# Step 2: Process a review with "Good" grade
echo "Step 2: Processing review with grade '$GRADE'..."
REVIEW_RESULT=$($CLI_BIN --server "$SERVER_URL" debug process-review "$USER_ID" "$NODE_ID" "$GRADE")
echo "$REVIEW_RESULT"
echo ""

# Step 3: Get final state
echo "Step 3: Getting final state..."
FINAL_STATE=$($CLI_BIN --server "$SERVER_URL" debug get-state "$USER_ID" "$NODE_ID")
echo "$FINAL_STATE"

# Verify state changes
if command -v jq &> /dev/null; then
    FINAL_ENERGY=$(echo "$FINAL_STATE" | jq -r '.energy')
    FINAL_REVIEW_COUNT=$(echo "$FINAL_STATE" | jq -r '.review_count')
    FINAL_STABILITY=$(echo "$FINAL_STATE" | jq -r '.stability')
    
    echo ""
    echo "=== Verification ==="
    echo "Initial energy: $INITIAL_ENERGY"
    echo "Final energy: $FINAL_ENERGY"
    echo "Initial review count: $INITIAL_REVIEW_COUNT"
    echo "Final review count: $FINAL_REVIEW_COUNT"
    echo "Final stability: $FINAL_STABILITY"
    
    # Check that energy increased
    if (( $(echo "$FINAL_ENERGY > $INITIAL_ENERGY" | bc -l) )); then
        echo "✓ Energy increased"
    else
        echo "✗ Energy did not increase"
        exit 1
    fi
    
    # Check that review count increased
    if [ "$FINAL_REVIEW_COUNT" -gt "$INITIAL_REVIEW_COUNT" ]; then
        echo "✓ Review count increased"
    else
        echo "✗ Review count did not increase"
        exit 1
    fi
    
    # Check that stability is positive
    if (( $(echo "$FINAL_STABILITY > 0" | bc -l) )); then
        echo "✓ Stability is positive"
    else
        echo "✗ Stability is not positive"
        exit 1
    fi
fi

echo ""
echo "=== Test PASSED ==="
