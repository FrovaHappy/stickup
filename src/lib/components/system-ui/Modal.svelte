<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { X } from '@lucide/svelte';

  export let open = false;
  export let title: string | undefined = undefined;
  export let maxWidth = '450px';
  export let busy = false;
  export let closeOnBackdrop = true;
  export let closeOnEscape = true;
  export let showCloseButton = true;
  export let onClose: (() => void) | undefined = undefined;

  const dispatch = createEventDispatcher<{ close: void }>();

  function requestClose() {
    if (busy) return;
    open = false;
    onClose?.();
    dispatch('close');
  }

  function handleBackdropClick(e: MouseEvent) {
    if (closeOnBackdrop && e.target === e.currentTarget) {
      requestClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (open && closeOnEscape && e.key === 'Escape') {
      requestClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
  <div
    class="ui-modal-backdrop"
    on:click={handleBackdropClick}
    role="presentation"
  >
    <div
      class="ui-modal-card"
      style:max-width={maxWidth}
      role="dialog"
      aria-modal="true"
      aria-label={title}
    >
      {#if title || $$slots.header}
        <div class="ui-modal-header">
          {#if $$slots.header}
            <slot name="header" />
          {:else if title}
            <h2 class="ui-modal-title">{title}</h2>
          {/if}

          {#if showCloseButton}
            <button
              type="button"
              class="ui-modal-close"
              disabled={busy}
              aria-label="Cerrar ventana"
              on:click={requestClose}
            >
              <X size={16} />
            </button>
          {/if}
        </div>
      {:else if showCloseButton}
        <button
          type="button"
          class="ui-modal-close floating"
          disabled={busy}
          aria-label="Cerrar ventana"
          on:click={requestClose}
        >
          <X size={16} />
        </button>
      {/if}

      <div class="ui-modal-body">
        <slot />
      </div>

      {#if $$slots.footer}
        <div class="ui-modal-footer">
          <slot name="footer" />
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .ui-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.72);
    backdrop-filter: blur(7px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 16px;
    animation: modalFadeIn 0.15s ease-out;
  }

  @keyframes modalFadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .ui-modal-card {
    position: relative;
    background: #ffffff;
    color: #0f172a;
    border-radius: 18px;
    padding: 22px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    box-sizing: border-box;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  @media (prefers-color-scheme: dark) {
    .ui-modal-card {
      background: #1e1e20;
      color: #f3f4f6;
      box-shadow:
        0 20px 40px rgba(0, 0, 0, 0.6),
        0 0 0 1px rgba(255, 255, 255, 0.08);
    }
  }

  .ui-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .ui-modal-title {
    margin: 0;
    font-size: 18px;
    font-weight: 800;
    letter-spacing: 0.5px;
    color: inherit;
  }

  .ui-modal-close {
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 18px;
    font-weight: 700;
    cursor: pointer;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition:
      background 0.15s ease,
      color 0.15s ease,
      transform 0.15s ease;
    z-index: 2;
  }

  .ui-modal-close.floating {
    position: absolute;
    top: 12px;
    right: 14px;
  }

  .ui-modal-close:hover:not(:disabled) {
    background: rgba(148, 163, 184, 0.15);
    color: #ef4444;
  }

  .ui-modal-close:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .ui-modal-body {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .ui-modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-top: 1px solid rgba(148, 163, 184, 0.16);
    padding-top: 12px;
  }
</style>

