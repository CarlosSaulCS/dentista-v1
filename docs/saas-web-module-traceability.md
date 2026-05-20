# SaaS Web Module Traceability

This document links the current local-first DentalCare Manager implementation to
the future SaaS web module in `css-aion-new-main`.

It is documentation only. Do not move Tauri/Rust into the web app, do not connect
Neon, and do not enable R2 from this phase.

## Migration Boundary

- `dentista-v1` remains the reference implementation for domain behavior.
- `css-aion-new-main` becomes the future web shell and contract owner.
- Rust services should be ported into a separate HTTP backend later.
- React components can be adapted, but desktop services must be replaced with a
  typed HTTP client.

## Module To Source Map

| SaaS module | Rust source | Frontend source | SQLite tables |
| --- | --- | --- | --- |
| Bootstrap/auth/session | `auth_service`, `license_service`, `audit_service` | `features/auth`, `store/auth-store` | `clinics`, `users`, `roles`, `permissions`, `user_sessions`, `audit_logs`, `app_license` |
| Patients | `patient_service` | `features/patients` | `patients`, `patient_contacts` |
| Appointments | `appointment_service` | `features/appointments` | `appointments`, `appointment_events` |
| Clinical records | `clinical_service` | `features/clinical-records` | `clinical_records`, `clinical_evolutions`, `clinical_evolution_revisions`, `vital_signs` |
| Odontogram | `odontogram_service` | `features/odontogram` | `odontogram_records`, `odontogram_entries`, `odontogram_entry_history` |
| Periodontal | `office_service` | `features/periodontal` | `periodontal_records` |
| Treatments/plans | `office_service` | `features/treatments`, `features/treatment-plans` | `treatment_catalog`, `treatment_plans`, `treatment_plan_items` |
| Estimates/payments/cash | `office_service`, `reports` | `features/estimates`, `features/payments`, `features/cash` | `estimates`, `estimate_items`, `payments`, `payment_allocations`, `receipts`, `cash_registers`, `cash_movements`, `cash_closures` |
| Files/documents | `office_service`, `files`, `reports` | `features/files`, `features/consents`, `features/reports` | `files`, `file_categories`, `consent_templates`, `generated_documents`, `report_exports` |
| Inventory/admin | `office_service`, `dashboard_service` | `features/inventory`, `features/suppliers`, `features/alerts`, `features/users`, `features/settings` | `suppliers`, `inventory_items`, `inventory_movements`, `alerts`, `system_alert_state`, `settings`, `message_templates` |
| Backups/imports | `backup_service`, `backups` | `features/backups` | `backups`, `backup_settings`, `restore_jobs` |

## Current Command Surface

The Tauri command names are the current boundary between React and Rust. Later,
each one should map to HTTP endpoints documented in `css-aion-new-main`.

- Auth: `get_bootstrap_status`, `setup_clinic_and_admin`, `login`, `logout`,
  `list_users`, `get_license_status`.
- Patients: `create_patient`, `list_patients`, `get_patient`, `update_patient`,
  `soft_delete_patient`.
- Appointments: `create_appointment`, `list_appointments`,
  `get_next_patient_appointment`, `update_appointment`,
  `update_appointment_status`, `soft_delete_appointment`.
- Clinical: `create_clinical_record`, `create_clinical_evolution`,
  `list_clinical_records`, `list_clinical_evolutions`.
- Odontogram/periodontal: `get_odontogram`, `upsert_odontogram_entry`,
  `list_periodontal_records`, `create_periodontal_record`.
- Office/finance/admin: treatments, treatment plans, estimates, payments, cash,
  suppliers, inventory, alerts, consents, reports, settings, templates, roles,
  users, global search.
- Files: `save_patient_file`, `list_patient_files`, `open_patient_file`,
  `open_external_url`.
- Backups: `create_backup`, `list_backups`, `verify_backup`,
  `preview_restore`, `prepare_restore`, `get_backup_settings`,
  `update_backup_settings`.

## Rules That Must Survive The Port

- Tenant resolution must happen on the backend using authenticated user context.
- Permission checks from `validate_session` and `validate_session_for_intent`
  must become API middleware.
- License read-only behavior from `AccessIntent` must become plan/subscription
  gates.
- Soft deletes must remain soft deletes for patients, appointments, and
  inventory.
- Folio generation must remain clinic-scoped and transactional.
- Payment totals, treatment balances, cash closures, and inventory movements
  must be calculated server-side.
- Clinical file validation must keep MIME/extension checks, size limits,
  sanitized names, audit logs, and checksum strategy.
- Backup restore must not become a common SaaS UI action; only verified imports
  should be exposed to admins.

## Web Contract Location

The initial web-side contracts live in:

```text
../css-aion-new-main/src/lib/dental/types.ts
../css-aion-new-main/src/lib/dental/api-contracts.ts
../css-aion-new-main/docs/dental-saas-migration-strategy.md
```

These files define names, routes, and data shapes only. They do not call a
backend and do not change either app's runtime behavior.

## Future Acceptance Criteria

- No `@tauri-apps/api` imports exist in `css-aion-new-main`.
- No Rust/Tauri files are copied into the web project.
- The future web client calls `/api/dental/*`, not `invokeCommand`.
- Neon and R2 are enabled only after schema, permissions, migration scripts, and
  verification checks are approved.
- A pilot import validates table counts, clinical reads, financial balances, and
  file checksums before any tenant is considered migrated.
