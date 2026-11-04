<script lang="ts">
  import type { Emote } from '$lib/types/emote';
  import EmoteCard from './EmoteCard.svelte';
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
  <div class="empty-state">No se encontraron emotes con los filtros actuales.</div>
{:else}
  <div class="grid">
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
    grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
    gap: 14px;
    padding: 10px 0 40px 0;
    width: 100%;
  }
  .empty-state {
    padding: 36px 16px;
    color: #9ca3af;
    font-size: 14px;
  }
</style>
