<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ChipVariant } from '$lib/types/system-ui';

  export let variant: ChipVariant = 'default';
  export let active = false;
  export let removable = false;
  export let pill = false;
  export let disabled = false;
  export let count: number | undefined = undefined;
  export let title: string | undefined = undefined;
  export let removeTitle = 'Quitar';

  const dispatch = createEventDispatcher<{
    click: MouseEvent;
    remove: MouseEvent;
  }>();

  function handleChipClick(e: MouseEvent) {
    if (disabled) return;
    dispatch('click', e);
  }

  function handleRemoveClick(e: MouseEvent) {
    e.stopPropagation();
    if (disabled) return;
    dispatch('remove', e);
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="ui-chip variant-{variant} {active ? 'is-active' : ''} {pill ? 'is-pill' : ''} {disabled ? 'is-disabled' : ''}"
  {title}
  role="button"
  tabindex={disabled ? -1 : 0}
  aria-pressed={active}
  aria-disabled={disabled}
  on:click={handleChipClick}
  on:keydown={(e) => {
    if (!disabled && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      dispatch('click', new MouseEvent('click'));
    }
  }}
>
  {#if $$slots.icon}
    <span class="ui-chip-icon">
      <slot name="icon" />
    </span>
  {/if}

  <span class="ui-chip-label">
    <slot />
  </span>

  {#if count !== undefined}
    <span class="ui-chip-count">{count}</span>
  {/if}

  {#if removable}
    <button
      type="button"
      class="ui-chip-remove"
      title={removeTitle}
      aria-label={removeTitle}
      {disabled}
      on:click={handleRemoveClick}
    >
      ✕
    </button>
  {/if}
</div>

<style>
  .ui-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border-radius: 7px;
    font-size: 12px;
    font-weight: 600;
    background: rgba(156, 163, 175, 0.12);
    border: 1px solid rgba(156, 163, 175, 0.22);
    color: inherit;
    cursor: pointer;
    user-select: none;
    transition: all 0.15s ease;
    box-sizing: border-box;
  }

  .ui-chip.is-pill {
    border-radius: 999px;
  }

  .ui-chip:hover:not(.is-disabled) {
    background: rgba(99, 102, 241, 0.14);
    border-color: rgba(99, 102, 241, 0.35);
  }

  .ui-chip.is-disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Variants (unselected & active) */
  .variant-indigo {
    background: rgba(99, 102, 241, 0.14);
    border-color: rgba(99, 102, 241, 0.35);
    color: #6366f1;
  }
  @media (prefers-color-scheme: dark) {
    .variant-indigo {
      color: #c7d2fe;
    }
  }

  .variant-amber {
    background: rgba(245, 158, 11, 0.18);
    border-color: rgba(245, 158, 11, 0.4);
    color: #d97706;
  }
  @media (prefers-color-scheme: dark) {
    .variant-amber {
      color: #fbbf24;
    }
  }

  .variant-green {
    background: rgba(16, 185, 129, 0.18);
    border-color: rgba(16, 185, 129, 0.4);
    color: #059669;
  }
  @media (prefers-color-scheme: dark) {
    .variant-green {
      color: #34d399;
    }
  }

  .variant-sky {
    background: rgba(2, 132, 199, 0.16);
    border-color: rgba(56, 189, 248, 0.4);
    color: #0284c7;
  }
  @media (prefers-color-scheme: dark) {
    .variant-sky {
      color: #38bdf8;
    }
  }

  /* Active / Selected states */
  .ui-chip.is-active,
  .variant-indigo.is-active {
    background: #4f46e5;
    border-color: #6366f1;
    color: #ffffff;
    box-shadow: 0 2px 10px rgba(99, 102, 241, 0.35);
  }

  .variant-hot.is-active {
    background: linear-gradient(135deg, #f59e0b, #ef4444);
    border-color: #f59e0b;
    color: #ffffff;
    box-shadow: 0 2px 10px rgba(245, 158, 11, 0.35);
  }

  .variant-green.is-active {
    background: #10b981;
    border-color: #34d399;
    color: #ffffff;
    box-shadow: 0 2px 10px rgba(16, 185, 129, 0.35);
  }

  .variant-sky.is-active {
    background: #0284c7;
    border-color: #38bdf8;
    color: #ffffff;
    box-shadow: 0 2px 10px rgba(2, 132, 199, 0.35);
  }

  .ui-chip-icon {
    display: inline-flex;
    align-items: center;
    line-height: 1;
  }

  .ui-chip-label {
    line-height: 1.2;
    white-space: nowrap;
  }

  .ui-chip-count {
    font-size: 10px;
    opacity: 0.75;
    background: rgba(0, 0, 0, 0.15);
    padding: 1px 5px;
    border-radius: 999px;
  }

  .ui-chip-remove {
    background: transparent;
    border: none;
    color: inherit;
    opacity: 0.75;
    font-size: 11px;
    line-height: 1;
    padding: 1px 2px;
    cursor: pointer;
    border-radius: 3px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition:
      background 0.15s ease,
      color 0.15s ease,
      opacity 0.15s ease;
  }

  .ui-chip-remove:hover:not(:disabled) {
    opacity: 1;
    background: rgba(239, 68, 68, 0.28);
    color: #fca5a5;
  }
</style>

