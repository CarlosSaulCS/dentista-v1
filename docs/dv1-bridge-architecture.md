# Arquitectura Dentista v1 Bridge

## Principio

Dentista v1 Professional es software local instalable para Windows. La base SQLite local sigue
siendo la fuente de verdad para agenda, pacientes, expedientes, pagos, inventario y operación diaria.

`css-aion-new` es una página/SaaS separado y ya productivo. No debe recibir migraciones, rutas,
layouts, settings, adapters ni endpoints nuevos para integrar Dentista v1. El SaaS dental existente
no es el backend de Dentista v1.

El bridge/portal de Dentista v1 debe ser un servicio independiente. Puede convivir dentro del
ecosistema web y de soporte de Code Solution Studio, pero debe tener despliegue, rutas, datos y
contratos propios.

## Alcance de Fase 1

El bridge solo maneja:

- agenda administrativa y proyecciones mínimas de citas;
- confirmación/cancelación remota por comandos idempotentes;
- estado de comandos `pending`, `applied` y `failed`;
- pairing de instalación/dispositivo;
- auditoría mínima de register, push, pull, ack y revoke;
- reportes operativos mínimos cuando existan datos proyectados suficientes.

No se sincroniza expediente clínico completo, odontograma, notas clínicas, archivos clínicos, pagos,
facturación, presupuestos, inventario ni documentos privados. Esos datos permanecen locales.

## Repositorio Sugerido

Nombre recomendado:

- `dentista-v1-bridge`

Alternativa:

- `dentalcare-manager-portal`

## Rutas Sugeridas

```text
/api/dv1/register-installation
/api/dv1/sync/refresh
/api/dv1/sync/push
/api/dv1/sync/pull
/api/dv1/sync/ack
/api/dv1/mobile/appointments
/api/dv1/mobile/appointments/[id]/confirm
/api/dv1/mobile/appointments/[id]/cancel
```

## Datos

El bridge debe usar base de datos o schema propio. No debe depender de tablas productivas del SaaS
dental actual, incluyendo:

- `dental_clinics`
- `dental_members`
- `dental_quotes`
- `dental_patients`
- `dental_appointments`
- cualquier otra entidad clínica/productiva de `css-aion-new`

Tablas sugeridas para el bridge:

- `dv1_pairing_codes`
- `dv1_installations`
- `dv1_devices`
- `dv1_appointment_projections`
- `dv1_sync_events`
- `dv1_remote_commands`
- `dv1_remote_command_receipts`
- `dv1_audit_events`

## Contrato Operativo

Dentista v1 empuja eventos locales al bridge. El bridge guarda proyecciones y expone una vista móvil
limitada. Cuando un usuario remoto confirma o cancela una cita, el bridge encola un comando estable
con `commandId`. Dentista v1 hace pull, aplica el cambio sobre SQLite local, registra auditoría local
y envía ack.

El bridge no debe marcar una cita como aplicada hasta recibir ack de la instalación local. Si el
dispositivo está revocado, pull y ack deben fallar de forma segura.

## Configuración Local

Dentista v1 puede recibir la URL base del bridge desde:

- Configuración > Portal remoto;
- variable de entorno `DV1_SYNC_BASE_URL`.

La app local debe seguir funcionando aunque el bridge no exista o esté fuera de línea. En ese caso,
se acumulan eventos locales en `sync_outbox` y las acciones explícitas de sincronización reportan un
error de conexión/configuración sin bloquear la operación local.
