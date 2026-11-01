<script lang="ts">
  import { onMount } from 'svelte';
  import EmoteGrid from '$lib/components/EmoteGrid.svelte';
  import ImportButton from '$lib/components/ImportButton.svelte';
  import SearchFilterBar from '$lib/components/SearchFilterBar.svelte';
  import StorageSettings from '$lib/components/StorageSettings.svelte';
  import EmoteDetailModal from '$lib/components/detail/EmoteDetailModal.svelte';
  import { loadEmotes, type Emote } from '$lib/stores/emotes';

  let selected: Emote | null = null;
  let filteredEmotes: Emote[] = [];

  onMount(() => {
    loadEmotes();
  });

  function closeDetail() {
    selected = null;
    loadEmotes();
  }
</script>

<main class="container">
  <h1>Emotes</h1>
  <StorageSettings />
  <ImportButton />
  <SearchFilterBar bind:filteredEmotes />
  <EmoteGrid emotes={filteredEmotes} on:select={(e) => (selected = e.detail)} />
  {#if selected}
    <EmoteDetailModal emote={selected} onClose={closeDetail} />
  {/if}
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    color: #0f0f0f;
    background-color: #f6f6f6;
    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    padding-top: 5vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
    max-width: 900px;
    margin: 0 auto;
  }

  h1 {
    text-align: center;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }
  }
</style>
