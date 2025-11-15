-- Migration v2: Add app settings table (proves migration harness works)

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

-- Insert default settings
INSERT INTO app_settings (key, value) VALUES
    ('schema_version', '2'),
    ('migration_date', CAST(strftime('%s', 'now') AS TEXT));
