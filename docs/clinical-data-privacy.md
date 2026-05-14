# Clinical Data Privacy

## Datos Sensibles

El sistema maneja datos personales, clínicos y financieros:

- pacientes y contactos;
- antecedentes, alergias, padecimientos y medicamentos;
- expedientes, evoluciones, odontograma y periodontal;
- consentimientos;
- archivos clínicos;
- citas;
- presupuestos, pagos y caja.

## Principios Técnicos

- Mínimo acceso por rol.
- Auditoría de acciones críticas.
- Baja lógica para evitar pérdida destructiva.
- Trazabilidad en expedientes y evoluciones.
- Respaldos verificables.
- Exportación controlada.
- Preparación para multi-clínica y SaaS.

## Roles

La aplicación debe separar funciones como:

- administración;
- odontólogo;
- recepción;
- caja;
- sólo lectura o auditoría.

Las acciones administrativas, financieras, de respaldo y restauración requieren permisos explícitos.

## Baja Lógica

Los pacientes y recursos críticos no deben borrarse destructivamente desde la operación normal. La baja lógica conserva historial clínico, financiero y de auditoría.

## Exportación

Las exportaciones deben registrarse en `report_exports` y, cuando correspondan a datos clínicos, deben ser tratadas como información sensible. El consultorio es responsable de proteger archivos exportados fuera de la app.

## Respaldos

Los respaldos contienen SQLite y archivos clínicos. Deben guardarse en una ubicación protegida, verificarse periódicamente y cifrarse en una fase futura antes de enviarse a nube.

## Consentimiento

Los consentimientos generados deben asociarse a paciente, tratamiento o archivo clínico. La plantilla legal final debe revisarla el consultorio con asesoría profesional.

## Futura Nube

Para SaaS será necesario:

- aislamiento estricto por `clinic_id`;
- autenticación central;
- control de sesiones web;
- almacenamiento de archivos con URLs temporales;
- bitácora de acceso a expediente;
- políticas de retención y exportación;
- contratos y avisos de privacidad por jurisdicción.

## Responsabilidad Del Consultorio

Esta documentación es una base técnica. No sustituye asesoría legal, fiscal o regulatoria. El consultorio debe definir políticas internas de acceso, respaldo, exportación, conservación y respuesta a incidentes.
