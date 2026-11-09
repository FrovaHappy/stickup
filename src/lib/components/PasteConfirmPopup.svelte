<script lang="ts">
  import { X, Download, Loader2 } from '@lucide/svelte';
  import { setAskBeforePaste } from '$lib/stores/preferences';

  export let open = false;
  export let previewUrl: string = '';
  export let name: string = '';
  export let isBusy = false;
  export let onSave: (dontAskAgain: boolean) => Promise<void> | void;
  export let onCancel: () => void;

  let dontAskAgain = false;

  async function handleSaveClick() {
    if (isBusy) return;
    if (dontAskAgain) {
      setAskBeforePaste(false);
    }
    await onSave(dontAskAgain);
  }

  function handleCloseClick() {
    if (isBusy) return;
    onCancel();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (open && e.key === 'Escape') {
      handleCloseClick();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
  <div
    class="paste-popup-card"
    role="dialog"
    aria-label="Confirmación de elemento pegado"
    aria-modal="false"
  >
    <button
      type="button"
      class="close-btn"
      aria-label="Cerrar y descartar"
      disabled={isBusy}
      on:click={handleCloseClick}
    >
      <X size={18} strokeWidth={2.2} />
    </button>

    <h3 class="popup-title">Se Pegó este Elemento</h3>

    <div class="preview-container">
      {#if previewUrl}
        <img src={previewUrl} alt={name || 'Elemento pegado'} class="preview-img" />
      {/if}
    </div>

    <label class="checkbox-row">
      <input
        type="checkbox"
        bind:checked={dontAskAgain}
        disabled={isBusy}
        class="custom-checkbox"
      />
      <span class="checkbox-label">No volver a preguntar</span>
    </label>

    <button
      type="button"
      class="save-btn"
      disabled={isBusy}
      on:click={handleSaveClick}
    >
      {#if isBusy}
        <Loader2 size={18} class="spin-icon" />
        <span>Guardando...</span>
      {:else}
        <Download size={18} strokeWidth={2.2} />
        <span>Guardar</span>
      {/if}
    </button>
  </div>
{/if}

<style>
  .paste-popup-card {
    position: fixed;
    right: 24px;
    bottom: 24px;
    width: 260px;
    background: var(--color-bg-surface, #1c1c1f);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 14px;
    padding: 16px 18px 18px;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.55), 0 0 0 1px rgba(255, 255, 255, 0.04);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    align-items: center;
    box-sizing: border-box;
    animation: slideInUp 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes slideInUp {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .close-btn {
    position: absolute;
    top: 12px;
    right: 12px;
    background: transparent;
    border: none;
    color: var(--color-text-dim, #71717a);
    cursor: pointer;
    padding: 4px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s ease, background 0.15s ease;
  }

  .close-btn:hover:not(:disabled) {
    color: var(--color-text-main, #f4f4f5);
    background: var(--color-bg-hover, rgba(255, 255, 255, 0.06));
  }

  .close-btn:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 1px;
  }

  .popup-title {
    margin: 4px 0 14px;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-main, #f4f4f5);
    text-align: center;
    letter-spacing: -0.01em;
  }

  .preview-container {
    width: 100%;
    height: 128px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 12px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.06));
    border-radius: 10px;
    overflow: hidden;
    padding: 8px;
  }

  .preview-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    user-select: none;
  }

  .checkbox-row {
    display: inline-flex;
    align-items: center;
    gap: 9px;
    margin-bottom: 14px;
    cursor: pointer;
    user-select: none;
    font-size: 13.5px;
    color: var(--color-text-secondary, #e4e4e7);
  }

  .custom-checkbox {
    width: 17px;
    height: 17px;
    accent-color: var(--color-orange-primary, #f97316);
    cursor: pointer;
    margin: 0;
  }

  .save-btn {
    width: 100%;
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 14.5px;
    font-weight: 600;
    color: #ffffff;
    background: var(--color-orange-primary, #f97316);
    border: none;
    border-radius: 8px;
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(249, 115, 22, 0.32);
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
  }

  .save-btn:hover:not(:disabled) {
    background: var(--color-orange-hover, #ea580c);
  }

  .save-btn:active:not(:disabled) {
    transform: scale(0.98);
  }

  .save-btn:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 2px;
  }

  .save-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  :global(.spin-icon) {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>

