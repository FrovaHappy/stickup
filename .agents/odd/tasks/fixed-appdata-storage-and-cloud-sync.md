# Task: Almacenamiento Fijo en %APPDATA%\stickup\resources, Apertura Rápida de Carpeta y Sistema de Sincronización en la Nube

- **Estado:** `in-progress` (Fase A completada, Fase B pendiente)
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave previstos:**
  - `src-tauri/src/db.rs`
  - `src-tauri/src/commands.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/services/sync/` (nuevo módulo de sincronización para Fase B)
  - `src/lib/services/api.ts`
  - `src/lib/components/StorageSettings.svelte`
  - `src/lib/components/CloudSyncSettings.svelte` (para Fase B)
  - `src/lib/types/sync.ts` (para Fase B)

---

## 1. Objetivo y Alcance

### 1.1 Fijar el Directorio de Almacenamiento en `%APPDATA%\stickup\resources`
- **Eliminar por completo la selección de carpeta:** Quitar cualquier selector manual de ruta en el backend y frontend para evitar problemas de permisos de sistema (`os error 5 / Acceso denegado`).
- **Ruta fija predeterminada y garantizada:**
  - Windows: `%APPDATA%\stickup\resources` (`C:\Users\<Usuario>\AppData\Roaming\stickup\resources`).
  - Linux: `~/.config/stickup/resources` (o `$XDG_DATA_HOME/stickup/resources`).
  - macOS: `~/Library/Application Support/stickup/resources`.
- **Migración transparente automática:**
  - Al iniciar, si existen emotes o `.store.db` en las ubicaciones heredadas (`Documentos/emotes` o el directorio local del proyecto), migrarlos automáticamente a `%APPDATA%\stickup\resources` sin fricción para el usuario.
- **Zero Dead Code:**
  - Eliminar `set_storage_path`, `set_active_db_dir`, modales de confirmación de conflicto de bases de datos y selectores de carpetas que queden en desuso.

### 1.2 Menú / Acción de Acceso Directo al Directorio ("Abrir en el Explorador")
- Añadir un comando nativo en Rust (`open_storage_directory`) que abra el explorador de archivos nativo del SO (`explorer.exe` en Windows, `xdg-open` en Linux, `open` en macOS) posicionado directamente en la carpeta `%APPDATA%\stickup\resources`.
- Añadir botón visible e intuitivo con icono Lucide (`FolderOpen` / `ExternalLink`) en la interfaz de Ajustes / Barra lateral.

### 1.3 Sistema de Sincronización en la Nube (Cloud Sync)
- Diseñar una arquitectura offline-first para respaldar y sincronizar tanto los metadatos de los emotes como los archivos binarios de imágenes entre múltiples dispositivos.

---

## 2. Checklist de Ejecución (Fases ODD)

### Fase A: Almacenamiento Fijo en AppData (Backend & Frontend)
- [x] 1. Refactorizar `src-tauri/src/db.rs`:
  - Definir la ruta fija del almacenamiento usando `%APPDATA%\stickup\resources` (o resolver mediante el directorio de datos de la app).
  - Implementar migración automática de datos existentes desde `Documents/emotes` o `.stickup_db_dir` hacia la nueva ruta de `%APPDATA%`.
  - Eliminar `.stickup_db_dir` y la dependencia de `std::env::current_dir()`.
- [x] 2. Eliminar comando `set_storage_path` y funciones muertas de migración manual en `src-tauri/src/commands.rs`.
- [x] 3. Implementar comando `open_storage_directory` en `src-tauri/src/commands.rs` y registrarlo en `src-tauri/src/lib.rs`.
- [x] 4. Actualizar `src/lib/services/api.ts` (eliminar `setStoragePath`, añadir `openStorageDirectory`).
- [x] 5. Rediseñar `src/lib/components/StorageSettings.svelte`:
  - Quitar selector de rutas, modales de migración y alertas de conflicto.
  - Mostrar la ruta fija `%APPDATA%\stickup\resources` con botón destacado: **"Abrir en el Explorador"** (`<FolderOpen />`).
- [x] 6. Verificar compilación y tests (`pnpm run prebuild`).

---

### Fase B: Arquitectura del Sistema de Sincronización en la Nube (Plan Detallado)

#### B.1 Principio Rector: Offline-First con Content-Addressing (SHA-256)
1. **Separación de Datos:**
   - **Metadatos (SQLite):** Nombres, tags, favoritos, contadores de uso, orden y soft-deletes (`deleted_at`).
   - **Archivos Binarios (Imágenes):** Se nombran o identifican por su hash SHA-256 (`{hash}.png`). Son inmutables. Si una imagen no cambia, nunca se vuelve a subir o descargar (deduplicación automática).

#### B.2 Proveedores en la Nube Priorizados
El usuario ha seleccionado enfocarse en nubes de uso personal masivo: **Google Drive**, **OneDrive** y **MEGA**.

1. **Google Drive (Google Drive API v3):**
   - **Flujo de Auth:** OAuth 2.0 con PKCE (flujo nativo para aplicaciones de escritorio sin secretos expuestos). Se abre el navegador del usuario y se captura el callback en un servidor loopback local (`http://localhost:PORT/callback`).
   - **Ubicación en la nube:**
     - *Opción A (Recomendada):* Carpeta dedicada visible `Google Drive/StickUp/` (permite al usuario ver sus archivos si lo desea).
     - *Opción B:* Carpeta oculta de aplicación (`drive.appdata`), aislada del resto del Drive del usuario.
   - **Ventajas:** 15 GB gratuitos, sincronización veloz y disponibilidad ubicua.

2. **Microsoft OneDrive (Microsoft Graph API):**
   - **Flujo de Auth:** OAuth 2.0 PKCE mediante Microsoft Identity Platform.
   - **Ubicación en la nube:** Carpeta dedicada de app (`/drive/special/approot` o carpeta `OneDrive/StickUp/`).
   - **Ventajas:** 5 GB gratuitos, integración nativa de Windows y excelente API REST de archivos y deltas.

3. **MEGA (Mega.nz):**
   - **Flujo de Auth:** MEGA utiliza autenticación cifrada en cliente con derivación de claves (Email + Password -> SRP / clave maestra). Se almacena el Session ID / Auth Token cifrado localmente en el almacén de credenciales del sistema.
   - **Ubicación en la nube:** Carpeta raíz dedicada `Root/StickUp/` en el árbol de nodos de MEGA.
   - **Ventajas:** 20 GB gratuitos por cuenta, **cifrado de extremo a extremo (E2EE)** nativo de MEGA de punto a punto, y soporte para almacenamiento y descarga eficiente de fragmentos binarios.

#### B.3 Flujo de Sincronización Paso a Paso
```text
[Local DB & Resources] ──(1. Comparar Hashes)──> [Manifest Local vs Remoto]
                                                           │
        ┌──────────────────────────────────────────────────┴──────────────────────────────────────────────────┐
        ▼                                                                                                     ▼
 [2. Subir Imágenes Nuevas]                                                                           [3. Descargar Imágenes Nuevas]
 (Solo las que faltan en la nube)                                                                     (Solo las que no existen localmente)
        │                                                                                                     │
        └──────────────────────────────────────────────────┬──────────────────────────────────────────────────┘
                                                           ▼
                                                [4. Sincronizar Base de Datos]
                                       (CRDT o Registro de Cambios / Last-Write-Wins con updated_at)
                                                           │
                                                           ▼
                                                [5. Recargar UI de StickUp]
```

1. **Manifest y Estado (`sync_manifest.json`):**
   - El cliente genera un manifiesto con la lista de emotes locales: `{ id, hash, name, tags, updated_at, is_deleted }`.
   - Se compara con el manifiesto remoto en la nube para calcular el delta (qué hay que subir y qué hay que descargar).
2. **Transferencia Eficiente de Binarios:**
   - Se transfieren únicamente los hashes de imágenes faltantes en paquetes concurrentes con reintentos.
3. **Resolución de Conflictos en Base de Datos:**
   - Cada emote tendrá columnas `updated_at` (timestamp UTC) e `is_deleted` (booleano para borrado lógico).
   - Regla **Last-Write-Wins (LWW):** Si el mismo emote se editó en dos dispositivos, el cambio con fecha más reciente prevalece.
   - Si un emote fue eliminado en un dispositivo (`is_deleted = true`), se propaga la eliminación al resto de equipos sin borrar de golpe antes de confirmar la sincronización.
4. **Seguridad y Cifrado Opcional (E2EE):**
   - Opción para cifrar el archivo de metadatos y las imágenes con AES-256-GCM usando una contraseña maestra definida por el usuario antes de subirlos a la nube.

#### B.4 Modos de Disparo de la Sincronización
- **Manual:** Botón **"Sincronizar ahora"** en la interfaz.
- **Al Iniciar / Cerrar la aplicación:** Sincronización automática de verificación rápida al arrancar StickUp.
- **En Segundo Plano (Debounced):** 30 segundos después de añadir o modificar emotes localmente.

---

## 3. Checklist de Implementación del Cloud Sync
- [ ] 1. Añadir campos `updated_at` (INTEGER) y `is_deleted` (BOOLEAN DEFAULT 0) a la tabla `emotes` en `src-tauri/src/db.rs`.
- [ ] 2. Crear módulo `src-tauri/src/services/sync/`:
  - `manifest.rs`: Generación, lectura y cálculo de diferencias de manifiestos con hashes SHA-256.
  - `provider.rs`: Trait unificado `CloudStorageProvider` (`login`, `logout`, `upload_file`, `download_file`, `list_files`, `get_manifest`).
  - `gdrive.rs`: Implementación para Google Drive (OAuth2 PKCE + REST API v3).
  - `onedrive.rs`: Implementación para OneDrive (Microsoft Graph API).
  - `mega.rs`: Implementación para MEGA (API E2EE con sesión de usuario).
- [ ] 3. Comandos IPC en Tauri: `start_sync`, `get_sync_status`, `configure_sync_provider`.
- [ ] 4. Componente de configuración y estado en frontend: `src/lib/components/CloudSyncSettings.svelte` con botón de sincronización y estado del último sync.
- [ ] 5. Tests unitarios deterministas para comparación de manifiestos y resolución LWW.

