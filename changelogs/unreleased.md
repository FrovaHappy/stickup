# Unreleased

### Added
- Soporte para espacios en el nombre de base de datos de los emotes, manteniendo nombres de archivo compactos en disco.
- Filtro de formato en la cabecera principal para alternar entre emotes animados (GIF), estáticos o todos.
- Registro de historial de notificaciones persistido en local storage y menú de notificaciones con popover y badge en la cabecera del Home.
- Selector de nivel de importancia de toasts en Configuración para filtrar notificaciones emergentes (Todas, Medias y altas, Solo importantes, Silencioso).

### Changed
- Menú desplegable de sugerencias de tags en el modal de detalle orientado hacia arriba para mejorar la visibilidad.
- Rediseño visual de las notificaciones Toast con estilo flotante pastel, contenedor de icono y tipografía estructurada acorde a la nueva referencia visual.
- Eliminado Alert.svelte y reemplazado completamente por Toast con soporte inline y de slots.
- Migración integral de avisos locales y retroalimentación de copiado hacia el store unificado de toasts en toda la app.

### Fixed
- Eliminado remanente de aviso viejo tipo píldora en el modal de detalle del emote.
