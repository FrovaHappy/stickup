<script lang="ts">
  import { onMount } from 'svelte';
  import EmoteGrid from '$lib/components/EmoteGrid.svelte';
  import ImportButton from '$lib/components/ImportButton.svelte';
  import SearchFilterBar from '$lib/components/SearchFilterBar.svelte';
  import StorageSettings from '$lib/components/StorageSettings.svelte';
  import EmoteDetailModal from '$lib/components/detail/EmoteDetailModal.svelte';
  import { Alert } from '$lib/components/system-ui';
  import { api } from '$lib/services/api';
  import { loadEmotes, type Emote } from '$lib/stores/emotes';

  let selected: Emote | null = null;
  let filteredEmotes: Emote[] = [];
  let copyToast = '';
  let copyToastTimeout: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    loadEmotes();
  });

  function closeDetail() {
    selected = null;
    loadEmotes();
  }

  async function handleEmoteCopy(emote: Emote) {
    if (selected) return;
    try {
      await api.copyToClipboard(
        emote.id,
        emote.path,
        emote.format,
        emote.original_width,
        emote.original_height
      );
      copyToast = `✓ ¡Emote "${emote.name}" copiado al portapapeles!`;
      await loadEmotes();
      if (copyToastTimeout) clearTimeout(copyToastTimeout);
      copyToastTimeout = setTimeout(() => {
        copyToast = '';
      }, 2500);
    } catch (err: any) {
      copyToast = `Error al copiar: ${err?.message || err}`;
      if (copyToastTimeout) clearTimeout(copyToastTimeout);
      copyToastTimeout = setTimeout(() => {
        copyToast = '';
      }, 3000);
    }
  }
</script>

<main class="container">
  <h1>Emotes</h1>
  <StorageSettings />
  <ImportButton />
  <SearchFilterBar bind:filteredEmotes />
  <EmoteGrid
    emotes={filteredEmotes}
    disabled={selected !== null}
    on:select={(e) => (selected = e.detail)}
    on:copy={(e) => handleEmoteCopy(e.detail)}
  />
  {#if selected}
    <EmoteDetailModal emote={selected} onClose={closeDetail} />
  {/if}
  {#if copyToast}
    <div class="toast-wrapper">
      <Alert variant="toast">{copyToast}</Alert>
    </div>
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

  .toast-wrapper {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9999;
    pointer-events: none;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }
  }
</style>
