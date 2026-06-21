# Future API Contracts

## Alcance

Contratos de referencia para la futura migración SaaS. No están implementados en esta fase local.

## Autenticación

```http
POST /api/auth/login
POST /api/auth/logout
GET /api/auth/me
POST /api/auth/refresh
```

Respuesta base:

```json
{
  "user": {
    "id": "uuid",
    "clinicId": "uuid",
    "fullName": "Admin",
    "permissions": ["patients.read", "patients.write"]
  },
  "accessToken": "short-lived-token"
}
```

## Licencia

```http
POST /api/licenses/check-in
GET /api/licenses/current
POST /api/licenses/offline-token
```

Debe devolver `status`, `accessMode`, `canWrite`, `planCode`, `planLimits`, `lastCheckAt`, `nextCheckAt` y `message`.

## Pacientes

```http
GET /api/patients
POST /api/patients
GET /api/patients/:id
PATCH /api/patients/:id
DELETE /api/patients/:id
```

`DELETE` debe ser baja lógica.

## Citas

```http
GET /api/appointments?date=YYYY-MM-DD&status=programada
POST /api/appointments
PATCH /api/appointments/:id
POST /api/appointments/:id/status
POST /api/appointments/:id/cancel
GET /api/appointments/availability
```

Estados válidos:

- `programada`
- `confirmada`
- `en_espera`
- `en_consulta`
- `finalizada`
- `cancelada`
- `no_asistio`

## Dentista v1 Bridge Independiente

```http
POST /api/dv1/register-installation
POST /api/dv1/sync/refresh
POST /api/dv1/sync/push
GET /api/dv1/sync/pull
POST /api/dv1/sync/ack
GET /api/dv1/mobile/appointments
POST /api/dv1/mobile/appointments/:id/confirm
POST /api/dv1/mobile/appointments/:id/cancel
```

Estos endpoints pertenecen a un bridge/portal independiente de Dentista v1, no al SaaS dental
existente de `css-aion-new`. El bridge trabaja con proyecciones propias y comandos remotos
propios; no debe depender de `dental_clinics`, `dental_members`, `dental_quotes` ni de las tablas
productivas del SaaS actual. No confirma ni cancela una cita hasta recibir `ack` aplicado desde la
instalación local.

## Expediente Clínico

```http
GET /api/patients/:patientId/clinical-record
POST /api/clinical-records
POST /api/clinical-evolutions
GET /api/patients/:patientId/odontogram
PUT /api/patients/:patientId/odontogram
GET /api/patients/:patientId/periodontal
POST /api/patients/:patientId/periodontal
```

## Finanzas

```http
GET /api/estimates
POST /api/estimates
POST /api/estimates/:id/status
GET /api/payments
POST /api/payments
POST /api/payments/:id/cancel
GET /api/cash/current
POST /api/cash/open
POST /api/cash/close
POST /api/cash/movements
```

## Inventario

```http
GET /api/inventory/items
POST /api/inventory/items
PATCH /api/inventory/items/:id
DELETE /api/inventory/items/:id
POST /api/inventory/items/:id/movements
GET /api/suppliers
POST /api/suppliers
```

## Archivos

```http
POST /api/files/presign-upload
POST /api/files/complete-upload
GET /api/files/:id/download-url
POST /api/files/:id/archive
```

## Reportes

```http
GET /api/reports/operational
GET /api/reports/restock
POST /api/report-exports
GET /api/report-exports
```

## Backups Locales

En SaaS no se debe restaurar una base tenant desde UI común. Los respaldos locales se convierten en herramienta de exportación/migración:

```http
POST /api/migrations/local-backup/import
GET /api/migrations/:id/status
POST /api/migrations/:id/verify
```

## Errores

Formato recomendado:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "El paciente requiere nombre completo.",
    "details": {
      "field": "fullName"
    }
  }
}
```

## Auditoría

Todo endpoint de escritura debe registrar:

- usuario;
- clínica;
- acción;
- entidad;
- id de entidad;
- metadata mínima;
- IP o device cuando aplique;
- fecha UTC.
