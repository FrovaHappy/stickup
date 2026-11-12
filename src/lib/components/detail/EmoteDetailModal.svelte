<script lang="ts">
  import { onMount } from "svelte";
  import {
    Copy,
    Check,
    Download,
    Trash2,
    Plus,
    X,
    ArrowRight,
    Flame,
    Sparkles,
    MousePointerClick,
    AlertCircle,
    RotateCcw,
    Wand2,
  } from "@lucide/svelte";
  import type { Emote, GifLoopStatus } from "$lib/types/emote";
  import { api } from "$lib/services/api";
  import emotes, {
    loadEmotes,
    topHotRanks,
    isEmoteNewByDate,
  } from "$lib/stores/emotes";
  import {
    allSizePresets,
    customSizes,
    defaultResolution,
    addCustomSize,
    removeCustomSize,
  } from "$lib/stores/sizes";
  import { minimizeOnCopyEnabled } from "$lib/stores/systemSettings";
  import { recordCopiedEmote } from "$lib/stores/preferences";
  import { toast } from "$lib/stores/toast";
  import {
    SIZE_PRESETS,
    formatFileSize,
    getAvailableFormats,
    getMimeType,
    isAnimatedEmote,
    renderEmoteToCanvas,
    resolveDimensions,
    toPascalCase,
    sanitizeEmoteName,
  } from "$lib/utils/format";
  import {
    Badge,
    Button,
    Chip,
    Input,
    Modal,
  } from "$lib/components/system-ui";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import TagManager from "./TagManager.svelte";
  import DeleteEmoteModal from "../DeleteEmoteModal.svelte";
  import RemoveBgModal from "./RemoveBgModal.svelte";

  export let emote: Emote | null = null;
  export let onClose: () => void;

  let selectedSize = $defaultResolution || "original";
  let selectedFormat = "webp";

  let emoteName = "";
  let tagList: string[] = [];
  let copyButtonText = "Copy to Clipboard (Ctrl+C)";
  let copyCount = 0;
  let interactionCount = 0;
  let initializedEmoteId: number | null = null;

  let newFileSize: number | null = null;
  let isCalculatingSize = false;

  let showCustomSizeInput = false;
  let customWidth = 96;
  let customHeight = 96;
  let customSizeError = "";
  let nameError = "";
  let showDeleteConfirm = false;
  let showRemoveBgModal = false;

  let gifLoopStatus: GifLoopStatus | null = null;
  let isUpdatingLoop = false;
  let previewCacheBuster = 0;

  async function checkGifLoop() {
    if (!emote || !isAnimatedEmote(emote)) return;
    try {
      gifLoopStatus = await api.getEmoteGifLoop(emote.id);
    } catch (e) {
      console.error("Error al consultar bucle de GIF:", e);
    }
  }

  async function toggleGifLoop() {
    if (!emote || !isAnimatedEmote(emote) || isUpdatingLoop) return;
    isUpdatingLoop = true;
    try {
      const targetInfinite = !gifLoopStatus?.is_infinite;
      gifLoopStatus = await api.setEmoteGifLoop(emote.id, targetInfinite);
      previewCacheBuster += 1;
      if (targetInfinite) {
        toast.success("Bucle infinito activado para este GIF");
      } else {
        toast.info(
          "Bucle infinito desactivado (se reproducirá 1 vez y se frenará)",
        );
      }
    } catch (e) {
      console.error("Error al actualizar bucle de GIF:", e);
      toast.error("No se pudo actualizar el bucle del GIF");
    } finally {
      isUpdatingLoop = false;
    }
  }

  function handleAddCustomSize() {
    customSizeError = "";
    const preset = addCustomSize(customWidth, customHeight);
    if (!preset) {
      customSizeError = "Las dimensiones deben ser entre 8 y 4096 px.";
      return;
    }
    selectedSize = preset.value;
    showCustomSizeInput = false;
    toast.success(`Tamaño ${preset.label} guardado`);
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
    selectedSize = $defaultResolution || "original";
    emoteName = emote.name ?? "";
    nameError = "";
    copyCount = emote.copy_count ?? 0;
    interactionCount = emote.interaction_count ?? 0;
    tagList = (emote.tag ?? "")
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    if (isAnimatedEmote(emote)) {
      checkGifLoop();
    }
  }

  $: isAnimated = isAnimatedEmote(emote);
  $: availableFormats = getAvailableFormats(emote);
  $: if (emote && !availableFormats.includes(selectedFormat)) {
    selectedFormat = isAnimated ? "gif" : "webp";
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
    return Math.round(
      ((newFileSize - emote.file_size) / emote.file_size) * 100,
    );
  })();

  $: resolutionDiffPercent = (() => {
    if (!emote) return null;
    const origArea = emote.original_width * emote.original_height;
    const newArea = targetDimensions.width * targetDimensions.height;
    if (origArea <= 0) return null;
    return Math.round(((newArea - origArea) / origArea) * 100);
  })();

  function checkNameOccupied(candidate: string): boolean {
    if (!emote) return false;
    const currentId = emote.id;
    const clean = sanitizeEmoteName(candidate);
    if (!clean || clean.toLowerCase() === (emote.name ?? "").toLowerCase()) {
      return false;
    }
    return $emotes.some(
      (e) =>
        e.id !== currentId &&
        (e.name ?? "").toLowerCase() === clean.toLowerCase(),
    );
  }

  function handleNameInput() {
    if (checkNameOccupied(emoteName)) {
      nameError = "Este nombre ya está ocupado";
    } else {
      nameError = "";
    }
  }

  async function persistMetadata(newName: string, newTags: string[]) {
    if (!emote) return;
    const cleanName = sanitizeEmoteName(newName) || newName;
    const cleanTags = newTags.map((t) => toPascalCase(t)).filter(Boolean);
    tagList = cleanTags;
    const joinedTags = cleanTags.join(", ");

    try {
      const updatedPath = await api.updateTag(emote.id, cleanName, joinedTags);
      nameError = "";
      emote.name = cleanName;
      emote.tag = joinedTags;
      if (updatedPath) {
        emote.path = updatedPath;
      }
      await loadEmotes();
    } catch (e: any) {
      const msg = e?.message || String(e);
      if (msg.toLowerCase().includes("ocupado")) {
        nameError = "Este nombre ya está ocupado";
      } else {
        toast.error(`Error al guardar: ${msg}`);
      }
    }
  }

  function handleNameBlur() {
    if (!emote) return;
    const normalized = sanitizeEmoteName(emoteName) || emote.name;
    emoteName = normalized;
    if (checkNameOccupied(normalized)) {
      nameError = "Este nombre ya está ocupado";
      return;
    }
    nameError = "";
    if (normalized !== emote.name) {
      persistMetadata(normalized, tagList);
    }
  }

  async function calculateNewSize() {
    if (!emote) return;
    if (
      selectedSize === "original" &&
      selectedFormat.toLowerCase() === (emote.format ?? "").toLowerCase()
    ) {
      newFileSize = emote.file_size ?? null;
      return;
    }

    isCalculatingSize = true;
    try {
      const canvas = await renderEmoteToCanvas(
        emote.path,
        targetDimensions.width,
        targetDimensions.height,
      );
      canvas.toBlob(
        (blob) => {
          if (blob) newFileSize = blob.size;
          isCalculatingSize = false;
        },
        getMimeType(selectedFormat),
        0.9,
      );
    } catch {
      isCalculatingSize = false;
    }
  }

  async function copyText(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`${label} copiado al portapapeles`);
    } catch {
      toast.error(`Error al copiar ${label.toLowerCase()}`);
    }
  }

  async function copyImageToClipboard() {
    if (!emote) return;
    const isGif = isAnimatedEmote(emote) || selectedFormat === "gif";
    let resizingToastId: string | null = null;
    if (isGif) {
      resizingToastId = toast.info("Redimensionando...", 0);
      await new Promise((r) => setTimeout(r, 20));
    }
    copyButtonText = "Copiando...";

    try {
      const { width: targetW, height: targetH } = targetDimensions;
      await api.copyToClipboard(
        emote.id,
        emote.path,
        selectedFormat,
        targetW,
        targetH,
      );

      copyCount += 1;
      emote.copy_count = copyCount;
      recordCopiedEmote(emote);
      await loadEmotes();

      if (resizingToastId) {
        toast.dismiss(resizingToastId);
        resizingToastId = null;
      }
      copyButtonText = "✓ ¡Copiado!";
      toast.success(
        `¡Copiado como ${selectedFormat.toUpperCase()} (${targetW}x${targetH})!`,
      );
      if ($minimizeOnCopyEnabled) {
        await api.minimizeWindow();
      }
      setTimeout(() => {
        copyButtonText = "Copy to Clipboard (Ctrl+C)";
      }, 2200);
    } catch (err: any) {
      if (resizingToastId) {
        toast.dismiss(resizingToastId);
        resizingToastId = null;
      }
      toast.error(`Error al copiar: ${err?.message || err}`);
      copyButtonText = "Copy to Clipboard (Ctrl+C)";
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && (e.key === "c" || e.key === "C")) {
      const active = document.activeElement as HTMLElement | null;
      const isTextInput =
        active &&
        (active.tagName === "INPUT" ||
          active.tagName === "TEXTAREA" ||
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

      const a = document.createElement("a");
      a.href = dataUrl;
      const safeStem = toPascalCase(emoteName || emote.name) || "Emote";
      a.download = `${safeStem}_${targetW}x${targetH}.${selectedFormat}`;
      a.click();
      toast.success("Imagen descargada correctamente");
    } catch (e: any) {
      toast.error(`Error al descargar: ${e?.message || e}`);
    }
  }

  function handleDelete() {
    if (!emote) return;
    showDeleteConfirm = true;
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if emote}
  <Modal open={true} title="INFORMACIÓN DEL EMOTE" maxWidth="600px" {onClose}>
    <div class="top-row">
      <div class="preview-box">
        <img
          src={`${convertFileSrc(emote.path)}${previewCacheBuster ? `?t=${previewCacheBuster}` : ""}`}
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
          on:keydown={(e) => {
            if (e.detail?.key === "Enter" || (e as any).key === "Enter") {
              handleNameBlur();
            }
          }}
        >
          <Button
            slot="trailing"
            variant="icon"
            size="sm"
            title="Copiar nombre"
            on:click={() => copyText(emoteName, "Nombre")}
          >
            <Copy size={14} />
          </Button>
        </Input>
        {#if nameError}
          <div class="name-occupied-legend">
            <AlertCircle size={13} />
            <span>{nameError}</span>
          </div>
        {/if}

        <Input label="Link" readonly value={emote.path} title={emote.path}>
          <Button
            slot="trailing"
            variant="icon"
            size="sm"
            title="Copiar ruta"
            on:click={() => copyText(emote?.path ?? "", "Ruta")}
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
          {@const isCustom = preset.value.startsWith("custom_")}
          <Chip
            active={selectedSize === preset.value}
            removable={isCustom}
            on:click={() => (selectedSize = preset.value)}
            on:remove={() => {
              removeCustomSize(preset.value);
              if (selectedSize === preset.value) {
                selectedSize = "original";
              }
              toast.info(`Tamaño ${preset.label} eliminado`);
            }}
          >
            {preset.label}
          </Chip>
        {/each}

        <Chip
          variant="amber"
          active={showCustomSizeInput}
          on:click={() => (showCustomSizeInput = !showCustomSizeInput)}
        >
          {#if showCustomSizeInput}
            <X size={13} /> Cancelar
          {:else}
            <Plus size={13} /> Personalizado
          {/if}
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
        Formato a copiar (Default: {isAnimated ? "GIF" : "WebP"}):
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

    {#if isAnimated}
      <div class="options-section">
        <div class="options-label">Bucle de animación del GIF:</div>
        <div class="gif-loop-row">
          <div class="gif-loop-info">
            <span
              class="loop-badge"
              class:loop-infinite={gifLoopStatus?.is_infinite}
            >
              {gifLoopStatus?.is_infinite
                ? "Bucle Infinito (Continuo)"
                : "1 Reproducción (Se frena)"}
            </span>
            <span class="loop-desc">
              {gifLoopStatus?.is_infinite
                ? "El emote se anima sin detenerse en la app y al pegarlo en Discord/chats."
                : "El emote se reproduce 1 vez y se congela en el último frame en la app y al pegarlo."}
            </span>
          </div>
          <Button
            variant="secondary"
            size="sm"
            disabled={isUpdatingLoop || !gifLoopStatus}
            on:click={toggleGifLoop}
          >
            <RotateCcw size={14} />
            <span
              >{gifLoopStatus?.is_infinite
                ? "Desactivar bucle (Frenar)"
                : "Activar bucle infinito"}</span
            >
          </Button>
        </div>
      </div>
    {/if}

    <Button
      variant="primary"
      size="lg"
      fullWidth
      on:click={copyImageToClipboard}
    >
      <Copy size={16} />
      <span>{copyButtonText}</span>
    </Button>

    <TagManager
      {tagList}
      onChange={(newTags) => persistMetadata(emoteName, newTags)}
    />

    <div class="meta-section">
      <div class="meta-line">
        <span class="meta-key">Resolución:</span>
        <div class="meta-comparison">
          {#if selectedSize === "original" || (emote.original_width === targetDimensions.width && emote.original_height === targetDimensions.height)}
            <span class="meta-value bold">
              {emote.original_width}×{emote.original_height} px
            </span>
            <Badge variant="gray">Original</Badge>
          {:else}
            <span class="meta-value muted">
              {emote.original_width}×{emote.original_height} px
            </span>
            <ArrowRight size={12} class="arrow-sep" />
            <span class="meta-value bold highlight">
              {targetDimensions.width}×{targetDimensions.height} px
            </span>
            {#if resolutionDiffPercent !== null}
              <Badge variant="amber">{resolutionDiffPercent}% px</Badge>
            {/if}
          {/if}
        </div>
      </div>

      <div class="meta-line">
        <span class="meta-key">Formato:</span>
        <div class="meta-comparison">
          {#if emote.format && selectedFormat.toUpperCase() !== emote.format.toUpperCase()}
            <span class="meta-value muted">{emote.format.toUpperCase()}</span>
            <ArrowRight size={12} class="arrow-sep" />
            <span class="meta-value bold highlight"
              >{selectedFormat.toUpperCase()}</span
            >
          {:else}
            <span class="meta-value bold">{selectedFormat.toUpperCase()}</span>
          {/if}
        </div>
      </div>

      <div class="meta-line">
        <span class="meta-key">Peso:</span>
        <div class="meta-comparison">
          {#if isCalculatingSize}
            <span class="meta-value muted">Calculando...</span>
          {:else if newFileSize !== null && emote.file_size && emote.file_size !== newFileSize}
            <span class="meta-value muted"
              >{formatFileSize(emote.file_size)}</span
            >
            <ArrowRight size={12} class="arrow-sep" />
            <span class="meta-value bold highlight"
              >{formatFileSize(newFileSize)}</span
            >
            {#if sizeDiffPercent !== null}
              <Badge variant={sizeDiffPercent < 0 ? "green" : "amber"}>
                {sizeDiffPercent < 0
                  ? `-${Math.abs(sizeDiffPercent)}%`
                  : `+${sizeDiffPercent}%`}
              </Badge>
            {/if}
          {:else}
            <span class="meta-value bold">
              {formatFileSize(newFileSize ?? emote.file_size)}
            </span>
            {#if selectedSize === "original"}
              <Badge variant="gray">Original</Badge>
            {/if}
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

      <div class="meta-line full-span">
        <span class="meta-key">Estadísticas:</span>
        <div class="meta-comparison">
          <Badge variant="amber">
            <Copy size={11} />
            <span>{copyCount} {copyCount === 1 ? "copia" : "copias"}</span>
          </Badge>
          <Badge variant="gray">
            <MousePointerClick size={11} />
            <span
              >{interactionCount}
              {interactionCount === 1 ? "interacción" : "interacciones"}</span
            >
          </Badge>
          {#if isHot}
            <Badge variant="hot">
              <Flame size={11} />
              <span>Top #{hotRank}</span>
            </Badge>
          {/if}
          {#if isNew}
            <Badge variant="new">
              <Sparkles size={11} />
              <span>Nuevo</span>
            </Badge>
          {/if}
        </div>
      </div>
    </div>

    <svelte:fragment slot="footer">
      <Button
        variant="secondary"
        size="sm"
        on:click={() => (showRemoveBgModal = true)}
        title="Eliminar fondo de la imagen o GIF animado usando IA local"
      >
        <Wand2 size={14} /> Quitar fondo
      </Button>
      <Button variant="outline" size="sm" on:click={downloadResizedImage}>
        <Download size={14} /> Descargar copia
      </Button>
      <Button variant="danger" size="sm" on:click={handleDelete}>
        <Trash2 size={14} /> Eliminar
      </Button>
    </svelte:fragment>
  </Modal>
{/if}

{#if showDeleteConfirm && emote}
  <DeleteEmoteModal
    {emote}
    open={true}
    onClose={() => (showDeleteConfirm = false)}
    onDeleted={() => {
      showDeleteConfirm = false;
      onClose();
    }}
  />
{/if}

{#if showRemoveBgModal && emote}
  <RemoveBgModal
    {emote}
    open={true}
    onClose={() => (showRemoveBgModal = false)}
    onApplied={(updated) => {
      emote = updated;
      showRemoveBgModal = false;
      previewCacheBuster += 1;
    }}
  />
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
    background: var(--color-bg-input, rgba(0, 0, 0, 0.25));
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px;
    flex-shrink: 0;
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
    color: var(--color-text-muted, #9ca3af);
  }
  .pills-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .meta-section {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 6px 16px;
    text-align: left;
    font-size: 13px;
    border-top: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    padding-top: 12px;
  }
  .meta-line {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 24px;
  }
  .meta-line.full-span {
    grid-column: 1 / -1;
  }
  .meta-key {
    color: var(--color-text-muted, #9ca3af);
    min-width: 85px;
    font-size: 13px;
  }
  .meta-comparison {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  :global(.arrow-sep) {
    color: var(--color-text-dim, #71717a);
  }
  .meta-value {
    color: inherit;
  }
  .meta-value.bold {
    font-weight: 700;
  }
  .meta-value.muted {
    color: var(--color-text-dim, #71717a);
    text-decoration: line-through;
    opacity: 0.75;
    font-size: 12px;
  }
  .meta-value.highlight {
    color: var(--color-orange-primary, #f97316);
  }
  .custom-size-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--color-bg-input, rgba(0, 0, 0, 0.25));
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
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
    color: var(--color-text-muted, #9ca3af);
    font-weight: 500;
  }
  .size-num-input {
    width: 60px;
    background: var(--color-bg-input, rgba(0, 0, 0, 0.25));
    border: 1px solid var(--color-border-strong, rgba(255, 255, 255, 0.16));
    border-radius: 6px;
    padding: 4px 6px;
    font-size: 12px;
    color: inherit;
    text-align: center;
  }
  .size-num-input:focus {
    outline: none;
    border-color: var(--color-orange-primary, #f97316);
    box-shadow: 0 0 0 2px
      var(--color-orange-focus-ring, rgba(249, 115, 22, 0.25));
  }
  .size-cross {
    color: var(--color-text-dim, #71717a);
    font-weight: 700;
  }
  .custom-size-error {
    font-size: 11px;
    color: var(--color-warning-primary, #ea580c);
  }
  .name-occupied-legend {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--color-warning-primary, #ea580c);
    margin-top: -6px;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .gif-loop-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    background: var(--color-bg-input, rgba(0, 0, 0, 0.2));
    padding: 10px 12px;
    border-radius: 8px;
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
  }
  .gif-loop-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .loop-badge {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-dim, #a1a1aa);
  }
  .loop-badge.loop-infinite {
    color: var(--color-orange-primary, #f97316);
  }
  .loop-desc {
    font-size: 11px;
    color: var(--color-text-dim, #71717a);
    line-height: 1.3;
  }
</style>
