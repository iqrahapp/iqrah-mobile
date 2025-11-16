# iqrah-cli

CLI tool for testing and interacting with the Iqrah headless test server.

## Building

```bash
cargo build --package iqrah-cli --release
```

## Usage

```bash
iqrah [OPTIONS] <COMMAND>
```

### Options

- `-s, --server <URL>` - Server URL (default: http://127.0.0.1:3000)

### Commands

#### Debug Commands

```bash
# Get node metadata
iqrah debug get-node <NODE_ID>

# Get user memory state
iqrah debug get-state <USER_ID> <NODE_ID>

# Process a review
iqrah debug process-review <USER_ID> <NODE_ID> <GRADE>
```

Valid grades: `Again`, `Hard`, `Good`, `Easy`

#### Exercise Commands

```bash
# Run an interactive exercise via WebSocket
iqrah exercise run <EXERCISE_TYPE> <NODE_ID>
```

Reads JSON commands from stdin. Example:

```bash
echo '{"type": "UpdateMemorizationWord", "word_node_id": "WORD:1:1:1", "action": "Tap"}' | \
  iqrah exercise run MemorizationAyah VERSE:1:1
```

Or pipe multiple commands:

```bash
cat commands.json | iqrah exercise run MemorizationAyah VERSE:1:1
```

## Examples

```bash
# Check a node
iqrah debug get-node VERSE:1:1

# Get current state
iqrah debug get-state test_user VERSE:1:1

# Submit a review
iqrah debug process-review test_user VERSE:1:1 Good

# Run memorization exercise
cat << 'EOF' | iqrah exercise run MemorizationAyah VERSE:1:1
{"type": "UpdateMemorizationWord", "word_node_id": "WORD:1:1:1", "action": "Tap"}
{"type": "UpdateMemorizationWord", "word_node_id": "WORD:1:1:2", "action": "Tap"}
{"type": "EndSession"}
EOF
```
