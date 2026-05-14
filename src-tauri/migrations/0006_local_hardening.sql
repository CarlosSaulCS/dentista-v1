ALTER TABLE app_license ADD COLUMN status TEXT NOT NULL DEFAULT 'trial_active';
ALTER TABLE app_license ADD COLUMN access_mode TEXT NOT NULL DEFAULT 'full';
ALTER TABLE app_license ADD COLUMN last_check_at TEXT;
ALTER TABLE app_license ADD COLUMN next_check_at TEXT;
ALTER TABLE app_license ADD COLUMN device_id TEXT;
ALTER TABLE app_license ADD COLUMN installation_id TEXT;
ALTER TABLE app_license ADD COLUMN clinic_id TEXT;
ALTER TABLE app_license ADD COLUMN customer_id TEXT;
ALTER TABLE app_license ADD COLUMN subscription_id TEXT;
ALTER TABLE app_license ADD COLUMN plan_code TEXT NOT NULL DEFAULT 'local_trial';
ALTER TABLE app_license ADD COLUMN plan_limits_json TEXT;
ALTER TABLE app_license ADD COLUMN grace_period_ends_at TEXT;
ALTER TABLE app_license ADD COLUMN offline_grace_until TEXT;
ALTER TABLE app_license ADD COLUMN read_only_reason TEXT;

UPDATE app_license
SET
  status = CASE
    WHEN activated_at IS NOT NULL THEN 'active'
    WHEN trial_ends_at <= datetime('now') THEN 'expired'
    ELSE 'trial_active'
  END,
  access_mode = CASE
    WHEN activated_at IS NOT NULL THEN 'full'
    WHEN trial_ends_at <= datetime('now') THEN 'read_only'
    ELSE 'full'
  END,
  clinic_id = COALESCE(clinic_id, (SELECT id FROM clinics ORDER BY created_at LIMIT 1)),
  installation_id = COALESCE(installation_id, lower(hex(randomblob(16)))),
  device_id = COALESCE(device_id, lower(hex(randomblob(16)))),
  last_check_at = COALESCE(last_check_at, updated_at),
  next_check_at = COALESCE(next_check_at, datetime('now', '+1 day')),
  read_only_reason = CASE
    WHEN activated_at IS NULL AND trial_ends_at <= datetime('now') THEN 'trial_expired'
    ELSE read_only_reason
  END;

ALTER TABLE backups ADD COLUMN checksum_sha256 TEXT;
ALTER TABLE backups ADD COLUMN verification_status TEXT;
ALTER TABLE backups ADD COLUMN verified_at TEXT;
ALTER TABLE backups ADD COLUMN backup_type TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE backups ADD COLUMN app_version TEXT;
ALTER TABLE backups ADD COLUMN migration_version INTEGER;
ALTER TABLE backups ADD COLUMN file_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE backups ADD COLUMN table_counts_json TEXT;

CREATE TABLE IF NOT EXISTS backup_settings (
  clinic_id TEXT PRIMARY KEY REFERENCES clinics(id),
  automatic_enabled INTEGER NOT NULL DEFAULT 1,
  frequency TEXT NOT NULL DEFAULT 'daily',
  include_files INTEGER NOT NULL DEFAULT 1,
  encrypt_backups INTEGER NOT NULL DEFAULT 0,
  retention_limit INTEGER NOT NULL DEFAULT 30,
  updated_by_user_id TEXT REFERENCES users(id),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS restore_jobs (
  id TEXT PRIMARY KEY,
  clinic_id TEXT REFERENCES clinics(id),
  backup_id TEXT,
  source_path TEXT NOT NULL,
  staged_path TEXT NOT NULL,
  safety_backup_path TEXT,
  status TEXT NOT NULL,
  manifest TEXT,
  verification_json TEXT,
  prepared_by_user_id TEXT REFERENCES users(id),
  prepared_at TEXT NOT NULL,
  applied_at TEXT,
  error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_backups_status_created
  ON backups(clinic_id, status, created_at);

CREATE INDEX IF NOT EXISTS idx_restore_jobs_status
  ON restore_jobs(clinic_id, status, prepared_at);

INSERT OR IGNORE INTO permissions (id, code, name, module, description) VALUES
('perm-settings-edit', 'settings.edit', 'Editar configuración', 'settings', 'Permite modificar configuración del consultorio'),
('perm-files-manage', 'files.manage', 'Administrar archivos', 'files', 'Permite guardar y clasificar archivos clínicos'),
('perm-inventory-manage', 'inventory.manage', 'Administrar inventario', 'inventory', 'Permite modificar insumos y movimientos'),
('perm-alerts-manage', 'alerts.manage', 'Administrar alertas', 'alerts', 'Permite crear y resolver alertas'),
('perm-license-view', 'license.view', 'Ver licencia', 'license', 'Permite consultar estado de licencia');

INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.code IN (
  'settings.edit',
  'files.manage',
  'inventory.manage',
  'alerts.manage',
  'license.view'
)
WHERE r.system_key = 'administrator';
