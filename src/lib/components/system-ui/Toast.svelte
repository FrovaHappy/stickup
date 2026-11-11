<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { X, Info, CircleCheck, TriangleAlert, CircleAlert } from '@lucide/svelte';
  import { toasts, dismissToast } from '$lib/stores/toast';
  import type { ToastItem, ToastVariant } from '$lib/types/system-ui';

  // Props para uso standalone / inline (reemplazo completo de Alert)
  export let item: ToastItem | undefined = undefined;
  export let variant: ToastVariant = 'info';
  export let title: string | undefined = undefined;
  export let message: string | undefined = undefined;
  export let dismissible: boolean = false;

  const dispatch = createEventDispatcher<{ dismiss: void }>();

  function getVariantClass(v?: ToastVariant): string {
    switch (v) {
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

  function getDefaultTitle(v?: ToastVariant): string {
    switch (v) {
      case 'error':
        return 'Error';
      case 'warning':
        return 'Atención';
      case 'info':
        return 'Información';
      default:
        return 'Éxito';
    }
  }
</script>

{#if item || title || message || $$slots.default}
  {@const resolvedVariant = item?.variant || variant || 'info'}
  {@const resolvedTitle = item?.title || title || (message ? getDefaultTitle(resolvedVariant) : undefined)}
  {@const resolvedMsg = item?.message || message}
  <div
    class="toast-card toast-inline {getVariantClass(resolvedVariant)}"
    role={resolvedVariant === 'error' ? 'alert' : 'status'}
  >
    <div class="toast-icon-box icon-box-{resolvedVariant}">
      {#if resolvedVariant === 'error'}
        <CircleAlert size={22} strokeWidth={2} />
      {:else if resolvedVariant === 'warning'}
        <TriangleAlert size={22} strokeWidth={2} />
      {:else if resolvedVariant === 'info'}
        <Info size={22} strokeWidth={2} />
      {:else}
        <CircleCheck size={22} strokeWidth={2} />
      {/if}
    </div>

    <div class="toast-content">
      {#if resolvedTitle}
        <span class="toast-title">{resolvedTitle}</span>
      {/if}
      {#if resolvedMsg}
        <p class="toast-message">{resolvedMsg}</p>
      {/if}
      <slot />
      {#if $$slots.actions}
        <div class="toast-actions">
          <slot name="actions" />
        </div>
      {/if}
    </div>

    {#if dismissible || item?.id}
      <button
        type="button"
        class="toast-close"
        aria-label="Cerrar notificación"
        on:click={() => {
          if (item?.id) dismissToast(item.id);
          dispatch('dismiss');
        }}
      >
        <X size={16} strokeWidth={2.2} />
      </button>
    {/if}
  </div>
{:else}
  <div class="toast-container" aria-live="polite">
    {#each $toasts as t (t.id)}
      <div class="toast-card {getVariantClass(t.variant)}" role={t.variant === 'error' ? 'alert' : 'status'}>
        <div class="toast-icon-box icon-box-{t.variant || 'success'}">
          {#if t.variant === 'error'}
            <CircleAlert size={22} strokeWidth={2} />
          {:else if t.variant === 'warning'}
            <TriangleAlert size={22} strokeWidth={2} />
          {:else if t.variant === 'info'}
            <Info size={22} strokeWidth={2} />
          {:else}
            <CircleCheck size={22} strokeWidth={2} />
          {/if}
        </div>

        <div class="toast-content">
          <span class="toast-title">{t.title || getDefaultTitle(t.variant)}</span>
          <p class="toast-message">{t.message}</p>
        </div>

        <button
          type="button"
          class="toast-close"
          aria-label="Cerrar notificación"
          on:click={() => dismissToast(t.id)}
        >
          <X size={16} strokeWidth={2.2} />
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 99999;
    display: flex;
    flex-direction: column-reverse;
    gap: 12px;
    pointer-events: none;
    max-width: calc(100vw - 32px);
  }

  .toast-card {
    pointer-events: auto;
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 14px;
    width: 380px;
    max-width: calc(100vw - 48px);
    padding: 14px 16px;
    border-radius: 16px;
    background-color: #ffffff;
    box-shadow: 0 12px 30px -4px rgba(0, 0, 0, 0.16), 0 4px 12px -2px rgba(0, 0, 0, 0.06);
    animation: toast-enter 0.24s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    transition: transform 0.2s ease, opacity 0.2s ease;
  }

  .toast-card.toast-inline {
    width: 100%;
    max-width: 100%;
    box-sizing: border-box;
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.22);
    animation: none;
  }

  /* Variantes con el degradado pastel característico en el lado izquierdo */
  .toast-info {
    background: linear-gradient(115deg, #e0f2fe 0%, #f0f9ff 28%, #ffffff 58%);
    border: 1px solid rgba(186, 230, 253, 0.75);
  }

  .toast-success {
    background: linear-gradient(115deg, #dcfce7 0%, #f0fdf4 28%, #ffffff 58%);
    border: 1px solid rgba(187, 247, 208, 0.75);
  }

  .toast-warning {
    background: linear-gradient(115deg, #fef3c7 0%, #fffbeb 28%, #ffffff 58%);
    border: 1px solid rgba(254, 240, 138, 0.75);
  }

  .toast-error {
    background: linear-gradient(115deg, #fee2e2 0%, #fef2f2 28%, #ffffff 58%);
    border: 1px solid rgba(254, 202, 202, 0.75);
  }

  /* Caja contenedora del icono a la izquierda */
  .toast-icon-box {
    flex-shrink: 0;
    width: 42px;
    height: 42px;
    border-radius: 12px;
    background-color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05), 0 1px 2px rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.04);
  }

  .icon-box-info {
    color: #0284c7;
  }

  .icon-box-success {
    color: #10b981;
  }

  .icon-box-warning {
    color: #f59e0b;
  }

  .icon-box-error {
    color: #ef4444;
  }

  /* Contenido central */
  .toast-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding-top: 1px;
  }

  .toast-title {
    font-size: 15px;
    font-weight: 700;
    color: #0f172a;
    line-height: 1.25;
    letter-spacing: -0.01em;
  }

  .toast-message {
    margin: 0;
    font-size: 13px;
    font-weight: 400;
    color: #64748b;
    line-height: 1.45;
    word-break: break-word;
  }

  .toast-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 10px;
  }

  /* Botón de cierre en la esquina superior derecha */
  .toast-close {
    flex-shrink: 0;
    margin-top: -2px;
    margin-right: -4px;
    background: transparent;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    padding: 4px;
    border-radius: 6px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s ease, background-color 0.15s ease;
  }

  .toast-close:hover {
    color: #334155;
    background-color: rgba(0, 0, 0, 0.05);
  }

  @keyframes toast-enter {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
