# Dentista v1 Professional

Sistema integral para consultorio dental construido como aplicación local-first de escritorio para Windows.

Esta fase mantiene el producto local: Tauri 2, Rust, React, TypeScript, SQLite y SQLx. No integra todavía la web principal, Neon, Stripe, R2 ni Code Solutions Studio.

## Stack

- Tauri 2 + Rust
- React + TypeScript + Vite
- Tailwind CSS + shadcn/ui
- SQLite + SQLx + migraciones versionadas
- TanStack Query, TanStack Table, Zustand, React Hook Form, Zod, Recharts

## Instalación Y Desarrollo

```bash
npm install
npm run tauri:dev
```

Validación local:

```bash
npm run typecheck
npm run lint
npm run build
cd src-tauri && cargo test
```

Empaquetado:

```bash
npm run tauri:build
```

Después del build, los instaladores quedan en:

- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

## Rutas De Datos

La aplicación usa rutas administradas por Tauri. En instalaciones nuevas quedan bajo el directorio de datos de Dentista v1 Professional; instalaciones heredadas pueden conservar carpetas con el nombre anterior para no romper compatibilidad.

- Base SQLite: `appDataDir/data/dentalcare.sqlite`
- Archivos clínicos: `appDataDir/files/`
- Respaldos: `Documents/Dentista v1 Backups/`
- Reportes exportados: directorio elegido por el usuario desde diálogo local

## Módulos

- Setup inicial del consultorio.
- Login local.
- Dashboard con datos reales de SQLite.
- Pacientes y baja lógica.
- Agenda diaria, conflictos y cambios de estado.
- Historia clínica y evoluciones.
- Odontograma por pieza dental con historial.
- Periodontograma base por paciente.
- Catálogo de tratamientos.
- Planes de tratamiento.
- Presupuestos.
- Pagos y abonos.
- Caja.
- Inventario y proveedores.
- Archivos clínicos locales.
- Consentimientos.
- Alertas.
- Reportes y exportaciones.
- Usuarios y roles.
- Configuración del consultorio.
- Respaldos, verificación y restauración staged.

## Licencia Local

La licencia local soporta estados de prueba, activo, gracia, suspendido, expirado y sólo lectura. Si la licencia vence, la app no bloquea el acceso a datos: permite consultar, exportar y crear respaldos, pero bloquea escrituras y restauraciones.

La activación hardcodeada en login fue retirada. La integración real de suscripción queda documentada para una fase futura.

Ver: `docs/licensing-roadmap.md`.

## Backups

Los respaldos se generan como ZIP verificable:

```text
manifest.json
checksums.sha256
data/dentalcare.sqlite
files/
metadata/system-info.json
```

El sistema guarda historial, tamaño, checksum, conteos de tablas, versión de app, versión de migración y estado de verificación. También puede crear respaldo automático diario al iniciar sesión si el último respaldo completado tiene más de 24 horas.

Ver: `docs/backups-and-restore.md`.

## Restore

La restauración usa flujo staged:

1. Verificar ZIP.
2. Mostrar preview.
3. Crear respaldo de seguridad.
4. Copiar ZIP a `restore-pending`.
5. Aplicar al siguiente arranque antes de abrir SQLite.
6. Registrar auditoría.

No se sobrescribe la base activa directamente desde la UI.

Ver: `docs/restore-procedure.md`.

## Seguridad

- CSP no nulo en Tauri.
- Capabilities reducidas.
- Contraseñas con Argon2id.
- Sesiones hasheadas y expirables.
- `sessionToken` no persistido en Zustand.
- Permisos por comando.
- `AccessIntent` para bloquear escrituras en modo sólo lectura.
- Validación de rutas y nombres de archivo.
- Límite de 100 MB para archivos clínicos.
- Auditoría de acciones críticas.

Ver: `docs/security-hardening.md` y `docs/clinical-data-privacy.md`.

## Documentación Técnica

- `docs/licensing-roadmap.md`
- `docs/backups-and-restore.md`
- `docs/restore-procedure.md`
- `docs/security-hardening.md`
- `docs/clinical-data-privacy.md`
- `docs/files-to-r2-migration.md`
- `docs/saas-migration-plan.md`
- `docs/sqlite-to-postgres-map.md`
- `docs/api-contracts-future.md`

## Roadmap SaaS

La base queda preparada para una migración futura:

- licencia remota y suscripción;
- endpoints HTTP por dominio;
- PostgreSQL/Neon;
- archivos en Cloudflare R2;
- Stripe en backend SaaS;
- integración posterior con Code Solutions Studio.

Nada de eso está conectado en esta fase.

## Limitaciones Actuales

- Cifrado de respaldos preparado como interfaz, no implementado.
- Restauración completa requiere reinicio controlado.
- Suscripción remota no conectada.
- Purga automática de retención pendiente.
- SaaS, Neon, Stripe y R2 sólo están documentados.

## Notas De Uso

Si ya creaste el administrador y vuelves a ver el asistente inicial, usa el build actual: detecta la base existente y cambia a login. En una instalación existente, inicia sesión con el usuario y contraseña creados en el primer arranque.

El ejecutable release generado por `npm run tauri:build` está compilado como aplicación Windows GUI y no abre una consola. Si ejecutas binarios debug o procesos lanzados desde consola, Windows puede mantener esa consola asociada al proceso.
