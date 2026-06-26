# Files To R2 Migration

## Estado Actual

Los archivos clínicos se almacenan localmente bajo el directorio de datos Tauri:

```text
appDataDir/files/
```

En instalaciones antiguas, `appDataDir` puede conservar una carpeta con el nombre comercial anterior.

La tabla `files` guarda metadatos como paciente, categoría, ruta, MIME, tamaño, hash y entidad relacionada.

## Seguridad Local

La fase local aplica:

- sanitización de nombres;
- límite de tamaño;
- validación de tipo/extensión;
- rutas dentro del directorio controlado;
- inclusión opcional en respaldos.

## Modelo Futuro En R2

R2 debe almacenar objetos con claves estables y tenant-aware:

```text
clinics/{clinic_id}/patients/{patient_id}/files/{file_id}/{safe_filename}
clinics/{clinic_id}/payments/{payment_id}/receipts/{file_id}/{safe_filename}
clinics/{clinic_id}/generated/{document_id}.pdf
```

Campos nuevos recomendados:

- `storage_provider`: `local`, `r2`;
- `bucket`;
- `object_key`;
- `etag`;
- `content_sha256`;
- `uploaded_at`;
- `storage_status`;
- `deleted_at`;

## Endpoints Futuros

- `POST /api/files/presign-upload`
- `POST /api/files/complete-upload`
- `GET /api/files/:id/download-url`
- `POST /api/files/:id/archive`
- `GET /api/patients/:patientId/files`

## Migración

1. Congelar escrituras o ejecutar por lotes con checkpoint.
2. Leer registros `files` locales.
3. Validar existencia y checksum del archivo.
4. Subir a R2.
5. Registrar `object_key`, `etag` y `storage_provider`.
6. Verificar descarga temporal.
7. Mantener copia local hasta completar auditoría.
8. Activar lectura desde R2 en SaaS.

## Riesgos

- archivos faltantes en instalaciones antiguas;
- rutas con nombres no normalizados;
- datos sensibles en descargas persistentes;
- permisos cruzados entre clínicas;
- respaldo incompleto si se mezcla local y nube sin manifest claro.
