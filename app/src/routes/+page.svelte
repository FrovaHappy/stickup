<script lang="ts">
  import { onMount } from 'svelte';
  import EmoteGrid from '$lib/components/EmoteGrid.svelte';
  import ImportButton from '$lib/components/ImportButton.svelte';
  import StorageSettings from '$lib/components/StorageSettings.svelte';
  import EmoteDetail from '$lib/components/EmoteDetail.svelte';
  import emotes, { loadEmotes, type Emote } from '$lib/stores/emotes';

  let selected: Emote | null = null;
  onMount(() => {
    loadEmotes();
  });

  function handleSelect(e: CustomEvent<Emote>) {
    selected = e.detail;
  }

  function closeDetail() {
    selected = null;
    loadEmotes();
  }
</script>

<main class="container">
  <h1>Emotes</h1>
  <StorageSettings />
  <ImportButton />
  <EmoteGrid emotes={$emotes} on:select={handleSelect} />
  {#if selected}
    <EmoteDetail emote={selected} onClose={closeDetail} />
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
  margin: 0;
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
