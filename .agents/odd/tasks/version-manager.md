# Task: Sistema de Gestión de Versiones y Comprobación de Actualizaciones con GitHub

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src/lib/types/updater.ts` (contratos TypeScript para releases de GitHub, assets, resultado de verificación y modalidades de actualización)
  - `src/lib/services/updater.ts` (lógica de consulta a GitHub Releases API, comparación SemVer, filtrado de pre-releases, resolución de assets por plataforma)
  - `src/lib/services/updater.test.ts` (tests unitarios TDD para parseo SemVer, orden de versiones, pre-releases y asignación de assets por SO)
  - `src/lib/stores/updater.ts` (store reactivo con estado de comprobación, última versión disponible, caché de periodicidad y preferencias de usuario)
  - `src/lib/components/UpdateModal.svelte` (modal interactivo que presenta la nueva versión, changelog oficial y opciones de actualización)
  - `src-tauri/src/commands.rs` y `src-tauri/src/lib.rs` (comandos auxiliares: apertura de URLs externas seguras y descarga directa del instalador a disco con notificación)
  - `src/lib/services/api.ts` (declaración de llamadas IPC correspondientes)
  - `src/routes/settings/+page.svelte` (reemplazo de versión hardcodeada por versión dinámica del backend, controles de actualización manual y toggles de configuración)
  - `src/routes/+layout.svelte` (verificación periódica no intrusiva al iniciar y montaje del modal de actualización)

## Objetivo y Alcance

1. **Detección Dinámica de Versión y Consulta a GitHub**:
   - Obtener la versión instalada real a través de `api.getAppVersion()` (vinculada a `Cargo.toml`).
   - Consultar el endpoint público de GitHub Releases (`https://api.github.com/repos/FrovaHappy/stickup/releases`) respetando rate limits mediante caché (comprobación automática al inicio espaciada estrictamente cada **24 horas**).
   - Implementar un comparador SemVer robusto que contemple tags estándar (`vX.Y.Z`) y opcionalmente versiones de desarrollo (`vX.Y.Z-dev.W`).
   - **Indicador de Versión Caliente / Recién Lanzada (< 24h)**:
     - Evaluar la fecha de publicación (`published_at`) de la nueva versión.
     - Si tiene **menos de 24 horas** de haber sido publicada, etiquetarla visualmente como **"Versión Caliente / Reciente"** con un aviso orientativo (*"Lanzada hace pocas horas. Puedes actualizar ahora o esperar un poco por estabilidad"*).

2. **Doble Modalidad de Actualización a Elección del Usuario**:
   - **Modalidad 1 (Descarga en la app)**: Descarga el archivo instalador (.exe NSIS para Windows, .deb / .AppImage para Linux) directamente a la carpeta de descargas del usuario mediante backend, notificando el avance y ofreciendo ejecutarlo o abrir su ubicación.
   - **Modalidad 2 (Asistida / Navegador)**: Abre la URL directa de descarga del instalador o la página del release en GitHub mediante el navegador web predeterminado.
   - Ambas modalidades estarán disponibles en el diálogo de actualización y configurables en las preferencias.

3. **Integración Visual en Ajustes y Notificación**:
   - Eliminar la versión hardcodeada (`v0.1.1`) en la sección *Acerca de StickUp* y mostrar la versión real con badge dinámico.
   - Añadir tarjeta de estado de actualizaciones:
     - Indicador de estado: "Tu versión está al día" vs "Nueva versión vX.Y.Z disponible".
     - Botón manual "Buscar actualizaciones" con feedback visual de carga.
     - Toggle: "Buscar actualizaciones automáticamente al iniciar".
     - Toggle: "Incluir versiones preliminares / pre-release (dev)".
   - Modal estilizado con el Design System de StickUp (naranja primario, superficies oscuras, tipografía limpia) mostrando el Changelog de GitHub y las acciones de actualización.

## Checklist de Ejecución

- [x] 1. **Definición de Contratos y Modelos de Datos**:
  - Crear `src/lib/types/updater.ts` con interfaces para `GitHubRelease`, `ReleaseAsset`, `UpdateCheckResult`, `UpdatePreferences`.
- [x] 2. **Servicio y Comparador SemVer con TDD (Vitest)**:
  - Crear `src/lib/services/updater.test.ts` con casos de prueba para:
    - Versiones mayores, menores, parches y pre-releases (`0.2.0` vs `0.2.1`, `0.2.0` vs `0.2.0-dev.1`, etc.).
    - Detección de versión "caliente" (< 24 horas de publicación) y cálculo de horas transcurridas.
    - Detección de asset correcto según plataforma (Windows vs Linux).
  - Implementar `src/lib/services/updater.ts` resolviendo todos los tests en verde (`GREEN`).
- [x] 3. **Comandos de Backend (Rust) para Descarga y Apertura**:
  - Implementar comando `download_update_asset(url, filename)` en `src-tauri/src/commands.rs` para descargar el instalador a `Downloads` de forma controlada.
  - Implementar o verificar comando de apertura de URL externa en el navegador nativo.
  - Registrar comandos en `src-tauri/src/lib.rs` y exponer en `src/lib/services/api.ts`.
- [x] 4. **Store Reactivo de Actualizaciones**:
  - Implementar `src/lib/stores/updater.ts` con persistencia en `localStorage` para preferencias y timestamp de último chequeo (intervalo de 24 horas).
  - Implementar función `checkForUpdates(manual: boolean)` con manejo de errores y estados reactivos (`isChecking`, `hasUpdate`, `availableUpdate`, `isHotRelease`).
- [x] 5. **Componente de Notificación y Modal de Actualización**:
  - Crear `src/lib/components/UpdateModal.svelte` con:
    - Badge o alerta de "Versión Caliente" (< 24h) orientando al usuario si desea esperar por estabilidad.
    - Opciones de actualización: "Descargar en la app", "Abrir en navegador / Ver en GitHub" y "Recordar más tarde".
  - Renderizar notas de la versión (changelog oficial) con formato legible.
- [x] 6. **Actualización de la Vista de Ajustes**:
  - Modificar `src/routes/settings/+page.svelte` para cargar la versión real de `api.getAppVersion()`.
  - Integrar la sección de "Actualizaciones de StickUp" con controles interactivos y estados.
- [x] 7. **Integración en Layout Principal**:
  - Invocar `checkForUpdates(false)` al arrancar en `src/routes/+layout.svelte` condicionado a la preferencia de usuario y el intervalo de caché.
  - Integrar `UpdateModal` a nivel de layout para presentar la propuesta de actualización sin importar la pantalla activa.
- [x] 8. **Fase de Verificación y Documentación**:
  - Ejecutar `pnpm run prebuild` (svelte-check, vitest, build y cargo test).
  - Actualizar `changelogs/unreleased.md` bajo la categoría `Added` y `Changed` siguiendo `release-notes`.
  - Marcar la tarea como completada en `.agents/odd/tasks/version-manager.md`.

## Notas de Continuidad / Decisiones

- **Sin dependencias pesadas de terceros**: Se usa la API pública de GitHub Releases sin requerir tokens ni crates externos invasivos.
- **Respeto a Rate Limits & Margen de 24 horas**: Se impone un umbral estricto de **24 horas** entre comprobaciones automáticas al iniciar la aplicación (`lastUpdateCheckTimestamp`); las comprobaciones manuales desde Ajustes se ejecutan siempre bajo demanda.
- **Criterio de Versión "Caliente"**: Si `Date.now() - Date.parse(published_at) < 24 * 3600 * 1000`, la versión se marca como caliente (`isHot: true`), mostrando una alerta informativa suave que indica el tiempo transcurrido (ej: *"Publicada hace 3 horas"*) y le permite al usuario decidir libremente si actualizar de inmediato o esperar.
- **Respeto al Design System**: Los botones primarios usan `--color-orange-primary`, las tarjetas mantienen transparencia con bordes sutiles y se usa iconografía exclusiva de `@lucide/svelte`.

