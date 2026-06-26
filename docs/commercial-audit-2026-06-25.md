# Auditoría Comercial Dentista v1 Professional

Fecha: 2026-06-25

## Alcance

Se revisó el proyecto instalable/local-first `dentista-v1` como producto premium para Windows. No se encontró `AGENTS.md` en el repositorio. La revisión cubrió documentación, Tauri/Rust, migraciones SQLite/SQLx, servicios, comandos, seguridad, backups, licencia, frontend React/TypeScript, rutas principales y preparación futura para sync/app remota.

## Arquitectura

- Escritorio: Tauri 2 con Rust. `src-tauri/src/lib.rs` registra comandos por dominio y expone estado compartido con `AppState`.
- Datos: SQLite local con SQLx, migraciones versionadas en `src-tauri/migrations/`, WAL, `foreign_keys = ON` y `busy_timeout`.
- Frontend: React + TypeScript + Vite, Tailwind/shadcn, Zustand para sesión, TanStack Query para lectura/invalidation, React Hook Form + Zod en formularios principales.
- Flujo Tauri: la UI llama wrappers en `src/lib/*` hacia comandos `src-tauri/src/commands/*`, que delegan a servicios Rust.
- Archivos: clínicos bajo `appDataDir/files/`, con nombre sanitizado y apertura protegida por canonicalización.
- Backups: ZIP con `manifest.json`, checksums SHA-256, SQLite y archivos; restore staged prepara respaldo de seguridad y aplica al reiniciar.
- Licencia: local trial/active/grace/offline/read-only; escrituras y restore pasan por `validate_session_for_intent`.
- Seguridad: Argon2id para contraseñas, tokens de sesión hasheados, expiración de sesión, auditoría, CSP Tauri restrictivo y opener limitado a WhatsApp/mailto.
- Sync futuro: existen tablas de base para instalaciones, dispositivos, outbox/inbox y comandos remotos, pero no hay bridge cloud productivo activo en este repo.

## Estado De Módulos

- Setup inicial: funcional, crea clínica, roles, permisos, admin, licencia trial y datos base.
- Login: funcional, sesión local, modo bloqueo/inactividad y read-only por licencia.
- Dashboard: funcional con métricas reales, alertas, citas, ingresos e inventario.
- Pacientes: CRUD operativo, búsqueda, edición y baja lógica. Se reforzaron accesos rápidos a agenda, expediente, odontograma, pagos, archivos y presupuestos.
- Agenda/citas: crear, editar, estados, cancelar/baja lógica, conflicto por paciente/dentista y próximas citas por paciente.
- Clínica: historias y evoluciones funcionales con validación de paciente activo y relaciones opcionales de cita/historia.
- Odontograma: funcional por pieza/superficie, historial y auditoría.
- Periodontograma: base funcional con notas/status; pendiente chart periodontal completo por sitio.
- Tratamientos/planes: catálogo, planes con items y totales; se reforzó validación de importes y referencias.
- Presupuestos: folio, estados, exportación CSV y aprobación; se validan estados permitidos y relaciones.
- Pagos/caja: pagos, comprobantes, caja abierta, movimientos automáticos y cierre; falta cancelación/devolución completa de pagos.
- Inventario/proveedores: productos, proveedores, movimientos, stock bajo y alertas; se hicieron permisos explícitos.
- Archivos clínicos: carga local, tipo/tamaño permitido, apertura segura; ahora usa permiso `files.manage` salvo comprobantes de pago.
- Consentimientos: plantillas y registros base; pendiente firma digital/legal y plantillas clínicas finales.
- Alertas: manuales y sistema por inventario, citas, presupuestos y backups.
- Reportes: operativo/financiero/inventario con CSV/XLSX/PDF y branding actualizado.
- Usuarios/roles: listado, creación y roles base; se validó que el rol pertenezca a la clínica.
- Configuración: datos de clínica editables.
- Backups/restore staged: funcional; pendiente prueba limpia de restore completo en Windows instalado.
- Licencia local: funcional como control offline; integración SaaS/entitlements queda documentada.

## Correcciones Aplicadas

- Se agregó guard compartido para impedir escrituras contra pacientes inexistentes o dados de baja.
- Se aplicó el guard en citas, clínica, odontograma, planes, presupuestos, pagos, archivos, alertas y periodontograma.
- Se reforzaron relaciones opcionales: plan-presupuesto, pago-presupuesto, pago-item, proveedor-inventario, rol-usuario, cita/historia clínica.
- Se hicieron permisos explícitos en inventario, archivos clínicos y plantillas de consentimiento.
- Se validaron estados de presupuesto, métodos de pago e importes por concepto.
- Se corrigió el filtro de enlaces externos para permitir WhatsApp/mailto con query percent-encoded y rechazar dominios/esquemas no permitidos.
- Se evitó crear asignaciones de pago vacías cuando los IDs opcionales vienen como cadena vacía.
- Se actualizó branding visible y metadatos fuente a `Dentista v1 Professional`.
- Se agregó migración para que el rol `dentist` conserve `files.manage` en instalaciones existentes.

## Pendientes Comerciales Reales

- Firmar updater real: reemplazar `REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY` y publicar manifiestos firmados.
- Ejecutar smoke test de MSI/NSIS en Windows limpio con instalación, upgrade, backup, restore staged y desinstalación.
- Definir migrador de ruta si se desea mover datos de instalaciones antiguas con nombre `DentalCare Manager`.
- Completar periodontograma por sitio, movilidad, furcas, sangrado, supuración y profundidad.
- Agregar cancelación/anulación de pagos con reversa de caja y auditoría financiera.
- Cifrado de backups y política de retención automática efectiva.
- Plantillas legales finales de consentimiento, firma, exportación PDF y versionado.
- Bridge cloud/app remota: implementar conexión real con `css-aion-new`, entitlements y sync bidireccional con resolución de conflictos.
