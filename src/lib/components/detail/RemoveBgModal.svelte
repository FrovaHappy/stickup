<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import {
    Wand2,
    Download,
    Check,
    AlertCircle,
    Copy,
    RefreshCw,
    Sliders,
    RotateCcw,
    ChevronsLeftRight,
    Film
  } from '@lucide/svelte';
  import type { Emote } from '$lib/types/emote';
  import type { BgGifProgress } from '$lib/types/bgRemoval';
  import { isAnimatedEmote } from '$lib/utils/format';
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';
  import { toast } from '$lib/stores/toast';
  import {
    bgModelStatus,
    modelPresets,
    activeModelId,
    isDownloadingModel,
    downloadProgress,
    checkBgModelStatus,
    loadModelPresets,
    downloadBgModel,
    formatBytes
  } from '$lib/stores/bgRemoval';
  import { Modal, Button } from '$lib/components/system-ui';
  import { convertFileSrc } from '@tauri-apps/api/core';

  export let emote: Emote;
  export let open = false;
  export let onClose: () => void;
  export let onApplied: (updatedEmote: Emote) => void;

  let isProcessingPreview = false;
  let previewDataUrl: string | null = null;
  let previewError = '';
  let splitPercent = 50;
  let isDragging = false;
  let sliderContainerRef: HTMLDivElement | null = null;
  let isApplying = false;
  let temperature = 1.0;
  let tempDebounceTimeout: ReturnType<typeof setTimeout> | null = null;
  let gifProgress: BgGifProgress | null = null;
  let unlistenGifProgress: UnlistenFn | null = null;

  $: isGif = isAnimatedEmote(emote);
  $: currentPreset = $modelPresets.find((p) => p.id === $activeModelId) || $modelPresets[0] || null;

  onMount(async () => {
    await checkBgModelStatus();
    await loadModelPresets();
    if ($bgModelStatus?.is_downloaded) {
      await generatePreview();
    }
  });

  onDestroy(() => {
    if (unlistenGifProgress) {
      unlistenGifProgress();
      unlistenGifProgress = null;
    }
  });

  async function handleDownloadAndStart() {
    const ok = await downloadBgModel($activeModelId);
    if (ok) {
      await generatePreview();
    }
  }

  async function generatePreview() {
    if (!emote) return;
    isProcessingPreview = true;
    previewError = '';
    try {
      previewDataUrl = await api.previewRemoveBackground(emote.path, temperature, $activeModelId);
    } catch (err) {
      previewError = typeof err === 'string' ? err : 'Error al procesar el recorte de fondo.';
    } finally {
      isProcessingPreview = false;
    }
  }

  function handleTemperatureInput(newVal: number) {
    temperature = newVal;
    if (tempDebounceTimeout) clearTimeout(tempDebounceTimeout);
    tempDebounceTimeout = setTimeout(() => {
      if ($bgModelStatus?.is_downloaded) {
        generatePreview();
      }
    }, 200);
  }

  function resetTemperature() {
    temperature = 1.0;
    if (tempDebounceTimeout) clearTimeout(tempDebounceTimeout);
    if ($bgModelStatus?.is_downloaded) {
      generatePreview();
    }
  }

  function updateSplitFromPointer(clientX: number) {
    if (!sliderContainerRef) return;
    const rect = sliderContainerRef.getBoundingClientRect();
    if (rect.width <= 0) return;
    const x = clientX - rect.left;
    const pct = Math.max(0, Math.min(100, (x / rect.width) * 100));
    splitPercent = Math.round(pct * 10) / 10;
  }

  function handlePointerDown(e: PointerEvent) {
    isDragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    updateSplitFromPointer(e.clientX);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging) return;
    updateSplitFromPointer(e.clientX);
  }

  function handlePointerUp(e: PointerEvent) {
    if (!isDragging) return;
    isDragging = false;
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {
      // Ignore if pointer capture release fails
    }
  }

  async function handleApply(asCopy: boolean) {
    if (!emote || isApplying) return;
    isApplying = true;
    gifProgress = null;

    if (isGif && !unlistenGifProgress) {
      try {
        unlistenGifProgress = await listen<BgGifProgress>('bg-gif-progress', (event) => {
          gifProgress = event.payload;
        });
      } catch (e) {
        console.error('Error al escuchar progreso de GIF:', e);
      }
    }

    try {
      const updated = await api.applyRemoveBackground(emote.id, asCopy, temperature, $activeModelId);
      await loadEmotes();
      toast.success(asCopy ? 'Se creó una copia del emote sin fondo' : 'Fondo del emote eliminado');
      onApplied(updated);
      onClose();
    } catch (err) {
      const msg = typeof err === 'string' ? err : 'No se pudo aplicar el recorte de fondo';
      toast.error(msg);
    } finally {
      isApplying = false;
      gifProgress = null;
      if (unlistenGifProgress) {
        unlistenGifProgress();
        unlistenGifProgress = null;
      }
    }
  }
</script>

<Modal
  {open}
  title="Eliminar fondo con IA Local"
  onClose={() => {
    if (!isApplying) onClose();
  }}
>
  <div class="remove-bg-container">
    {#if !$bgModelStatus?.is_downloaded}
      <!-- Estado 1: El modelo no está descargado aún -->
      <div class="model-download-box">
        <div class="model-icon-circle">
          <Wand2 size={32} class="text-orange" />
        </div>
        <h3>Modelo de IA requerido</h3>
        <p class="model-desc">
          Para recortar el fondo sin enviar imágenes a la nube, StickUp utiliza
          <strong>{currentPreset ? currentPreset.name : 'BiRefNet Lite'}</strong> (~{currentPreset?.approx_size_mb || 224} MB).
          Se descarga una sola vez y funcionará de por vida sin conexión.
        </p>

        {#if $isDownloadingModel}
          <div class="download-progress-container">
            <div class="progress-info-row">
              <span class="progress-text">Descargando {currentPreset ? currentPreset.name : 'modelo'} desde Hugging Face...</span>
              <span class="progress-percent">
                {$downloadProgress?.percentage ? `${Math.round($downloadProgress.percentage)}%` : '0%'}
              </span>
            </div>
            <div class="progress-bar-bg">
              <div
                class="progress-bar-fill"
                style="width: {$downloadProgress?.percentage || 0}%;"
              ></div>
            </div>
            <div class="progress-bytes-row">
              <span class="bytes-text">
                {formatBytes($downloadProgress?.downloaded_bytes || 0)} /
                {$downloadProgress?.total_bytes ? formatBytes($downloadProgress.total_bytes) : `~${currentPreset?.approx_size_mb || 224} MB`}
              </span>
            </div>
          </div>
        {:else}
          <div class="download-action-row">
            <Button
              variant="primary"
              size="md"
              on:click={handleDownloadAndStart}
            >
              <Download size={16} />
              <span>Descargar modelo e iniciar (~{currentPreset?.approx_size_mb || 224} MB)</span>
            </Button>
          </div>
        {/if}
      </div>
    {:else if isProcessingPreview}
      <!-- Estado 2: Procesando inferencia local -->
      <div class="processing-box">
        <div class="spinner-circle">
          <RefreshCw size={28} class="spin-icon text-orange" />
        </div>
        <p class="processing-title">Procesando recorte con IA local...</p>
        <span class="processing-desc">Detectando bordes y aislando el primer plano...</span>
      </div>
    {:else if previewError}
      <!-- Estado 3: Error durante la previsualización -->
      <div class="error-box">
        <AlertCircle size={32} class="text-warning" />
        <h4>No se pudo eliminar el fondo</h4>
        <p>{previewError}</p>
        <Button variant="secondary" size="sm" on:click={generatePreview}>
          <RefreshCw size={14} /> Reintentar
        </Button>
      </div>
    {:else if previewDataUrl}
      <!-- Estado 4: Previsualización lista con comparador superpuesto -->
      <div class="preview-layout">
        {#if isApplying}
          <div class="applying-overlay" role="status" aria-live="polite">
            <div class="spinner-circle">
              <RefreshCw size={28} class="spin-icon text-orange" />
            </div>
            {#if isGif}
              <h4 class="applying-title">Eliminando fondo del GIF animado...</h4>
              <p class="applying-subtitle">
                Procesando fotograma a fotograma con IA y preservando la animación continua.
              </p>
              <div class="gif-progress-card">
                <div class="gif-progress-header">
                  <span class="gif-frames-text">
                    {#if gifProgress && gifProgress.total_frames > 0}
                      Fotograma {gifProgress.current_frame} de {gifProgress.total_frames}
                    {:else}
                      Preparando fotogramas...
                    {/if}
                  </span>
                  <span class="gif-percent-text">
                    {gifProgress ? `${Math.round(gifProgress.percentage)}%` : '0%'}
                  </span>
                </div>
                <div class="progress-bar-bg">
                  <div
                    class="progress-bar-fill"
                    style="width: {gifProgress?.percentage || 0}%;"
                  ></div>
                </div>
              </div>
            {:else}
              <h4 class="applying-title">Aplicando recorte con IA...</h4>
              <p class="applying-subtitle">
                Guardando el nuevo emote con canal alfa transparente.
              </p>
            {/if}
          </div>
        {/if}

        <!-- Visor superpuesto Before / After con cortina clip-path -->
        <div
          class="comparison-container"
          bind:this={sliderContainerRef}
          on:pointerdown={handlePointerDown}
          on:pointermove={handlePointerMove}
          on:pointerup={handlePointerUp}
          on:pointercancel={handlePointerUp}
          role="slider"
          aria-label="Comparación antes y después"
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow={splitPercent}
          tabindex="0"
        >
          <!-- Capa inferior: Resultado sin fondo sobre checkerboard -->
          <div class="layer base-layer">
            <img src={previewDataUrl} alt="Sin fondo" class="comparison-img" />
          </div>

          <!-- Capa superior: Original con fondo, recortada según splitPercent -->
          <div
            class="layer overlay-layer"
            style="clip-path: inset(0 {100 - splitPercent}% 0 0);"
          >
            <img src={convertFileSrc(emote.path)} alt="Original" class="comparison-img" />
          </div>

          <!-- Línea divisoria y manija interactiva -->
          <div class="divider-line" style="left: {splitPercent}%;">
            <div class="divider-handle">
              <ChevronsLeftRight size={14} />
            </div>
          </div>

          <!-- Etiquetas flotantes informativas -->
          <div class="floating-tag tag-left" style="opacity: {splitPercent < 15 ? 0 : 1};">
            <span>Original</span>
          </div>
          <div class="floating-tag tag-right" style="opacity: {splitPercent > 85 ? 0 : 1};">
            <span class="text-orange">Sin Fondo</span>
          </div>
        </div>

        <!-- Control de Temperatura de Recorte -->
        <div class="temperature-control-box">
          <div class="temperature-header">
            <div class="temperature-label-row">
              <Sliders size={14} class="text-orange" />
              <span class="temperature-title">Temperatura de recorte</span>
              <span class="temperature-badge">{temperature.toFixed(2)}</span>
            </div>
            {#if Math.abs(temperature - 1.0) > 0.01}
              <button
                type="button"
                class="reset-temp-btn"
                on:click={resetTemperature}
                title="Restablecer temperatura por defecto (1.0)"
              >
                <RotateCcw size={12} />
                <span>Restablecer</span>
              </button>
            {/if}
          </div>

          <div class="slider-row">
            <span class="slider-side-label">Firme (0.2)</span>
            <input
              type="range"
              min="0.2"
              max="2.0"
              step="0.05"
              value={temperature}
              on:input={(e) => handleTemperatureInput(parseFloat(e.currentTarget.value))}
              class="temp-slider"
            />
            <span class="slider-side-label">Suave (2.0)</span>
          </div>

          <p class="temperature-hint">
            {#if temperature < 0.8}
              Borde más firme y afilado. Útil para eliminar halos residuales del fondo.
            {:else if temperature > 1.2}
              Borde más suave y difuso. Conserva detalles finos, pelaje o semitransparencias.
            {:else}
              Ajuste neutral equilibrado de la red neuronal.
            {/if}
          </p>
        </div>

        <div class="preview-info-strip">
          {#if isGif}
            <span class="info-badge">
              <Film size={12} class="text-orange" /> GIF Animado (Frame 1 en visor)
            </span>
          {:else}
            <span class="info-badge">
              <Check size={12} class="text-orange" /> {currentPreset ? currentPreset.name.split(' (')[0] : 'IA'} (PNG transparente)
            </span>
          {/if}
          <span class="info-badge">
            {emote.original_width}×{emote.original_height} px
          </span>
        </div>

        {#if isGif}
          <div class="gif-hint-box">
            <Film size={14} class="text-orange flex-shrink-0" />
            <span>
              Previsualización instantánea del primer fotograma. Al aplicar, la IA procesará la secuencia completa manteniendo el bucle infinito.
            </span>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <svelte:fragment slot="footer">
    <Button variant="ghost" size="sm" on:click={onClose} disabled={isApplying}>
      Cancelar
    </Button>

    {#if $bgModelStatus?.is_downloaded && previewDataUrl && !isProcessingPreview}
      <Button
        variant="secondary"
        size="sm"
        disabled={isApplying}
        loading={isApplying}
        on:click={() => handleApply(true)}
        title="Guarda una nueva copia del emote sin modificar el original"
      >
        <Copy size={14} />
        <span>Guardar como copia</span>
      </Button>

      <Button
        variant="primary"
        size="sm"
        disabled={isApplying}
        loading={isApplying}
        on:click={() => handleApply(false)}
        title="Reemplaza la imagen del emote actual por la versión sin fondo"
      >
        <Check size={14} />
        <span>Reemplazar actual</span>
      </Button>
    {/if}
  </svelte:fragment>
</Modal>

<style>
  .remove-bg-container {
    min-height: 240px;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .model-download-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 24px 16px;
  }

  .model-icon-circle {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background: rgba(249, 115, 22, 0.1);
    border: 1px solid rgba(249, 115, 22, 0.25);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 16px;
  }

  .model-download-box h3 {
    margin: 0 0 8px 0;
    font-size: 1.15rem;
    color: #fff;
  }

  .model-desc {
    max-width: 440px;
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.65);
    line-height: 1.45;
    margin: 0 0 20px 0;
  }

  .download-progress-container {
    width: 100%;
    max-width: 420px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    padding: 16px;
    border-radius: 10px;
  }

  .progress-info-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    margin-bottom: 8px;
    color: rgba(255, 255, 255, 0.85);
  }

  .progress-percent {
    font-weight: 600;
    color: var(--color-orange-primary, #f97316);
  }

  .progress-bar-bg {
    width: 100%;
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--color-orange-primary, #f97316);
    transition: width 0.2s ease;
  }

  .progress-bytes-row {
    margin-top: 6px;
    text-align: right;
    font-size: 0.78rem;
    color: rgba(255, 255, 255, 0.5);
  }

  .download-action-row {
    display: flex;
    justify-content: center;
  }

  .processing-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px 16px;
  }

  .spinner-circle {
    margin-bottom: 16px;
  }

  :global(.spin-icon) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .processing-title {
    font-size: 1.05rem;
    font-weight: 600;
    color: #fff;
    margin: 0 0 6px 0;
  }

  .processing-desc {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.55);
  }

  .error-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 30px 16px;
    text-align: center;
    gap: 12px;
  }

  .preview-layout {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .applying-overlay {
    position: absolute;
    inset: -6px;
    z-index: 20;
    background: rgba(22, 22, 25, 0.92);
    backdrop-filter: blur(8px);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px 20px;
    text-align: center;
  }

  .applying-title {
    margin: 0 0 6px 0;
    font-size: 1.05rem;
    font-weight: 600;
    color: #ffffff;
  }

  .applying-subtitle {
    margin: 0 0 16px 0;
    font-size: 0.83rem;
    color: rgba(255, 255, 255, 0.65);
    max-width: 400px;
    line-height: 1.4;
  }

  .gif-progress-card {
    width: 100%;
    max-width: 360px;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .gif-progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.82rem;
  }

  .gif-frames-text {
    color: rgba(255, 255, 255, 0.85);
    font-weight: 500;
  }

  .gif-percent-text {
    font-weight: 700;
    font-family: monospace;
    color: var(--color-orange-primary, #f97316);
  }

  .gif-hint-box {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(249, 115, 22, 0.08);
    border: 1px solid rgba(249, 115, 22, 0.2);
    border-radius: 8px;
    padding: 8px 12px;
    font-size: 0.78rem;
    color: rgba(255, 255, 255, 0.8);
    line-height: 1.4;
  }

  /* Contenedor principal de comparación superpuesta */
  .comparison-container {
    width: 100%;
    height: 270px;
    position: relative;
    border-radius: 10px;
    overflow: hidden;
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    cursor: ew-resize;
    user-select: none;
    touch-action: none;
    background-color: #1a1a1d;
    background-image:
      linear-gradient(45deg, #252528 25%, transparent 25%),
      linear-gradient(-45deg, #252528 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #252528 75%),
      linear-gradient(-45deg, transparent 75%, #252528 75%);
    background-size: 16px 16px;
    background-position: 0 0, 0 8px, 8px -8px, -8px 0px;
  }

  .comparison-container:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 2px;
  }

  .layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .base-layer {
    z-index: 1;
  }

  .overlay-layer {
    z-index: 2;
    will-change: clip-path;
  }

  .comparison-img {
    max-width: 90%;
    max-height: 90%;
    width: auto;
    height: auto;
    object-fit: contain;
    filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.45));
    pointer-events: none;
  }

  /* Línea divisoria y manija */
  .divider-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: #ffffff;
    box-shadow: 0 0 8px rgba(0, 0, 0, 0.8), 0 0 2px rgba(0, 0, 0, 0.6);
    pointer-events: none;
    transform: translateX(-50%);
    z-index: 3;
    will-change: left;
  }

  .divider-handle {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: #1c1c1f;
    border: 2px solid #ffffff;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-orange-primary, #f97316);
    pointer-events: none;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }

  .comparison-container:hover .divider-handle {
    transform: translate(-50%, -50%) scale(1.08);
    border-color: var(--color-orange-primary, #f97316);
    color: #ffffff;
  }

  /* Etiquetas flotantes en esquinas */
  .floating-tag {
    position: absolute;
    bottom: 10px;
    z-index: 4;
    background: rgba(20, 20, 22, 0.78);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    padding: 3px 10px;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.85);
    pointer-events: none;
    transition: opacity 0.2s ease;
  }

  .tag-left {
    left: 10px;
  }

  .tag-right {
    right: 10px;
  }

  .preview-info-strip {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.65);
    padding: 4px 2px;
  }

  .info-badge {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .text-orange {
    color: var(--color-orange-primary, #f97316);
  }

  :global(.text-warning) {
    color: var(--color-warning-primary, #ea580c);
  }

  .temperature-control-box {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .temperature-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .temperature-label-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .temperature-title {
    font-size: 0.83rem;
    font-weight: 600;
    color: #fff;
  }

  .temperature-badge {
    font-size: 0.75rem;
    font-family: monospace;
    background: rgba(249, 115, 22, 0.15);
    color: var(--color-orange-primary, #f97316);
    padding: 1px 6px;
    border-radius: 4px;
    font-weight: 600;
  }

  .reset-temp-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    font-size: 0.75rem;
    cursor: pointer;
    transition: color 0.15s ease;
  }

  .reset-temp-btn:hover {
    color: var(--color-orange-primary, #f97316);
  }

  .slider-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .slider-side-label {
    font-size: 0.72rem;
    color: rgba(255, 255, 255, 0.45);
    flex-shrink: 0;
  }

  .temp-slider {
    flex: 1;
    accent-color: var(--color-orange-primary, #f97316);
    height: 4px;
    cursor: pointer;
  }

  .temperature-hint {
    margin: 0;
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.55);
    line-height: 1.35;
  }
</style>
