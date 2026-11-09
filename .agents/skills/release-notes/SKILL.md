---
name: release-notes
description: >-
  Gestión del registro de cambios (Changelog / Version Log) de StickUp.
  Reglas basadas en Keep a Changelog y Semantic Versioning para mantener
  changelogs/unreleased.md al día y preparar notas de release para el CI.
  Activar cuando se pida "changelog", "version log", "release notes", "actualiza changelog",
  "documenta cambios" o al concluir una tarea sustancial/feature/fix.
---

# Release Notes & Changelog Skill

Esta skill define el estándar y flujo de trabajo para registrar cambios en **StickUp**, asegurando compatibilidad con el sistema de CI / GitHub Actions mediante archivos modulares en `changelogs/`.

## 1. Principios y Formato

El registro de cambios se rige por:

- **[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)**: Redacción clara dirigida a humanos, agrupada por categorías semánticas.
- **[Semantic Versioning (SemVer)](https://semver.org/)**:
  - `MAJOR`: Cambios incompatibles o que rompen la API / esquema.
  - `MINOR`: Nuevas funcionalidades retrocompatibles.
  - `PATCH`: Correcciones de errores retrocompatibles.
- **Estructura Modular (Zero Monolithic Changelog)**:
  - No mantener un único archivo `.md` infinito.
  - Los cambios en desarrollo residen **exclusivamente** en `changelogs/unreleased.md`.
  - Cada versión publicada posee su archivo congelado independiente: `changelogs/v{VERSION}.md` (ej. `changelogs/v0.1.1.md`).

---

## 2. Categorías Estándar de Cambios

Dentro de `changelogs/unreleased.md`, clasificar cada cambio bajo una de las siguientes secciones:

- `### Added`: Nuevas características, nuevos componentes de UI, comandos de Tauri o configuraciones.
- `### Changed`: Modificaciones a funcionalidades existentes, refactorizaciones de componentes o rediseño de flujos.
- `### Deprecated`: Funcionalidades que dejarán de tener soporte en futuras versiones.
- `### Removed`: Funcionalidades o código muerto eliminado (respetando *Zero Dead Code*).
- `### Fixed`: Corrección de errores, fallos de renderizado o bugs de plataforma.
- `### Security`: Mejoras de seguridad o resolución de vulnerabilidades.

### Estilo de Redacción

- Usar un título breve en negrita con el componente o módulo afectado, seguido de dos puntos y una descripción concisa.
- Ejemplo:

  ```markdown
  ### Added
  - **Modal flotante de confirmación (`PasteConfirmPopup`)**: Interfaz emergente para previsualizar y asignar tags al pegar desde el portapapeles.
  ```

---

## 3. Plantilla Base de `changelogs/unreleased.md`

```markdown
# Unreleased

### Added
- 

### Changed
- 

### Fixed
- 
```

---

## 4. Flujo de Trabajo para el Agente (Durante el Desarrollo)

1. **Al completar una tarea, feature, corrección o refactor:**
   - Abrir y verificar [changelogs/unreleased.md](file:///c:/Users/happy/repos/stickup/changelogs/unreleased.md).
   - Añadir una viñeta concisa en la sección correspondiente (`Added`, `Changed`, `Fixed`, etc.).
   - Mantener el archivo limpio, eliminando secciones vacías si no tienen elementos.

2. **Verificación de cierre en ODD (Fase `Verify`):**
   - Asegurarse de que toda modificación funcional relevante cuente con su entrada en `changelogs/unreleased.md` antes de dar la tarea por concluida.

---

## 5. Integración con el CI (GitHub Actions)

El workflow de release ([.github/workflows/build.yml](file:///.github/workflows/build.yml)) automatiza el ciclo de vida:

1. **Al disparar un release (`major`, `minor`, `fix`):**
   - Renombra automáticamente `changelogs/unreleased.md` a `changelogs/v{NUEVA_VERSION}.md`.
   - Reemplaza el encabezado `# Unreleased` por `# Release v{NUEVA_VERSION} (YYYY-MM-DD)`.
   - Inicializa un nuevo `changelogs/unreleased.md` en blanco.
   - Agrega los cambios de `changelogs/` al commit `chore(release): {NUEVA_VERSION}`.
2. **Al publicar en GitHub Releases:**
   - Lee directamente `changelogs/v{TAG}.md` y lo pasa a GitHub CLI con `--notes-file`, evitando consumo innecesario de recursos o parseos con expresiones regulares.
