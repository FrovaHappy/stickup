<script lang="ts">
  import type { Emote } from '../stores/emotes';
  import EmoteCard from './EmoteCard.svelte';
  import { createEventDispatcher } from 'svelte';

  export let emotes: Emote[] = [];
  const dispatch = createEventDispatcher<{ select: Emote }>();

  function handleCardSelect(e: CustomEvent<Emote>) {
    dispatch('select', e.detail);
  }
</script>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
    gap: 14px;
    padding: 10px 0 40px 0;
    width: 100%;
  }
</style>

<div class="grid">
  {#each emotes as e (e.id)}
    <EmoteCard {e} on:select={handleCardSelect} />
  {/each}
</div>
