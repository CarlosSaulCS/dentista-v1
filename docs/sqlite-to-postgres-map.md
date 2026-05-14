# SQLite To PostgreSQL Map

## Convenciones Futuras

En PostgreSQL se recomienda:

- `TEXT` id actual a `uuid` cuando sea viable.
- Fechas `TEXT` ISO a `timestamptz`.
- JSON en `TEXT` a `jsonb`.
- Montos `REAL` a `numeric(12,2)`.
- Banderas `INTEGER` 0/1 a `boolean`.
- Agregar `client_id`, `clinic_id`, `created_at`, `updated_at`, `deleted_at` donde aplique.

## Mapa De Tablas

| SQLite actual | PostgreSQL futura | Notas |
| --- | --- | --- |
| `clinics` | `clinics` | Agregar `client_id`, datos fiscales normalizados. |
| `permissions` | `permissions` | Catálogo global. |
| `roles` | `roles` | Tenant por `clinic_id` o `client_id`. |
| `role_permissions` | `role_permissions` | Igual. |
| `users` | `users` | Puede separarse de identidad central. |
| `user_permissions` | `user_permissions` | Overrides por usuario. |
| `user_sessions` | `sessions` | En SaaS puede moverse a auth central. |
| `audit_logs` | `audit_logs` | Usar `jsonb` para metadata. |
| `settings` | `clinic_settings` | JSONB para configuración agrupada. |
| `folio_sequences` | `folio_sequences` | Usar transacciones fuertes. |
| `message_templates` | `message_templates` | Tenant-aware. |
| `backups` | `backups` | En SaaS será historial de exportaciones/migración. |
| `backup_settings` | `backup_settings` | Local o por tenant. |
| `restore_jobs` | `restore_jobs` | Puede ser sólo herramienta local/admin. |
| `app_license` | `subscriptions` y `license_devices` | Separar suscripción, cliente y dispositivo. |
| `patients` | `patients` | Agregar campos normalizados de búsqueda. |
| `patient_contacts` | `patient_contacts` | Igual. |
| `appointments` | `appointments` | Indexar por `clinic_id`, `starts_at`, `status`. |
| `appointment_events` | `appointment_events` | Historial de cambios. |
| `clinical_records` | `clinical_records` | Datos sensibles. |
| `clinical_evolutions` | `clinical_evolutions` | Trazabilidad fuerte. |
| `clinical_evolution_revisions` | `clinical_evolution_revisions` | `snapshot_json` a `jsonb`. |
| `vital_signs` | `vital_signs` | Tipos numéricos. |
| `odontogram_records` | `odontogram_records` | Igual. |
| `odontogram_entries` | `odontogram_entries` | JSONB si se agregan superficies. |
| `odontogram_entry_history` | `odontogram_entry_history` | Auditoría clínica. |
| `periodontal_records` | `periodontal_records` | JSONB para mediciones. |
| `treatment_catalog` | `treatment_catalog` | Tenant o catálogo global configurable. |
| `treatment_plans` | `treatment_plans` | Estados normalizados. |
| `treatment_plan_items` | `treatment_plan_items` | Montos numeric. |
| `estimates` | `estimates` | Folios únicos por clínica. |
| `estimate_items` | `estimate_items` | Montos numeric. |
| `payments` | `payments` | Montos numeric, conciliación futura. |
| `payment_allocations` | `payment_allocations` | Igual. |
| `receipts` | `receipts` | Relación con archivo R2. |
| `cash_registers` | `cash_registers` | Turnos de caja. |
| `cash_movements` | `cash_movements` | Montos numeric. |
| `cash_closures` | `cash_closures` | JSONB para desglose. |
| `file_categories` | `file_categories` | Catálogo. |
| `files` | `files` | Agregar `storage_provider`, `bucket`, `object_key`. |
| `consent_templates` | `consent_templates` | Versionado de plantilla. |
| `generated_documents` | `generated_documents` | Archivos PDF en R2. |
| `suppliers` | `suppliers` | Tenant-aware. |
| `inventory_items` | `inventory_items` | Evitar stock negativo con constraints. |
| `inventory_movements` | `inventory_movements` | Kardex por insumo. |
| `alerts` | `alerts` | Estados y severidad normalizados. |
| `system_alert_state` | `system_alert_state` | Deduplicación. |
| `report_exports` | `report_exports` | Auditoría de exportaciones. |

## Campos Nuevos Recomendados

- `client_id`
- `clinic_id`
- `subscription_id`
- `created_at`
- `updated_at`
- `deleted_at`
- `created_by_user_id`
- `updated_by_user_id`
- `search_text`
- `external_id` para importaciones.

## Validaciones A Convertir En Constraints

- citas: `ends_at > starts_at`;
- pagos: `amount > 0`;
- inventario: `current_quantity >= 0`;
- estados mediante enums o check constraints;
- folios únicos por clínica y entidad;
- unicidad suave de paciente por clínica y normalización.
