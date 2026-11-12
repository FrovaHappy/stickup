# Unreleased

### Added
- Vista expandida (lightbox) interactiva de emotes en el menú de información con zoom ajustable, fondo de transparencia y soporte para teclado.
- Reordenamiento dinámico en la barra de tags para posicionar al inicio de la fila los tags seleccionados.
- Barra horizontal de filtrado múltiple por tags con orden descendente, etiqueta de 'Sin tag', navegación con botones y arrastre con mouse.
- Soporte para espacios en el nombre de base de datos de los emotes, manteniendo nombres de archivo compactos en disco.
- Filtro de formato en la cabecera principal para alternar entre emotes animados (GIF), estáticos o todos.
- Registro de historial de notificaciones persistido en local storage y menú de notificaciones con popover y badge en la cabecera del Home.
- Selector de nivel de importancia de toasts en Configuración para filtrar notificaciones emergentes (Todas, Medias y altas, Solo importantes, Silencioso).

### Changed
- Reemplazado el toast inline en el modal de agregar por una barra de estado integrada con progreso visual y soporte ampliado para directorios de Linux.
- Importación por URL en el modal de agregar siempre visible por encima de la búsqueda local, con soporte de archivos o directorios y adición directa sin solicitar nombre.
- Menú desplegable de sugerencias de tags en el modal de detalle orientado hacia arriba para mejorar la visibilidad.
- Rediseño visual de las notificaciones Toast con estilo flotante pastel, contenedor de icono y tipografía estructurada acorde a la nueva referencia visual.
- Eliminado Alert.svelte y reemplazado completamente por Toast con soporte inline y de slots.
- Migración integral de avisos locales y retroalimentación de copiado hacia el store unificado de toasts en toda la app.

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
