# Tarea: Control de Bucle (Loop) en GIFs (Global e Individual)

## 1. Contexto y Problema
En el estándar GIF89a, las animaciones se detienen tras reproducirse una vez a menos que incluyan una extensión de aplicación (`NETSCAPE2.0` o `ANIMEXTS1.0`) que indique repeticiones. Muchos GIFs importados desde la web o mensajería no contienen este bloque o tienen un contador finito (ej. 1 repetición), lo que causa que se congelen en el último fotograma tanto dentro de la app como al pegarlos fuera (en Discord, Telegram, etc.).
El usuario requiere:
- **Ajuste global:** Configurar en Ajustes si los GIFs deben normalizarse a bucle infinito al importarse.
- **Ajuste individual:** Poder inspeccionar y cambiar el comportamiento de bucle (infinito vs frenar tras 1 reproducción) directamente en el modal de detalle del emote, afectando tanto la visualización en la app como el archivo al copiarlo fuera.

## 2. Enfoque Técnico
1. **Servicio dedicado de backend (`src-tauri/src/services/gif/index.rs`):**
   - Lectura / inspección binaria sin decodificación pesada (`get_gif_loop_status`).
   - Parcheo binario sin pérdida de calidad ni re-codificación (`patch_gif_loop`):
     - Si `infinite == true`: setea sub-bloque a `0x00, 0x00` o inserta bloque `NETSCAPE2.0` tras la Global Color Table.
     - Si `infinite == false`: setea sub-bloque a `0x01, 0x00` (reproducir 1 vez y frenar).
2. **Comandos IPC y Persistencia (`db.rs`, `commands.rs`, `lib.rs`):**
   - `get_force_gif_loop` y `set_force_gif_loop` almacenados en tabla `settings` (default: `true`).
   - `get_emote_gif_loop(id: i64) -> Result<GifLoopStatus, String>`
   - `set_emote_gif_loop(id: i64, infinite: bool) -> Result<GifLoopStatus, String>`
   - En `import_*`: Si `force_gif_loop` está activo, parchear GIFs a bucle infinito al guardarlos.
   - En `copy_emote`: Si un GIF animado se redimensiona, respetar si el original era infinito o finito en lugar de forzar siempre infinito.
3. **Frontend (`systemSettings.ts`, `api.ts`, `settings/+page.svelte`, `EmoteDetailModal.svelte`):**
   - Store y ajuste en `settings/+page.svelte`.
   - Control interactivo en `EmoteDetailModal.svelte` con badge de estado y botón de alternancia.

## 3. Checklist de Implementación
- [x] 1. Crear servicio `src-tauri/src/services/gif/index.rs` con tests unitarios TDD (`test_detect_loop`, `test_patch_to_infinite`, `test_patch_to_finite`).
- [x] 2. Añadir helpers en `db.rs` para `get_force_gif_loop` y `set_force_gif_loop` con tests unitarios.
- [x] 3. Implementar comandos Tauri en `commands.rs` y registrarlos en `lib.rs`.
- [x] 4. Ajustar importación y redimensionado en `clipboard/mod.rs` para respetar o forzar bucle según configuración.
- [x] 5. Actualizar types y wrapper `api.ts` en frontend.
- [x] 6. Añadir store y UI de configuración en `systemSettings.ts` y `settings/+page.svelte`.
- [x] 7. Añadir control de bucle en `EmoteDetailModal.svelte`.
- [x] 8. Actualizar `changelogs/unreleased.md`.
- [x] 9. Ejecutar `pnpm run prebuild` y verificar compilación y tests al 100%.
