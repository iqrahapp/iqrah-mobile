# iqrah-backend-storage

Storage layer for the Iqrah backend, providing PostgreSQL repositories for users, packs, and sync operations.

## Testing

This crate uses `sqlx::test` macro for integration tests, which requires a PostgreSQL database to run.

### Prerequisites

1. **PostgreSQL Server**: You need a running PostgreSQL instance
2. **Test Database**: A database for running tests (will be created automatically by sqlx::test)
3. **DATABASE_URL Environment Variable**: Must be set before running tests

### Setting Up Tests

```bash
# Option 1: Set DATABASE_URL for a local PostgreSQL instance
export DATABASE_URL="postgresql://postgres:password@localhost/iqrah_test"

# Option 2: Use a temporary PostgreSQL instance (Docker)
docker run --name postgres-test -e POSTGRES_PASSWORD=testpass -p 5432:5432 -d postgres:15
export DATABASE_URL="postgresql://postgres:testpass@localhost/iqrah_test"

# Run tests
cargo test --package iqrah-backend-storage

# Or run all backend tests
cd backend
cargo test --all-features
```

### How sqlx::test Works

The `#[sqlx::test(migrations = "../../migrations")]` macro:
1. Creates a fresh test database for each test function
2. Runs all migrations from the specified directory
3. Provides an isolated `PgPool` to the test
4. Cleans up after the test completes

This ensures tests are:
- **Isolated**: Each test gets a clean database
- **Repeatable**: No state leakage between tests
- **Migration-validated**: Tests run against the actual schema

### CI/CD Setup

For continuous integration, ensure `DATABASE_URL` is configured in your CI environment:

```yaml
# Example GitHub Actions
env:
  DATABASE_URL: postgresql://postgres:postgres@localhost/test_db

services:
  postgres:
    image: postgres:15
    env:
      POSTGRES_PASSWORD: postgres
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
```

### Without DATABASE_URL

If `DATABASE_URL` is not set, tests will fail with:
```
error: DATABASE_URL environment variable required
```

This is intentional - storage tests require a real database to verify SQL queries and data integrity.

## Repository Structure

- `pack/` - Content pack storage and retrieval
- `sync/` - Multi-device sync with last-write-wins (LWW)
- `user/` - User and device management
