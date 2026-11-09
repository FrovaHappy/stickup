# Task: Selección de Resolución Favorita y Minimizado Automático al Copiar

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src-tauri/src/db.rs` (persistencia de `minimize_on_copy` en tabla `settings` y tests TDD)
  - `src-tauri/src/commands.rs` (comandos `get_minimize_on_copy`, `set_minimize_on_copy`, `minimize_window`)
  - `src-tauri/src/lib.rs` (registro de comandos IPC en `invoke_handler`)
  - `src/lib/services/api.ts` (wrappers tipados Tauri)
  - `src/lib/services/system.ts` (métodos en `SystemService`)
  - `src/lib/stores/sizes.ts` (store reactivo `defaultResolution`, `setDefaultResolution`, `resolveDefaultDimensions`)
  - `src/lib/stores/sizes.test.ts` (tests unitarios para resolución favorita y resolución de dimensiones)
  - `src/lib/stores/systemSettings.ts` (store reactivo `minimizeOnCopyEnabled`, `setMinimizeOnCopySetting`)
  - `src/lib/stores/systemSettings.test.ts` (tests unitarios para `minimizeOnCopyEnabled`)
  - `src/routes/settings/+page.svelte` (UI para seleccionar resolución favorita y toggle de minimizar al copiar)
  - `src/routes/+page.svelte` (uso de resolución favorita en atajo `Ctrl+C` y menú contextual; minimizado reactivo)
  - `src/lib/components/detail/EmoteDetailModal.svelte` (resolución por defecto favorita y minimizado al copiar)

## Objetivo y Alcance
1. **Resolución Favorita por Defecto (Afecta al Shortcut)**:
   - Permitir al usuario elegir su resolución favorita o predeterminada entre todas las resoluciones disponibles (`Original`, `48x48`, `64x64`, `128x128`, `256x256`, `512x512` y tamaños personalizados).
   - Por defecto: `'original'`.
   - El atajo de teclado (`Ctrl+C` sobre un emote en el grid) debe copiar automáticamente en la resolución favorita elegida.
   - El menú contextual refleja dinámicamente la resolución favorita en la acción del shortcut (`Copiar emote ({Resolución Favorita}) - Ctrl+C`).
   - El modal de detalle de emote (`EmoteDetailModal`) inicializa la resolución seleccionada con la favorita por defecto.
   - Si se elimina un tamaño personalizado que era la resolución favorita, se restablece automáticamente a `'original'`.

2. **Minimizar la Aplicación al Copiar (Desactivada por Defecto)**:
   - Opción en "Comportamiento del Sistema" con switch desactivado por defecto (`false`).
   - Persistir la preferencia en SQLite (`settings.minimize_on_copy`).
   - Al copiar cualquier emote al portapapeles (mediante atajo `Ctrl+C`, menú contextual o modal de detalle), si la opción está activada, minimizar automáticamente la ventana de StickUp a la barra de tareas.

## Checklist de Ejecución
- [x] 1. Crear documento de seguimiento ODD en `.agents/odd/tasks/favorite-resolution-and-minimize-on-copy.md`
- [x] 2. Backend (Rust): Persistencia de `minimize_on_copy` en `src-tauri/src/db.rs` con tests unitarios TDD
- [x] 3. Backend (Rust): Comandos `get_minimize_on_copy`, `set_minimize_on_copy`, `minimize_window` en `commands.rs` y registro en `lib.rs`
- [x] 4. Frontend: Stores y servicios (`api.ts`, `system.ts`, `systemSettings.ts`, `sizes.ts`)
- [x] 5. Frontend: Tests unitarios (`systemSettings.test.ts`, `sizes.test.ts`)
- [x] 6. Frontend: Integración en vistas (`+page.svelte`, `EmoteDetailModal.svelte`, `settings/+page.svelte`)
- [x] 7. Verificación completa (`cargo test --manifest-path src-tauri/Cargo.toml --lib`, chequeos de compilación y Zero Dead Code)

## Notas de Continuidad / Decisiones
- La resolución favorita se almacena en `localStorage` mediante `stickup_pref_default_resolution`, coexistiendo fluidamente con `stickup_custom_sizes`.
- `minimize_on_copy` se guarda en SQLite en la tabla `settings`, manteniendo paridad de diseño con `minimize_to_tray` y `global_shortcut`.
- `minimize_window` utiliza `app.get_webview_window("main").minimize()`, asegurando que la minimización funcione en Windows, macOS y Linux.
