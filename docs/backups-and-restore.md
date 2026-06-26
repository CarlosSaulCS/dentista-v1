# Backups And Restore

## Objetivo

El respaldo local ahora busca ser verificable, auditable y apto para restauración staged. No cifra todavía el ZIP, pero deja interfaz y configuración para activarlo en una fase posterior.

## Estructura Del ZIP

```text
backup.zip
  manifest.json
  checksums.sha256
  data/dentalcare.sqlite
  files/
  metadata/system-info.json
```

`manifest.json` incluye:

- `backupId`
- `clinicId`
- `createdAt`
- `appVersion`
- `databaseVersion`
- `migrationVersion`
- `includes`
- `tableCounts`
- `fileCount`
- `checksum`
- `compression`
- `encrypted`
- `createdByUserId`

`checksums.sha256` registra hashes SHA-256 de las entradas principales del ZIP. La verificación recalcula los hashes y detecta archivos faltantes o modificados.

## Comandos Tauri

- `create_backup`
- `list_backups`
- `verify_backup`
- `preview_restore`
- `prepare_restore`
- `get_backup_settings`
- `update_backup_settings`

## Tipos De Respaldo

- `manual`: creado desde la UI.
- `automatic`: creado al iniciar sesión si el último respaldo completado tiene más de 24 horas y el respaldo automático está activo.
- `pre_restore`: creado automáticamente antes de preparar una restauración.

Los respaldos manuales y la verificación de ZIP deben seguir disponibles en modo sólo lectura. La
preparación de restauración requiere escritura porque reemplaza la base SQLite activa en el siguiente
arranque.

## Historial

La tabla `backups` guarda:

- ruta;
- estado;
- tamaño;
- manifest;
- checksum global;
- estado de verificación;
- fecha de verificación;
- tipo de respaldo;
- versión de app;
- versión de migración;
- conteo de archivos;
- conteos de tablas.

## Configuración

La tabla `backup_settings` controla:

- respaldo automático activo o no;
- frecuencia declarada: `daily`, `weekly`, `manual`;
- inclusión de archivos clínicos;
- bandera de cifrado futuro;
- límite de retención.

En esta fase la frecuencia automática implementada es diaria al login. `weekly` y `manual` quedan modelados para evolución.

## Integridad

La verificación valida:

- existencia del ZIP;
- existencia de `manifest.json`;
- existencia de `checksums.sha256`;
- presencia de `data/dentalcare.sqlite`;
- checksum de entradas listadas;
- manifest parseable;
- estado final `valid` o `invalid`.

## Archivos Clínicos

El respaldo incluye `files/` cuando `include_files` está activo. Se omiten archivos mayores a 100 MB para evitar respaldos inestables. La subida futura a R2 debe operar sobre el mismo inventario lógico de archivos.

## Cifrado Futuro

`src-tauri/src/backups/backup_encryption.rs` define la interfaz preparada. En esta fase devuelve error controlado porque no se activó cifrado real.

Diseño recomendado:

- derivación de llave local con Argon2id o integración con keychain del sistema;
- cifrado AEAD por archivo o por ZIP completo;
- salt y nonce únicos por respaldo;
- manifest con `encrypted: true` y metadatos no sensibles;
- prueba de restauración con clave incorrecta.

## Retención

El límite de retención se guarda, pero la purga automática de respaldos antiguos debe tratarse con cuidado. No debe eliminar el único respaldo válido ni borrar archivos sin registro visible.
