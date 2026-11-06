# Task: Sistema de Copias de Seguridad de la Base de Datos

- **Estado:** `completed`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src-tauri/src/models.rs`
  - `src-tauri/src/db.rs`
  - `src-tauri/src/commands.rs`
  - `src-tauri/src/lib.rs`
  - `src/lib/types/backup.ts`
  - `src/lib/services/api.ts`
  - `src/lib/components/StorageSettings.svelte`

## Objetivo y Alcance
1. **Creación de Copias de Seguridad (Exportar / Rápido):**
   - Permitir al usuario crear copias de seguridad de la base de datos SQLite activa (`.store.db`).
   - Soportar tanto copia rápida con timestamp en la carpeta `backups/` del directorio de almacenamiento como exportación a una ruta personalizada elegida mediante el diálogo del sistema (`save()` de `@tauri-apps/plugin-dialog`).
   - Uso de `VACUUM INTO` nativo de SQLite para generar copias consistentes, limpias y libres de fragmentación sin bloquear operaciones.
2. **Restauración Segura con Snapshot Preventivo:**
   - Permitir restaurar una copia de seguridad seleccionada por el usuario (`open()` de `@tauri-apps/plugin-dialog`) o desde el historial de copias locales.
   - Validación previa de integridad y estructura (comprobación de que es SQLite válido y contiene la tabla `emotes`).
   - Creación automática de un snapshot preventivo (*safety backup*) antes de sobrescribir la base de datos activa para garantizar cero riesgo de pérdida de datos.
   - Ejecución automática de migraciones posteriores (`ensure_db`) y recarga inmediata del store de emotes en la UI.
3. **Registro de Versión y Backup Automático al Instalar/Actualizar:**
   - Registrar la versión actual de la aplicación (`env!("CARGO_PKG_VERSION")`) en la tabla `settings` de la base de datos.
   - Al ejecutar una nueva versión instalada por encima, detectar la discrepancia de versión antes de iniciar la app o ejecutar migraciones y generar automáticamente una copia de seguridad preventiva (`pre_upgrade_v{old}_to_v{new}_{timestamp}.db`).
   - Registrar la nueva versión en `settings` tras completar el respaldo.
4. **Gestión e Historial de Backups:**
   - Listado de copias de seguridad existentes en la carpeta `backups/`, mostrando fecha, tamaño y cantidad de emotes.
   - Diálogo o panel intuitivo en `StorageSettings.svelte` para gestionar copias de forma visual.
5. **Límite de 15 Copias de Seguridad y Eliminación:**
   - Mantener un límite estricto de hasta 15 copias locales en la subcarpeta `backups/`.
   - Rotación automática (`prune_backups_dir`): al crearse nuevas copias (manuales o automáticas), purgar las más antiguas excedentes de 15.
   - Opción para eliminar copias individuales desde la interfaz con confirmación preventiva.
6. **Cumplimiento ODD y Zero Dead Code:**
   - Comandos Tauri tipados en Rust y frontend (`delete_backup`).
   - Tests unitarios deterministas para límite de rotación y borrado de copias.

## Checklist de Ejecución
- [x] 1. Definición de modelos serializables en Rust (`BackupInfo`, `BackupResult`, `RestoreResult`) y TypeScript (`src/lib/types/backup.ts`)
- [x] 2. Lógica de backup, restauración, listado e inspección en `src-tauri/src/db.rs` con tests unitarios TDD
- [x] 3. Comandos IPC en `src-tauri/src/commands.rs` y registro en `src-tauri/src/lib.rs`
- [x] 4. Métodos tipados en `src/lib/services/api.ts`
- [x] 5. Interfaz de usuario en `src/lib/components/StorageSettings.svelte` con acciones de Crear Copia, Restaurar e Historial
- [x] 6. Registro de versión en `settings` y backup automático al actualizar versión (`check_and_backup_on_version_change`)
- [x] 7. Lógica de rotación a 15 copias (`prune_backups_dir`) y borrado (`delete_db_backup`) en `src-tauri/src/db.rs` con tests unitarios
- [x] 8. Comando IPC `delete_backup` en backend y método en `api.ts`
- [x] 9. Interfaz de eliminación y visualización de cupo (X/15) en `StorageSettings.svelte`
- [x] 10. Verificación determinista (`cargo test --lib`, `cargo check`, `pnpm run check`, `pnpm run test`) y confirmación de Zero Dead Code

## Notas de Continuidad / Decisiones
- Se utilizó `VACUUM INTO` de SQLite para crear backups atómicos, limpios y desfragmentados.
- Se implementó creación automática de snapshot preventivo (`pre_restore_safety_{timestamp}.db`) previo a cualquier restauración de base de datos para máxima seguridad.
- Se implementó `check_and_backup_on_version_change` que se dispara en el inicio del backend (en `setup` de Tauri y antes de cualquier migración de `ensure_db`), detectando cambios de versión y creando una copia `pre_upgrade_v{old}_to_v{new}_{timestamp}.db`.
- Se estableció una constante `MAX_LOCAL_BACKUPS = 15` con purga automática (`prune_backups_dir`) que elimina los respaldos más antiguos al superar el límite tanto en copias rápidas, snapshots preventivos y actualizaciones de versión.
- Se añadió comando `delete_backup` validado y modal de confirmación en la UI con botón `🗑️` por cada ítem.
- Se visualiza el cupo de almacenamiento (`X de 15 copias guardadas`) en el modal de historial.
