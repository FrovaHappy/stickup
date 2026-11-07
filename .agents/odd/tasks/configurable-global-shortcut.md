# Task: Acceso por Tecla Configurable (Global Shortcut) para Mostrar/Ocultar StickUp

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src-tauri/Cargo.toml` (dependencia `tauri-plugin-global-shortcut`)
  - `src-tauri/capabilities/default.json` (permiso `global-shortcut:default`)
  - `src-tauri/src/db.rs` (persistencia del shortcut en la tabla `settings`, funciones `get_global_shortcut` y `set_global_shortcut`)
  - `src-tauri/src/commands.rs` (comandos `get_global_shortcut` y `set_global_shortcut`)
  - `src-tauri/src/lib.rs` (inicialización de plugin con `with_handler`, registro del shortcut activo y alternancia de ventana)
  - `src/lib/services/api.ts` (wrapper tipado de `get_global_shortcut` y `set_global_shortcut`)
  - `src/lib/services/system.ts` (métodos `getGlobalShortcut` y `setGlobalShortcut` en `SystemService`)
  - `src/lib/stores/systemSettings.ts` (store reactivo `globalShortcut`, helpers de actualización y feedback con `toast`)
  - `src/lib/stores/systemSettings.test.ts` (tests unitarios para el store del atajo global)
  - `src/routes/settings/+page.svelte` (interfaz visual para visualizar, grabar y restablecer el atajo de teclado en "Comportamiento del Sistema")

## Objetivo y Alcance
1. **Atajo Global por Defecto**:
   - Por defecto: `Win + Ctrl + .` (en formato acelerador Tauri: `"Super+Control+Period"`).
   - Al ser presionado desde cualquier parte del sistema operativo:
     - Si la ventana está oculta o no tiene el foco: mostrarla (`window.show()`), desminimizarla (`window.unminimize()`) y enfocarla (`window.set_focus()`).
     - Si la ventana ya está visible y tiene el foco: ocultarla (`window.hide()`).
2. **Configurabilidad y Persistencia**:
   - Almacenar el atajo configurado en la tabla `settings` de SQLite (`emotes.db` / `.store.db`) con la clave `global_shortcut`.
   - Si no hay valor persistido o está vacío, utilizar por defecto `"Super+Control+Period"`.
   - Permitir al usuario cambiar el atajo desde la interfaz de Configuración (`/settings`), grabando combinaciones de teclas o restableciendo el valor por defecto.
   - Si el atajo ingresado no es válido o hay conflicto de registro a nivel OS, notificar el error sin romper la persistencia previa.
3. **Frontend y UX**:
   - Representar visualmente el atajo actual mediante badges o teclas estilizadas (ej. `Win` + `Ctrl` + `.`).
   - Modo de captura/grabación de atajo con validación de teclas y soporte para restablecer ("Por defecto").
   - Integración armoniosa con el sistema de diseño oscuro y naranja en la sección "Comportamiento del Sistema".

## Checklist de Ejecución
- [x] 1. Documentación de la tarea en `.agents/odd/tasks/configurable-global-shortcut.md`
- [x] 2. Dependencias de backend (`tauri-plugin-global-shortcut` en `src-tauri/Cargo.toml` y permisos en `capabilities/default.json`)
- [x] 3. Persistencia en SQLite (`src-tauri/src/db.rs`) con tests unitarios TDD
- [x] 4. Comandos IPC en `src-tauri/src/commands.rs` y registro del plugin con handler de alternancia de ventana en `src-tauri/src/lib.rs`
- [x] 5. Tipos y servicios en frontend (`src/lib/services/api.ts`, `src/lib/services/system.ts`)
- [x] 6. Store y tests unitarios en frontend (`src/lib/stores/systemSettings.ts`, `src/lib/stores/systemSettings.test.ts`)
- [x] 7. Componente UI en `/settings` para visualizar, capturar y restablecer el atajo
- [x] 8. Verificación integral (`cargo check`, `cargo test`, `pnpm run check`, `pnpm test`, `prebuild`)

## Notas de Continuidad / Decisiones
- El plugin oficial `tauri-plugin-global-shortcut` (v2) gestiona los atajos a nivel de sistema operativo de forma no bloqueante.
- El handler del plugin en Rust opera en segundo plano independientemente de si la ventana de Tauri está visible u oculta.
- Al activarse el atajo global: si la ventana está visible y con foco, se oculta; si está en segundo plano u oculta, se muestra, desminimiza y enfoca.
- Si el registro de un atajo nuevo falla en el sistema operativo, se realiza un rollback automático al atajo anterior y se notifica al usuario.
