# Restore Procedure

## Principio

La restauración no sobrescribe la base activa directamente desde la UI. Se usa un flujo staged con validación previa y aplicación al siguiente arranque, antes de abrir SQLite.

## Flujo De Usuario

1. Seleccionar un ZIP de respaldo.
2. Ejecutar `preview_restore`.
3. Revisar resumen: versión, migración, conteos de tablas, archivos incluidos y errores.
4. Confirmar preparación.
5. Ejecutar `prepare_restore`.
6. El sistema crea un respaldo `pre_restore`.
7. El ZIP se copia a `restore-pending/restore.zip`.
8. La app solicita reinicio.
9. En el siguiente arranque se valida el ZIP staged y se aplica antes de abrir la base.
10. Se registra auditoría de preparación y aplicación.

## Validaciones

Antes de preparar restauración:

- permiso `backups.restore`;
- licencia con escritura habilitada;
- ZIP válido;
- manifest válido;
- checksum correcto;
- entrada `data/dentalcare.sqlite`;
- `encrypted: false` en esta fase.

## Archivos Restaurados

La base se restaura desde `data/dentalcare.sqlite`. Los archivos bajo `files/` se restauran dentro del directorio controlado de archivos clínicos. Las rutas del ZIP se sanitizan para evitar path traversal.

## Respaldo De Seguridad

Antes de preparar una restauración, el sistema crea un respaldo `pre_restore`. Durante la aplicación staged también conserva una copia local de seguridad de la base y archivos actuales.

## Estados De Restore

La tabla `restore_jobs` registra:

- `prepared`;
- `applied`;
- `failed` en futuras mejoras;
- rutas staged y de seguridad;
- manifest y resultado de verificación;
- usuario que preparó;
- fechas de preparación y aplicación.

## Limitaciones Actuales

- No hay cifrado real de respaldos.
- No hay migración automática hacia versiones anteriores.
- No hay UI de comparación campo por campo.
- Si un ZIP pertenece a una versión futura incompatible, debe revisarse manualmente antes de aplicar.

## Procedimiento Técnico De Emergencia

1. Cerrar la aplicación.
2. Copiar manualmente la carpeta de datos actual.
3. Verificar el ZIP con `verify_backup` desde la UI o una herramienta interna.
4. Iniciar la app y usar el flujo staged.
5. Si el arranque falla, recuperar la copia de seguridad generada antes de restaurar.

No restaurar sobre una base activa abierta por SQLite.
