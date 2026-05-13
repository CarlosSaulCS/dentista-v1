CREATE TABLE IF NOT EXISTS app_license (
  id TEXT PRIMARY KEY CHECK (id = 'local'),
  trial_started_at TEXT NOT NULL,
  trial_ends_at TEXT NOT NULL,
  activated_at TEXT,
  activated_by_user_id TEXT REFERENCES users(id),
  activation_fingerprint TEXT,
  updated_at TEXT NOT NULL
);

INSERT INTO app_license (id, trial_started_at, trial_ends_at, updated_at)
SELECT
  'local',
  COALESCE((SELECT created_at FROM clinics ORDER BY created_at LIMIT 1), datetime('now')),
  datetime(COALESCE((SELECT created_at FROM clinics ORDER BY created_at LIMIT 1), datetime('now')), '+30 days'),
  datetime('now')
WHERE EXISTS (SELECT 1 FROM clinics)
  AND NOT EXISTS (SELECT 1 FROM app_license WHERE id = 'local');

CREATE INDEX IF NOT EXISTS idx_app_license_trial_window
  ON app_license (trial_ends_at, activated_at);
