# Task: Tamaños Personalizados, Menú Contextual y Toasts en System-UI

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src/lib/types/system-ui.ts`
  - `src/lib/types/emote.ts`
  - `src/lib/stores/sizes.ts`
  - `src/lib/stores/toast.ts`
  - `src/lib/utils/format.ts`
  - `src/lib/components/system-ui/Toast.svelte`
  - `src/lib/components/system-ui/ContextMenu.svelte`
  - `src/lib/components/system-ui/index.ts`
  - `src/lib/components/EmoteCard.svelte`
  - `src/lib/components/EmoteGrid.svelte`
  - `src/lib/components/AddEmoteModal.svelte`
  - `src/lib/components/detail/EmoteDetailModal.svelte`
  - `src/routes/+page.svelte`

## Objetivo y Alcance
1. **Tamaños personalizados:** Permitir al usuario definir y guardar dimensiones personalizadas (anchura x altura) tanto en la selección de tamaños como al resolver dimensiones para copiar o descargar.
2. **Menú contextual personalizado:** Agregar un menú contextual (`ContextMenu` en `system-ui`) al hacer click derecho en un emote con atajos rápidos: copiar original (Ctrl+C), copiar en tamaños estándar/personalizados, copiar nombre, copiar ruta de archivo, abrir detalle y eliminar.
3. **Pegado con Ctrl+V sin modal y Toasts:** Al presionar `Ctrl+V` (o evento de paste global), procesar la importación directamente en segundo plano sin abrir el modal de `AddEmoteModal`, informando el resultado mediante notificaciones toast flotantes usando un nuevo componente `Toast` en `system-ui`.

## Checklist de Ejecución
- [x] 1. Tipos de datos para Toast y ContextMenu en `src/lib/types/system-ui.ts`
- [x] 2. Stores de Toast (`src/lib/stores/toast.ts`) y de Tamaños Personalizados (`src/lib/stores/sizes.ts`)
- [x] 3. Lógica de tamaños en `src/lib/utils/format.ts` y tests unitarios en `src/lib/utils/format.test.ts`
- [x] 4. Componente `Toast.svelte` y `ContextMenu.svelte` en `src/lib/components/system-ui/` y exportación en `index.ts`
- [x] 5. Integración de menú contextual en `EmoteCard.svelte`, `EmoteGrid.svelte` y `+page.svelte`
- [x] 6. Integración de tamaños personalizados en `EmoteDetailModal.svelte`
- [x] 7. Ajuste en `AddEmoteModal.svelte` para importar con Ctrl+V sin abrir modal y usar `toast`
- [x] 8. Verificación de compilación de Rust y consistencia de tipos

## Notas de Continuidad / Decisiones
- Se creó `Toast.svelte` y `ContextMenu.svelte` dentro de `src/lib/components/system-ui/` cumpliendo el principio Zero Dead Code (ambos exportados en `index.ts` y consumidos en la UI principal).
- El pegado con `Ctrl+V` ejecuta la importación de manera silenciosa en segundo plano y notifica el resultado mediante toasts sin desplegar el modal.
- Los tamaños personalizados se persisten automáticamente en `localStorage` (`stickup_custom_sizes`) y están disponibles tanto en el modal de detalle del emote como en los atajos del menú contextual.
