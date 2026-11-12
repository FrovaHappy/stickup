<script lang="ts">
  import { createEventDispatcher, onDestroy } from "svelte";
  import { X, ZoomIn, ZoomOut, Copy, Check, Sparkles } from "@lucide/svelte";
  import type { Emote } from "$lib/types/emote";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { api } from "$lib/services/api";
  import { toast } from "$lib/stores/toast";
  import { recordCopiedEmote } from "$lib/stores/preferences";
  import { formatFileSize, isAnimatedEmote } from "$lib/utils/format";
  import { Badge, Button } from "$lib/components/system-ui";

  export let emote: Emote;
  export let open = false;
  export let previewCacheBuster = 0;
  export let onClose: () => void;

  const dispatch = createEventDispatcher<{ close: void }>();

  let isActualSize = false;
  let isPixelated = (emote?.original_width ?? 0) <= 128 && (emote?.original_height ?? 0) <= 128;
  let copied = false;
  let copyTimeout: ReturnType<typeof setTimeout> | null = null;

  $: isAnimated = isAnimatedEmote(emote);
  $: imgSrc = emote
    ? `${convertFileSrc(emote.path)}${previewCacheBuster ? `?t=${previewCacheBuster}` : ""}`
    : "";

  function closeLightbox() {
    open = false;
    onClose?.();
    dispatch("close");
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeLightbox();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "Escape") {
      e.stopPropagation();
      e.preventDefault();
      closeLightbox();
    } else if (e.key === "z" || e.key === "Z") {
      isActualSize = !isActualSize;
    }
  }

  async function handleCopy() {
    if (!emote) return;
    try {
      await api.copyToClipboard(
        emote.id,
        emote.path,
        isAnimated ? "gif" : (emote.format || "png"),
        emote.original_width,
        emote.original_height
      );
      recordCopiedEmote(emote);
      copied = true;
      toast.success("¡Emote copiado al portapapeles!");
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (err: any) {
      toast.error(`Error al copiar: ${err?.message || err}`);
    }
  }

  onDestroy(() => {
    if (copyTimeout) clearTimeout(copyTimeout);
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open && emote}
  <div
    class="lightbox-backdrop"
    on:click={handleBackdropClick}
    role="presentation"
  >
    <div
      class="lightbox-card"
      role="dialog"
      aria-modal="true"
      aria-label={`Vista expandida de ${emote.name}`}
    >
      <div class="lightbox-header">
        <div class="lightbox-meta">
          <h2 class="lightbox-title" title={emote.name}>{emote.name}</h2>
          <div class="lightbox-badges">
            <Badge variant="gray">{emote.original_width} × {emote.original_height} px</Badge>
            {#if emote.format}
              <Badge variant="amber">{emote.format.toUpperCase()}</Badge>
            {/if}
            {#if emote.file_size}
              <Badge variant="gray">{formatFileSize(emote.file_size)}</Badge>
            {/if}
            {#if isAnimated}
              <Badge variant="new">Animado</Badge>
            {/if}
          </div>
        </div>

        <div class="lightbox-actions">
          <Button
            variant="outline"
            size="sm"
            on:click={() => (isPixelated = !isPixelated)}
            title={isPixelated ? "Renderizado suave" : "Renderizado nítido / pixel-art"}
          >
            <Sparkles size={13} />
            <span>{isPixelated ? "Nítido" : "Suave"}</span>
          </Button>

          <Button
            variant="outline"
            size="sm"
            on:click={() => (isActualSize = !isActualSize)}
            title={isActualSize ? "Ajustar a ventana" : "Ver tamaño real 100%"}
          >
            {#if isActualSize}
              <ZoomOut size={13} />
              <span>Ajustar</span>
            {:else}
              <ZoomIn size={13} />
              <span>100%</span>
            {/if}
          </Button>

          <Button
            variant="primary"
            size="sm"
            on:click={handleCopy}
            title="Copiar imagen al portapapeles"
          >
            {#if copied}
              <Check size={13} />
              <span>Copiado</span>
            {:else}
              <Copy size={13} />
              <span>Copiar</span>
            {/if}
          </Button>

          <button
            type="button"
            class="lightbox-close-btn"
            on:click={closeLightbox}
            aria-label="Cerrar vista expandida"
            title="Cerrar (Esc)"
          >
            <X size={18} />
          </button>
        </div>
      </div>

      <div
        class="lightbox-stage lightbox-checkerboard"
        on:click={handleBackdropClick}
        role="presentation"
      >
        <button
          type="button"
          class="lightbox-img-btn"
          on:click={() => (isActualSize = !isActualSize)}
          title={isActualSize ? "Clic para ajustar a ventana" : "Clic para ver tamaño 100%"}
          aria-label={isActualSize ? "Ajustar imagen a ventana" : "Ver imagen en tamaño real 100%"}
        >
          <img
            src={imgSrc}
            alt={emote.name}
            class="lightbox-image"
            class:actual-size={isActualSize}
            class:pixelated={isPixelated}
          />
        </button>
      </div>

      <div class="lightbox-footer">
        <span class="lightbox-hint">
          Clic en la imagen o <kbd>Z</kbd> para alternar zoom • <kbd>Esc</kbd> para cerrar
        </span>
      </div>
    </div>
  </div>
{/if}

<style>
  .lightbox-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1200;
    background: rgba(10, 10, 12, 0.88);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
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

  .lightbox-card {
    position: relative;
    max-width: 92vw;
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-surface, #1c1c1f);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 16px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.7);
    overflow: hidden;
    color: var(--color-text-main, #f4f4f5);
  }

  .lightbox-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 18px;
    border-bottom: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    background: var(--color-bg-sidebar, #141416);
    flex-shrink: 0;
  }

  .lightbox-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    flex-wrap: wrap;
  }

  .lightbox-title {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    color: var(--color-text-main, #f4f4f5);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 260px;
  }

  .lightbox-badges {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .lightbox-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .lightbox-close-btn {
    background: transparent;
    border: none;
    color: var(--color-text-muted, #94a3b8);
    cursor: pointer;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .lightbox-close-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-orange-primary, #f97316);
  }

  .lightbox-close-btn:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 2px;
  }

  .lightbox-stage {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    min-width: 320px;
    min-height: 260px;
    max-height: calc(88vh - 110px);
    overflow: auto;
    position: relative;
  }

  .lightbox-checkerboard {
    background-color: #151518;
    background-image:
      linear-gradient(45deg, #1f1f23 25%, transparent 25%),
      linear-gradient(-45deg, #1f1f23 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #1f1f23 75%),
      linear-gradient(-45deg, transparent 75%, #1f1f23 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
  }

  .lightbox-img-btn {
    background: transparent;
    border: none;
    padding: 0;
    margin: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    outline: none;
  }

  .lightbox-img-btn:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 4px;
    border-radius: 8px;
  }

  .lightbox-image {
    display: block;
    max-width: min(78vw, 750px);
    max-height: min(65vh, 600px);
    object-fit: contain;
    filter: drop-shadow(0 4px 16px rgba(0, 0, 0, 0.5));
    cursor: pointer;
    transition: transform 0.15s ease;
    user-select: none;
  }

  .lightbox-image.actual-size {
    max-width: none;
    max-height: none;
    width: auto;
    height: auto;
  }

  .lightbox-image.pixelated {
    image-rendering: pixelated;
  }

  .lightbox-footer {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px 16px;
    background: var(--color-bg-sidebar, #141416);
    border-top: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    flex-shrink: 0;
  }

  .lightbox-hint {
    font-size: 11.5px;
    color: var(--color-text-dim, #71717a);
  }

  kbd {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    padding: 1px 5px;
    font-size: 10.5px;
    color: var(--color-text-muted, #a1a1aa);
    font-family: inherit;
  }
</style>

