-- Initialize app_settings with schema version
-- This migration runs after the user schema is created

INSERT INTO app_settings (key, value) VALUES ('schema_version', '2');
