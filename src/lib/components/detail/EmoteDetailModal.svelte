<script lang="ts">
  import { onMount } from 'svelte';
  import type { Emote } from '$lib/types/emote';
  import { api } from '$lib/services/api';
  import { loadEmotes, topHotEmoteIds, isEmoteNewByDate } from '$lib/stores/emotes';
  import {
    FORMAT_PRESETS,
    SIZE_PRESETS,
    formatFileSize,
    getMimeType,
    renderEmoteToCanvas,
    resolveDimensions
  } from '$lib/utils/format';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import TagManager from './TagManager.svelte';

  export let emote: Emote | null = null;
  export let onClose: () => void;

  let selectedSize = '48';
  let selectedFormat = 'webp';

  let emoteName = '';
  let tagList: string[] = [];
  let copyFeedback = '';
  let copyButtonText = 'Copy to Clipboard';
  let copyCount = 0;
  let interactionCount = 0;

  let newFileSize: number | null = null;
  let isCalculatingSize = false;

  onMount(async () => {
    if (emote) {
      copyCount = emote.copy_count ?? 0;
      interactionCount = (emote.interaction_count ?? 0) + 1;
      emote.interaction_count = interactionCount;
      try {
        await api.recordInteraction(emote.id);
        loadEmotes();
      } catch {
        // Ignorar errores silenciosos de telemetría local
      }
    }
  });

  $: if (emote) {
    emoteName = emote.name ?? '';
    copyCount = emote.copy_count ?? 0;
    interactionCount = emote.interaction_count ?? 0;
    tagList = (emote.tag ?? '')
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);
  }

  $: isHot = emote ? $topHotEmoteIds.has(emote.id) : false;
  $: isNew = emote ? isEmoteNewByDate(emote.created_at) : false;

  $: targetDimensions = resolveDimensions(emote, selectedSize);

  $: if (emote && (selectedSize || selectedFormat)) {
    calculateNewSize();
  }

  $: sizeDiffPercent = (() => {
    if (!emote?.file_size || !newFileSize || emote.file_size <= 0) return null;
    return Math.round(((newFileSize - emote.file_size) / emote.file_size) * 100);
  })();

  $: resolutionDiffPercent = (() => {
    if (!emote) return null;
    const origArea = emote.original_width * emote.original_height;
    const newArea = targetDimensions.width * targetDimensions.height;
    if (origArea <= 0) return null;
    return Math.round(((newArea - origArea) / origArea) * 100);
  })();

  function showToast(msg: string) {
    copyFeedback = msg;
    setTimeout(() => {
      copyFeedback = '';
    }, 2500);
  }

  async function persistMetadata(newName: string, newTags: string[]) {
    if (!emote) return;
    tagList = newTags;
    const joinedTags = newTags.join(', ');
    emote.name = newName;
    emote.tag = joinedTags;
    try {
      await api.updateTag(emote.id, newName, joinedTags);
    } catch (e: any) {
      showToast(`Error al guardar: ${e?.message || e}`);
    }
  }

  function handleNameBlur() {
    if (emote && emoteName.trim() !== emote.name) {
      persistMetadata(emoteName.trim() || emote.name, tagList);
    }
  }

  async function calculateNewSize() {
    if (!emote) return;
    if (
      selectedSize === 'original' &&
      selectedFormat.toLowerCase() === (emote.format ?? '').toLowerCase()
    ) {
      newFileSize = emote.file_size ?? null;
      return;
    }

    isCalculatingSize = true;
    try {
      const canvas = await renderEmoteToCanvas(
        emote.path,
        targetDimensions.width,
        targetDimensions.height
      );
      canvas.toBlob(
        (blob) => {
          if (blob) newFileSize = blob.size;
          isCalculatingSize = false;
        },
        getMimeType(selectedFormat),
        0.9
      );
    } catch {
      isCalculatingSize = false;
    }
  }

  async function copyText(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      showToast(`¡${label} copiado!`);
    } catch {
      showToast('Error al copiar texto');
    }
  }

  async function copyImageToClipboard() {
    if (!emote) return;
    copyButtonText = 'Copiando...';

    try {
      const { width: targetW, height: targetH } = targetDimensions;
      await api.copyToClipboard(emote.id, emote.path, selectedFormat, targetW, targetH);

      copyCount += 1;
      interactionCount += 1;
      emote.copy_count = copyCount;
      emote.interaction_count = interactionCount;
      loadEmotes();

      copyButtonText = '✓ ¡Copiado!';
      showToast(`✓ Copiado como ${selectedFormat.toUpperCase()} (${targetW}x${targetH})`);
      setTimeout(() => {
        copyButtonText = 'Copy to Clipboard';
      }, 2200);
    } catch (err: any) {
      showToast(`Error: ${err?.message || err}`);
      copyButtonText = 'Copy to Clipboard';
    }
  }

  async function downloadResizedImage() {
    if (!emote) return;
    try {
      const { width: targetW, height: targetH } = targetDimensions;
      const canvas = await renderEmoteToCanvas(emote.path, targetW, targetH);
      const dataUrl = canvas.toDataURL(getMimeType(selectedFormat), 0.95);

      const a = document.createElement('a');
      a.href = dataUrl;
      a.download = `${emoteName || emote.name}_${targetW}x${targetH}.${selectedFormat}`;
      a.click();
      showToast('✓ Imagen descargada');
    } catch (e: any) {
      showToast(`Error al descargar: ${e?.message || e}`);
    }
  }

  async function handleDelete() {
    if (!emote) return;
    if (confirm(`¿Eliminar emote "${emote.name}" de forma permanente?`)) {
      try {
        await api.deleteEmote(emote.id);
        onClose();
      } catch (e: any) {
        showToast(`Error: ${e?.message || e}`);
      }
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if emote}
  <div class="modal-backdrop" on:click={handleBackdropClick} role="presentation">
    <div class="modal-card">
      <div class="modal-header">
        <h2 class="modal-title">INFO</h2>
        <button type="button" class="close-btn" on:click={onClose} aria-label="Cerrar modal">
          ✕
        </button>
      </div>

      <div class="top-row">
        <div class="preview-box">
          <img src={convertFileSrc(emote.path)} alt={emote.name} class="preview-img" />
        </div>

        <div class="fields-col">
          <div class="field-group">
            <span class="field-label">Name</span>
            <div class="input-with-action">
              <input
                type="text"
                bind:value={emoteName}
                on:blur={handleNameBlur}
                placeholder="Nombre del emote"
                class="field-input"
              />
              <button
                type="button"
                class="copy-icon-btn"
                title="Copiar nombre"
                on:click={() => copyText(emoteName, 'Nombre')}
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            </div>
          </div>

          <div class="field-group">
            <span class="field-label">Link</span>
            <div class="input-with-action">
              <input
                type="text"
                readonly
                value={emote.path}
                title={emote.path}
                class="field-input readonly"
              />
              <button
                type="button"
                class="copy-icon-btn"
                title="Copiar ruta"
                on:click={() => copyText(emote.path, 'Ruta')}
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="options-section">
        <div class="options-label">Tamaño / Dimensiones:</div>
        <div class="pills-row">
          {#each SIZE_PRESETS as preset}
            <button
              type="button"
              class="pill-btn {selectedSize === preset.value ? 'active' : ''}"
              on:click={() => (selectedSize = preset.value)}
            >
              {preset.label}
            </button>
          {/each}
        </div>
      </div>

      <div class="options-section">
        <div class="options-label">Formato a copiar (Default: WebP):</div>
        <div class="pills-row">
          {#each FORMAT_PRESETS as fmt}
            <button
              type="button"
              class="pill-btn {selectedFormat === fmt ? 'active' : ''}"
              on:click={() => (selectedFormat = fmt)}
            >
              {fmt.toUpperCase()}
            </button>
          {/each}
        </div>
      </div>

      <button type="button" class="copy-cta-btn" on:click={copyImageToClipboard}>
        {copyButtonText}
      </button>

      <div class="meta-section">
        <div class="meta-line">
          <span class="meta-key">Resolución:</span>
          <div class="meta-comparison">
            {#if selectedSize === 'original' || (emote.original_width === targetDimensions.width && emote.original_height === targetDimensions.height)}
              <span class="meta-value bold">{emote.original_width}×{emote.original_height} px</span>
              <span class="diff-badge gray">Original</span>
            {:else}
              <span class="meta-value muted">{emote.original_width}×{emote.original_height} px</span>
              <span class="arrow-sep">➜</span>
              <span class="meta-value bold highlight">{targetDimensions.width}×{targetDimensions.height} px</span>
              {#if resolutionDiffPercent !== null}
                <span class="diff-badge blue">{resolutionDiffPercent}% px</span>
              {/if}
            {/if}
          </div>
        </div>

        <div class="meta-line">
          <span class="meta-key">Peso:</span>
          <div class="meta-comparison">
            {#if isCalculatingSize}
              <span class="meta-value muted">Calculando...</span>
            {:else if newFileSize !== null && emote.file_size && emote.file_size !== newFileSize}
              <span class="meta-value muted">{formatFileSize(emote.file_size)}</span>
              <span class="arrow-sep">➜</span>
              <span class="meta-value bold highlight">{formatFileSize(newFileSize)}</span>
              {#if sizeDiffPercent !== null}
                <span class="diff-badge {sizeDiffPercent < 0 ? 'green' : 'amber'}">
                  {sizeDiffPercent < 0 ? `-${Math.abs(sizeDiffPercent)}%` : `+${sizeDiffPercent}%`}
                </span>
              {/if}
            {:else}
              <span class="meta-value bold">{formatFileSize(newFileSize ?? emote.file_size)}</span>
              {#if selectedSize === 'original'}
                <span class="diff-badge gray">Original</span>
              {/if}
            {/if}
          </div>
        </div>

        <div class="meta-line">
          <span class="meta-key">Type:</span>
          <div class="meta-comparison">
            {#if emote.format && selectedFormat.toUpperCase() !== emote.format.toUpperCase()}
              <span class="meta-value muted">{emote.format.toUpperCase()}</span>
              <span class="arrow-sep">➜</span>
              <span class="meta-value bold highlight">{selectedFormat.toUpperCase()}</span>
            {:else}
              <span class="meta-value bold">{selectedFormat.toUpperCase()}</span>
            {/if}
          </div>
        </div>

        <div class="meta-line">
          <span class="meta-key">Estadísticas:</span>
          <div class="meta-comparison">
            <span class="diff-badge blue">📋 {copyCount} {copyCount === 1 ? 'copia' : 'copias'}</span>
            <span class="diff-badge gray">👆 {interactionCount} {interactionCount === 1 ? 'interacción' : 'interacciones'}</span>
            {#if isHot}
              <span class="diff-badge amber">🔥 Top 20</span>
            {/if}
            {#if isNew}
              <span class="diff-badge green">🍃 Nuevo</span>
            {/if}
          </div>
        </div>

        <div class="meta-line">
          <span class="meta-key">Creado:</span>
          <div class="meta-comparison">
            <span class="meta-value">{new Date(emote.created_at).toLocaleString()}</span>
          </div>
        </div>
      </div>

      <TagManager
        {tagList}
        onChange={(newTags) => persistMetadata(emoteName, newTags)}
      />

      <div class="modal-footer">
        <button type="button" class="secondary-action-btn" on:click={downloadResizedImage}>
          💾 Descargar copia
        </button>
        <button type="button" class="danger-action-btn" on:click={handleDelete}>
          🗑️ Eliminar
        </button>
      </div>

      {#if copyFeedback}
        <div class="toast-popup">{copyFeedback}</div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 16px;
    animation: fadeIn 0.15s ease-out;
  }
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  .modal-card {
    background: #1e1e20;
    color: #f3f4f6;
    border-radius: 16px;
    width: 100%;
    max-width: 440px;
    padding: 24px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(255, 255, 255, 0.08);
    display: flex;
    flex-direction: column;
    gap: 16px;
    position: relative;
    max-height: 90vh;
    overflow-y: auto;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .modal-title {
    font-size: 18px;
    font-weight: 800;
    letter-spacing: 0.5px;
    margin: 0;
    color: #ffffff;
  }
  .close-btn {
    background: transparent;
    border: none;
    color: #facc15;
    font-size: 22px;
    font-weight: bold;
    cursor: pointer;
    line-height: 1;
    padding: 4px;
    transition: transform 0.15s ease, color 0.15s ease;
  }
  .close-btn:hover {
    transform: scale(1.15) rotate(90deg);
    color: #fef08a;
  }
  .top-row {
    display: flex;
    gap: 16px;
    align-items: stretch;
  }
  .preview-box {
    width: 140px;
    height: 140px;
    background: #2b2b2f;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px;
    flex-shrink: 0;
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
  }
  .preview-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.4));
  }
  .fields-col {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }
  .field-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    text-align: left;
  }
  .field-label {
    font-size: 13px;
    font-weight: 600;
    color: #9ca3af;
  }
  .input-with-action {
    display: flex;
    align-items: center;
    background: #2b2b2f;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    overflow: hidden;
    transition: border-color 0.15s;
  }
  .input-with-action:focus-within {
    border-color: #6366f1;
  }
  .field-input {
    background: transparent;
    border: none;
    color: #f3f4f6;
    padding: 8px 10px;
    font-size: 13px;
    flex: 1;
    min-width: 0;
    outline: none;
  }
  .field-input.readonly {
    color: #9ca3af;
    cursor: default;
  }
  .copy-icon-btn {
    background: transparent;
    border: none;
    color: #9ca3af;
    padding: 8px 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s ease, background 0.15s ease;
  }
  .copy-icon-btn:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.08);
  }
  .options-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    text-align: left;
  }
  .options-label {
    font-size: 12px;
    font-weight: 600;
    color: #9ca3af;
  }
  .pills-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .pill-btn {
    background: #2b2b2f;
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: #d1d5db;
    padding: 5px 10px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .pill-btn:hover {
    background: #374151;
    border-color: rgba(255, 255, 255, 0.2);
  }
  .pill-btn.active {
    background: #4f46e5;
    border-color: #6366f1;
    color: #ffffff;
    box-shadow: 0 0 10px rgba(99, 102, 241, 0.4);
  }
  .copy-cta-btn {
    background: #facc15;
    color: #111827;
    border: none;
    border-radius: 8px;
    padding: 12px 16px;
    font-size: 15px;
    font-weight: 700;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
    box-shadow: 0 4px 14px rgba(250, 204, 21, 0.3);
    margin-top: 4px;
  }
  .copy-cta-btn:hover {
    background: #eab308;
    box-shadow: 0 6px 18px rgba(250, 204, 21, 0.4);
  }
  .copy-cta-btn:active {
    transform: scale(0.98);
  }
  .meta-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
    text-align: left;
    font-size: 13px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    padding-top: 12px;
  }
  .meta-line {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 24px;
  }
  .meta-key {
    color: #9ca3af;
    min-width: 85px;
    font-size: 13px;
  }
  .meta-comparison {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .arrow-sep {
    color: #6b7280;
    font-size: 11px;
  }
  .meta-value {
    color: #e5e7eb;
  }
  .meta-value.bold {
    font-weight: 700;
  }
  .meta-value.muted {
    color: #9ca3af;
    text-decoration: line-through;
    opacity: 0.75;
    font-size: 12px;
  }
  .meta-value.highlight {
    color: #facc15;
  }
  .diff-badge {
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 11px;
    font-weight: 700;
    line-height: 1.4;
  }
  .diff-badge.green {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }
  .diff-badge.blue {
    background: rgba(99, 102, 241, 0.2);
    color: #818cf8;
  }
  .diff-badge.amber {
    background: rgba(245, 158, 11, 0.2);
    color: #fbbf24;
  }
  .diff-badge.gray {
    background: rgba(156, 163, 175, 0.15);
    color: #9ca3af;
  }
  .modal-footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin-top: 4px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    padding-top: 12px;
  }
  .secondary-action-btn {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #d1d5db;
    border-radius: 8px;
    padding: 8px 12px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .secondary-action-btn:hover {
    background: rgba(255, 255, 255, 0.06);
    color: #ffffff;
  }
  .danger-action-btn {
    background: transparent;
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #f87171;
    border-radius: 8px;
    padding: 8px 12px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .danger-action-btn:hover {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }
  .toast-popup {
    position: absolute;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: #10b981;
    color: #ffffff;
    font-size: 13px;
    font-weight: 600;
    padding: 8px 16px;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    animation: toastIn 0.2s ease-out;
  }
  @keyframes toastIn {
    from {
      opacity: 0;
      transform: translate(-50%, 10px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }
</style>

