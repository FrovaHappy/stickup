# StickUp

**StickUp** es un gestor de emotes y stickers de escritorio rápido, ligero y multiplataforma, construido con **Tauri 2 (Rust)**, **SvelteKit 5 (TypeScript)** y **SQLite**. Permite importar, organizar, optimizar, redimensionar y copiar emotes (estáticos y animados) directamente al portapapeles nativo para pegarlos al instante en Discord, Telegram, WhatsApp, Slack y cualquier aplicación.

![Tauri 2](https://img.shields.io/badge/Tauri-2.0-24C8DB?logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)
![Svelte 5](https://img.shields.io/badge/SvelteKit-5.0-FF3E00?logo=svelte&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5.6-3178C6?logo=typescript&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-Bundled-003B57?logo=sqlite&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green)

---

## ✨ Características Principales

### 📥 Importación Versátil y Optimización Automática

- **Múltiples fuentes de entrada**:
  - Selección individual o múltiple de archivos (`PNG`, `JPG`, `JPEG`, `GIF`, `WebP`).
  - Importación por lotes escaneando carpetas completas.
  - **Drag & Drop** directo sobre la aplicación (archivos locales o enlaces web).
  - **Pegado rápido desde el portapapeles (`Ctrl+V`)** en cualquier momento.
  - **Descarga desde URL**: Limpia automáticamente parámetros de tamaño por defecto (como `?size=32` en CDNs de Discord) para obtener siempre la resolución original máxima.
- **Conversión inteligente de formatos**:
  - **Imágenes estáticas a WebP**: Convierte automáticamente `PNG` y `JPG/JPEG` a `.webp` para reducir el uso de disco y acelerar la carga.
  - **Preservación de GIFs animados**: Detecta animaciones multifotograma (`GifDecoder`) y las conserva intactas con sus tiempos y bucles originales.
  - **WebP animado a GIF**: Detecta archivos `.webp` animados y los convierte automáticamente a `.gif` conservando todos sus fotogramas y retardos (`Delay`).

### 📋 Portapapeles Nativo Multiplataforma

- **Windows (Win32 API nativo)**:
  - Inyecta `CF_HDROP` con el archivo procesado para pegado directo en **Discord, Telegram, WhatsApp Web, Slack y el Explorador de archivos**.
  - Registra formatos MIME nativos (`image/gif`, `image/webp`, `PNG`) y `CF_DIB` para editores gráficos.
  - Omite `CF_DIB` en GIFs animados para evitar que las aplicaciones de chat congelen la animación en el primer fotograma.
- **Linux**:
  - Soporte nativo para **Wayland** mediante `wl-copy` (`text/uri-list`) y **X11** mediante `xclip` (`image/gif`, `image/webp`, `image/png`).
- **Android**:
  - Arquitectura modular desacoplada preparada para integración mediante URI / `ClipData`.

### 📐 Redimensionamiento y Exportación en Tiempo Real

- **Presets de resolución rápidos**: `Original`, `48×48`, `64×64`, `128×128`, `256×256` y `512×512`.
- **Re-escalado de GIFs fotograma a fotograma**: Redimensiona animaciones completas con filtro `Lanczos3` manteniendo la transparencia, el retardo de cada cuadro y el bucle infinito.
- **Conversión al vuelo**: Copia o exporta emotes estáticos en `WebP`, `PNG` o `JPG` y emotes animados en `GIF`.
- **Comparativa en vivo**: Calcula el peso estimado del archivo resultante y muestra el porcentaje de ahorro o variación de resolución antes de copiar o descargar.
- **Descarga directa**: Permite guardar en disco una copia procesada con las dimensiones y el formato elegidos.

### 🏷️ Organización, Búsqueda y Estadísticas de Uso

- **Gestor de etiquetas en chips**: Añade o elimina tags de cada emote con guardado instantáneo en SQLite.
- **Autocompletado global**: Sugerencias en tiempo real con todas las etiquetas existentes en tu biblioteca para reutilizarlas con un clic.
- **Búsqueda y filtrado dinámico**: Filtra instantáneamente la cuadrícula por nombre o etiquetas.
- **Insignias automáticas y telemetría local**:
  - **🔥 Top #1–20**: Destaca los 20 emotes más utilizados según su puntuación de copias e interacciones.
  - **🍃 Nuevo**: Identifica automáticamente los emotes agregados en las últimas 24 horas.
  - Contadores persistentes de copias (`copy_count`), interacciones (`interaction_count`) y fecha de último uso (`last_used_at`).

### 🗄️ Almacenamiento Configurable y Resincronización

- **Carpeta de biblioteca personalizable**: Cambia la ubicación de tus emotes en cualquier momento con migración automática de archivos y fusión inteligente de metadatos entre bases de datos (`emotes.db`).
- **Persistencia aislada del árbol de compilación**: La base de datos SQLite y los archivos residen fuera de `src-tauri/` para evitar recompilaciones innecesarias durante el desarrollo.
- **Resincronización y optimización con un clic**: Escanea el directorio activo, convierte `PNG`/`JPG` sueltos a `.webp` y WebP animados a `.gif`, indexa archivos nuevos, limpia registros huérfanos en la base de datos y reporta el espacio en disco ahorrado.

---

## 🛠️ Stack Tecnológico

| Capa | Tecnología | Propósito |
| :--- | :--- | :--- |
| **Core / Backend** | **Rust** + **Tauri 2** | Procesamiento de imágenes, portapapeles nativo e IPC seguro |
| **Base de Datos** | **SQLite** (`rusqlite` bundled) | Persistencia local de metadatos, etiquetas y estadísticas |
| **Imagen / Animación** | `image` (`webp`, `gif`, `png`, `jpeg`) | Decodificación/codificación multifotograma y re-escalado `Lanczos3` |
| **Frontend** | **SvelteKit 5** + **TypeScript** + **Vite 6** | Interfaz reactiva, rápida y tipada de extremo a extremo |
| **Testing** | `cargo test` + **Vitest** + `svelte-check` | Pruebas unitarias deterministas y validación estática de tipos |

---

## 🚀 Requisitos e Instalación

### Requisitos Previos

1. **Rust** (edición 2021, estable) a través de [rustup](https://rustup.rs/).
2. **Node.js** (v18+) y **pnpm** (o `npm`).
3. **Dependencias del sistema para Tauri 2**:
   - **Windows**: Microsoft Visual Studio C++ Build Tools y WebView2 (incluido en Windows 10/11).
   - **Linux**: Paquetes de desarrollo de `webkit2gtk`, `libayatana-appindicator` y `wl-clipboard` (Wayland) o `xclip` (X11).

### Ejecución en Desarrollo

```bash
# 1. Instalar dependencias del frontend
pnpm install

# 2. Iniciar la aplicación en modo desarrollo (Vite + Tauri)
pnpm tauri dev
```

### Compilación para Producción

```bash
# Ejecuta las validaciones previas y genera el instalador nativo
pnpm tauri build
```

---

## 🧪 Verificación y Metodología de Desarrollo (ODD)

El desarrollo en **StickUp** se rige por **Organic Driven Development (ODD)** (`Explore → Plan → Implement → Verify`), manteniendo **Zero Dead Code** y aplicando **TDD (`RED → GREEN → REFACTOR`)** en la lógica de negocio.

Antes de cerrar cualquier cambio, puedes ejecutar la suite de verificación completa:

```bash
# Verificar compilación y warnings de Rust
cargo check --manifest-path src-tauri/Cargo.toml

# Ejecutar tests unitarios del backend en Rust
cargo test --manifest-path src-tauri/Cargo.toml

# Verificar tipos de Svelte y TypeScript
pnpm run check

# Ejecutar tests unitarios del frontend con Vitest
pnpm run test
```

---

## 📄 Licencia

Distribuido bajo la licencia **MIT**. Creado por [FrovaHappy](https://github.com/FrovaHappy).
