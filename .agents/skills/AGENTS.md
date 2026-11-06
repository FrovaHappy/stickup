---
name: ODD & StickUp Architecture
description: ODD & StickUp Architecture skills for TDD & Organic Development in Rust/SvelteKit Project
---

# ODD & StickUp Architecture

Esta skill define las reglas arquitectónicas, la estructura de directorios, los estándares de código mínimo e indispensable y la metodología **Organic Driven Development (ODD)** para **StickUp**.

## 1. Metodología de Trabajo: Organic Driven Development (ODD)

Todo cambio en **StickUp** se rige por **ODD** (ver detalle completo en [.agents/skills/odd-workflow/SKILL.md](.agents/skills/odd-workflow/SKILL.md)), adaptando la planificación, documentación y verificación de forma proporcional al tamaño de la tarea (**Task Size**):

1. **Proporcionalidad según el Tamaño de Tarea:**
   - **Cambios pequeños o locales:** Se ejecutan de forma directa e inline tras explorar en modo lectura, sin generar artefactos pesados.
   - **Cambios sustanciales o complejos:** Generan documentación persistente en `.agents/odd/tasks/{feature-slug}.md` antes de escribir código (*Track Before First Write*), manteniendo un checklist actualizado paso a paso para permitir continuidad entre sesiones (*Durable Resume*).
2. **Ciclo de 4 Fases (`Explore → Plan → Implement → Verify`):**
   - **Explore (Read-Only):** Investigar los módulos afectados en `src-tauri/` y `src/` antes de editar. Hacer preguntas solo ante decisiones ambiguas reales.
   - **Plan:** Trazar pasos pragmáticos y proporcionales al alcance real.
   - **Implement (TDD & Slices):** Ejecutar en unidades manejables (~400 líneas máx. por corte). Aplicar **TDD (`RED → GREEN → REFACTOR`)** por defecto cuando exista un test determinista aplicable.
   - **Verify:** Validar compilación, tipado y tests antes de dar por concluida la tarea.

## 2. Principios de Diseño (Zero Dead Code)

0. **Gestor de Paquetes Exclusivo**:
   - El gestor de paquetes del proyecto es estrictamente **pnpm** (`pnpm run check`, `pnpm test`, `pnpm build`, `pnpm dev`, etc.). No usar npm.

1. **Mínimo e Indispensable**:
   - No mantener funciones, comandos de Tauri, plugins ni dependencias en `Cargo.toml` o `package.json` que no sean invocados activamente.
   - Si un comando del backend no tiene llamador en el frontend, debe eliminarse.
2. **Responsabilidad Única (SRP)**:
   - Ningún archivo de Rust o componente Svelte debe convertirse en un monolito.
   - Separar claramente: **Modelos/Tipos**, **Base de Datos**, **Procesamiento de Imagen**, **Portapapeles Nativo** y **Comandos IPC**.
3. **Persistencia Fuera del Árbol de Compilación**:
   - La base de datos SQLite (`emotes.db`) y la carpeta por defecto de emotes deben residir fuera de `src-tauri/` para evitar que el watcher de Tauri recompile la app en desarrollo al modificar metadatos.
4. **Iconografía Estándar con Lucide (`@lucide/svelte`)**:
   - Toda la iconografía de la interfaz de usuario debe construirse utilizando **Lucide Icons** mediante `@lucide/svelte` ([Guía oficial](https://lucide.dev/guide/svelte/)), evitando emojis como iconos o SVGs dispersos y desalineados.
   - Importar exclusivamente los iconos requeridos (cumpliendo *Zero Dead Code*), directamente desde `@lucide/svelte` o mediante sus subrutas (`@lucide/svelte/icons/*`).
   - Mantener coherencia visual en propiedades clave (`size`, `strokeWidth`, clases CSS auxiliares y color `currentColor`).

---

## 3. Estructura de Ficheros Estándar

### Backend (`src-tauri/src/`)

```text
src-tauri/src/
├── main.rs              # Punto de entrada mínimo (invoca stickup_lib::run)
├── lib.rs               # Registro del Builder de Tauri y declaración de módulos
├── models.rs            # Estructuras de datos serializables (Emote, ResyncStats)
├── db.rs                # Conexión SQLite, migración de tablas y rutas de almacenamiento
├── commands.rs          # Comandos #[tauri::command] expuestos al frontend
└── services/            # Módulos para servicios
    ├── [submodule]      # Módulo para servicios
    │   ├── index.rs     # Modulo base del servicio en caso que no se requiera separar por plataforma
    │   └── [platform]   # Módulo para servicios específicos de la plataforma (windows, linux) puede ser carpeta o fichero
```

### Frontend (`src/`)

```text
src/
├── app.html
├── +routes/
│   └── +page.svelte                 # Vista principal limpia
└── lib/
    ├── types/
        ├── [...].ts                 # Tipos específicos de cada feature
        └── global.ts                # Tipos globales 
    ├── services/
        ├── [...].ts                 # Servicios agrupados por funcionalidad
        └── api.ts                   # Wrapper tipado para invocaciones Tauri (invoke)
    ├── stores/
        └── [...].ts                 # Stores agrupados por funcionalidad
    ├── utils/
        └── [...].ts                 # Utilidades agrupados por funcionalidad
    └── components/
        └── system-ui/               # Componentes del sistema
        └── [...].ts                 # Componentes agrupados por funcionalidad
```

## 4. Checklist de Verificación (Fase `Verify` de ODD)

1. Verificar compilación de Rust:
   `cargo check --manifest-path src-tauri/Cargo.toml`
2. Verificar tipos de Svelte/TypeScript:
   `pnpm run check`
3. Ejecutar tests unitarios cuando aplique:
   `pnpm test` / `cargo test --manifest-path src-tauri/Cargo.toml`
4. Confirmar que no existan warnings de código muerto (`unused_imports`, `dead_code`) en Rust y actualizar `odd/tasks/{feature-slug}.md` si la tarea fue sustancial.
