# Task: Extracción e Integración de Componentes Base (`system-ui`)

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-componente
- **Archivos clave:**
  - `src/lib/types/system-ui.ts`
  - `src/lib/components/system-ui/*`
  - `src/lib/components/EmoteCard.svelte`
  - `src/lib/components/ImportButton.svelte`
  - `src/lib/components/StorageSettings.svelte`
  - `src/lib/components/AddEmoteModal.svelte`
  - `src/lib/components/SearchFilterBar.svelte`
  - `src/lib/components/detail/TagManager.svelte`
  - `src/lib/components/detail/EmoteDetailModal.svelte`

## Objetivo y Alcance
1. Extraer los primitivos de interfaz reutilizables dentro de `src/lib/components/system-ui/`.
2. Integrar `system-ui` en todos los componentes de dominio de la aplicación eliminando estilos y estructuras duplicadas y cumpliendo el principio **Zero Dead Code** (todos los componentes de `system-ui` en uso activo).

## Checklist de Ejecución
- [x] 1. Exploración en modo lectura de todos los componentes en `src/lib/components/`
- [x] 2. Definición de tipos compartidos en `src/lib/types/system-ui.ts`
- [x] 3. Implementación de primitivos de acción y entrada (`Button`, `Input`, `Chip`, `Badge`)
- [x] 4. Implementación de contenedores y superposiciones (`Modal`, `Dropdown`, `Dropzone`, `Alert`, `Card`) y barrel `index.ts`
- [x] 5. Corte 1 — Integrar `system-ui` en `EmoteCard`, `ImportButton`, `StorageSettings` y `TagManager`
- [x] 6. Corte 2 — Integrar `system-ui` en `AddEmoteModal`, `EmoteDetailModal` y `SearchFilterBar`
- [x] 7. Verificación final (`pnpm run check` — 0 errores/warnings, `pnpm run test` — 7/7 tests en verde)

## Notas de Continuidad / Decisiones
- Todos los 9 componentes de `system-ui` (`Alert`, `Badge`, `Button`, `Card`, `Chip`, `Dropdown`, `Dropzone`, `Input`, `Modal`) están activamente consumidos en la aplicación (Zero Dead Code), reduciendo el código CSS duplicado en más de 800 líneas en los componentes de dominio.
