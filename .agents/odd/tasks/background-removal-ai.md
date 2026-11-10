# Tarea: Eliminación de Fondo de Imágenes con IA Local Bajo Demanda (ONNX Runtime + BiRefNet / RMBG)

## 1. Contexto y Problema
En StickUp, los emotes y stickers requieren fondos transparentes (canal alfa) para visualizarse correctamente en chats (Discord, Telegram, Slack) y en la propia aplicación. 
Muchos usuarios importan imágenes estáticas (memes, avatares, fotos) con fondos sólidos (blancos, negros) o fondos complejos que ensucian la experiencia como emote.

Se requiere:
- **Alcance Inicial:** Soporte tanto para **imágenes estáticas** (PNG, JPG, WebP) como para **GIFs animados** (procesamiento fotograma a fotograma con reporte de progreso y retención de delays y bucle).
- **Inferencia Local, Privada y Multiplataforma:** Procesamiento 100% en la máquina del usuario (sin enviar imágenes a la nube). Compatible con Windows, Linux y macOS vía `ort` (ONNX Runtime).
- **Selector de Modelos con BiRefNet Lite por Defecto:**
  - `birefnet_lite`: SOTA en segmentación y matting de alta resolución con ImageNet normalization y activación sigmoide (~224 MB).
  - `rmbg14_full`: Precisión flotante completa FP32 de BRIA AI (~176 MB).
  - `rmbg14_quantized`: Versión ligera de 8 bits para ahorrar espacio (~43 MB).
- **Descarga Bajo Demanda (*Lazy Fetch*) desde Hugging Face:** Los modelos **no** vienen empaquetados en el instalador inicial. Se descargan solo cuando el usuario lo solicita.
- **Resiliencia de Red y Avisos de Disponibilidad:** Detección de fallos de red, timeouts o indisponibilidad de servidores de Hugging Face con mensajes de error claros y accionables en la interfaz.
- **Feedback en Tiempo Real:** Barra de progreso con bytes y porcentaje durante la descarga del modelo e indicador visual de carga durante la inferencia.
- **Ajuste de Temperatura (Nitidez / Suavizado de Bordes):** Control interactivo para modular la temperatura de probabilidad de la máscara ($\text{logit} / T$), permitiendo elegir entre bordes duros/estrictos ($T < 1.0$) y bordes suaves/conservadores ($T > 1.0$).
- **Previsualización Interactiva:** En el modal de detalle/edición de emote o durante la importación, comparar original vs resultado recortado con fondo ajedrezado (*checkerboard*) antes de sobrescribir o crear una copia.
- **Gestión de Almacenamiento:** Panel en Ajustes para ver modelos disponibles, tamaño en disco, cambiar el modelo activo y liberar espacio.

---

## 2. Decisiones Técnicas y Arquitectura

### 2.1 Backend (Rust / Tauri v2)
1. **Motor de Inferencia:** Crate `ort` (ONNX Runtime 1.19+ / 2.x) con soporte de CPU multihilo y DirectML (en Windows si aplica).
2. **Modelo Neuronal:** `RMBG-1.4` (ONNX cuantizado INT8/FP16, ~40-44 MB).
   - Entrada: Tensor `[1, 3, 1024, 1024]` normalizado.
   - Salida: Máscara de probabilidad `[1, 1, 1024, 1024]`.
   - Post-proceso: Redimensionado bilineal a las dimensiones originales de la imagen y aplicación como canal alfa `RgbaImage`.
3. **Módulo de Servicio (`src-tauri/src/services/ai/` o `services/bg_removal/`):**
   - `model_manager.rs`:
     - Rutas de almacenamiento: `{app_data_dir}/models/rmbg-1.4.onnx`.
     - Verificación de integridad mediante hash SHA-256.
     - Descarga con streaming y reporte de progreso mediante eventos de Tauri (`bg-model-download-progress`).
     - Eliminación / consulta de estado del modelo.
   - `inference.rs`:
     - Carga diferida (*lazy loading*) de `ort::session::Session` en un `Mutex<Option<Session>>` en memoria.
     - Ejecución de preprocesamiento, inferencia y generación de imagen PNG en memoria o guardada en disco.
4. **Comandos IPC (`commands.rs` & `lib.rs`):**
   - `get_bg_model_status() -> Result<BgModelStatus, String>`
   - `download_bg_model() -> Result<(), String>`
   - `delete_bg_model() -> Result<(), String>`
   - `remove_image_background(source_path: String, target_path: Option<String>) -> Result<RemoveBgResult, String>`
   - `preview_remove_background(source_path: String) -> Result<String, String>` (retorna DataURL o PNG temporal para el visor)

### 2.2 Frontend (SvelteKit + Lucide Icons)
1. **Servicio y Store (`src/lib/services/bgRemoval.ts`):**
   - Manejo reactivo del estado del modelo (descargando, instalado, porcentaje, error).
2. **UI en Ajustes (`src/routes/settings/+page.svelte`):**
   - Sección "Inteligencia Artificial / Recorte de Fondo": Estado del modelo, botón de descarga manual y botón de liberar espacio.
3. **Modal de Detalle / Edición (`EmoteDetailModal.svelte`):**
   - Botón de acción con icono de tijeras o varita mágica (*Wand2* o *Scissors* de `@lucide/svelte`).
   - Modal de previsualización antes/después con toggle/slider sobre fondo ajedrezado.
   - Opciones: "Guardar como copia" o "Reemplazar actual".

---

## 3. Checklist de Implementación

- [x] 1. **Fase de Análisis & Dependencias:**
  - Evaluar dependencias en `src-tauri/Cargo.toml` (`ort`, downloader HTTP con streaming y TLS).
  - Diseñar estructura de directorios en `src-tauri/src/services/bg_removal/`.
- [x] 2. **Gestor de Modelo (`model_manager.rs`):**
  - Implementar verificación de existencia de archivo y tamaño/checksum.
  - Implementar descarga con emisión de eventos de progreso (`Tauri AppHandle`).
  - Implementar comando de eliminación de archivo.
  - Tests unitarios TDD para estados y validaciones de rutas.
- [x] 3. **Pipeline de Inferencia (`inference.rs`):**
  - Carga de sesión `ort` en memoria (`lazy_static` o `Mutex<Option<Session>>`).
  - Hilo vigilante (*idle session reaper*): liberación automática de la memoria RAM del modelo y del runtime de ONNX tras 5 minutos de inactividad.
  - Preprocesamiento de imagen (`image::DynamicImage` $\rightarrow$ Tensor normalizado 1024x1024).
  - Ejecución de inferencia y extracción de la máscara alfa.
  - Postprocesamiento: Mapeo de máscara a tamaño original y generación de `RgbaImage`.
  - Tests unitarios con fixtures sintéticos.
- [x] 4. **Comandos IPC & Integración Tauri:**
  - Exponer comandos en `commands.rs` y registrarlos en el builder de `lib.rs`.
- [x] 5. **Frontend - Tipos y API Wrapper:**
  - Actualizar `src/lib/types/` y `src/lib/services/api.ts`.
  - Crear listener de eventos de progreso en `bgRemoval.ts`.
- [x] 6. **Frontend - UI y Experiencia de Usuario:**
  - Agregar sección de gestión en `settings/+page.svelte`.
  - Integrar flujo de recorte y diálogo de previsualización en `EmoteDetailModal.svelte`.
  - Implementar visor de comparación superpuesto Before/After con cortina interactiva (`clip-path: inset`) y arrastre por puntero.
  - Respetar la guía de diseño oscura con acentos naranja y `@lucide/svelte`.
- [x] 7. **Verificación & Documentación:**
  - Documentar cambios en `changelogs/unreleased.md` (formato Keep a Changelog).
  - Verificar que compile sin errores ni warnings con `pnpm run prebuild`.
- [x] 8. **Soporte de GIFs Animados:**
  - Decodificación de frames con delays y preservación de loop Netscape.
  - Previsualización rápida sobre el primer fotograma en el Before/After slider.
  - Inferencia secuencial fotograma a fotograma con reporte de progreso (`bg-gif-progress`).
  - Reensamblado de GIF animado con transparencia (RGBA).
  - Habilitar botón en `EmoteDetailModal.svelte` para GIFs y barra de progreso de fotogramas en `RemoveBgModal.svelte`.

