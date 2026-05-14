# SaaS Migration Plan

## Alcance

Esta fase no migra a SaaS. Sólo prepara el sistema local para que la lógica, datos y UI puedan evolucionar hacia una plataforma web multi-tenant dentro de Code Solutions Studio.

## Principios

- SQLite sigue siendo la fuente local.
- Tauri sigue siendo el contenedor desktop.
- No se conecta Neon.
- No se conecta Stripe.
- No se conecta R2.
- No se copia código a la web principal.

## Preparación Técnica Actual

- `clinic_id` ya existe en la mayoría de tablas críticas.
- Licencia local modela `customer_id`, `subscription_id` y plan.
- Respaldos tienen manifest y conteos.
- Restauración staged evita sobrescrituras directas.
- Comandos Rust concentran validación de dominio.
- React usa servicios por módulo que luego pueden apuntar a API HTTP.

## Conversión A API

Los servicios Rust se deben convertir por dominio:

- `auth_service`: autenticación central y sesiones web.
- `patient_service`: API de pacientes.
- `appointment_service`: agenda y disponibilidad.
- `clinical_service`: expediente y evolución.
- `odontogram_service`: odontograma.
- `office_service`: tratamientos, presupuestos, pagos, caja, inventario, archivos, usuarios, configuración.
- `backup_service`: respaldos locales, exportación y herramienta de migración.
- `license_service`: cliente de licencia remota.

## Componentes React Reutilizables

Reutilizables con ajustes:

- tablas de pacientes, citas, inventario y reportes;
- formularios con React Hook Form y Zod;
- badges de estado;
- layouts internos;
- páginas de dashboard y reportes.

Requieren rediseño:

- login y sesión;
- permisos multi-tenant;
- archivos y descargas;
- backup/restore local;
- configuración de suscripción;
- experiencia móvil.

## Multi-Tenant

Toda tabla SaaS debe tener:

- `client_id` si hay organización dueña;
- `clinic_id` para consultorio;
- `created_at`;
- `updated_at`;
- `deleted_at` cuando aplique.

Las queries deben filtrar por tenant desde el backend, no desde el frontend.

## Migración De Datos

1. Verificar respaldo local.
2. Exportar manifest.
3. Crear tenant en SaaS.
4. Transformar SQLite a PostgreSQL.
5. Subir archivos a R2.
6. Verificar conteos y checksums.
7. Ejecutar prueba de lectura clínica.
8. Activar suscripción.
9. Mantener copia local en modo sólo lectura como contingencia.

## Riesgos

- diferencias de tipos entre SQLite y PostgreSQL;
- datos históricos con estados no normalizados;
- archivos locales faltantes;
- referencias financieras incompletas;
- expectativas de operación offline;
- privacidad y permisos por rol en web;
- migraciones grandes sin ventana de mantenimiento.
