# Unreleased

### Added
- **Modal flotante de confirmación al pegar (`PasteConfirmPopup`)**: Interfaz emergente para previsualizar, asignar tags y confirmar la adición de nuevos emotes rápidamente al detectar imágenes o URLs en el portapapeles.
- **Preferencia de resolución favorita y auto-minimizado al copiar**: Opciones configurables en Ajustes para copiar emotes directamente en la resolución predefinida y minimizar automáticamente la aplicación a la bandeja del sistema tras la copia.
- **Notificación y manejo específico para GIFs**: Mensaje informativo aclarando que los archivos GIF animados se copian en su resolución original para preservar la animación.

### Changed
- **Flujo y modal de eliminación de emotes (`DeleteEmoteModal`)**: Rediseño del diálogo de confirmación para eliminar emotes siguiendo las directrices de diseño y colores del sistema (ODD / Design System).
- **Documentación y capturas de pantalla**: Actualización del archivo `README.md` con nuevas capturas de pantalla del flujo de trabajo (galería, detalle, scanner e interfaz de configuración).
- **Reglas del agente y contexto (`AGENTS.md`)**: Limpieza de referencias huérfanas y optimización del contexto arquitectónico.

### Fixed
- **Optimización en portapapeles para GIFs**: Corrección en el servicio de clipboard para evitar reescalados erróneos que rompían la animación de archivos GIF.

