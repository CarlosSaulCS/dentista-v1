# Licensing Roadmap

## Objetivo

La licencia de `dentista-v1` queda preparada para operar primero como producto local robusto y después como cliente de una suscripción remota. En esta fase no se conecta Stripe, no se llama una API externa y no se guardan secretos reales.

## Estado Local Actual

La tabla `app_license` conserva la prueba local y agrega campos de suscripción futura:

- `status`: `trial_active`, `active`, `grace_period`, `past_due`, `suspended`, `read_only`, `expired`, `offline_grace`, `not_configured`.
- `access_mode`: `full`, `read_only` o `setup`.
- `last_check_at` y `next_check_at`: ventana de validación local o remota futura.
- `device_id` e `installation_id`: identificadores locales no secretos.
- `clinic_id`, `customer_id`, `subscription_id`: llaves para la futura plataforma.
- `plan_code` y `plan_limits_json`: plan contratado y límites.
- `grace_period_ends_at`, `offline_grace_until`, `read_only_reason`: control de gracia y modo seguro.

El servicio principal es `src-tauri/src/services/license_service.rs`. Los comandos expuestos están en `src-tauri/src/commands/license.rs`.

## Reglas De Acceso

La aplicación usa `AccessIntent`:

- `Read`: permite consultar datos.
- `ExportOrBackup`: permite exportar y crear respaldos.
- `DataWrite`: permite mutaciones clínicas, administrativas y financieras.
- `Restore`: permite preparar restauración.

Cuando la licencia vence o queda suspendida, el sistema entra en `read_only`. En ese modo se permite:

- consultar pacientes, expedientes, citas, pagos e inventario;
- generar exportaciones;
- crear respaldos;
- revisar configuración.

En ese modo se bloquea:

- crear o editar datos;
- cancelar o reprogramar citas;
- registrar pagos;
- modificar usuarios, inventario, archivos o configuración;
- preparar restauraciones.

## Activación Local DEV/Legacy

La activación local con secreto hardcodeado fue retirada del login. Cualquier helper local queda aislado para pruebas o migraciones controladas y no se expone como flujo de producción.

## Validación Remota Futura

Cuando exista la plataforma SaaS, el cliente local podrá llamar endpoints como:

- `POST /api/licenses/check-in`
- `POST /api/licenses/activate-device`
- `POST /api/licenses/deactivate-device`
- `GET /api/licenses/current`
- `POST /api/licenses/offline-token`

Payload mínimo de check-in:

```json
{
  "installationId": "local-installation-id",
  "deviceId": "local-device-id",
  "clinicId": "clinic-id",
  "appVersion": "0.1.0",
  "lastKnownStatus": "trial_active"
}
```

Respuesta esperada:

```json
{
  "status": "active",
  "accessMode": "full",
  "customerId": "cus_xxx",
  "subscriptionId": "sub_xxx",
  "planCode": "clinic-pro",
  "planLimits": {
    "patients": null,
    "users": 10,
    "storageMb": 102400
  },
  "lastCheckAt": "2026-05-14T00:00:00Z",
  "nextCheckAt": "2026-05-15T00:00:00Z",
  "offlineGraceUntil": "2026-05-21T00:00:00Z",
  "message": "Suscripción activa."
}
```

## Stripe Futuro

Stripe debe vivir en el backend SaaS, no en la app local. La app local nunca debe guardar claves secretas de Stripe.

Flujo esperado:

1. El backend recibe webhooks de Stripe.
2. Actualiza cliente, suscripción y plan.
3. Publica el estado de licencia.
4. El cliente local consume estado firmado o validado por API.
5. Ante falta de pago, se entra primero a `grace_period` o `past_due`; si no se regulariza, a `read_only`.

## Offline Y Gracia

La app local debe tolerar falta de internet cuando exista licencia remota:

- Si `next_check_at` vence pero existe `offline_grace_until`, usar `offline_grace`.
- Si vence la gracia offline, cambiar a `read_only`.
- Nunca bloquear completamente la consulta ni el respaldo de datos clínicos.

## Pendiente Para Producción Comercial

- Firma criptográfica de respuestas de licencia.
- Pinning o validación estricta del endpoint remoto.
- Rotación de identificadores de dispositivo si se reinstala.
- Reglas de límite de plan aplicadas por módulo.
- Pantalla de administración de dispositivos.
- Webhooks Stripe en la plataforma SaaS.
