ALTER TABLE patients ADD COLUMN deleted_by_user_id TEXT;

ALTER TABLE appointments ADD COLUMN deleted_at TEXT;
ALTER TABLE appointments ADD COLUMN deleted_by_user_id TEXT;

ALTER TABLE inventory_items ADD COLUMN deleted_at TEXT;
ALTER TABLE inventory_items ADD COLUMN deleted_by_user_id TEXT;

CREATE INDEX IF NOT EXISTS idx_appointments_active_date
  ON appointments(clinic_id, starts_at, status)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_patients_active_lookup
  ON patients(clinic_id, full_name, phone, whatsapp, email)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_inventory_active_lookup
  ON inventory_items(clinic_id, active, name, category, unit)
  WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_inventory_unique_active
  ON inventory_items(clinic_id, lower(name), lower(category), lower(unit))
  WHERE active = 1 AND deleted_at IS NULL;
