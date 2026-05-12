# DentalCare Manager

Sistema Integral para Consultorio Dental construido como aplicación de escritorio local-first para Windows.

## Stack

- Tauri 2 + Rust
- React + TypeScript + Vite
- Tailwind CSS + shadcn/ui
- SQLite + SQLx + migraciones versionadas
- TanStack Query, TanStack Table, Zustand, React Hook Form, Zod, Recharts

## Comandos

```bash
npm install
npm run tauri:dev
npm run tauri:build
npm run typecheck
npm run lint
cd src-tauri && cargo test
```

## Build Generado

Después de `npm run tauri:build`, los instaladores quedan en:

- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

## Almacenamiento Local

La aplicación usa rutas de datos administradas por Tauri:

- Base SQLite: `%APPDATA%/DentalCare Manager/data/dentalcare.sqlite`
- Archivos clínicos: `%APPDATA%/DentalCare Manager/files/`
- Respaldos: `Documents/DentalCare Backups/`

## Seguridad V1

- Primer arranque con asistente para crear consultorio y administrador.
- No hay credenciales por defecto.
- Contraseñas con Argon2id.
- Sesiones locales con token hasheado en base de datos.
- Permisos por rol y validación por comando Tauri.
- Auditoría de acciones críticas.
- Registros clínicos diseñados para trazabilidad, no borrado destructivo.

## Módulos Implementados En Esta Base

- Setup inicial del consultorio.
- Login local.
- Dashboard con datos reales de SQLite.
- Pacientes.
- Agenda diaria y cambios de estado.
- Historia clínica y evoluciones.
- Odontograma por pieza dental con historial.
- Periodontograma base por paciente.
- Catálogo de tratamientos.
- Planes de tratamiento.
- Presupuestos con folio y exportación CSV.
- Pagos y abonos con folio.
- Caja con apertura, entradas por pago y cierre.
- Inventario dental y proveedores.
- Archivos clínicos guardados localmente.
- Consentimientos con plantillas y PDF al expediente.
- Alertas.
- Reportes con exportación CSV/Excel.
- Usuarios y roles.
- Configuración del consultorio y plantillas de mensajes.
- Respaldos ZIP de base de datos y archivos.

Los módulos siguen siendo local-first: no dependen de internet ni de servicios cloud.

## Notas De Uso

Si ya creaste el administrador y vuelves a ver el asistente inicial, usa el nuevo build: detecta la base existente y cambia a login. En una instalación existente, inicia sesión con el usuario y contraseña que creaste en el primer arranque.

El ejecutable release generado por `npm run tauri:build` está compilado como aplicación Windows GUI y no abre una consola. Si ejecutas binarios debug o procesos lanzados desde consola, Windows puede mantener esa consola asociada al proceso.
