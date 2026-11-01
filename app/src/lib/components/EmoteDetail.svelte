<script lang="ts">
  import type { Emote } from '../stores/emotes';
  import { allTags } from '../stores/emotes';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';

  export let emote: Emote | null = null;
  export let onClose: () => void;

  interface Preset {
    label: string;
    value: string;
    width?: number;
    height?: number;
  }

  // Size presets
  const sizePresets: Preset[] = [
    { label: 'Original', value: 'original' },
    { label: '25x25', width: 25, height: 25, value: '25' },
    { label: '32x32', width: 32, height: 32, value: '32' },
    { label: '48x48', width: 48, height: 48, value: '48' },
    { label: '64x64', width: 64, height: 64, value: '64' },
    { label: '112x112', width: 112, height: 112, value: '112' },
    { label: '128x128', width: 128, height: 128, value: '128' }
  ];

  // Format presets (default: webp as requested)
  const formatPresets = ['webp', 'png', 'gif', 'jpg'];

  let selectedSize = '48';
  let selectedFormat = 'webp';

  let emoteName = '';
  let emoteTag = '';
  let tagList: string[] = [];
  let tagInputText = '';
  let showSuggestions = false;
  let copyFeedback = '';
  let copyButtonText = 'Copy to Clipboard';
  let isSavingMeta = false;

  $: if (emote) {
    emoteName = emote.name ?? '';
    emoteTag = emote.tag ?? '';
    tagList = (emote.tag ?? '')
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);
  }

  // Tags sugeridos del catálogo global que aún no tenga este emote
  $: suggestedTags = ($allTags || [])
    .filter((t) => !tagList.some((existing) => existing.toLowerCase() === t.toLowerCase()))
    .filter((t) => {
      if (!tagInputText.trim()) return true;
      return t.toLowerCase().includes(tagInputText.trim().toLowerCase());
    });

  async function persistTags(newTags: string[]) {
    tagList = newTags;
    emoteTag = newTags.join(', ');
    if (emote) {
      emote.tag = emoteTag;
      try {
        await invoke('update_tag', { id: emote.id, name: emoteName, tag: emoteTag });
      } catch (e: any) {
        showToast(`Error al guardar tag: ${e?.message || e}`);
      }
    }
  }

  function addTag(tagName: string) {
    const clean = tagName.trim();
    if (!clean) return;
    if (tagList.some((t) => t.toLowerCase() === clean.toLowerCase())) {
      tagInputText = '';
      return;
    }
    const updated = [...tagList, clean];
    persistTags(updated);
    tagInputText = '';
  }

  function removeTag(tagToRemove: string) {
    const updated = tagList.filter((t) => t !== tagToRemove);
    persistTags(updated);
  }

  function handleTagInputKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (tagInputText.trim()) {
        addTag(tagInputText);
      }
    } else if (e.key === 'Backspace' && !tagInputText && tagList.length > 0) {
      removeTag(tagList[tagList.length - 1]);
    } else if (e.key === 'Escape') {
      showSuggestions = false;
    }
  }

  function formatFileSize(bytes?: number | null): string {
    if (!bytes || bytes <= 0) return 'Desconocido';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  let newFileSize: number | null = null;
  let isCalculatingSize = false;

  function getTargetDimensions(): { width: number; height: number } {
    if (!emote) return { width: 48, height: 48 };
    if (selectedSize === 'original') {
      return { width: emote.original_width, height: emote.original_height };
    }
    const preset = sizePresets.find((p) => p.value === selectedSize);
    if (preset && typeof preset.width === 'number') {
      return { width: preset.width, height: preset.height ?? preset.width };
    }
    return { width: emote.original_width, height: emote.original_height };
  }

  $: targetDimensions = getTargetDimensions();

  async function calculateNewSize() {
    if (!emote) return;
    if (selectedSize === 'original' && selectedFormat.toLowerCase() === (emote.format ?? '').toLowerCase()) {
      newFileSize = emote.file_size ?? null;
      return;
    }

    isCalculatingSize = true;
    try {
      const img = new Image();
      img.crossOrigin = 'anonymous';
      img.src = convertFileSrc(emote.path);

      await new Promise<void>((resolve, reject) => {
        if (img.complete) resolve();
        else {
          img.onload = () => resolve();
          img.onerror = () => reject();
        }
      });

      const { width: targetW, height: targetH } = getTargetDimensions();
      const canvas = document.createElement('canvas');
      canvas.width = targetW;
      canvas.height = targetH;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;
      ctx.drawImage(img, 0, 0, targetW, targetH);

      const mime =
        selectedFormat === 'webp'
          ? 'image/webp'
          : selectedFormat === 'jpg'
          ? 'image/jpeg'
          : 'image/png';

      canvas.toBlob((blob) => {
        if (blob) {
          newFileSize = blob.size;
        }
        isCalculatingSize = false;
      }, mime, 0.9);
    } catch {
      isCalculatingSize = false;
    }
  }

  $: if (emote && (selectedSize || selectedFormat)) {
    calculateNewSize();
  }

  $: sizeDiffPercent = (() => {
    if (!emote?.file_size || !newFileSize || emote.file_size <= 0) return null;
    const diff = ((newFileSize - emote.file_size) / emote.file_size) * 100;
    return Math.round(diff);
  })();

  $: resolutionDiffPercent = (() => {
    if (!emote) return null;
    const { width: targetW, height: targetH } = getTargetDimensions();
    const origArea = emote.original_width * emote.original_height;
    const newArea = targetW * targetH;
    if (origArea <= 0) return null;
    const diff = ((newArea - origArea) / origArea) * 100;
    return Math.round(diff);
  })();

  function showToast(msg: string) {
    copyFeedback = msg;
    setTimeout(() => {
      copyFeedback = '';
    }, 2500);
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
      const { width: targetW, height: targetH } = getTargetDimensions();

      // Invocar el comando nativo de Tauri que coloca el archivo y los bytes con su formato exacto en el portapapeles
      await invoke('copy_to_clipboard', {
        path: emote.path,
        format: selectedFormat,
        targetWidth: targetW,
        targetHeight: targetH
      });

      copyButtonText = '✓ ¡Copiado!';
      showToast(`✓ Copiado como ${selectedFormat.toUpperCase()} (${targetW}x${targetH})`);
      setTimeout(() => {
        copyButtonText = 'Copy to Clipboard';
      }, 2200);
    } catch (err: any) {
      console.warn('Fallback al portapapeles del navegador tras error nativo:', err);
      // Fallback web si el backend nativo no estuviera disponible
      try {
        const img = new Image();
        img.crossOrigin = 'anonymous';
        img.src = convertFileSrc(emote.path);

        await new Promise<void>((resolve, reject) => {
          if (img.complete) resolve();
          else {
            img.onload = () => resolve();
            img.onerror = () => reject(new Error('No se pudo cargar la imagen'));
          }
        });

        const { width: targetW, height: targetH } = getTargetDimensions();
        const canvas = document.createElement('canvas');
        canvas.width = targetW;
        canvas.height = targetH;
        const ctx = canvas.getContext('2d');
        if (!ctx) throw new Error('No se pudo inicializar canvas');
        ctx.drawImage(img, 0, 0, targetW, targetH);

        canvas.toBlob(async (blob) => {
          if (blob) {
            await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
            copyButtonText = '✓ ¡Copiado!';
            showToast(`✓ Imagen copiada (${targetW}x${targetH} PNG fallback)`);
          }
          setTimeout(() => {
            copyButtonText = 'Copy to Clipboard';
          }, 2200);
        }, 'image/png');
      } catch (fallbackErr: any) {
        console.error('Error al copiar al portapapeles:', fallbackErr);
        showToast(`Error: ${fallbackErr?.message || fallbackErr}`);
        copyButtonText = 'Copy to Clipboard';
      }
    }
  }

  function downloadResizedImage() {
    if (!emote) return;
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.src = convertFileSrc(emote.path);

    img.onload = () => {
      const { width: targetW, height: targetH } = getTargetDimensions();

      const canvas = document.createElement('canvas');
      canvas.width = targetW;
      canvas.height = targetH;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;
      ctx.drawImage(img, 0, 0, targetW, targetH);

      const mime =
        selectedFormat === 'webp'
          ? 'image/webp'
          : selectedFormat === 'jpg'
          ? 'image/jpeg'
          : 'image/png';
      const dataUrl = canvas.toDataURL(mime, 0.95);

      const a = document.createElement('a');
      a.href = dataUrl;
      a.download = `${emoteName || emote.name}_${targetW}x${targetH}.${selectedFormat}`;
      a.click();
      showToast('✓ Imagen descargada');
    };
  }

  async function saveMetadata() {
    if (!emote || isSavingMeta) return;
    isSavingMeta = true;
    try {
      await invoke('update_tag', { id: emote.id, name: emoteName, tag: emoteTag });
      emote.name = emoteName;
      emote.tag = emoteTag;
      showToast('✓ Cambios guardados');
    } catch (e: any) {
      showToast(`Error: ${e?.message || e}`);
    } finally {
      isSavingMeta = false;
    }
  }

  async function handleDelete() {
    if (!emote) return;
    if (confirm(`¿Eliminar emote "${emote.name}" de forma permanente?`)) {
      try {
        await invoke('delete_emote', { id: emote.id });
        onClose && onClose();
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
  <!-- Backdrop -->
  <div class="modal-backdrop" on:click={handleBackdropClick} role="presentation">
    <div class="modal-card">
      <!-- Header -->
      <div class="modal-header">
        <h2 class="modal-title">INFO</h2>
        <button
          type="button"
          class="close-btn"
          on:click={onClose}
          aria-label="Cerrar modal"
        >
          ✕
        </button>
      </div>

      <!-- Top section: Preview + Name & Link -->
      <div class="top-row">
        <!-- Image Preview Card -->
        <div class="preview-box">
          <img
            src={convertFileSrc(emote.path)}
            alt={emote.name}
            class="preview-img"
          />
        </div>

        <!-- Name & Link inputs with copy buttons -->
        <div class="fields-col">
          <div class="field-group">
            <span class="field-label">Name</span>
            <div class="input-with-action">
              <input
                type="text"
                bind:value={emoteName}
                placeholder="Nombre del emote"
                class="field-input"
              />
              <button
                type="button"
                class="copy-icon-btn"
                title="Copiar nombre"
                on:click={() => copyText(emoteName, 'Nombre')}
              >
                <!-- Copy icon -->
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
                <!-- Copy icon -->
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Size Selector -->
      <div class="options-section">
        <div class="options-label">Tamaño / Dimensiones:</div>
        <div class="pills-row">
          {#each sizePresets as preset}
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

      <!-- Format Selector -->
      <div class="options-section">
        <div class="options-label">Formato a copiar (Default: WebP):</div>
        <div class="pills-row">
          {#each formatPresets as fmt}
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

      <!-- Main Action: Yellow CTA Copy Button -->
      <button
        type="button"
        class="copy-cta-btn"
        on:click={copyImageToClipboard}
      >
        {copyButtonText}
      </button>

      <!-- Metadata comparison section -->
      <div class="meta-section">
        <!-- Resolution comparison -->
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

        <!-- File size comparison -->
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

        <!-- Format comparison -->
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
      </div>

      <!-- Tags Section: Modern Tag Manager with Global Autocomplete -->
      <div class="tags-section">
        <div class="section-header-row">
          <h3 class="section-title">TAGS</h3>
          <span class="tags-counter">{tagList.length} {tagList.length === 1 ? 'tag' : 'tags'}</span>
        </div>

        <div class="tag-manager-wrapper">
          <!-- Tag input box with embedded pills like the reference -->
          <div
            class="tag-box-container"
            on:click={() => {
              const el = document.getElementById('tag-inline-input');
              if (el) el.focus();
            }}
            role="presentation"
          >
            {#each tagList as t}
              <span class="tag-badge">
                <span class="tag-badge-text">{t}</span>
                <button
                  type="button"
                  class="tag-badge-remove"
                  title="Eliminar tag"
                  on:click|stopPropagation={() => removeTag(t)}
                >
                  ✕
                </button>
              </span>
            {/each}

            <input
              id="tag-inline-input"
              type="text"
              class="tag-inline-input"
              placeholder={tagList.length === 0 ? "Escribe o busca un tag..." : "Agregar tag..."}
              bind:value={tagInputText}
              on:focus={() => (showSuggestions = true)}
              on:keydown={handleTagInputKeydown}
            />
          </div>

          <!-- Suggestions Dropdown (Global Tags) -->
          {#if showSuggestions && suggestedTags.length > 0}
            <!-- Backdrop to close dropdown on outer click -->
            <div
              class="dropdown-backdrop"
              on:click|stopPropagation={() => (showSuggestions = false)}
              role="presentation"
            ></div>

            <div class="tag-dropdown">
              <div class="tag-dropdown-header">
                <span class="dropdown-header-title">🏷️ Tags globales existentes</span>
                <span class="dropdown-header-count">{suggestedTags.length} disponibles</span>
              </div>
              <div class="tag-dropdown-list">
                {#each suggestedTags as st}
                  <button
                    type="button"
                    class="tag-dropdown-item"
                    on:click|stopPropagation={() => {
                      addTag(st);
                      // Mantener foco en el input
                      const el = document.getElementById('tag-inline-input');
                      if (el) el.focus();
                    }}
                  >
                    <span class="tag-item-name">{st}</span>
                    <span class="tag-item-action">+ Agregar</span>
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Footer extra actions -->
      <div class="modal-footer">
        <button
          type="button"
          class="secondary-action-btn"
          on:click={downloadResizedImage}
        >
          💾 Descargar copia
        </button>
        <button
          type="button"
          class="danger-action-btn"
          on:click={handleDelete}
        >
          🗑️ Eliminar
        </button>
      </div>

      <!-- Floating toast message -->
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
    from { opacity: 0; }
    to { opacity: 1; }
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

  /* Header */
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

  /* Top Row */
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

  /* Options Section (Size & Format) */
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

  /* Main Action Button (Yellow CTA) */
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

  /* Metadata */
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

  /* Tags */
  .tags-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    padding-top: 12px;
    position: relative;
  }
  .section-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .section-title {
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.5px;
    color: #9ca3af;
    margin: 0;
  }
  .tags-counter {
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
  }
  .tag-manager-wrapper {
    position: relative;
    width: 100%;
  }
  .tag-box-container {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    background: #2b2b2f;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 6px 10px;
    min-height: 42px;
    cursor: text;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }
  .tag-box-container:focus-within {
    border-color: #6366f1;
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
  }
  .tag-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: rgba(99, 102, 241, 0.18);
    border: 1px solid rgba(99, 102, 241, 0.4);
    color: #c7d2fe;
    font-size: 12px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 6px;
    animation: tagAppear 0.12s ease-out;
  }
  @keyframes tagAppear {
    from {
      opacity: 0;
      transform: scale(0.92);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
  .tag-badge-text {
    line-height: 1.2;
    white-space: nowrap;
  }
  .tag-badge-remove {
    background: transparent;
    border: none;
    color: #a5b4fc;
    font-size: 11px;
    line-height: 1;
    padding: 0 1px;
    cursor: pointer;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
  }
  .tag-badge-remove:hover {
    background: rgba(239, 68, 68, 0.3);
    color: #fca5a5;
  }
  .tag-inline-input {
    flex: 1;
    min-width: 100px;
    background: transparent;
    border: none;
    color: #f3f4f6;
    font-size: 13px;
    outline: none;
    padding: 4px 2px;
  }
  .tag-inline-input::placeholder {
    color: #6b7280;
  }

  /* Dropdown & Backdrop */
  .dropdown-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1050;
  }
  .tag-dropdown {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    background: #232326;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.05);
    z-index: 1100;
    max-height: 200px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: fadeIn 0.12s ease-out;
  }
  .tag-dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    font-size: 11px;
    color: #9ca3af;
    font-weight: 600;
  }
  .dropdown-header-count {
    color: #6b7280;
    font-size: 10px;
  }
  .tag-dropdown-list {
    overflow-y: auto;
    max-height: 160px;
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .tag-dropdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 10px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #e5e7eb;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.12s ease;
  }
  .tag-dropdown-item:hover {
    background: rgba(99, 102, 241, 0.18);
    color: #ffffff;
  }
  .tag-item-name {
    font-weight: 500;
  }
  .tag-item-action {
    font-size: 11px;
    color: #818cf8;
    font-weight: 600;
  }

  /* Footer Actions */
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

  /* Toast */
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
