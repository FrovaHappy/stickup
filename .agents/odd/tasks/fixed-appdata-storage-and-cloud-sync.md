# Task: Almacenamiento Fijo en %APPDATA%\stickup\resources, Apertura Rápida de Carpeta y Sistema de Sincronización en la Nube

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src-tauri/src/db.rs`
  - `src-tauri/src/commands.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/services/sync/manifest.rs`
  - `src-tauri/src/services/sync/provider.rs`
  - `src-tauri/src/services/sync/index.rs`
  - `src-tauri/src/services/sync/mod.rs`
  - `src/lib/types/sync.ts`
  - `src/lib/services/api.ts`
  - `src/lib/components/CloudSyncSettings.svelte`
  - `src/lib/components/StorageSettings.svelte`
  - `changelogs/unreleased.md`

---

## 1. Objetivo y Alcance

### 1.1 Fijar el Directorio de Almacenamiento en `%APPDATA%\stickup\resources`
- **Eliminar por completo la selección de carpeta:** Quitar selector manual de ruta en backend y frontend para evitar problemas de permisos de sistema (`os error 5 / Acceso denegado`).
- **Ruta fija predeterminada y garantizada:**
  - Windows: `%APPDATA%\stickup\resources` (`C:\Users\<Usuario>\AppData\Roaming\stickup\resources`).
  - Linux: `~/.config/stickup/resources` (o `$XDG_DATA_HOME/stickup/resources`).
  - macOS: `~/Library/Application Support/stickup/resources`.
- **Migración transparente automática:**
  - Migración inteligente desde `Documents/emotes`, `.stickup_db_dir` o carpetas de sincronización previas con inspección del conteo de emotes.
- **Zero Dead Code:**
  - Eliminados `set_storage_path`, `set_active_db_dir`, modales de confirmación de conflicto de BD y selectores obsoletos.

### 1.2 Menú / Acción de Acceso Directo al Directorio ("Abrir en el Explorador")
- Comando nativo en Rust (`open_storage_directory`) que invoca el explorador del sistema (`explorer.exe` / `xdg-open` / `open`) directamente en `%APPDATA%\stickup\resources`.
- Botón visible e intuitivo con icono Lucide `<FolderOpen /> Abrir carpeta`.

### 1.3 Sistema de Sincronización en la Nube (Cloud Sync)
- Arquitectura offline-first con content-addressing (SHA-256) para respaldar y sincronizar emotes e imágenes entre múltiples computadoras.
- Soporte para **MEGA**, **Google Drive**, **OneDrive** y **Carpetas personalizadas de nube / red**.
- Detección automática en Windows de carpetas de sincronización existentes (ej. `Documents\MEGA\MEGAsync`, `OneDrive`, `Google Drive`).

---

## 2. Checklist de Ejecución (ODD)

### Fase A: Almacenamiento Fijo en AppData (Backend & Frontend)
- [x] 1. Refactorizar `src-tauri/src/db.rs` con `get_app_resources_dir()` fijo en `%APPDATA%\stickup\resources`.
- [x] 2. Implementar migración automática con conteo de emotes (`migrate_legacy_locations_to_resources`).
- [x] 3. Implementar comando `open_storage_directory` en `src-tauri/src/commands.rs` y registrar en `src-tauri/src/lib.rs`.
- [x] 4. Actualizar `src/lib/services/api.ts` (quitar `setStoragePath`, añadir `openStorageDirectory`).
- [x] 5. Rediseñar `src/lib/components/StorageSettings.svelte` con botón `Abrir carpeta`.
- [x] 6. Verificar compilación y tests (`pnpm run prebuild`).

---

### Fase B: Sistema de Sincronización en la Nube (Cloud Sync)
- [x] 1. Añadir campos `updated_at` (INTEGER), `is_deleted` (BOOLEAN DEFAULT 0) y `hash` (TEXT) a la tabla `emotes` en `src-tauri/src/db.rs`.
- [x] 2. Actualizar operaciones de emotes en `src-tauri/src/commands.rs` para soft-deletes y actualización de timestamps UTC.
- [x] 3. Crear módulo `src-tauri/src/services/sync/`:
  - `manifest.rs`: Construcción de `SyncManifest`, hash SHA-256 de imágenes y cálculo de deltas con resolución Last-Write-Wins (LWW) y propagación de soft-deletes.
  - `provider.rs`: Trait `CloudStorageProvider`, enum `AnyCloudProvider`, implementación `MemoryStorageProvider` (para tests) y `FolderStorageProvider` (para carpetas sincronizadas de nube).
  - `index.rs`: Motor de sincronización `execute_sync`, `execute_sync_with_db_opener`, persistencia de configuración `SyncConfig` y detección de rutas de nube con `detect_cloud_candidates()`.
  - `mod.rs`: Exportaciones públicas limpias.
- [x] 4. Comandos IPC en Tauri (`src-tauri/src/commands.rs` y `src-tauri/src/lib.rs`):
  - `get_cloud_sync_config`, `save_cloud_sync_config`
  - `get_cloud_sync_status`
  - `detect_cloud_folders`
  - `start_cloud_sync`
- [x] 5. Frontend:
  - `src/lib/types/sync.ts`: Tipos tipados `SyncConfig` y `SyncStatus`.
  - `src/lib/services/api.ts`: Métodos tipados para invocar comandos de Tauri.
  - `src/lib/components/CloudSyncSettings.svelte`: Panel completo de sincronización con selector de proveedores (MEGA, Google Drive, OneDrive, Carpeta), detección automática de carpetas, botón "Sincronizar ahora" con spinner, toggle de sincronización automática y métricas de última sincronización.
  - Integrado dentro de `StorageSettings.svelte`.
- [x] 6. Verificación exhaustiva:
  - `svelte-check`: 0 errores, 0 advertencias.
  - `vitest`: 101/101 tests unitarios superados.
  - `vite build`: bundle SSR y cliente compilado correctamente.
  - `cargo test`: 96/96 tests unitarios superados.
