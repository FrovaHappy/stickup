# Unreleased

### Added
- Sistema de gestión de versiones con comprobación automática (intervalo de 24 horas) y manual contra GitHub Releases.
- Detección de "versión caliente / reciente" (< 24 horas de publicación) con advertencia suave para el usuario.
- Doble modalidad de actualización: descarga directa en la app (con ejecución automática) o apertura asistida en navegador web.
- Comandos Tauri `open_external_url`, `download_update_asset` y `launch_installer` con saneamiento de rutas.
- Componente `UpdateModal` para previsualizar changelog oficial y gestionar la actualización.
- Indicador visual reactivo en el icono de Ajustes de la barra lateral al detectar una versión superior.

### Changed
- Conexión de la versión en Ajustes ("Acerca de StickUp") a la versión real de `Cargo.toml` (`get_app_version`).

### Fixed
- 
