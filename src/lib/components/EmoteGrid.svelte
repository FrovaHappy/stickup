<script lang="ts">
  import type { Emote } from '$lib/types/emote';
  import EmoteCard from './EmoteCard.svelte';
  import { gridSize } from '$lib/stores/preferences';
  import { createEventDispatcher } from 'svelte';

  export let emotes: Emote[] = [];
  export let disabled = false;

  const dispatch = createEventDispatcher<{
    select: Emote;
    copy: Emote;
    contextmenu: { emote: Emote; event: MouseEvent };
  }>();

  let hoveredEmote: Emote | null = null;

  function handleKeyDown(e: KeyboardEvent) {
    if (disabled || !hoveredEmote) return;
    if ((e.ctrlKey || e.metaKey) && (e.key === 'c' || e.key === 'C')) {
      const activeEl = document.activeElement as HTMLElement | null;
      const isInput =
        activeEl &&
        (activeEl.tagName === 'INPUT' ||
          activeEl.tagName === 'TEXTAREA' ||
          activeEl.isContentEditable);
      const selection = window.getSelection()?.toString();
      if (isInput && selection && selection.length > 0) {
        return;
      }
      e.preventDefault();
      dispatch('copy', hoveredEmote);
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if emotes.length === 0}
  <div class="empty-state">
    <p class="empty-title">No se encontraron emotes</p>
    <p class="empty-desc">Prueba a buscar con otro término o cambia tus filtros.</p>
  </div>
{:else}
  <div class="grid grid-size-{$gridSize}">
    {#each emotes as e (e.id)}
      <EmoteCard
        {e}
        on:select={(ev) => dispatch('select', ev.detail)}
        on:contextmenu={(ev) => dispatch('contextmenu', ev.detail)}
        on:hover={(ev) => (hoveredEmote = ev.detail)}
        on:unhover={(ev) => {
          if (hoveredEmote?.id === ev.detail.id) {
            hoveredEmote = null;
          }
        }}
      />
    {/each}
  </div>
{/if}

<style>
  .grid {
    display: grid;
    width: 100%;
    padding: 12px 28px 48px 28px;
    box-sizing: border-box;
    transition: all 0.2s ease;
  }

  .grid.grid-size-small {
    grid-template-columns: repeat(auto-fill, minmax(78px, 1fr));
    gap: 10px;
  }

  .grid.grid-size-medium {
    grid-template-columns: repeat(auto-fill, minmax(105px, 1fr));
    gap: 12px;
  }

  .grid.grid-size-large {
    grid-template-columns: repeat(auto-fill, minmax(145px, 1fr));
    gap: 16px;
  }

  .empty-state {
    padding: 64px 24px;
    text-align: center;
    color: #a1a1aa;
    margin: 0 auto;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 600;
    color: #e4e4e7;
    margin: 0 0 6px 0;
  }

  .empty-desc {
    font-size: 13px;
    margin: 0;
  }
</style>
