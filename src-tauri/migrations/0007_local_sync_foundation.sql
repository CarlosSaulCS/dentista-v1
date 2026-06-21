CREATE TABLE IF NOT EXISTS sync_device_tokens (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  installation_id TEXT NOT NULL,
  device_id TEXT NOT NULL,
  device_label TEXT,
  portal_base_url TEXT,
  access_token TEXT,
  access_token_expires_at TEXT,
  refresh_token TEXT,
  refresh_token_expires_at TEXT,
  status TEXT NOT NULL DEFAULT 'pending_pairing',
  last_registered_at TEXT,
  last_refreshed_at TEXT,
  last_sync_at TEXT,
  last_error TEXT,
  revoked_at TEXT,
  revoked_reason TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, installation_id, device_id),
  CHECK (status IN ('pending_pairing', 'active', 'revoked', 'error'))
);

CREATE TABLE IF NOT EXISTS sync_outbox (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  aggregate_type TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  occurred_at TEXT NOT NULL,
  available_at TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'pending',
  last_attempt_at TEXT,
  synced_at TEXT,
  remote_ack_id TEXT,
  error_message TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  CHECK (status IN ('pending', 'in_flight', 'synced', 'failed', 'dead'))
);

CREATE TABLE IF NOT EXISTS sync_inbox (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  command_id TEXT NOT NULL,
  command_type TEXT NOT NULL,
  aggregate_type TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  requested_by_json TEXT,
  received_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  applied_at TEXT,
  failed_at TEXT,
  error_message TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, command_id),
  CHECK (status IN ('pending', 'applied', 'failed', 'ignored'))
);

CREATE TABLE IF NOT EXISTS sync_cursor (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  cursor_type TEXT NOT NULL,
  cursor_value TEXT,
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, cursor_type)
);

CREATE TABLE IF NOT EXISTS remote_command_receipts (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  command_id TEXT NOT NULL,
  command_type TEXT NOT NULL,
  appointment_id TEXT,
  status TEXT NOT NULL,
  result_json TEXT,
  error_message TEXT,
  received_at TEXT NOT NULL,
  applied_at TEXT,
  acked_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, command_id),
  CHECK (status IN ('applied', 'failed', 'ignored'))
);

CREATE INDEX IF NOT EXISTS idx_sync_device_tokens_clinic_status
  ON sync_device_tokens (clinic_id, status, updated_at);

CREATE INDEX IF NOT EXISTS idx_sync_outbox_pending
  ON sync_outbox (clinic_id, status, available_at, occurred_at);

CREATE INDEX IF NOT EXISTS idx_sync_outbox_aggregate
  ON sync_outbox (clinic_id, aggregate_type, aggregate_id, occurred_at);

CREATE INDEX IF NOT EXISTS idx_sync_inbox_pending
  ON sync_inbox (clinic_id, status, received_at);

CREATE INDEX IF NOT EXISTS idx_remote_command_receipts_pending_ack
  ON remote_command_receipts (clinic_id, acked_at, created_at)
  WHERE acked_at IS NULL;

INSERT OR IGNORE INTO permissions (id, code, name, module, description) VALUES
('perm-sync-view', 'sync.view', 'Ver sincronización', 'sync', 'Permite consultar estado de sincronización local'),
('perm-sync-manage', 'sync.manage', 'Administrar sincronización', 'sync', 'Permite vincular, sincronizar y revocar dispositivos locales');

INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.code IN ('sync.view', 'sync.manage')
WHERE r.system_key = 'administrator';
