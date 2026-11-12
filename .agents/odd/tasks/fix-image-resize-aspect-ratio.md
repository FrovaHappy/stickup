# Tarea ODD: Preservar Relación de Aspecto al Redimensionar Imágenes

## 1. Contexto y Problema
Al redimensionar emotes o imágenes (tanto presets como 48x48, 128x128, etc., como dimensiones personalizadas o copiado rápido), las imágenes que no son cuadradas (landscape o portrait) se deforman/aplanan/estiran porque las dimensiones se fuerzan de forma exacta al ancho y alto máximos sin respetar su aspect ratio original.

## 2. Objetivos
- Mantener la relación de aspecto original (`aspect-ratio`) ajustando la imagen dentro del cuadro contenedor (bounding box) indicado por el preset o tamaño personalizado.
- En el frontend:
  - Crear/exportar función `fitDimensions(origW, origH, maxW, maxH)` en `format.ts`.
  - Usar `fitDimensions` en `resolveDimensions` y `resolveDefaultDimensions`.
  - Asegurar que `renderEmoteToCanvas` respete el aspect ratio al dibujar en canvas.
  - Asegurar que las acciones del menú contextual en `+page.svelte` calculen dimensiones proporcionales.
  - Actualizar tests unitarios (`format.test.ts` y `sizes.test.ts`).
- En el backend (Rust):
  - Crear `fit_dimensions(orig_w, orig_h, max_w, max_h) -> (u32, u32)` en `src-tauri/src/clipboard/mod.rs`.
  - En `copy_emote`, ajustar `(target_width, target_height)` para respetar el aspect ratio original antes de redimensionar GIFs o imágenes estáticas.
  - Agregar pruebas unitarias en Rust para `fit_dimensions` y redimensionado.
- Documentar en `changelogs/unreleased.md`.
- Verificar con `pnpm run prebuild`.

## 3. Checklist de Ejecución
- [x] Implementar `fitDimensions` en `src/lib/utils/format.ts`
- [x] Actualizar `resolveDimensions` en `src/lib/utils/format.ts`
- [x] Actualizar `resolveDefaultDimensions` en `src/lib/stores/sizes.ts`
- [x] Actualizar menú contextual y copiado en `src/routes/+page.svelte`
- [x] Actualizar y expandir tests en `format.test.ts` y `sizes.test.ts`
- [x] Implementar `fit_dimensions` y ajuste en `src-tauri/src/clipboard/mod.rs`
- [x] Agregar tests en Rust para `fit_dimensions`
- [x] Actualizar `changelogs/unreleased.md`
- [x] Ejecutar `pnpm run prebuild` y verificar cero errores
