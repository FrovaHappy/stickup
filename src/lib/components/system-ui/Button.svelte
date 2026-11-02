<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ButtonSize, ButtonVariant } from '$lib/types/system-ui';

  export let variant: ButtonVariant = 'primary';
  export let size: ButtonSize = 'md';
  export let type: 'button' | 'submit' | 'reset' = 'button';
  export let disabled = false;
  export let loading = false;
  export let fullWidth = false;
  export let active = false;
  export let pill = false;
  export let title: string | undefined = undefined;
  export let ariaLabel: string | undefined = undefined;

  const dispatch = createEventDispatcher<{ click: MouseEvent }>();

  function handleClick(event: MouseEvent) {
    if (disabled || loading) return;
    dispatch('click', event);
  }
</script>

<button
  {type}
  class="ui-btn variant-{variant} size-{size} {fullWidth ? 'full-width' : ''} {active ? 'is-active' : ''} {pill ? 'is-pill' : ''}"
  disabled={disabled || loading}
  {title}
  aria-label={ariaLabel}
  aria-busy={loading}
  on:click={handleClick}
>
  {#if loading}
    <span class="ui-btn-spinner" aria-hidden="true">⏳</span>
  {/if}
  <slot />
</button>

<style>
  .ui-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 1px solid transparent;
    border-radius: 9px;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.2;
    cursor: pointer;
    white-space: nowrap;
    box-sizing: border-box;
    transition:
      background 0.15s ease,
      border-color 0.15s ease,
      color 0.15s ease,
      transform 0.1s ease,
      box-shadow 0.15s ease,
      opacity 0.15s ease;
  }

  .ui-btn:active:not(:disabled) {
    transform: scale(0.98);
  }

  .ui-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .ui-btn.full-width {
    width: 100%;
  }

  .ui-btn.is-pill {
    border-radius: 999px;
  }

  /* Sizes */
  .size-sm {
    padding: 6px 12px;
    font-size: 12px;
    border-radius: 7px;
  }

  .size-md {
    padding: 9px 18px;
    font-size: 13.5px;
  }

  .size-lg {
    padding: 12px 20px;
    font-size: 15px;
    font-weight: 700;
  }

  /* Variants */
  .variant-primary {
    background: #3b5bff;
    color: #ffffff;
    box-shadow: 0 4px 12px rgba(59, 91, 255, 0.3);
  }
  .variant-primary:hover:not(:disabled) {
    background: #2f49d1;
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(59, 91, 255, 0.42);
  }

  .variant-secondary {
    background: rgba(59, 91, 255, 0.1);
    color: #3b5bff;
    border-color: rgba(59, 91, 255, 0.25);
  }
  .variant-secondary:hover:not(:disabled),
  .variant-secondary.is-active {
    background: rgba(59, 91, 255, 0.18);
    border-color: #3b5bff;
    transform: translateY(-1px);
  }
  @media (prefers-color-scheme: dark) {
    .variant-secondary {
      background: rgba(76, 110, 245, 0.16);
      color: #93c5fd;
      border-color: rgba(76, 110, 245, 0.35);
    }
    .variant-secondary:hover:not(:disabled),
    .variant-secondary.is-active {
      background: rgba(76, 110, 245, 0.26);
      color: #ffffff;
    }
  }

  .variant-accent {
    background: #facc15;
    color: #111827;
    font-weight: 700;
    box-shadow: 0 4px 14px rgba(250, 204, 21, 0.3);
  }
  .variant-accent:hover:not(:disabled) {
    background: #eab308;
    box-shadow: 0 6px 18px rgba(250, 204, 21, 0.4);
  }

  .variant-success {
    background: rgba(16, 185, 129, 0.14);
    color: #059669;
    border-color: rgba(16, 185, 129, 0.35);
  }
  .variant-success:hover:not(:disabled) {
    background: rgba(16, 185, 129, 0.24);
    transform: translateY(-1px);
  }
  @media (prefers-color-scheme: dark) {
    .variant-success {
      color: #34d399;
    }
  }

  .variant-danger {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.3);
  }
  .variant-danger:hover:not(:disabled) {
    background: #ef4444;
    color: #ffffff;
  }
  @media (prefers-color-scheme: dark) {
    .variant-danger {
      color: #f87171;
    }
  }

  .variant-outline {
    background: #ffffff;
    color: #1f2937;
    border-color: #d1d5db;
  }
  .variant-outline:hover:not(:disabled) {
    background: #f3f4f6;
    border-color: #9ca3af;
  }
  @media (prefers-color-scheme: dark) {
    .variant-outline {
      background: #374151;
      color: #f3f4f6;
      border-color: #4b5563;
    }
    .variant-outline:hover:not(:disabled) {
      background: #4b5563;
    }
  }

  .variant-ghost {
    background: rgba(156, 163, 175, 0.16);
    color: inherit;
    border-color: transparent;
  }
  .variant-ghost:hover:not(:disabled) {
    background: rgba(156, 163, 175, 0.26);
  }

  .variant-icon {
    background: transparent;
    color: #9ca3af;
    padding: 6px;
    border-radius: 50%;
    min-width: 30px;
    min-height: 30px;
  }
  .variant-icon:hover:not(:disabled) {
    background: rgba(148, 163, 184, 0.15);
    color: #ffffff;
  }

  .ui-btn-spinner {
    font-size: 13px;
    line-height: 1;
  }
</style>

