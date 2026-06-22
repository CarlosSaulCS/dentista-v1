# Local Installation Manual

## Instalación Limpia

1. Instalar Dentista v1 Professional desde MSI o NSIS firmado.
2. Abrir la aplicación.
3. Confirmar que aparece setup inicial cuando no existe SQLite local.
4. Capturar datos del consultorio y usuario administrador.
5. Entrar con el usuario administrador.
6. Confirmar que el dashboard carga sin conexión a internet.

## Primer Uso

1. Crear un paciente de prueba.
2. Crear una cita para ese paciente.
3. Cambiar la cita a `confirmada`.
4. Crear un presupuesto.
5. Registrar un pago.
6. Abrir caja, revisar movimientos y cerrar caja.
7. Crear un item de inventario y registrar entrada/salida.
8. Generar reporte de rango del día.

## Respaldos

1. Ir a Respaldos.
2. Crear respaldo manual.
3. Verificar el ZIP.
4. Guardar la ruta del archivo.
5. Seleccionar el ZIP para restauración.
6. Revisar preview.
7. Preparar restauración.
8. Reiniciar la app.
9. Confirmar que la app abre y los datos esperados siguen disponibles.

## Trabajo Offline

1. Desconectar internet.
2. Iniciar sesión.
3. Crear paciente, cita, presupuesto y pago.
4. Crear respaldo.
5. Entrar a Configuración > Portal remoto.
6. Confirmar que no se requiere `DV1_SYNC_BASE_URL` para seguir operando.
7. Si se intenta sincronizar sin bridge activo, tratar el mensaje como advertencia.

## Licencia Expirada

1. Forzar una licencia expirada en entorno de prueba.
2. Confirmar que el login sigue permitido.
3. Confirmar que pacientes, citas, pagos, inventario y reportes se pueden consultar.
4. Confirmar que crear o editar datos queda bloqueado.
5. Confirmar que crear y verificar respaldo sigue permitido.
6. Confirmar que preparar restore queda bloqueado hasta activar licencia.

## Portal Remoto

El Portal remoto/Bridge DV1 es opcional. No configurar este campo durante una instalación local
normal. Usarlo solo cuando exista un bridge independiente con rutas `/api/dv1/*`.

No usar `css-aion-new` como backend de Dentista v1.
