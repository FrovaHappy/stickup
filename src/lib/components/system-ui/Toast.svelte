<script lang="ts">
  import { toasts, dismissToast } from '$lib/stores/toast';
  import type { ToastItem, ToastVariant } from '$lib/types/system-ui';

  // Opcional: si se pasa un item específico para uso standalone
  export let item: ToastItem | undefined = undefined;

  function getVariantClass(variant?: ToastVariant): string {
    switch (variant) {
      case 'error':
        return 'toast-error';
      case 'warning':
        return 'toast-warning';
      case 'info':
        return 'toast-info';
      default:
        return 'toast-success';
    }
  }

  function getIcon(variant?: ToastVariant): string {
    switch (variant) {
      case 'error':
        return '✕';
      case 'warning':
        return '⚠️';
      case 'info':
        return 'ℹ️';
      default:
        return '✓';
    }
  }
</script>

{#if item}
  <div class="toast-bubble {getVariantClass(item.variant)}" role={item.variant === 'error' ? 'alert' : 'status'}>
    <span class="toast-icon">{getIcon(item.variant)}</span>
    <span class="toast-message">{item.message}</span>
    <button
      type="button"
      class="toast-close"
      aria-label="Cerrar notificación"
      on:click={() => item?.id && dismissToast(item.id)}
    >
      ✕
    </button>
  </div>
{:else}
  <div class="toast-container" aria-live="polite">
    {#each $toasts as t (t.id)}
      <div class="toast-bubble {getVariantClass(t.variant)}" role={t.variant === 'error' ? 'alert' : 'status'}>
        <span class="toast-icon">{getIcon(t.variant)}</span>
        <span class="toast-message">{t.message}</span>
        <button
          type="button"
          class="toast-close"
          aria-label="Cerrar notificación"
          on:click={() => dismissToast(t.id)}
        >
          ✕
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 99999;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    pointer-events: none;
    max-width: 90vw;
  }

  .toast-bubble {
    pointer-events: auto;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 9px 16px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 600;
    line-height: 1.4;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(8px);
    animation: toast-enter 0.22s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    white-space: nowrap;
    max-width: 650px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .toast-icon {
    font-size: 14px;
    font-weight: 800;
    line-height: 1;
  }

  .toast-message {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .toast-close {
    background: transparent;
    border: none;
    color: inherit;
    opacity: 0.7;
    cursor: pointer;
    font-size: 12px;
    line-height: 1;
    padding: 2px 4px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    margin-left: 2px;
    transition: opacity 0.15s ease, background 0.15s ease;
  }

  .toast-close:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.2);
  }

  /* Variantes */
  .toast-success {
    background: #059669;
    color: #ffffff;
    border: 1px solid rgba(52, 211, 153, 0.3);
  }

  .toast-error {
    background: #dc2626;
    color: #ffffff;
    border: 1px solid rgba(248, 113, 113, 0.35);
  }

  .toast-warning {
    background: #d97706;
    color: #ffffff;
    border: 1px solid rgba(251, 191, 36, 0.35);
  }

  .toast-info {
    background: #4f46e5;
    color: #ffffff;
    border: 1px solid rgba(165, 180, 252, 0.35);
  }

  @keyframes toast-enter {
    from {
      opacity: 0;
      transform: translateY(12px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>

