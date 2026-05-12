# Arquitectura

## Flujo Principal

```txt
React pages
  -> feature services
  -> Tauri invoke
  -> Rust commands
  -> Rust services
  -> SQLx / SQLite
```

Los comandos Tauri son la frontera de seguridad. Cada comando que toca datos sensibles valida sesión y permisos antes de operar.

## Frontend

```txt
src/
  app/             bootstrap, providers y estilos globales
  layouts/         shell principal con sidebar/header
  routes/          composición de rutas
  components/ui/   shadcn/ui
  components/data/ componentes reutilizables de producto
  features/        dominios clínicos y administrativos
  lib/             API invoke y utilidades
  store/           estado de sesión local
  types/           tipos compartidos
```

Cada feature contiene solo carpetas con código real para evitar estructura vacía. La UI usa componentes shadcn, Tailwind tokens, formularios validados y cache con TanStack Query.

## Backend

```txt
src-tauri/src/
  commands/        API expuesta a React
  services/        reglas de negocio y permisos
  database/        inicialización SQLite/SQLx
  models/          DTOs serializables
  security/        Argon2id y tokens
  files/           punto de extensión para archivos clínicos
  reports/         punto de extensión para reportes
  backups/         punto de extensión de restauración
  errors/          errores de dominio
```

La migración `0001_initial_schema.sql` crea la base local para seguridad, pacientes, agenda, clínica, odontograma, tratamientos, finanzas, archivos, inventario, alertas, reportes y respaldos.

## Escalabilidad Preparada

- IDs UUID en texto.
- `clinic_id` desde la primera migración.
- Auditoría centralizada.
- Folios por entidad.
- Separación de comandos, servicios y persistencia.
- Respaldo ZIP con manifiesto.
- Preparación para extraer repositorios por dominio antes de PostgreSQL/sync.

## Límites Actuales

Esta base no implementa aún pagos completos, caja, inventario operativo, reportes exportables, consentimientos, firma digital, WhatsApp API, sync cloud ni IA administrativa. Esos módulos están preparados en navegación y esquema para continuar sin rehacer la arquitectura.
