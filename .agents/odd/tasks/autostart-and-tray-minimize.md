# Task: Inicio Automático con el Sistema Operativo y Minimizar en la Bandeja del Sistema

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src-tauri/Cargo.toml` (dependencia `tauri-plugin-autostart` y feature `tray-icon`)
  - `src-tauri/capabilities/default.json` (permiso `autostart:default`)
  - `src-tauri/src/lib.rs` (configuración del TrayIcon, menú contextual de bandeja, interceptor CloseRequested y plugin autostart)
  - `src-tauri/src/db.rs` (persistencia del valor `minimize_to_tray` en la tabla `settings`)
  - `src-tauri/src/commands.rs` (comandos `get_minimize_to_tray` y `set_minimize_to_tray`)
  - `package.json` (`@tauri-apps/plugin-autostart`)
  - `src/lib/services/system.ts` (servicio de integración con autostart y preferencia de bandeja)
  - `src/lib/stores/systemSettings.ts` (store reactivo para autostart y minimize_to_tray con tests unitarios)
  - `src/routes/settings/+page.svelte` (sección "Comportamiento del Sistema" con toggles interactivos)

## Objetivo y Alcance
1. **Inicio con el Sistema Operativo (Autostart)**:
   - Permitir al usuario activar o desactivar que StickUp inicie automáticamente con el sistema operativo (Windows).
   - Si se inicia con el sistema, opcionalmente arrancar en modo minimizado/oculto en la bandeja del sistema si la opción de minimizar a la bandeja está activa.
2. **Minimizar en la Bandeja del Sistema (Tray)**:
   - Crear el icono de bandeja del sistema (System Tray Icon) usando `tauri::tray::TrayIconBuilder`.
   - Menú contextual de la bandeja con opciones: "Mostrar StickUp" y "Salir".
   - Clic izquierdo en el icono de la bandeja para alternar visibilidad (mostrar/ocultar y enfocar la ventana).
   - Opción configurable en `/settings` para "Minimizar en la bandeja del sistema":
     - Si está activa: al hacer clic en cerrar [X], la ventana se oculta a la bandeja (`window.hide()`) sin terminar el proceso.
     - Si está inactiva: la ventana se cierra normalmente saliendo de la aplicación.
3. **Persistencia y UI**:
   - Persistir la preferencia de minimizar a la bandeja en la base de datos SQLite y reflejar el estado en tiempo real.
   - Añadir una nueva sección elegante en `/settings` ("Comportamiento del Sistema") con interruptores accesibles y retroalimentación mediante `toast`.

## Checklist de Ejecución
- [x] 1. Configuración de dependencias (`tauri-plugin-autostart`, `@tauri-apps/plugin-autostart`, capabilities)
- [x] 2. Persistencia en base de datos (`db.rs`) y comandos IPC (`commands.rs`) para `minimize_to_tray`
- [x] 3. Configuración de Tray Icon, menú y manejo de `CloseRequested` en Rust (`lib.rs`)
- [x] 4. Stores y servicios frontend con tests unitarios (`systemSettings.ts`, `systemSettings.test.ts`)
- [x] 5. Integración visual en `/settings` con controles toggle y estilos coherentes
- [x] 6. Verificación integral (`cargo check`, `cargo test`, `pnpm run check`, `pnpm test`, `prebuild`)

## Notas de Continuidad / Decisiones
- Se utiliza `tauri-plugin-autostart` oficial de Tauri v2.
- El menú del tray ofrece "Mostrar StickUp" y "Salir" para garantizar que el usuario siempre pueda terminar la aplicación completamente.
- Clic izquierdo sobre el icono del tray conmuta entre ocultar y mostrar la ventana activa.
- Para autostart silencioso en boot, se configura el flag `--hidden` y en `lib.rs` se oculta la ventana si el argumento está presente.
