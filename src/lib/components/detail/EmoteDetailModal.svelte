<script lang="ts">
  import { onMount } from 'svelte';
  import { Copy, Check, Download, Trash2 } from '@lucide/svelte';
  import type { Emote } from '$lib/types/emote';
  import { api } from '$lib/services/api';
  import emotes, {
    loadEmotes,
    topHotRanks,
    isEmoteNewByDate
  } from '$lib/stores/emotes';
  import {
    allSizePresets,
    customSizes,
    defaultResolution,
    addCustomSize,
    removeCustomSize
  } from '$lib/stores/sizes';
  import { minimizeOnCopyEnabled } from '$lib/stores/systemSettings';
  import { toast } from '$lib/stores/toast';
  import {
    SIZE_PRESETS,
    formatFileSize,
    getAvailableFormats,
    getMimeType,
    isAnimatedEmote,
    renderEmoteToCanvas,
    resolveDimensions,
    toPascalCase
  } from '$lib/utils/format';
  import { Alert, Badge, Button, Chip, Input, Modal } from '$lib/components/system-ui';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import TagManager from './TagManager.svelte';

  export let emote: Emote | null = null;
  export let onClose: () => void;

  let selectedSize = $defaultResolution || 'original';
  let selectedFormat = 'webp';

  let emoteName = '';
  let tagList: string[] = [];
  let copyFeedback = '';
  let copyButtonText = 'Copy to Clipboard (Ctrl+C)';
  let copyCount = 0;
  let interactionCount = 0;
  let initializedEmoteId: number | null = null;

  let newFileSize: number | null = null;
  let isCalculatingSize = false;

  let showCustomSizeInput = false;
  let customWidth = 96;
  let customHeight = 96;
  let customSizeError = '';
  let nameError = '';

  function handleAddCustomSize() {
    customSizeError = '';
    const preset = addCustomSize(customWidth, customHeight);
    if (!preset) {
      customSizeError = 'Las dimensiones deben ser entre 8 y 4096 px.';
      return;
    }
    selectedSize = preset.value;
    showCustomSizeInput = false;
    toast.success(`✓ Tamaño ${preset.label} guardado`);
  }

  onMount(async () => {
    if (emote) {
      copyCount = emote.copy_count ?? 0;
      interactionCount = (emote.interaction_count ?? 0) + 1;
      emote.interaction_count = interactionCount;
      try {
        await api.recordInteraction(emote.id);
        await loadEmotes();
      } catch {
        // Ignorar errores silenciosos de telemetría local
      }
    }
  });

  $: if (emote && emote.id !== initializedEmoteId) {
    initializedEmoteId = emote.id;
    selectedSize = $defaultResolution || 'original';
    emoteName = emote.name ?? '';
    nameError = '';
    copyCount = emote.copy_count ?? 0;
    interactionCount = emote.interaction_count ?? 0;
    tagList = (emote.tag ?? '')
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);
  }

  $: isAnimated = isAnimatedEmote(emote);
  $: availableFormats = getAvailableFormats(emote);
  $: if (emote && !availableFormats.includes(selectedFormat)) {
    selectedFormat = isAnimated ? 'gif' : 'webp';
  }

  $: hotRank = emote ? ($topHotRanks.get(emote.id) ?? null) : null;
  $: isHot = hotRank !== null;
  $: isNew = emote ? isEmoteNewByDate(emote.created_at) : false;

  $: targetDimensions = resolveDimensions(emote, selectedSize, $customSizes);

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

  function checkNameOccupied(candidate: string): boolean {
    if (!emote) return false;
    const clean = toPascalCase(candidate.trim());
    if (!clean || clean.toLowerCase() === (emote.name ?? '').toLowerCase()) {
      return false;
    }
    return $emotes.some(
      (e) => e.id !== emote.id && (e.name ?? '').toLowerCase() === clean.toLowerCase()
    );
  }

  function handleNameInput() {
    if (checkNameOccupied(emoteName)) {
      nameError = 'Este nombre ya está ocupado';
    } else {
      nameError = '';
    }
  }

  async function persistMetadata(newName: string, newTags: string[]) {
    if (!emote) return;
    const cleanName = toPascalCase(newName) || newName;
    const cleanTags = newTags.map((t) => toPascalCase(t)).filter(Boolean);
    tagList = cleanTags;
    const joinedTags = cleanTags.join(', ');

    try {
      const updatedPath = await api.updateTag(emote.id, cleanName, joinedTags);
      nameError = '';
      emote.name = cleanName;
      emote.tag = joinedTags;
      if (updatedPath) {
        emote.path = updatedPath;
      }
      await loadEmotes();
    } catch (e: any) {
      const msg = e?.message || String(e);
      if (msg.toLowerCase().includes('ocupado')) {
        nameError = 'Este nombre ya está ocupado';
      } else {
        showToast(`Error al guardar: ${msg}`);
      }
    }
  }

  function handleNameBlur() {
    if (!emote) return;
    const normalized = toPascalCase(emoteName.trim()) || emote.name;
    emoteName = normalized;
    if (checkNameOccupied(normalized)) {
      nameError = 'Este nombre ya está ocupado';
      return;
    }
    nameError = '';
    if (normalized !== emote.name) {
      persistMetadata(normalized, tagList);
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
      await api.copyToClipboard(
        emote.id,
        emote.path,
        selectedFormat,
        targetW,
        targetH
      );

      copyCount += 1;
      emote.copy_count = copyCount;
      await loadEmotes();

      copyButtonText = '✓ ¡Copiado!';
      showToast(`✓ Copiado como ${selectedFormat.toUpperCase()} (${targetW}x${targetH})`);
      if ($minimizeOnCopyEnabled) {
        await api.minimizeWindow();
      }
      setTimeout(() => {
        copyButtonText = 'Copy to Clipboard (Ctrl+C)';
      }, 2200);
    } catch (err: any) {
      showToast(`Error: ${err?.message || err}`);
      copyButtonText = 'Copy to Clipboard (Ctrl+C)';
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && (e.key === 'c' || e.key === 'C')) {
      const active = document.activeElement as HTMLElement | null;
      const isTextInput =
        active &&
        (active.tagName === 'INPUT' ||
          active.tagName === 'TEXTAREA' ||
          active.isContentEditable);
      const selection = window.getSelection()?.toString();
      if (isTextInput && selection && selection.length > 0) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      copyImageToClipboard();
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
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if emote}
  <Modal open={true} title="INFO" maxWidth="440px" {onClose}>
    <div class="top-row">
      <div class="preview-box">
        <img
          src={convertFileSrc(emote.path)}
          alt={emote.name}
          class="preview-img"
        />
      </div>

      <div class="fields-col">
        <Input
          label="Name"
          bind:value={emoteName}
          placeholder="Nombre del emote"
          on:input={handleNameInput}
          on:blur={handleNameBlur}
        >
          <Button
            slot="trailing"
            variant="icon"
            size="sm"
            title="Copiar nombre"
            on:click={() => copyText(emoteName, 'Nombre')}
          >
            <Copy size={14} />
          </Button>
        </Input>
        {#if nameError}
          <div class="name-occupied-legend">
            <span>⚠️ {nameError}</span>
          </div>
        {/if}

        <Input
          label="Link"
          readonly
          value={emote.path}
          title={emote.path}
        >
          <Button
            slot="trailing"
            variant="icon"
            size="sm"
            title="Copiar ruta"
            on:click={() => copyText(emote?.path ?? '', 'Ruta')}
          >
            <Copy size={14} />
          </Button>
        </Input>
      </div>
    </div>

    <div class="options-section">
      <div class="options-label">Tamaño / Dimensiones:</div>
      <div class="pills-row">
        {#each $allSizePresets as preset}
          {@const isCustom = preset.value.startsWith('custom_')}
          <Chip
            active={selectedSize === preset.value}
            removable={isCustom}
            on:click={() => (selectedSize = preset.value)}
            on:remove={() => {
              removeCustomSize(preset.value);
              if (selectedSize === preset.value) {
                selectedSize = 'original';
              }
              toast.info(`Tamaño ${preset.label} eliminado`);
            }}
          >
            {preset.label}
          </Chip>
        {/each}

        <Chip
          variant="sky"
          active={showCustomSizeInput}
          on:click={() => (showCustomSizeInput = !showCustomSizeInput)}
        >
          {showCustomSizeInput ? '✕ Cancelar' : '➕ Personalizado'}
        </Chip>
      </div>

      {#if showCustomSizeInput}
        <div class="custom-size-form">
          <div class="custom-size-inputs">
            <div class="custom-input-group">
              <span class="custom-input-label">Ancho:</span>
              <input
                type="number"
                min="8"
                max="4096"
                bind:value={customWidth}
                class="size-num-input"
              />
            </div>
            <span class="size-cross">×</span>
            <div class="custom-input-group">
              <span class="custom-input-label">Alto:</span>
              <input
                type="number"
                min="8"
                max="4096"
                bind:value={customHeight}
                class="size-num-input"
              />
            </div>
            <Button variant="primary" size="sm" on:click={handleAddCustomSize}>
              <Check size={14} /> Guardar
            </Button>
          </div>
          {#if customSizeError}
            <span class="custom-size-error">{customSizeError}</span>
          {/if}
        </div>
      {/if}
    </div>

    <div class="options-section">
      <div class="options-label">
        Formato a copiar (Default: {isAnimated ? 'GIF' : 'WebP'}):
      </div>
      <div class="pills-row">
        {#each availableFormats as fmt}
          <Chip
            active={selectedFormat === fmt}
            on:click={() => (selectedFormat = fmt)}
          >
            {fmt.toUpperCase()}
          </Chip>
        {/each}
      </div>
    </div>

    <Button
      variant="primary"
      size="lg"
      fullWidth
      on:click={copyImageToClipboard}
    >
      <Copy size={16} />
      <span>{copyButtonText}</span>
    </Button>

    <div class="meta-section">
      <div class="meta-line">
        <span class="meta-key">Resolución:</span>
        <div class="meta-comparison">
          {#if selectedSize === 'original' || (emote.original_width === targetDimensions.width && emote.original_height === targetDimensions.height)}
            <span class="meta-value bold">
              {emote.original_width}×{emote.original_height} px
            </span>
            <Badge variant="gray">Original</Badge>
          {:else}
            <span class="meta-value muted">
              {emote.original_width}×{emote.original_height} px
            </span>
            <span class="arrow-sep">➜</span>
            <span class="meta-value bold highlight">
              {targetDimensions.width}×{targetDimensions.height} px
            </span>
            {#if resolutionDiffPercent !== null}
              <Badge variant="blue">{resolutionDiffPercent}% px</Badge>
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
              <Badge variant={sizeDiffPercent < 0 ? 'green' : 'amber'}>
                {sizeDiffPercent < 0
                  ? `-${Math.abs(sizeDiffPercent)}%`
                  : `+${sizeDiffPercent}%`}
              </Badge>
            {/if}
          {:else}
            <span class="meta-value bold">
              {formatFileSize(newFileSize ?? emote.file_size)}
            </span>
            {#if selectedSize === 'original'}
              <Badge variant="gray">Original</Badge>
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
          <Badge variant="blue">
            📋 {copyCount} {copyCount === 1 ? 'copia' : 'copias'}
          </Badge>
          <Badge variant="gray">
            👆 {interactionCount} {interactionCount === 1 ? 'interacción' : 'interacciones'}
          </Badge>
          {#if isHot}
            <Badge variant="amber">🔥 Top #{hotRank}</Badge>
          {/if}
          {#if isNew}
            <Badge variant="green">🍃 Nuevo</Badge>
          {/if}
        </div>
      </div>

      <div class="meta-line">
        <span class="meta-key">Creado:</span>
        <div class="meta-comparison">
          <span class="meta-value">
            {new Date(emote.created_at).toLocaleString()}
          </span>
        </div>
      </div>
    </div>

    <TagManager
      {tagList}
      onChange={(newTags) => persistMetadata(emoteName, newTags)}
    />

    <svelte:fragment slot="footer">
      <Button variant="outline" size="sm" on:click={downloadResizedImage}>
        <Download size={14} /> Descargar copia
      </Button>
      <Button variant="danger" size="sm" on:click={handleDelete}>
        <Trash2 size={14} /> Eliminar
      </Button>
    </svelte:fragment>

    {#if copyFeedback}
      <Alert variant="toast">{copyFeedback}</Alert>
    {/if}
  </Modal>
{/if}

<style>
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
    color: inherit;
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
  .custom-size-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: rgba(0, 0, 0, 0.2);
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    margin-top: 4px;
  }
  .custom-size-inputs {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .custom-input-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .custom-input-label {
    font-size: 11px;
    color: #9ca3af;
    font-weight: 500;
  }
  .size-num-input {
    width: 60px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 6px;
    padding: 4px 6px;
    font-size: 12px;
    color: inherit;
    text-align: center;
  }
  .size-num-input:focus {
    outline: none;
    border-color: #6366f1;
  }
  .size-cross {
    color: #9ca3af;
    font-weight: 700;
  }
  .custom-size-error {
    font-size: 11px;
    color: #f87171;
  }
  .name-occupied-legend {
    font-size: 11.5px;
    font-weight: 600;
    color: #f87171;
    margin-top: -6px;
    display: flex;
    align-items: center;
    gap: 4px;
  }
</style>
