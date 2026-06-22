# Commercial Readiness

## Estado Actual

Dentista v1 Professional ya está orientado como producto local-first instalable. La operación diaria
depende de SQLite local y no requiere internet para setup, login, dashboard, pacientes, citas,
presupuestos, pagos, caja, inventario, reportes, respaldos ni configuración básica.

La integración remota queda limitada a un Bridge DV1 opcional. Si no existe URL, no hay internet o el
bridge falla, el sistema local debe seguir operando; la UI muestra advertencias de sincronización,
no fallos críticos de operación.

## Lo Que Está Listo

- Producto y ventana renombrados como `Dentista v1 Professional`.
- Migraciones locales para sync opcional.
- Outbox/inbox local sin dependencia de red.
- Licencia local con trial, activo, gracia, offline grace y sólo lectura.
- Respaldos ZIP con manifest, checksums SHA-256 y verificación.
- Restauración staged con respaldo `pre_restore` y aplicación al siguiente arranque.
- Updater Tauri integrado pero pendiente de firma/endpoint real.
- Documentación de bridge independiente.

## Riesgos Controlados

- El modo sólo lectura bloquea escritura, pero permite consultar, exportar y crear respaldos.
- La restauración requiere licencia con escritura porque reemplaza SQLite local.
- El updater no debe exponerse en UI ni publicarse hasta tener llave pública real, firma y endpoint final.
- El bridge remoto no debe guardar expediente clínico completo ni depender del SaaS dental productivo.

## Pendientes Antes De Venta

- Generar llave real de updater y reemplazar `REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY`.
- Firmar instaladores Windows.
- Definir endpoint final de releases.
- Ejecutar smoke test de MSI/NSIS en Windows limpio.
- Probar restore staged en una instalación real con datos de demo.
- Definir proceso de soporte para recuperar respaldos.
- Crear bridge independiente si se desea portal móvil/remoto.
- Preparar contrato comercial: vigencia de actualizaciones, soporte, garantía de respaldo y límites de uso.

## Criterio Comercial Mínimo

El producto puede considerarse listo para beta comercial cuando:

- la instalación limpia funcione sin internet;
- un consultorio pueda completar un flujo paciente-cita-presupuesto-pago-respaldo;
- licencia expirada mantenga acceso de lectura y respaldo;
- restore staged haya sido probado fuera del entorno de desarrollo;
- updater esté firmado o explícitamente deshabilitado para el build entregado.
