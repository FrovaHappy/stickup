<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { X } from '@lucide/svelte';
  import type { AlertVariant } from '$lib/types/system-ui';

  export let variant: AlertVariant = 'info';
  export let title: string | undefined = undefined;
  export let dismissible = false;

  const dispatch = createEventDispatcher<{ close: void }>();
</script>

<div class="ui-alert variant-{variant}" role={variant === 'error' ? 'alert' : 'status'}>
  {#if title || dismissible}
    <div class="ui-alert-header">
      {#if title}
        <span class="ui-alert-title">{title}</span>
      {/if}
      {#if dismissible}
        <button
          type="button"
          class="ui-alert-close"
          aria-label="Cerrar aviso"
          on:click={() => dispatch('close')}
        >
          <X size={14} />
        </button>
      {/if}
    </div>
  {/if}

  <div class="ui-alert-body">
    <slot />
  </div>

  {#if $$slots.actions}
    <div class="ui-alert-actions">
      <slot name="actions" />
    </div>
  {/if}
</div>

<style>
  .ui-alert {
    width: 100%;
    box-sizing: border-box;
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 12.5px;
    line-height: 1.45;
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
  }

  .variant-info {
    background: rgba(99, 102, 241, 0.08);
    border: 1px solid rgba(99, 102, 241, 0.35);
    color: inherit;
  }

  .variant-success {
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid rgba(16, 185, 129, 0.3);
    color: #059669;
    font-weight: 600;
  }
  @media (prefers-color-scheme: dark) {
    .variant-success {
      color: #34d399;
    }
  }

  .variant-warning {
    background: rgba(245, 158, 11, 0.14);
    border: 1px solid rgba(245, 158, 11, 0.35);
    color: #d97706;
  }
  @media (prefers-color-scheme: dark) {
    .variant-warning {
      color: #fbbf24;
    }
  }

  .variant-error {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    font-weight: 600;
  }
  @media (prefers-color-scheme: dark) {
    .variant-error {
      color: #f87171;
    }
  }

  .variant-toast {
    position: absolute;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    width: auto;
    max-width: 90%;
    background: #10b981;
    color: #ffffff;
    font-weight: 700;
    padding: 8px 16px;
    border-radius: 999px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.45);
    z-index: 1200;
    text-align: center;
    white-space: nowrap;
  }

  .ui-alert-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .ui-alert-title {
    font-weight: 700;
    font-size: 13px;
    color: #4f46e5;
  }
  @media (prefers-color-scheme: dark) {
    .ui-alert-title {
      color: #a5b4fc;
    }
  }

  .ui-alert-close {
    background: transparent;
    border: none;
    color: inherit;
    opacity: 0.7;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 13px;
  }
  .ui-alert-close:hover {
    opacity: 1;
    background: rgba(148, 163, 184, 0.15);
  }

  .ui-alert-body {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .ui-alert-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 2px;
  }
</style>

