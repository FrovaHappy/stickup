# Task: Extracción de Componentes Base (`system-ui`)

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-componente
- **Archivos clave:**
  - `src/lib/types/system-ui.ts`
  - `src/lib/components/system-ui/Button.svelte`
  - `src/lib/components/system-ui/Modal.svelte`
  - `src/lib/components/system-ui/Badge.svelte`
  - `src/lib/components/system-ui/Chip.svelte`
  - `src/lib/components/system-ui/Input.svelte`
  - `src/lib/components/system-ui/Alert.svelte`
  - `src/lib/components/system-ui/Dropdown.svelte`
  - `src/lib/components/system-ui/Dropzone.svelte`
  - `src/lib/components/system-ui/Card.svelte`
  - `src/lib/components/system-ui/index.ts`

## Objetivo y Alcance

Analizar todos los componentes actuales de StickUp (`AddEmoteModal`, `EmoteCard`, `EmoteGrid`, `ImportButton`, `SearchFilterBar`, `StorageSettings`, `EmoteDetailModal`, `TagManager`) y extraer los primitivos de interfaz reutilizables dentro de `src/lib/components/system-ui/` con todas las funcionalidades, estados, eventos, slots y estilos (light/dark mode) necesarios para operar en el proyecto.

En esta etapa solo se crean los componentes de `system-ui` y sus tipos asociados sin alterar todavía los componentes de dominio existentes.

## Mapeo de Componentes Extraídos

1. **`Button.svelte`**:
   - Variantes: `primary`, `secondary`, `accent`, `success`, `danger`, `outline`, `ghost`, `icon`.
   - Tamaños: `sm`, `md`, `lg`.
   - Soporte para `disabled`, `loading`, `fullWidth`, `active`, `pill`, `type`, `title`, `ariaLabel` y evento `on:click`.
2. **`Modal.svelte`**:
   - Backdrop con desenfoque (`backdrop-filter`), cierre con tecla `Escape` y clic fuera (`closeOnBackdrop`, `closeOnEscape`), bloqueo cuando `busy=true`, cabecera opcional con título y botón de cierre, y slots `header`, `default`, `footer`.
3. **`Badge.svelte`**:
   - Variantes semánticas y de ranking: `gray`, `blue`, `green`, `amber`, `hot`, `new`.
   - Formas: `rounded`, `pill`, `circle` y animación opcional `animate`.
4. **`Chip.svelte`**:
   - Píldoras interactivas para filtros, presets de tamaño/formato y tags.
   - Soporte para estado `active`, variantes de color (`default`, `indigo`, `amber`, `green`, `sky`, `hot`), contador opcional (`count`), slot `icon` y botón de eliminación (`removable` + evento `remove`).
5. **`Input.svelte`**:
   - Campo de texto/búsqueda/URL con `bind:value`, `bind:inputElement`, métodos exportados `focus()`/`blur()`, `label` opcional, modo `readonly`, variante `default` o `pill` (barra de búsqueda) y slots `leading` (icono) y `trailing` (botones de acción como copiar o limpiar).
6. **`Alert.svelte`**:
   - Banners de estado (`info`, `success`, `warning`, `error`), paneles de confirmación/migración con `title`, `dismissible` y slot `actions`, más variante flotante `toast`.
7. **`Dropdown.svelte`**:
   - Contenedor flotante con backdrop invisible para cerrar al hacer clic fuera, cabecera opcional con `title`/`subtitle` y animación de apertura.
8. **`Dropzone.svelte`**:
   - Zona de arrastrar y soltar archivos/URLs con gestión de estados `dragover`, `dragleave`, `drop`, ilustración SVG por defecto y slots para icono y acciones.
9. **`Card.svelte`**:
   - Superficie contenedora o tarjeta interactiva (`interactive`), variantes de borde destacado (`new`, `hot`, `indigo`) y tamaños de padding (`sm`, `md`, `lg`).

## Checklist de Ejecución

- [x] 1. Exploración en modo lectura de todos los componentes en `src/lib/components/`
- [x] 2. Definición de tipos compartidos en `src/lib/types/system-ui.ts`
- [x] 3. Implementación de primitivos de acción y entrada (`Button`, `Input`, `Chip`, `Badge`)
- [x] 4. Implementación de contenedores y superposiciones (`Modal`, `Dropdown`, `Dropzone`, `Alert`, `Card`) y barrel `index.ts`
- [x] 5. Verificación (`pnpm run check` — 0 errores/warnings, `pnpm run test` — 7/7 tests en verde)

## Notas de Continuidad / Decisiones

- Los componentes mantienen compatibilidad con la sintaxis de eventos y slots usada en el resto del código (`export let`, `on:click`, `<slot />`) sobre Svelte 5.
- Se preservan los esquemas de color para modo claro y oscuro (`@media (prefers-color-scheme: dark)`).
- Próximo paso potencial (cuando el usuario lo indique): migrar los componentes de dominio (`AddEmoteModal`, `EmoteDetailModal`, `SearchFilterBar`, `StorageSettings`, `ImportButton`, `EmoteCard`, `TagManager`) para consumir `system-ui`.
