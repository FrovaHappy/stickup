# Unreleased

### Added
- Filtro de formato en la cabecera principal para alternar entre emotes animados (GIF), estáticos o todos.
- Registro de historial de notificaciones persistido en local storage y menú de notificaciones con popover y badge en la cabecera del Home.
- Selector de nivel de importancia de toasts en Configuración para filtrar notificaciones emergentes (Todas, Medias y altas, Solo importantes, Silencioso).

### Changed
- Rediseño visual de las notificaciones Toast con estilo flotante pastel, contenedor de icono y tipografía estructurada acorde a la nueva referencia visual.
- Eliminado Alert.svelte y reemplazado completamente por Toast con soporte inline y de slots.
- Migración integral de avisos locales y retroalimentación de copiado hacia el store unificado de toasts en toda la app.

### Fixed
- Eliminado remanente de aviso viejo tipo píldora en el modal de detalle del emote.
