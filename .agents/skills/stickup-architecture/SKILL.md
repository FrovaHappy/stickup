---
name: stickup-architecture
description: >-
  Guía de arquitectura, estándares de modularidad, estructura de ficheros y reglas
  de refactorización del proyecto StickUp (Tauri v2 + Rust + SvelteKit). Usar esta
  skill antes de agregar funcionalidades, refactorizar módulos o limpiar código muerto.
---

# StickUp Architecture & Refactoring Skill

Esta skill define las reglas arquitectónicas, la estructura de directorios y los estándares de código mínimo e indispensable para **StickUp**.

---

## 1. Principios de Diseño (Zero Dead Code)

1. **Mínimo e Indispensable**:
   - No mantener funciones, comandos de Tauri, plugins ni dependencias en `Cargo.toml` o `package.json` que no sean invocados activamente.
   - Si un comando del backend no tiene llamador en el frontend, debe eliminarse.
2. **Responsabilidad Única (SRP)**:
   - Ningún archivo de Rust o componente Svelte debe convertirse en un monolito.
   - Separar claramente: **Modelos/Tipos**, **Base de Datos**, **Procesamiento de Imagen**, **Portapapeles Nativo** y **Comandos IPC**.
3. **Persistencia Fuera del Árbol de Compilación**:
   - La base de datos SQLite (`emotes.db`) y la carpeta por defecto de emotes deben residir fuera de `src-tauri/` para evitar que el watcher de Tauri recompile la app en desarrollo al modificar metadatos.

---

## 2. Estructura de Ficheros Estándar

### Backend (`src-tauri/src/`)
```text
src-tauri/src/
├── main.rs              # Punto de entrada mínimo (invoca app_lib::run)
├── lib.rs               # Registro del Builder de Tauri y declaración de módulos
├── models.rs            # Estructuras de datos serializables (Emote, ResyncStats)
├── db.rs                # Conexión SQLite, migración de tablas y rutas de almacenamiento
├── commands.rs          # Comandos #[tauri::command] expuestos al frontend
└── clipboard/           # Subsistema multiplataforma de portapapeles e imagen
    ├── mod.rs           # Orquestación de redimensionado (estático y GIF frame-by-frame)
    ├── windows.rs       # Implementación Win32 (CF_HDROP, MIME, CF_DIB condicional)
    ├── linux.rs         # Implementación Linux (Wayland wl-copy / X11 xclip)
    └── android.rs       # Stub / Adaptador Android
```

### Frontend (`src/`)
```text
src/
├── app.html
├── routes/
│   ├── +layout.ts
│   └── +page.svelte                 # Vista principal limpia
└── lib/
    ├── types/
    │   └── emote.ts                 # Interfaces TypeScript (Emote, SizePreset, ResyncStats)
    ├── services/
    │   └── api.ts                   # Wrapper tipado para invocaciones Tauri (invoke)
    ├── stores/
    │   └── emotes.ts                # Estado reactivo global (emotes, allTags, loadEmotes)
    ├── utils/
    │   └── format.ts                # Utilidades puras (formatFileSize, cálculo de dimensiones)
    └── components/
        ├── StorageSettings.svelte   # Selector de ruta de guardado
        ├── ImportButton.svelte      # Botones de importación y resincronización con estadísticas
        ├── EmoteGrid.svelte         # Cuadrícula de emotes
        ├── EmoteCard.svelte         # Tarjeta individual de emote
        └── detail/                  # Subcomponentes del modal de detalle
            ├── EmoteDetailModal.svelte  # Contenedor principal del modal
            └── TagManager.svelte        # Gestor de chips de tags con autocompletado global
```

---

## 3. Reglas Críticas de Dominio

1. **Importación de Imágenes (`import_folder`)**:
   - Imágenes estáticas (`png`, `jpg`, `jpeg`, `gif` de 1 frame) se convierten automáticamente a `.webp`.
   - GIFs animados (`frames.len() > 1`) se detectan con `GifDecoder` y se copian intactos como `.gif` para conservar su secuencia de fotogramas y retardos (`delay`).
2. **Portapapeles y GIFs Animados (`copy_to_clipboard`)**:
   - En Windows, **nunca** emitir `CF_DIB` cuando el archivo es un GIF animado, ya que Discord/Telegram priorizan el bitmap estático sobre `CF_HDROP` y congelan el GIF.
   - Si se redimensiona un GIF animado, redimensionar todos los fotogramas con `GifDecoder` + `GifEncoder` (`Repeat::Infinite`).
3. **Guardado de Nombre y Tags (`update_tag`)**:
   - Los cambios de nombre o de tags deben persistir de inmediato sin funciones duplicadas.

---

## 4. Checklist de Verificación tras Refactorizar

1. Verificar compilación de Rust:
   `cargo check --manifest-path app/src-tauri/Cargo.toml`
2. Verificar tipos de Svelte/TypeScript:
   `npm --prefix app run check`
3. Confirmar que no existan warnings de código muerto (`unused_imports`, `dead_code`) en Rust.

