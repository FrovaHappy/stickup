# StickUp - Lista de Características y Roadmap

---

## 📌 Funcionalidades Implementadas

### Importación y Procesamiento de Archivos

- [x] **Importación individual**: Carga múltiple de archivos de imagen (`PNG`, `JPG`, `JPEG`, `GIF`, `WebP`).
- [x] **Importación por lote (carpetas)**: Escaneo y carga recursiva de carpetas completas.
- [x] **Migración automática a WebP**: Conversión en lote de imágenes estáticas a `.webp` para optimizar almacenamiento y velocidad.
- [x] **Preservación de GIFs animados**: Detección de cuadros múltiples (`GifDecoder`); se conservan intactos en `.gif` con sus retardos y bucles sin perder fotogramas.

### Gestión de Tags y Metadatos

- [x] **Gestor de etiquetas en chips**: Agregar y eliminar tags de forma unitaria en cada emote.
- [x] **Autocompletado y sugerencias globales**: Desplegable flotante en tiempo real con tags existentes en la base de datos para reutilizarlos con un clic.
- [x] **Búsqueda y filtrado dinámico**: Filtrado en la vista de cuadrícula por nombre y etiquetas.
- [x] **Edición de nombre del emote**: Renombrado editable con guardado directo.
- [x] **Persistencia en SQLite**: Almacenamiento seguro en `emotes.db` sin provocar reinicios de la aplicación durante escrituras.

### Portapapeles Multiplataforma

- [x] **Copiado en Windows (Win32 nativo)**:
  - [x] `CF_HDROP`: Copia el archivo real en el portapapeles para pegarlo directamente en Discord, Telegram, WhatsApp Web, Slack y Explorador de Windows.
  - [x] Soporte especial para GIFs: Exclusión de `CF_DIB` para evitar que aplicaciones de chat congelen la animación en el primer frame.
  - [x] Registro de formatos MIME nativos (`image/gif`, `image/webp`, `PNG`).
  - [x] `CF_DIB`: Bitmap estándar para imágenes estáticas (compatible con Paint, Photoshop, Word).
- [x] **Copiado en Linux**:
  - [x] Soporte para Wayland vía `wl-copy` (`text/uri-list`).
  - [x] Soporte para X11 vía `xclip` (`image/gif`, `image/webp`, `image/png`).
- [x] **Copiado en Android**:
  - [x] Arquitectura desacoplada en Rust lista para gestión por URI / `ClipData`.
- [x] **Fallback en frontend**: Copiado secundario mediante navegador si falla el backend nativo.

### Redimensionamiento y Exportación

- [x] **Presets de tamaño rápidos**: Resoluciones predefinidas (32x32, 64x64, 128x128) y opción de tamaño personalizado.
- [x] **Redimensionamiento de GIFs frame a frame**: Re-escalado Lanczos3 cuadro por cuadro conservando los tiempos de retraso y bucle infinito.
- [x] **Selector de formato de exportación**: Posibilidad de elegir entre PNG, WEBP, GIF o JPG.
- [x] **Cálculo de peso en tiempo real**: Comparativa porcentual del cambio de tamaño antes de exportar o copiar.
- [x] **Descarga directa**: Botón para descargar el archivo procesado a la carpeta del usuario.

### Almacenamiento y Sistema

- [x] **Ruta de almacenamiento configurable**: Selector de carpeta personalizada para guardar la biblioteca de emotes.
- [x] **Eliminación sincronizada**: Al borrar un emote, se elimina de la base de datos y se suprime físicamente del disco.

---

## ⏳ En Progreso / Planificadas (Backlog)

- [ ]
- [ ]

---

## 🗑️ Dropeados / Descartados (Cancelados)
>
> Ideas, formatos o implementaciones evaluadas que se descartaron y el motivo técnico de su descarte.

- [ ]
- [ ]
