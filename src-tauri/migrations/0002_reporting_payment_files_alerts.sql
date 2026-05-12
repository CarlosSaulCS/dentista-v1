PRAGMA foreign_keys = ON;

ALTER TABLE files ADD COLUMN related_entity_type TEXT;
ALTER TABLE files ADD COLUMN related_entity_id TEXT;

CREATE INDEX IF NOT EXISTS idx_files_related_entity ON files(clinic_id, related_entity_type, related_entity_id);

CREATE TABLE IF NOT EXISTS system_alert_state (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  alert_type TEXT NOT NULL,
  related_entity_type TEXT NOT NULL,
  related_entity_id TEXT NOT NULL,
  last_generated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, alert_type, related_entity_type, related_entity_id)
);
