# Release Checklist

## Preflight Técnico

- Confirmar rama de release limpia.
- Ejecutar `cargo fmt`.
- Ejecutar `cargo check`.
- Ejecutar `cargo test`.
- Ejecutar `npm run typecheck`.
- Ejecutar `npm run lint`.
- Ejecutar `npm run build`.
- Ejecutar `git diff --check`.
- Revisar que `css-aion-new` no tenga cambios relacionados con Dentista v1.
- Confirmar que `src-tauri/tauri.conf.json` usa `Dentista v1 Professional`.

## Pruebas Manuales

- Instalación limpia en Windows.
- Primer uso y setup inicial.
- Login con usuario administrador.
- Dashboard carga sin internet.
- Crear paciente.
- Crear cita.
- Cambiar estatus de cita.
- Crear presupuesto.
- Registrar pago.
- Abrir y cerrar caja.
- Crear item de inventario y movimiento.
- Generar reporte.
- Crear respaldo manual.
- Verificar respaldo.
- Preparar restauración staged.
- Reiniciar y confirmar que la app abre con SQLite restaurado.
- Trabajar offline con `DV1_SYNC_BASE_URL` ausente.
- Probar licencia expirada y confirmar modo sólo lectura.
- Confirmar que modo sólo lectura permite consultar y crear respaldos.
- Confirmar que sync desactivado no bloquea pacientes, citas, caja ni reportes.

## Licencia

- `trial_active`: operación completa local.
- `active`: operación completa local.
- `grace_period`: operación completa temporal con mensaje claro.
- `offline_grace`: operación completa temporal sin internet.
- `expired`, `past_due`, `suspended`, `read_only`: lectura, exportación y respaldos permitidos; escritura y restore bloqueados.

## Updater

- No publicar updater productivo mientras `plugins.updater.pubkey` sea `REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY`.
- Generar par de llaves Tauri fuera del repo.
- Guardar llave privada como secreto de pipeline.
- Firmar el artefacto real MSI/NSIS.
- Publicar `latest.json` con URL y firma reales.
- Probar actualización desde una versión anterior en Windows limpio.

## Bridge DV1

- El bridge remoto es opcional.
- La app debe operar localmente sin `DV1_SYNC_BASE_URL`.
- Fallos de bridge se tratan como advertencias.
- No conectar Dentista v1 al SaaS dental productivo de `css-aion-new`.
