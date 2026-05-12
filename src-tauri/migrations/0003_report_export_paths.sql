ALTER TABLE report_exports ADD COLUMN path TEXT;
ALTER TABLE report_exports ADD COLUMN size_bytes INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_report_exports_created
  ON report_exports(clinic_id, report_type, format, created_at);

CREATE INDEX IF NOT EXISTS idx_treatment_catalog_lookup
  ON treatment_catalog(clinic_id, active, name, category);
