# Security Hardening

## Superficie Local

`dentista-v1` es una aplicación local-first. La seguridad se centra en proteger datos clínicos en disco, comandos Tauri, sesiones, archivos, respaldos y auditoría.

## CSP

La configuración Tauri ya no usa `csp: null`. La política actual limita scripts, imágenes, conexiones y frames a orígenes necesarios para la app local:

- `default-src 'self'`
- `script-src 'self'`
- `style-src 'self' 'unsafe-inline'`
- `img-src 'self' data: asset: http://asset.localhost`
- `connect-src 'self' ipc: http://ipc.localhost`
- `object-src 'none'`
- `frame-ancestors 'none'`

## Permisos Tauri

Las capabilities se reducen a permisos mínimos de core y diálogo para abrir/guardar archivos. Las acciones reales pasan por comandos Rust validados.

## Sesiones

- Contraseñas con Argon2id.
- Tokens de sesión hasheados en SQLite.
- Expiración de sesión en backend.
- Bloqueo por inactividad en frontend.
- El frontend ya no persiste `sessionToken` en Zustand.

## Roles Y Permisos

Los comandos sensibles validan permisos por módulo. Además, `AccessIntent` aplica política de licencia:

- lectura permitida en modo sólo lectura;
- exportación y respaldo permitidos en modo sólo lectura;
- escritura y restauración bloqueadas en modo sólo lectura.

## Archivos

Medidas implementadas:

- nombres sanitizados;
- rutas canonicalizadas dentro del directorio de archivos clínicos;
- límite de 100 MB por archivo;
- extensiones clínicas permitidas;
- bloqueo de path traversal;
- URLs externas con parseo estricto para WhatsApp y correo.

## Respaldos

- ZIP con manifest.
- Checksums SHA-256.
- Historial en SQLite.
- Verificación antes de restaurar.
- Restauración staged con respaldo previo.
- Interfaz para cifrado futuro sin prometer cifrado activo.

## Auditoría

Se registran acciones críticas como login, logout, pacientes, citas, expediente, pagos, caja, inventario, archivos, respaldos, restauración, configuración, usuarios y bloqueos por licencia.

## Secretos

Reglas vigentes:

- no guardar secretos en frontend;
- no hardcodear licencias, tokens, contraseñas ni claves privadas;
- Stripe y licenciamiento remoto vivirán en backend SaaS;
- no almacenar claves de Stripe ni R2 en la app local.

## Futura Nube

Antes de SaaS:

- separar API de dominio Rust;
- agregar `clinic_id` obligatorio en todos los recursos multi-tenant;
- firmar respuestas de licencia;
- cifrar respaldos;
- proteger endpoints con autenticación central;
- mover archivos a almacenamiento con URLs temporales.
