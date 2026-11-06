# Task: Rediseño de UI Principal, Ordenamiento, Tamaño de Grid y Panel /settings

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src/routes/+layout.svelte` (Shell global con Sidebar lateral, navegación Home/Settings, botón `+` para agregar emotes y Toast global)
  - `src/routes/+page.svelte` (Vista principal estilizada y minimalista con MainHeader, EmoteGrid y modales de detalle)
  - `src/routes/settings/+page.svelte` (Nuevo panel `/settings` que centraliza StorageSettings, sincronización de emotes, tamaños de portapapeles y acerca de StickUp)
  - `src/lib/components/MainHeader.svelte` (Header superior con barra de búsqueda tipo píldora, selector de ordenamiento `ArrowDownAZ` y selector de tamaño de grid `Grid2x2`)
  - `src/lib/components/EmoteGrid.svelte` (Grid reactivo a los tamaños pequeño, mediano y grande con estados vacíos claros)
  - `src/lib/components/EmoteCard.svelte` (Tarjeta cuadrada oscura, efecto glow amarillo en hover/foco, vista de nombre en overlay y badges de hot/new)
  - `src/lib/components/system-ui/Logo.svelte` (Componente oficial del logotipo con variantes `color`, `monochrome`, `grayscale`, `badge` y soporte interactivo)
  - `static/logo.svg` (Asset SVG oficial del logotipo)
  - `src/lib/components/AddEmoteModal.svelte` (Soporte para trigger custom desde el sidebar)
  - `src/lib/stores/preferences.ts` (Store persistente en localStorage para tamaño de grid y criterio de ordenamiento)
  - `src/lib/stores/preferences.test.ts` (Tests unitarios de ordenamiento y tamaño de grid)
  - `src/lib/types/preferences.ts` (Definición de tipos `GridSize`, `SortOption`)

## Objetivo y Alcance
1. Rediseñar la UI principal de StickUp basándose en la imagen de referencia:
   - Sidebar lateral izquierdo con navegación (Home activo en amarillo, botón `+` para nuevo emote directamente arriba de Settings, Settings al fondo, avatar/sticker decorativo).
   - Header superior centrado con barra de búsqueda "Busca por nombre o por tags..." en formato pill.
   - Controles en el header derecho: botón/menú de ordenamiento (`ArrowDownAZ`) y selector de tamaño de grid (`Grid2x2`).
   - Grid de emotes con tarjetas cuadradas oscuras y efecto glow amarillo en hover/active/focus.
   - Barra de scroll vertical derecha con thumb amarillo estilizado como en la referencia.
2. Añadir funcionalidad completa de ordenamiento (A-Z, Z-A, fecha reciente/antigua, más usados / hot).
3. Añadir funcionalidad completa de cambio de tamaño de grid (Compacto, Mediano, Grande), persistente en localStorage.
4. Mover todas las configuraciones (`StorageSettings`, gestión de base de datos, backups, etc.) al nuevo panel `/settings`.
5. Incorporar el botón `+` en la barra lateral por encima del icono de configuración para abrir el flujo de carga de nuevos elementos.

## Checklist de Ejecución
- [x] 1. Exploración y definición de contratos / tipos y reglas ODD
- [x] 2. Creación del store de preferencias (`preferences.ts`) con tests unitarios
- [x] 3. Creación del layout global (`+layout.svelte`) con Sidebar de navegación persistente
- [x] 4. Creación de la página `/settings` (`src/routes/settings/+page.svelte`) moviendo `StorageSettings`
- [x] 5. Adaptación del Header con controles de Ordenamiento y Tamaño de Grid
- [x] 6. Actualización de `EmoteGrid.svelte` y `EmoteCard.svelte` con estilos de la referencia (tarjetas, glow, scrollbar amarillo)
- [x] 7. Inclusión del icono `+` para cargar nuevo emote por encima de Settings en el Sidebar
- [x] 8. Integración del logotipo oficial SVG (`Logo.svelte`) y sus variantes en el Sidebar y `/settings`
- [x] 9. Ensamblado de `src/routes/+page.svelte` limpia y modular
- [x] 10. Verificación de compilación de Rust (`cargo check`) y cumplimiento de Zero Dead Code

## Notas de Continuidad / Decisiones
- El gestor de paquetes exclusivo es **pnpm**.
- Se preservan todos los atajos de teclado globales (Ctrl+C para copiar emote sobre el que está el cursor, Ctrl+V para pegar y cargar nuevo emote).
- El scrollbar amarillo se implementa a nivel global mediante reglas CSS nativas (`::-webkit-scrollbar-thumb`).
- El header superior pegajoso (`MainHeader.svelte`) implementa fondo translúcido con desenfoque (`backdrop-filter: blur(16px)`) y borde sutil inferior para aislar visualmente las tarjetas al hacer scroll.
