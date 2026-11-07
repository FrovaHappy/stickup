# Task: Sistema de Escáner de URLs y Carga Manual de HTML para Extracción y Guardado de Imágenes

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src-tauri/src/models.rs` (estructuras `ScannedImage`, `ScannedImageImportRequest`, `BatchImportResult`)
  - `src-tauri/src/services/scanner/index.rs` (lógica pura de parsing HTML, extracción de fuentes de imágenes, resolución de URLs relativas y curl para scan de URL)
  - `src-tauri/src/commands.rs` (comandos `scan_url`, `scan_html`, `import_scanned_images`)
  - `src-tauri/src/lib.rs` (registro de comandos IPC)
  - `src/lib/types/scanner.ts` (contratos TypeScript para imágenes escaneadas y resultados de importación)
  - `src/lib/services/api.ts` (métodos de invocación `scanUrl`, `scanHtml`, `importScannedImages`)
  - `src/routes/scanner/+page.svelte` (interfaz visual del escáner con modos URL y HTML, previsualización en grilla, selector múltiple y guardado a la biblioteca)
  - `src/routes/+layout.svelte` (icono de navegación en la barra lateral hacia `/scanner`)

## Objetivo y Alcance
1. **Escaneo de URLs y Carga Manual de HTML**:
   - Permitir al usuario ingresar una URL web para descargar su contenido HTML de forma segura desde el backend y extraer todas las imágenes candidatas (`<img>`, `data-src`, `srcset`, `<picture>`, estilos `background-image`, enlaces `<a>` a formatos de imagen, `data:image` inline).
   - Permitir la carga manual de una estructura HTML (vía pegado de texto o carga de archivo `.html`) con soporte para URL base opcional (`<base href>` o input).
   - Resolver rutas relativas contra la URL base y desduplicar resultados manteniendo el orden original.
2. **Selección y Gestión Visual**:
   - Visualización de imágenes en grilla responsiva con previsualización (`referrerpolicy="no-referrer"`), nombre PascalCase sugerido, formato y URL de origen.
   - Controles de selección: seleccionar todo, deseleccionar todo, selección individual por clic o checkbox, y contador reactivo.
3. **Guardado en la Biblioteca StickUp**:
   - Guardar las imágenes seleccionadas en lote o individualmente utilizando el flujo estándar de optimización y almacenamiento de StickUp (WebP/GIF, registro en SQLite `emotes`).
   - Notificación de progreso y confirmación mediante `Toast`.

## Checklist de Ejecución
- [x] 1. Definición de contratos y modelos de datos en `src-tauri/src/models.rs` y `src/lib/types/scanner.ts`
- [x] 2. Lógica de parsing y extracción en `src-tauri/src/services/scanner/index.rs` con tests TDD deterministas (`cargo test`)
- [x] 3. Comandos Tauri en `src-tauri/src/commands.rs` y registro en `src-tauri/src/lib.rs`
- [x] 4. Integración en `src/lib/services/api.ts`
- [x] 5. Vista interactiva en `src/routes/scanner/+page.svelte` con estilos de StickUp Design System
- [x] 6. Enlace de navegación en `src/routes/+layout.svelte` con icono `@lucide/svelte`
- [x] 7. Soporte para reglas de Discord (diferenciación entre WebP estático y GIF animado por query param `animated=true`, extensión `.gif`, prefijo `a_` en hashes y sintaxis de emojis personalizados `<a:name:id>` / `<:name:id>`)
- [x] 8. Verificación final de tests, compilación y Zero Dead Code

## Notas de Continuidad / Decisiones
- El motor de descarga reutiliza `curl` en el backend para evitar problemas de CORS y bloqueos de User-Agent.
- Las imágenes extraídas se desduplican por URL normalizada preservando atributos relevantes (alt / title) para sugerir nombres adecuados.
- Reglas de Discord implementadas:
  - `cdn.discordapp.com` y `media.discordapp.net`:
    - `animated=true`: reconocido como animación (`format_hint: "gif"`).
    - Prefijo `a_` en hash de avatars/iconos/emojis: reconocido como animación (`format_hint: "gif"`).
    - Extensión `.gif`: animado.
    - Sin indicador de animación y con extensión `.webp` o `.png`: reconocido como estático (`format_hint: "webp"` o `"png"`).
  - Soporte de sintaxis nativa de Discord en HTML/texto: `<a:name:id>` (animado -> .gif) y `<:name:id>` (estático -> .webp).
  - Detección de URLs directas de imagen (incluyendo links directos de Discord) en `scan_url` para procesarlas de inmediato sin requerir envoltorio HTML.
