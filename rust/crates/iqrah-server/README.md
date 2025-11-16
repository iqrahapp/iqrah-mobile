# iqrah-server

Headless test server for the Iqrah learning system. Exposes iqrah-core through REST and WebSocket APIs.

## Building

```bash
cargo build --package iqrah-server --release
```

## Running

```bash
./target/release/iqrah-server
```

## REST API

- `GET /health` - Health check
- `GET /debug/node/:node_id` - Get node metadata
- `GET /debug/user/:user_id/state/:node_id` - Get memory state
- `POST /debug/user/:user_id/review` - Process review

## WebSocket API

Connect to `/ws` for interactive exercises.

Commands:
- `StartExercise` - Begin a new exercise session
- `UpdateMemorizationWord` - Update word energy (MVP)
- `EndSession` - Finish and save session

Session IDs are auto-tracked by the server.

## Environment Variables

- `CONTENT_DB_PATH` - Content database path (default: data/content.db)
- `USER_DB_PATH` - User database path (default: data/user.db)
- `BIND_ADDR` - Server address (default: 127.0.0.1:3000)
