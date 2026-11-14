# Unreleased

### Added
- Botón y comando nativo `open_storage_directory` para abrir la carpeta de recursos directamente en el explorador del sistema.

### Changed
- Almacenamiento centralizado y fijo en `%APPDATA%\stickup\resources` (o equivalente del sistema) con migración automática de datos heredados.

### Fixed
- Error de permisos `Acceso denegado. (os error 5)` al guardar rutas de datos en carpetas protegidas del sistema.

### Removed
- Selector manual de ruta de almacenamiento y modales de conflicto de migración de base de datos.
