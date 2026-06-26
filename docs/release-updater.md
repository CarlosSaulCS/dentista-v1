# Dentista v1 Professional Release And Updater

## Updater Tauri v2

La app usa `tauri-plugin-updater` con manifest estático:

```json
{
  "version": "0.1.0",
  "notes": "Dentista v1 Professional update",
  "pub_date": "2026-06-21T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "contenido-del-archivo.sig",
      "url": "https://releases.codesolutionstudio.com.mx/dentista-v1/Dentista-v1-Professional.msi"
    }
  }
}
```

La llave privada de firma no debe guardarse en el repositorio. Genera el par de llaves con la CLI de Tauri, guarda la privada en un secreto del pipeline y pega la llave pública en `src-tauri/tauri.conf.json` en `plugins.updater.pubkey`.

Mientras `plugins.updater.pubkey` tenga `REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY`, el updater queda
como integración preparada, no como canal productivo. No agregar botones de actualización ni publicar
un `latest.json` real hasta tener llave pública, firma del instalador y endpoint final validados.

## Build Windows

```powershell
npm run release:windows
```

El build genera instaladores NSIS/MSI bajo `src-tauri/target/release/bundle`.

## Manifest

```powershell
npm run release:manifest:windows -- -InstallerUrl "https://releases.codesolutionstudio.com.mx/dentista-v1/Dentista-v1-Professional.msi" -SignaturePath "src-tauri/target/release/bundle/nsis/Dentista v1 Professional_0.1.0_x64-setup.nsis.zip.sig"
```

Publica `release/latest.json` en el endpoint configurado:

```text
https://releases.codesolutionstudio.com.mx/dentista-v1/latest.json
```

## Checklist Smoke

- instalar MSI/NSIS en Windows limpio;
- abrir app y validar SQLite existente;
- verificar que `get_sync_status` no expone tokens;
- publicar manifest con firma del artefacto real;
- confirmar que una versión superior aparece como update;
- validar rollback manual con respaldo local antes de instalar builds beta.
