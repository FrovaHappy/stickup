<script lang="ts">
  import { onMount } from 'svelte';
  import type { Emote } from '$lib/types/emote';
  import { api } from '$lib/services/api';
  import {
    loadEmotes,
    topHotRanks,
    isEmoteNewByDate
  } from '$lib/stores/emotes';
  import {
    SIZE_PRESETS,
    formatFileSize,
    getAvailableFormats,
    getMimeType,
    isAnimatedEmote,
    renderEmoteToCanvas,
    resolveDimensions
  } from '$lib/utils/format';
  import { Alert, Badge, Button, Chip, Input, Modal } from '$lib/components/system-ui';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import TagManager from './TagManager.svelte';

  export let emote: Emote | null = null;
  export let onClose: () => void;

  let selectedSize = 'original';
  let selectedFormat = 'webp';

  let emoteName = '';
  let tagList: string[] = [];
  let copyFeedback = '';
  let copyButtonText = 'Copy to Clipboard';
  let copyCount = 0;
  let interactionCount = 0;
  let initializedEmoteId: number | null = null;

  let newFileSize: number | null = null;
  let isCalculatingSize = false;

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
    emoteName = emote.name ?? '';
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
</script>

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
          on:blur={handleNameBlur}
        >
          <Button
            slot="trailing"
            variant="icon"
            size="sm"
            title="Copiar nombre"
            on:click={() => copyText(emoteName, 'Nombre')}
          >
            <svg
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
          </Button>
        </Input>

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
            <svg
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
          </Button>
        </Input>
      </div>
    </div>

    <div class="options-section">
      <div class="options-label">Tamaño / Dimensiones:</div>
      <div class="pills-row">
        {#each SIZE_PRESETS as preset}
          <Chip
            active={selectedSize === preset.value}
            on:click={() => (selectedSize = preset.value)}
          >
            {preset.label}
          </Chip>
        {/each}
      </div>
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
      variant="accent"
      size="lg"
      fullWidth
      on:click={copyImageToClipboard}
    >
      {copyButtonText}
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
        💾 Descargar copia
      </Button>
      <Button variant="danger" size="sm" on:click={handleDelete}>
        🗑️ Eliminar
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
</style>
