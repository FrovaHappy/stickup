<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Trash2 } from '@lucide/svelte';
  import type { Emote } from '$lib/types/emote';
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';
  import { toast } from '$lib/stores/toast';
  import { formatFileSize } from '$lib/utils/format';
  import { Button, Modal, Toast } from '$lib/components/system-ui';
  import { convertFileSrc } from '@tauri-apps/api/core';

  export let emote: Emote | null = null;
  export let open = false;
  export let onClose: () => void;
  export let onDeleted: (() => void) | undefined = undefined;

  const dispatch = createEventDispatcher<{
    close: void;
    deleted: Emote;
  }>();

  let isDeleting = false;

  async function handleConfirmDelete() {
    if (!emote || isDeleting) return;
    const target = emote;
    isDeleting = true;

    try {
      await api.deleteEmote(target.id);
      toast.success(`Emote "${target.name}" eliminado`);
      await loadEmotes();
      onDeleted?.();
      dispatch('deleted', target);
      handleClose();
    } catch (err: any) {
      toast.error(`Error al eliminar: ${err?.message || err}`);
    } finally {
      isDeleting = false;
    }
  }

  function handleClose() {
    if (isDeleting) return;
    onClose();
    dispatch('close');
  }
</script>

{#if open && emote}
  <Modal
    {open}
    title="Eliminar emote"
    maxWidth="420px"
    busy={isDeleting}
    onClose={handleClose}
  >
    <div class="delete-dialog-content">
      <div class="preview-card">
        <div class="preview-img-box">
          <img
            src={convertFileSrc(emote.path)}
            alt={emote.name}
            class="preview-img"
          />
        </div>
        <div class="preview-info">
          <span class="preview-name" title={emote.name}>{emote.name}</span>
          <div class="preview-meta">
            {#if emote.original_width && emote.original_height}
              <span>{emote.original_width}×{emote.original_height}</span>
            {/if}
            {#if emote.format}
              <span class="meta-format">{emote.format.toUpperCase()}</span>
            {/if}
            {#if emote.file_size}
              <span>{formatFileSize(emote.file_size)}</span>
            {/if}
          </div>
        </div>
      </div>

      <Toast
        variant="error"
        title="Atención"
        message="¿Deseas eliminar este emote permanentemente? El archivo y sus datos se borrarán del disco y esta acción no se puede deshacer."
      />
    </div>

    <svelte:fragment slot="footer">
      <Button
        variant="ghost"
        size="sm"
        disabled={isDeleting}
        on:click={handleClose}
      >
        Cancelar
      </Button>
      <Button
        variant="danger"
        size="sm"
        loading={isDeleting}
        disabled={isDeleting}
        on:click={handleConfirmDelete}
      >
        <Trash2 size={14} /> Eliminar
      </Button>
    </svelte:fragment>
  </Modal>
{/if}

<style>
  .delete-dialog-content {
    display: flex;
    flex-direction: column;
    gap: 14px;
    text-align: left;
  }

  .preview-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: transparent;
    border: 1px solid var(--color-border-subtle);
    border-radius: 8px;
  }

  .preview-img-box {
    width: 52px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
    overflow: hidden;
  }

  .preview-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .preview-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow: hidden;
    min-width: 0;
  }

  .preview-name {
    font-weight: 600;
    font-size: 13.5px;
    color: var(--color-text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .preview-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--color-text-dim);
  }

  .meta-format {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    color: var(--color-orange-primary);
    font-weight: 600;
  }
</style>

