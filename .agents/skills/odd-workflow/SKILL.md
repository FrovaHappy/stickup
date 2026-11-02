---
name: odd-workflow
description: >-
  Regido por Organic Driven Development (ODD). Adapta la planificación,
  documentación y verificación de forma proporcional al tamaño de la tarea.
  Activar cuando el usuario pida "implementa", "crea la feature", "desarrolla",
  "refactoriza", "corrige" o mencione "odd".
---

# Organic Driven Development (ODD) - Workflow Skill

## 1. Filosofía y Principio Rector

El proceso de desarrollo debe crecer de forma **proporcional y orgánica** según la complejidad, el alcance y el riesgo de la tarea (**Task Size**):

- **Cambios pequeños o locales (Inline):**
  - Correcciones puntuales, ajustes de UI aislados, renombres o cambios de pocas líneas con alcance claro.
  - Se ejecutan de forma directa e inline sin burocracia ni artefactos pesados.
- **Cambios sustanciales o complejos (Tracked):**
  - Nuevas features de extremo a extremo (Rust + SvelteKit), refactorizaciones multi-módulo, cambios de esquema SQLite o tareas que abarquen múltiples sesiones.
  - Generan documentación persistente de seguimiento en `odd/tasks/{feature-slug}.md` **antes de realizar la primera escritura de código** (*Track Before First Write*).

---

## 2. Ciclo de Vida ODD (4 Fases)

### Fase 1: Explore (Exploración en Solo Lectura)

1. **Lectura primero:** Analizar el código existente, dependencias y contratos IPC antes de mutar cualquier archivo.
2. **Preguntas dirigidas:** Preguntar al usuario **únicamente** cuando exista una decisión de producto o técnica genuinamente ambigua que no pueda deducirse del código.
3. **Evaluación de tamaño (Task Sizing):**
   - Si el cambio es **pequeño/local**, pasar directamente a ejecución tras confirmar el contexto.
   - Si el cambio es **sustancial/complejo**, inicializar o actualizar `odd/tasks/{feature-slug}.md` con el alcance, criterios de aceptación y checklist de tareas.

### Fase 2: Plan (Planificación Proporcional)

1. Diseñar una secuencia clara y pragmática de pasos basada en los hallazgos reales del repositorio (archivos afectados, contratos de tipos y límites de prueba).
2. Evitar especificaciones especulativas extensas; mantener el plan enfocado en entregables verificables.
3. **Continuidad (Durable Resume):** Si existe un archivo activo en `odd/tasks/{feature-slug}.md`, reconciliar el estado actual con el checklist para retomar exactamente donde quedó la sesión previa.

### Fase 3: Implement (Implementación Incremental y TDD)

1. **Ejecución por unidades manejables:** Implementar tarea por tarea en cortes revisables (~400 líneas modificadas como referencia máxima por bloque lógico), actualizando el progreso en `odd/tasks/{feature-slug}.md` cuando aplique.
2. **TDD por defecto (`RED → GREEN → REFACTOR`):**
   - Cuando exista un test determinista ejecutable y un resultado esperado claro, escribir o ajustar primero el test que falla (**RED**), implementar el código mínimo necesario para pasarlo (**GREEN**) y limpiar/modularizar manteniendo los tests en verde (**REFACTOR**).
   - Si aplica una excepción (por ejemplo, ajustes puramente visuales o interoperabilidad nativa de sistema operativo), dejar constancia explícita y validar mediante verificación funcional y chequeo de compilación.

### Fase 4: Verify (Verificación y Cierre)

1. Ejecutar las comprobaciones deterministas y de tipos correspondientes a las capas modificadas:
   - **Backend (Rust / Tauri):** `cargo check --manifest-path src-tauri/Cargo.toml` y `cargo test --manifest-path src-tauri/Cargo.toml`
   - **Frontend (SvelteKit / TypeScript):** `npm run check` y `npm run test`
2. Confirmar cumplimiento de **Zero Dead Code** (sin imports huérfanos, comandos IPC sin consumidor ni código muerto).
3. Marcar las tareas completadas en `odd/tasks/{feature-slug}.md` (si fue un cambio sustancial) y reportar los resultados de forma concisa.
4. **Control del usuario:** Acciones como `git commit`, `git push`, creación de PRs o merges quedan siempre bajo decisión explícita del usuario.

---

## 3. Plantilla para `odd/tasks/{feature-slug}.md`

Cuando una tarea requiera seguimiento persistente, utilizar esta estructura mínima:

```markdown
# Task: {Título de la tarea}

- **Estado:** `in-progress` | `completed` | `blocked`
- **Tamaño:** Sustancial / Multi-módulo
- **Archivos clave:**
  - `src-tauri/src/...`
  - `src/lib/...`

## Objetivo y Alcance
Breve descripción del problema y del resultado esperado.

## Checklist de Ejecución
- [ ] 1. Exploración y definición de contratos / tipos
- [ ] 2. Pruebas y lógica de backend (`src-tauri`)
- [ ] 3. Integración y estado en frontend (`src/lib`)
- [ ] 4. Verificación (`cargo check`, `npm run check`, tests)

## Notas de Continuidad / Decisiones
- Registro breve de decisiones técnicas tomadas o bloqueos encontrados.
```
