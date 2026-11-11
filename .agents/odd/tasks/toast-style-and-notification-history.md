# Task: Rediseño Visual de Toasts y Registro de Notificaciones en Home

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src/lib/types/system-ui.ts`
  - `src/lib/stores/notifications.ts`
  - `src/lib/stores/toast.ts`
  - `src/lib/components/system-ui/Toast.svelte`
  - `src/lib/components/MainHeader.svelte`
  - `src/routes/+page.svelte`
  - `changelogs/unreleased.md`

## Objetivo y Alcance
1. **Rediseño de Toasts según referencia visual:**
   - Adaptar `Toast.svelte` al diseño de la imagen de referencia: tarjetas blancas curvadas (`border-radius: 16px`) con degradado suave pastel en el lateral izquierdo según la variante (`#e0f2fe` Información, `#dcfce7` Éxito, `#fef3c7` Atención, `#fee2e2` Error).
   - Caja contenedora del icono en la izquierda (tarjeta blanca con bordes suaves y sombra sutil) con icono Lucide del color de la variante (`Info`, `CircleCheck`, `TriangleAlert`, `CircleAlert`).
   - Título destacado en negrita y mensaje descriptivo en tono slate/grisáceo.
   - Botón de cierre `X` en la esquina superior derecha.
   - Posicionamiento elegante en la esquina inferior derecha con apilamiento fluido.
2. **Registro de Notificaciones (Historial):**
   - Crear store `src/lib/stores/notifications.ts` con persistencia en `localStorage`.
   - Registrar automáticamente las notificaciones generadas por el sistema (evitando toasts transitorios de progreso con duración 0 como "Redimensionando...").
   - Métodos para marcar como leídas, borrar individuales y limpiar todo el historial.
3. **Menú de Notificaciones en el Home (`MainHeader.svelte`):**
   - Incorporar botón con icono de campana (`Bell`) en el encabezado del Home.
   - Indicador/badge numérico de notificaciones no leídas con acento naranja StickUp.
   - Menú desplegable (popover) con la lista de notificaciones recientes, tiempo relativo, indicador de no leído, botón para marcar todas como leídas y botón para vaciar historial.

## Checklist de Ejecución
- [x] 1. Tipos de datos para `ToastItem` (soporte de `title`) y `NotificationRecord` en `src/lib/types/system-ui.ts`.
- [x] 2. Crear store `src/lib/stores/notifications.ts` con persistencia en `localStorage`.
- [x] 3. Integrar `toast.ts` con `notifications.ts` para registrar en el historial y soportar títulos personalizados.
- [x] 4. Rediseñar `Toast.svelte` replicando fielmente la estética de la imagen de referencia con iconos Lucide.
- [x] 5. Implementar el menú de notificaciones con popover y badge en `MainHeader.svelte`.
- [x] 6. Limpiar llamadas existentes para evitar prefijos duplicados (como `✓`) en `+page.svelte`.
- [x] 7. Verificación de compilación (`prebuild`) y tipos (`pnpm run prebuild`).
- [x] 8. Actualizar `changelogs/unreleased.md` y marcar tarea como completada.

## Notas de Continuidad / Decisiones
- El diseño de los toasts reproduce exactamente la composición visual de la imagen de referencia: fondo con degradado pastel radial/lineal en la izquierda, recuadro blanco independiente con sombra para el icono de la variante, títulos estructurados ("Éxito", "Error", "Atención", "Información" o personalizados) y cuerpo de texto descriptivo.
- Se eliminó el viejo remanente local de alerta tipo píldora (`<Alert variant="toast">`) en `EmoteDetailModal.svelte` y la variante obsoleta `'toast'` en `AlertVariant` y `Alert.svelte`, unificando el copiado de imagen, nombre, ruta, guardado y descargas bajo el store global de toasts (`toast.success` y `toast.error`).
- Se reforzó la emisión de toasts en `StorageSettings.svelte`, `ImportButton.svelte`, `DeleteEmoteModal.svelte`, `settings/+page.svelte` y `preferences.ts` (al activar/desactivar confirmación al pegar), garantizando retroalimentación inmediata, uniforme y libre de prefijos duplicados (`✓`).
- Se eliminó completamente `Alert.svelte` y el tipo `AlertVariant` (Zero Dead Code), unificando todo componente de aviso bajo `Toast.svelte`, el cual ahora soporta renderizado flotante desde el store y modo inline/standalone con soporte de slots de acciones y cuerpo.
- Se reemplazaron todas las instancias de `<Alert>` en `AddEmoteModal.svelte`, `DeleteEmoteModal.svelte`, `ImportButton.svelte`, `StorageSettings.svelte` y `scanner/+page.svelte` por `<Toast>`.
- Se implementó un sistema de niveles de importancia (`low`, `medium`, `high`) en `ToastItem` y `NotificationRecord`, junto con un filtro configurable (`toastImportanceFilter`: `all`, `medium`, `high`, `none`) accesible desde la sección de Configuración, permitiendo a los usuarios filtrar las tarjetas emergentes flotantes mientras preservan el historial en el registro.
- Las notificaciones transitorias (con `duration === 0`, utilizadas durante tareas en progreso como redimensionamiento de imágenes) se omiten del historial persistente para mantener el registro limpio y enfocado en eventos relevantes concluidos.
- La verificación con `pnpm run prebuild` se ejecutó con éxito completo: 0 errores en `svelte-check`, 73 tests en `vitest`, compilación estática de Vite y 62 tests en `cargo test`.
