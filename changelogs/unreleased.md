# Unreleased

### Added
- **Eliminación de fondo con IA local**: Inferencia nativa bajo demanda con BiRefNet Lite por defecto y previsualización interactiva.
- **Control de memoria RAM de IA en Ajustes**: Selector de tiempo de inactividad (Instantáneo, 1m, 5m, 10m, 30m) y botón para descargar el modelo de la memoria de inmediato.
- **Optimizador de arena de memoria ONNX**: Prevención de retención masiva de RAM en CPU (`kSameAsRequested` y sin memory pattern fijo).
- **Liberación automática de memoria tras inactividad**: Descarga de memoria RAM de los modelos ONNX e hilo vigilante tras tiempo de inactividad.
- **Comparación superpuesta Before/After**: Deslizador interactivo para comparar en tiempo real el emote original y el recorte sin fondo.
- **Selector de modelos de IA en Ajustes**: Opciones para elegir entre BiRefNet Lite (SOTA), RMBG-1.4 Completo y RMBG-1.4 Ligero.
- **Ajuste de temperatura de recorte**: Control interactivo para modular la nitidez, dureza o suavizado de los bordes del canal alfa.
- **Gestión de modelo IA en Configuración**: Descarga bajo demanda desde Hugging Face y opción para liberar espacio en disco.
- **Control de bucle en GIFs**: Ajuste en Configuración y en detalle del emote para alternar bucle infinito o frenar.
- **Servicio `services::gif`**: Inspección y parcheo binario sin pérdida del bloque `NETSCAPE2.0`.

### Changed
- **Copiado de GIFs**: Respeta repeticiones finitas configuradas al redimensionar en portapapeles.

### Fixed
- 
