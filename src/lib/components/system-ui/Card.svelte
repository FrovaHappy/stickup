<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { CardHighlight, CardPadding } from '$lib/types/system-ui';

  export let interactive = false;
  export let highlight: CardHighlight = 'none';
  export let padding: CardPadding = 'md';
  export let title: string | undefined = undefined;

  const dispatch = createEventDispatcher<{ click: MouseEvent }>();
</script>

{#if interactive}
  <button
    type="button"
    class="ui-card is-interactive highlight-{highlight} pad-{padding}"
    {title}
    on:click={(e) => dispatch('click', e)}
  >
    <slot />
  </button>
{:else}
  <div class="ui-card highlight-{highlight} pad-{padding}" {title}>
    <slot />
  </div>
{/if}

<style>
  .ui-card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    box-sizing: border-box;
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 10px;
    color: inherit;
    font-family: inherit;
    overflow: hidden;
    transition:
      transform 0.15s ease,
      background 0.15s ease,
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }

  @media (prefers-color-scheme: dark) {
    .ui-card {
      background: rgba(255, 255, 255, 0.05);
      border-color: rgba(255, 255, 255, 0.1);
    }
  }

  /* Paddings */
  .pad-sm {
    padding: 8px;
  }
  .pad-md {
    padding: 12px 16px;
  }
  .pad-lg {
    padding: 18px 22px;
  }

  /* Interactive mode */
  .ui-card.is-interactive {
    cursor: pointer;
    text-align: center;
  }

  .ui-card.is-interactive:hover {
    transform: translateY(-2px);
    background: rgba(99, 102, 241, 0.08);
    border-color: #6366f1;
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.25);
  }
  @media (prefers-color-scheme: dark) {
    .ui-card.is-interactive:hover {
      background: rgba(255, 255, 255, 0.1);
    }
  }

  /* Highlights */
  .highlight-new {
    border-color: rgba(16, 185, 129, 0.5);
  }

  .highlight-hot {
    border-color: rgba(249, 115, 22, 0.55);
    box-shadow: 0 0 10px rgba(249, 115, 22, 0.16);
  }

  .highlight-indigo {
    background: rgba(99, 102, 241, 0.14);
    border-color: rgba(99, 102, 241, 0.35);
  }
</style>

