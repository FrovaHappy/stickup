# Unreleased

### Added

-

### Changed

-

### Fixed

- Retroalimentación visual prominente y corrección de eventos de clic al seleccionar tags en la barra horizontal.
- Eliminado remanente de aviso viejo tipo píldora en el modal de detalle del emote.

### Security

- Reemplazada la ejecución vía `cmd /c start` por `ShellExecuteW` nativo y eliminados los subprocesos externos de `curl` en favor de cliente HTTP `reqwest` con límites de tamaño y protección anti-SSRF.
- Activada Content Security Policy (CSP) estricta y restringido el scope de `assetProtocol` con autorización dinámica segura de la carpeta de emotes activa.
- Añadida verificación SHA-256 obligatoria, lista blanca de origen en GitHub y publicación automática de `SHA256SUMS.txt` para descargas e instaladores de actualización.
- Blindado el scanner HTML/DOM frente a esquemas peligrosos (`javascript:`, `vbscript:`, SVG inline) y el lector nativo de portapapeles `CF_DIB` frente a desbordamientos de dimensiones.
- Restringidas las operaciones de eliminación, restauración y creación de copias de seguridad y archivos de emotes a sus directorios y extensiones autorizadas.
- Añadida validación de origen HTTPS y checksum SHA-256 en la descarga de modelos IA desde Hugging Face.
