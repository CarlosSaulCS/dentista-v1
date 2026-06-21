# Dentista v1 Professional Sync

## Alcance

La sincronización beta se comunica con un bridge DV1 independiente. Ese bridge puede vivir junto al
ecosistema web de Code Solution Studio, pero no es el SaaS dental existente de `css-aion-new` ni usa
sus tablas productivas.

La fase 1 replica solamente proyecciones y comandos de agenda:

- citas administrativas;
- cambios de estatus;
- branding mínimo del consultorio cuando el bridge lo soporte;
- auditoría mínima de pairing, push, pull y ack.

No sincroniza expedientes clínicos, archivos clínicos, pagos ni información diagnóstica. SQLite local sigue siendo el source of truth.

## Tablas Locales

- `sync_outbox`: eventos locales pendientes de enviar al bridge DV1.
- `sync_inbox`: comandos remotos recibidos y su estado local.
- `sync_cursor`: cursor de pull por tipo.
- `sync_device_tokens`: instalación/dispositivo vinculado y tokens locales.
- `remote_command_receipts`: receipts idempotentes que se ackean al bridge DV1.

## Comandos Tauri

- `register_installation`: guarda URL del bridge, instalación/dispositivo y opcionalmente completa pairing con código.
- `refresh_sync_token`: rota access/refresh token contra el bridge.
- `sync_now`: ejecuta push, pull, aplicación local y ack.
- `get_sync_status`: estado de conectividad y contadores.
- `revoke_local_device`: revoca el dispositivo local y borra tokens.

La URL base puede venir de Configuración > Portal remoto o de la variable de entorno
`DV1_SYNC_BASE_URL`. Si no hay URL ni dispositivo activo, la app local sigue funcionando; solo las
acciones de sincronización devuelven un error de configuración.

## Endpoints Esperados

```http
POST /api/dv1/register-installation
POST /api/dv1/sync/refresh
POST /api/dv1/sync/push
GET /api/dv1/sync/pull
POST /api/dv1/sync/ack
```

## Comandos Remotos

El bridge debe enviar comandos con `commandId` estable. El local aplica solo:

- `confirm_appointment` -> `confirmada`
- `cancel_appointment` -> `cancelada`

Cada comando se registra en `sync_inbox` y `remote_command_receipts`. Si el mismo `commandId` vuelve a llegar después de aplicarse, el local devuelve el receipt existente.

## Seguridad

El frontend local nunca recibe tokens. El bridge resuelve instalación/dispositivo por pairing code o
token de dispositivo; no se aceptan ids de tenant enviados por UI. El bridge de Dentista v1 debe usar
base de datos o schema propio y no depender de `dental_clinics`, `dental_members`, `dental_quotes` ni
demás entidades productivas de `css-aion-new`.
