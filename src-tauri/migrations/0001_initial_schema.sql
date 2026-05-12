PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS clinics (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  subtitle TEXT NOT NULL DEFAULT 'Sistema Integral para Consultorio Dental',
  phone TEXT,
  whatsapp TEXT,
  email TEXT,
  address TEXT,
  tax_data TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS permissions (
  id TEXT PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  module TEXT NOT NULL,
  description TEXT
);

CREATE TABLE IF NOT EXISTS roles (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  name TEXT NOT NULL,
  system_key TEXT NOT NULL,
  description TEXT,
  is_system INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, system_key)
);

CREATE TABLE IF NOT EXISTS role_permissions (
  role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission_id TEXT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
  PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  role_id TEXT REFERENCES roles(id),
  full_name TEXT NOT NULL,
  username TEXT NOT NULL UNIQUE,
  email TEXT UNIQUE,
  password_hash TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  professional_license TEXT,
  specialty TEXT,
  last_login_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS user_permissions (
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  permission_id TEXT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
  allowed INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (user_id, permission_id)
);

CREATE TABLE IF NOT EXISTS user_sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id),
  token_hash TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  expires_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  revoked_at TEXT
);

CREATE TABLE IF NOT EXISTS audit_logs (
  id TEXT PRIMARY KEY,
  clinic_id TEXT REFERENCES clinics(id),
  user_id TEXT REFERENCES users(id),
  action TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT,
  severity TEXT NOT NULL DEFAULT 'info',
  metadata TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS settings (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  value_type TEXT NOT NULL DEFAULT 'string',
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, key)
);

CREATE TABLE IF NOT EXISTS folio_sequences (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  entity TEXT NOT NULL,
  prefix TEXT NOT NULL,
  next_number INTEGER NOT NULL DEFAULT 1,
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, entity)
);

CREATE TABLE IF NOT EXISTS message_templates (
  id TEXT PRIMARY KEY,
  clinic_id TEXT REFERENCES clinics(id),
  name TEXT NOT NULL,
  channel TEXT NOT NULL DEFAULT 'whatsapp_copy',
  body TEXT NOT NULL,
  is_system INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS backups (
  id TEXT PRIMARY KEY,
  clinic_id TEXT REFERENCES clinics(id),
  created_by_user_id TEXT REFERENCES users(id),
  path TEXT NOT NULL,
  status TEXT NOT NULL,
  size_bytes INTEGER NOT NULL DEFAULT 0,
  manifest TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS patients (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  full_name TEXT NOT NULL,
  birth_date TEXT,
  sex TEXT,
  phone TEXT,
  whatsapp TEXT,
  email TEXT,
  address TEXT,
  emergency_contact_name TEXT,
  emergency_contact_phone TEXT,
  occupation TEXT,
  allergies TEXT,
  systemic_diseases TEXT,
  current_medications TEXT,
  relevant_history TEXT,
  habits TEXT,
  general_notes TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  photo_file_id TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS patient_contacts (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  contact_type TEXT NOT NULL,
  name TEXT,
  value TEXT NOT NULL,
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS appointments (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  dentist_user_id TEXT REFERENCES users(id),
  starts_at TEXT NOT NULL,
  ends_at TEXT NOT NULL,
  duration_minutes INTEGER NOT NULL,
  reason TEXT NOT NULL,
  appointment_type TEXT NOT NULL,
  status TEXT NOT NULL,
  notes TEXT,
  created_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  cancelled_at TEXT,
  rescheduled_from_id TEXT REFERENCES appointments(id)
);

CREATE TABLE IF NOT EXISTS appointment_events (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  appointment_id TEXT NOT NULL REFERENCES appointments(id),
  user_id TEXT REFERENCES users(id),
  event_type TEXT NOT NULL,
  from_status TEXT,
  to_status TEXT,
  metadata TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS clinical_records (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  responsible_user_id TEXT REFERENCES users(id),
  appointment_id TEXT REFERENCES appointments(id),
  chief_complaint TEXT,
  current_condition TEXT,
  hereditary_history TEXT,
  pathological_history TEXT,
  non_pathological_history TEXT,
  allergies TEXT,
  current_medications TEXT,
  systemic_diseases TEXT,
  habits TEXT,
  clinical_exploration TEXT,
  diagnosis TEXT,
  prognosis TEXT,
  suggested_plan TEXT,
  indications TEXT,
  observations TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS clinical_evolutions (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  clinical_record_id TEXT REFERENCES clinical_records(id),
  appointment_id TEXT REFERENCES appointments(id),
  responsible_user_id TEXT NOT NULL REFERENCES users(id),
  reason TEXT NOT NULL,
  findings TEXT,
  procedures_done TEXT,
  indications TEXT,
  next_appointment_notes TEXT,
  signed_by TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  voided_at TEXT
);

CREATE TABLE IF NOT EXISTS clinical_evolution_revisions (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  evolution_id TEXT NOT NULL REFERENCES clinical_evolutions(id),
  user_id TEXT REFERENCES users(id),
  snapshot_json TEXT NOT NULL,
  reason TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS vital_signs (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  clinical_record_id TEXT REFERENCES clinical_records(id),
  blood_pressure TEXT,
  heart_rate INTEGER,
  temperature_celsius REAL,
  respiratory_rate INTEGER,
  oxygen_saturation INTEGER,
  weight_kg REAL,
  height_cm REAL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS odontogram_records (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  dentition_type TEXT NOT NULL DEFAULT 'permanent',
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, patient_id, dentition_type, status)
);

CREATE TABLE IF NOT EXISTS odontogram_entries (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  odontogram_record_id TEXT NOT NULL REFERENCES odontogram_records(id),
  tooth_number TEXT NOT NULL,
  surface TEXT,
  state TEXT NOT NULL,
  finding TEXT,
  treatment_plan_item_id TEXT,
  updated_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (odontogram_record_id, tooth_number, surface)
);

CREATE TABLE IF NOT EXISTS odontogram_entry_history (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  odontogram_entry_id TEXT REFERENCES odontogram_entries(id),
  odontogram_record_id TEXT NOT NULL REFERENCES odontogram_records(id),
  tooth_number TEXT NOT NULL,
  surface TEXT,
  previous_state TEXT,
  new_state TEXT NOT NULL,
  finding TEXT,
  changed_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS periodontal_records (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  status TEXT NOT NULL DEFAULT 'draft',
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS treatment_catalog (
  id TEXT PRIMARY KEY,
  clinic_id TEXT REFERENCES clinics(id),
  name TEXT NOT NULL,
  category TEXT NOT NULL,
  description TEXT,
  base_price_cents INTEGER NOT NULL DEFAULT 0,
  estimated_duration_minutes INTEGER,
  requires_follow_up INTEGER NOT NULL DEFAULT 0,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS treatment_plans (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  diagnosis TEXT,
  subtotal_cents INTEGER NOT NULL DEFAULT 0,
  discount_cents INTEGER NOT NULL DEFAULT 0,
  total_cents INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'draft',
  notes TEXT,
  created_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS treatment_plan_items (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  treatment_plan_id TEXT NOT NULL REFERENCES treatment_plans(id),
  treatment_catalog_id TEXT REFERENCES treatment_catalog(id),
  tooth_number TEXT,
  diagnosis TEXT,
  phase TEXT,
  priority TEXT,
  quantity INTEGER NOT NULL DEFAULT 1,
  unit_price_cents INTEGER NOT NULL DEFAULT 0,
  discount_cents INTEGER NOT NULL DEFAULT 0,
  total_cents INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'pending',
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS estimates (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  treatment_plan_id TEXT REFERENCES treatment_plans(id),
  folio TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft',
  valid_until TEXT,
  subtotal_cents INTEGER NOT NULL DEFAULT 0,
  discount_cents INTEGER NOT NULL DEFAULT 0,
  total_cents INTEGER NOT NULL DEFAULT 0,
  observations TEXT,
  terms TEXT,
  created_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, folio)
);

CREATE TABLE IF NOT EXISTS estimate_items (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  estimate_id TEXT NOT NULL REFERENCES estimates(id),
  treatment_catalog_id TEXT REFERENCES treatment_catalog(id),
  description TEXT NOT NULL,
  quantity INTEGER NOT NULL DEFAULT 1,
  unit_price_cents INTEGER NOT NULL DEFAULT 0,
  discount_cents INTEGER NOT NULL DEFAULT 0,
  total_cents INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS payments (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT NOT NULL REFERENCES patients(id),
  folio TEXT NOT NULL,
  concept TEXT NOT NULL,
  amount_cents INTEGER NOT NULL,
  method TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  paid_at TEXT NOT NULL,
  received_by_user_id TEXT NOT NULL REFERENCES users(id),
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  cancelled_at TEXT,
  cancelled_by_user_id TEXT REFERENCES users(id),
  UNIQUE (clinic_id, folio)
);

CREATE TABLE IF NOT EXISTS payment_allocations (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  payment_id TEXT NOT NULL REFERENCES payments(id),
  treatment_plan_item_id TEXT,
  estimate_id TEXT,
  amount_cents INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS receipts (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  payment_id TEXT NOT NULL REFERENCES payments(id),
  folio TEXT NOT NULL,
  pdf_file_id TEXT,
  printed_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (clinic_id, folio)
);

CREATE TABLE IF NOT EXISTS cash_registers (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  opened_by_user_id TEXT NOT NULL REFERENCES users(id),
  opened_at TEXT NOT NULL,
  opening_float_cents INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'open',
  closed_at TEXT,
  closed_by_user_id TEXT REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS cash_movements (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  cash_register_id TEXT REFERENCES cash_registers(id),
  payment_id TEXT REFERENCES payments(id),
  movement_type TEXT NOT NULL,
  category TEXT NOT NULL,
  amount_cents INTEGER NOT NULL,
  method TEXT NOT NULL,
  description TEXT,
  created_by_user_id TEXT NOT NULL REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  cancelled_at TEXT
);

CREATE TABLE IF NOT EXISTS cash_closures (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  cash_register_id TEXT NOT NULL REFERENCES cash_registers(id),
  expected_cash_cents INTEGER NOT NULL,
  counted_cash_cents INTEGER NOT NULL,
  difference_cents INTEGER NOT NULL,
  report_json TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS file_categories (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  description TEXT
);

CREATE TABLE IF NOT EXISTS files (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT REFERENCES patients(id),
  category_id TEXT REFERENCES file_categories(id),
  file_type TEXT NOT NULL,
  original_name TEXT NOT NULL,
  stored_name TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  mime_type TEXT,
  size_bytes INTEGER NOT NULL DEFAULT 0,
  description TEXT,
  uploaded_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS consent_templates (
  id TEXT PRIMARY KEY,
  clinic_id TEXT REFERENCES clinics(id),
  name TEXT NOT NULL,
  treatment_category TEXT,
  body TEXT NOT NULL,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS generated_documents (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT REFERENCES patients(id),
  document_type TEXT NOT NULL,
  related_entity_type TEXT,
  related_entity_id TEXT,
  file_id TEXT REFERENCES files(id),
  status TEXT NOT NULL DEFAULT 'generated',
  created_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS suppliers (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  name TEXT NOT NULL,
  phone TEXT,
  email TEXT,
  notes TEXT,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inventory_items (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  supplier_id TEXT REFERENCES suppliers(id),
  name TEXT NOT NULL,
  category TEXT NOT NULL,
  unit TEXT NOT NULL,
  current_quantity REAL NOT NULL DEFAULT 0,
  minimum_stock REAL NOT NULL DEFAULT 0,
  cost_cents INTEGER NOT NULL DEFAULT 0,
  purchase_date TEXT,
  expiration_date TEXT,
  location TEXT,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inventory_movements (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  inventory_item_id TEXT NOT NULL REFERENCES inventory_items(id),
  movement_type TEXT NOT NULL,
  quantity REAL NOT NULL,
  cost_cents INTEGER,
  reason TEXT,
  treatment_plan_item_id TEXT,
  created_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS alerts (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  patient_id TEXT REFERENCES patients(id),
  alert_type TEXT NOT NULL,
  priority TEXT NOT NULL,
  title TEXT NOT NULL,
  message TEXT NOT NULL,
  due_at TEXT,
  status TEXT NOT NULL DEFAULT 'open',
  related_entity_type TEXT,
  related_entity_id TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  resolved_at TEXT
);

CREATE TABLE IF NOT EXISTS report_exports (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL REFERENCES clinics(id),
  report_type TEXT NOT NULL,
  format TEXT NOT NULL,
  filters_json TEXT,
  file_id TEXT,
  created_by_user_id TEXT REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_users_clinic_status ON users(clinic_id, status);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON user_sessions(token_hash, revoked_at, expires_at);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity_type, entity_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_patients_name ON patients(clinic_id, full_name);
CREATE INDEX IF NOT EXISTS idx_patients_phone ON patients(clinic_id, phone, whatsapp);
CREATE INDEX IF NOT EXISTS idx_appointments_date ON appointments(clinic_id, starts_at);
CREATE INDEX IF NOT EXISTS idx_appointments_dentist_status ON appointments(clinic_id, dentist_user_id, status);
CREATE INDEX IF NOT EXISTS idx_clinical_patient ON clinical_records(patient_id, created_at);
CREATE INDEX IF NOT EXISTS idx_evolutions_patient ON clinical_evolutions(patient_id, created_at);
CREATE INDEX IF NOT EXISTS idx_odontogram_patient ON odontogram_records(patient_id, dentition_type);
CREATE INDEX IF NOT EXISTS idx_treatment_plans_patient ON treatment_plans(patient_id, status);
CREATE INDEX IF NOT EXISTS idx_estimates_patient ON estimates(patient_id, status);
CREATE INDEX IF NOT EXISTS idx_payments_patient ON payments(patient_id, paid_at, status);
CREATE INDEX IF NOT EXISTS idx_payments_date ON payments(clinic_id, paid_at);
CREATE INDEX IF NOT EXISTS idx_inventory_stock ON inventory_items(clinic_id, current_quantity, minimum_stock);
CREATE INDEX IF NOT EXISTS idx_inventory_expiration ON inventory_items(clinic_id, expiration_date);
CREATE INDEX IF NOT EXISTS idx_alerts_status_priority ON alerts(clinic_id, status, priority, due_at);

INSERT OR IGNORE INTO permissions (id, code, name, module, description) VALUES
('perm-patients-view', 'patients.view', 'Ver pacientes', 'patients', 'Permite consultar pacientes y expedientes administrativos'),
('perm-patients-create', 'patients.create', 'Crear pacientes', 'patients', 'Permite registrar pacientes'),
('perm-patients-edit', 'patients.edit', 'Editar pacientes', 'patients', 'Permite modificar datos de pacientes'),
('perm-patients-delete', 'patients.delete', 'Eliminar pacientes', 'patients', 'Permite baja lógica de pacientes'),
('perm-clinical-view', 'clinical.view', 'Ver expediente clínico', 'clinical', 'Permite consultar información clínica'),
('perm-clinical-edit', 'clinical.edit', 'Editar expediente clínico', 'clinical', 'Permite crear historia clínica y evoluciones'),
('perm-odontogram-view', 'odontogram.view', 'Ver odontograma', 'odontogram', 'Permite consultar odontograma'),
('perm-odontogram-edit', 'odontogram.edit', 'Editar odontograma', 'odontogram', 'Permite registrar hallazgos del odontograma'),
('perm-appointments-create', 'appointments.create', 'Crear citas', 'appointments', 'Permite agendar citas'),
('perm-appointments-reschedule', 'appointments.reschedule', 'Reprogramar citas', 'appointments', 'Permite mover citas'),
('perm-appointments-cancel', 'appointments.cancel', 'Cancelar citas', 'appointments', 'Permite cancelar citas'),
('perm-treatments-create', 'treatments.create', 'Crear tratamientos', 'treatments', 'Permite crear tratamientos y planes'),
('perm-prices-edit', 'prices.edit', 'Modificar precios', 'finance', 'Permite modificar precios y descuentos'),
('perm-payments-create', 'payments.create', 'Registrar pagos', 'finance', 'Permite registrar pagos y abonos'),
('perm-payments-cancel', 'payments.cancel', 'Cancelar pagos', 'finance', 'Permite cancelar pagos con trazabilidad'),
('perm-reports-financial', 'reports.financial', 'Ver reportes financieros', 'reports', 'Permite ver reportes financieros'),
('perm-users-admin', 'users.admin', 'Administrar usuarios', 'security', 'Permite administrar usuarios y permisos'),
('perm-backups-restore', 'backups.restore', 'Restaurar respaldos', 'backups', 'Permite restaurar respaldos'),
('perm-backups-create', 'backups.create', 'Crear respaldos', 'backups', 'Permite crear respaldos locales');

INSERT OR IGNORE INTO file_categories (id, name, description) VALUES
('file-radiographs', 'Radiografías', 'Radiografías y estudios de imagen'),
('file-photos', 'Fotografías clínicas', 'Fotografías clínicas del paciente'),
('file-pdfs', 'PDFs', 'Documentos PDF del expediente'),
('file-consents', 'Consentimientos', 'Consentimientos informados'),
('file-receipts', 'Recibos', 'Recibos y documentos financieros'),
('file-other', 'Varios', 'Otros documentos');

INSERT OR IGNORE INTO treatment_catalog (id, clinic_id, name, category, description, base_price_cents, estimated_duration_minutes, requires_follow_up) VALUES
('sys-treatment-consulta-inicial', NULL, 'Consulta inicial', 'Diagnóstico', 'Valoración odontológica inicial', 0, 30, 0),
('sys-treatment-profilaxis', NULL, 'Profilaxis', 'Preventivo', 'Limpieza dental profesional', 0, 45, 0),
('sys-treatment-resina', NULL, 'Resina', 'Restaurativo', 'Restauración con resina', 0, 45, 0),
('sys-treatment-extraccion-simple', NULL, 'Extracción simple', 'Cirugía', 'Extracción dental simple', 0, 45, 1),
('sys-treatment-endodoncia', NULL, 'Endodoncia', 'Endodoncia', 'Tratamiento de conductos', 0, 90, 1),
('sys-treatment-corona', NULL, 'Corona', 'Prótesis', 'Corona dental', 0, 60, 1),
('sys-treatment-implante', NULL, 'Implante', 'Implantología', 'Implante dental', 0, 90, 1),
('sys-treatment-blanqueamiento', NULL, 'Blanqueamiento', 'Estética', 'Blanqueamiento dental', 0, 60, 0),
('sys-treatment-ortodoncia', NULL, 'Ortodoncia', 'Ortodoncia', 'Tratamiento de ortodoncia', 0, 60, 1),
('sys-treatment-radiografia', NULL, 'Radiografía', 'Diagnóstico', 'Estudio radiográfico', 0, 15, 0);
