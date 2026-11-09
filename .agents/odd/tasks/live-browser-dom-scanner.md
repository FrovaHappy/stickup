# Task: Escáner Interactivo de Navegación Web (Captura Periódica de DOM en Vivo)

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src-tauri/src/models.rs` (contratos para URLs recolectadas `RawScannedItem` y `ScannedImage`)
  - `src-tauri/src/services/scanner/index.rs` (normalización de URLs en lote, desduplicación, inferencia de formato y generación de nombres PascalCase)
  - `src-tauri/src/commands.rs` (comando IPC `process_scanned_urls`)
  - `src-tauri/src/lib.rs` (registro de comandos IPC)
  - `src/lib/types/scanner.ts` (contratos TypeScript `RawScannedItem` y wrappers de escaneo)
  - `src/lib/services/api.ts` (wrapper de API para invocar `process_scanned_urls`)
  - `src/lib/utils/domCollector.ts` (generador de script inyectable/bookmarklet con panel flotante, polling cada 1s y parser tolerante)
  - `src/lib/utils/domCollector.test.ts` (batería de pruebas unitarias para el colector)
  - `src/routes/scanner/+page.svelte` (nueva pestaña "Captura en vivo" con integración a la grilla y controles intuitivos)

## Objetivo y Alcance

1. **Modo Captura Interactiva ("Capturar" y "Parar"):**
   - Permitir al usuario navegar por cualquier sitio web (Twitch, 7TV, Discord, Reddit, etc.) con una sesión de captura activa.
   - Polling automático cada 1 segundo (vía `setInterval` / `document.images`) extrayendo URLs de imágenes renderizadas en el DOM.
   - Desduplicación automática continua mediante `Set<string>`.
   - Filtro inteligente de dimensiones (`naturalWidth` / `naturalHeight` entre 16px y 512px) para ignorar banners publicitarios, iconos de interfaz diminutos y píxeles de rastreo.
2. **Consolidación y Presentación al Parar:**
   - Al pulsar "Parar", convertir el `Set` a un array de URLs y enviarlo al backend de Rust.
   - El backend normaliza las URLs (resolviendo URLs de Discord animadas vs estáticas, formatos WebP/GIF/PNG y nombres sugeridos en PascalCase) y retorna una lista `Vec<ScannedImage>`.
   - Inyección directa de las imágenes en la grilla visual existente de `/scanner` para selección múltiple, previsualización con `referrerpolicy="no-referrer"` y guardado en la biblioteca SQLite/WebP de StickUp.
3. **Mecanismo de Ejecución y Navegación:**
   - Script inyectable y Bookmarklet con panel flotante interactivo, auto-copia al portapapeles y carga directa con 1 clic en StickUp.

## Checklist de Ejecución

- [x] 1. Exploración y definición de contratos / tipos en `src-tauri/src/models.rs` y `src/lib/types/scanner.ts`
- [x] 2. Lógica de backend para procesamiento y resolución de lote de URLs directas en `src-tauri/src/services/scanner/index.rs` con tests unitarios (`cargo test`)
- [x] 3. Comandos IPC en `src-tauri/src/commands.rs` y registro en `src-tauri/src/lib.rs` (`process_scanned_urls`)
- [x] 4. Módulo frontend para el colector periódico de DOM (gestión de `Set<string>`, timer de 1s, normalización absoluta y filtrado de dimensiones) en `src/lib/utils/domCollector.ts` y tests en `src/lib/utils/domCollector.test.ts`
- [x] 5. Integración de la pestaña "Captura en vivo" en `src/routes/scanner/+page.svelte` con botones "Copiar Script (F12)", "Copiar Bookmarklet", "Pegar desde Portapapeles" y migración automática a la grilla de selección
- [x] 6. Verificación (`cargo test --lib`, 42 tests en verde) y Zero Dead Code
- [x] 7. Registro de cambios en `changelogs/unreleased.md` según la skill `release-notes`

## Notas de Continuidad / Decisiones

- **Enfoque técnico acordado:** Recuperación periódica del DOM cada 1 segundo en lugar de captura visual por screenshots (mayor fidelidad, menor consumo de CPU, soporte nativo de canal alfa/transparencia y animaciones GIF/WebP).
- **Desduplicación:** `Set<string>` en memoria durante la sesión de captura; desduplicación secundaria en Rust por URL limpia y hash.
- **Reutilización:** Se aprovechan directamente las estructuras `ScannedImage`, `ScannedImageImportRequest` y el comando `import_scanned_images` existentes en el proyecto.

