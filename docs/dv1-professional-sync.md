# Dentista v1 Professional Sync

## Alcance

La sincronización beta replica solamente proyecciones y comandos de agenda:

- citas administrativas;
- cambios de estatus;
- branding mínimo del consultorio cuando el portal lo soporte;
- auditoría mínima de pairing, push, pull y ack.

No sincroniza expedientes clínicos, archivos clínicos, pagos ni información diagnóstica. SQLite local sigue siendo el source of truth.

## Tablas Locales

- `sync_outbox`: eventos locales pendientes de enviar al portal.
- `sync_inbox`: comandos remotos recibidos y su estado local.
- `sync_cursor`: cursor de pull por tipo.
- `sync_device_tokens`: instalación/dispositivo vinculado y tokens locales.
- `remote_command_receipts`: receipts idempotentes que se ackean al portal.

## Comandos Tauri

- `register_installation`: guarda URL del portal, instalación/dispositivo y opcionalmente completa pairing con código.
- `refresh_sync_token`: rota access/refresh token contra el portal.
- `sync_now`: ejecuta push, pull, aplicación local y ack.
- `get_sync_status`: estado de conectividad y contadores.
- `revoke_local_device`: revoca el dispositivo local y borra tokens.

## Comandos Remotos

El portal debe enviar comandos con `commandId` estable. El local aplica solo:

- `confirm_appointment` -> `confirmada`
- `cancel_appointment` -> `cancelada`

Cada comando se registra en `sync_inbox` y `remote_command_receipts`. Si el mismo `commandId` vuelve a llegar después de aplicarse, el local devuelve el receipt existente.

## Seguridad

El frontend local nunca recibe tokens. El portal resuelve `client_id`/`clinic_id` por pairing code o token de dispositivo; no se aceptan ids de tenant enviados por UI.
