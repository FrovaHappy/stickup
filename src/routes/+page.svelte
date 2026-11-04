<script lang="ts">
  import { onMount } from 'svelte';
  import EmoteGrid from '$lib/components/EmoteGrid.svelte';
  import ImportButton from '$lib/components/ImportButton.svelte';
  import SearchFilterBar from '$lib/components/SearchFilterBar.svelte';
  import StorageSettings from '$lib/components/StorageSettings.svelte';
  import EmoteDetailModal from '$lib/components/detail/EmoteDetailModal.svelte';
  import { ContextMenu, Toast } from '$lib/components/system-ui';
  import { api } from '$lib/services/api';
  import { loadEmotes, type Emote } from '$lib/stores/emotes';
  import { customSizes } from '$lib/stores/sizes';
  import { toast } from '$lib/stores/toast';

  let selected: Emote | null = null;
  let filteredEmotes: Emote[] = [];
  let contextMenuOpen = false;
  let contextMenuPos = { x: 0, y: 0 };
  let contextMenuEmote: Emote | null = null;

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
      toast.success(`✓ ¡Emote "${emote.name}" copiado al portapapeles!`);
      await loadEmotes();
    } catch (err: any) {
      toast.error(`Error al copiar: ${err?.message || err}`);
    }
  }

  async function handleEmoteCopyWithSize(
    emote: Emote,
    width: number,
    height: number,
    sizeLabel: string
  ) {
    try {
      await api.copyToClipboard(
        emote.id,
        emote.path,
        emote.format,
        width,
        height
      );
      toast.success(`✓ ¡"${emote.name}" copiado (${sizeLabel})!`);
      await loadEmotes();
    } catch (err: any) {
      toast.error(`Error al copiar: ${err?.message || err}`);
    }
  }

  async function handleCopyText(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`✓ ${label} copiado al portapapeles`);
    } catch {
      toast.error(`Error al copiar ${label.toLowerCase()}`);
    }
  }

  async function handleDeleteEmote(emote: Emote) {
    if (confirm(`¿Eliminar emote "${emote.name}" de forma permanente?`)) {
      try {
        await api.deleteEmote(emote.id);
        toast.success(`✓ Emote "${emote.name}" eliminado`);
        await loadEmotes();
      } catch (err: any) {
        toast.error(`Error al eliminar: ${err?.message || err}`);
      }
    }
  }

  function handleContextMenu(detail: { emote: Emote; event: MouseEvent }) {
    contextMenuEmote = detail.emote;
    contextMenuPos = { x: detail.event.clientX, y: detail.event.clientY };
    contextMenuOpen = true;
  }

  $: contextMenuItems = (() => {
    if (!contextMenuEmote) return [];
    const emote = contextMenuEmote;

    const customSizeItems = $customSizes.map((cs) => ({
      label: `Copiar ${cs.label}`,
      icon: '📐',
      action: () =>
        cs.width &&
        cs.height &&
        handleEmoteCopyWithSize(emote, cs.width, cs.height, cs.label)
    }));

    return [
      {
        label: 'Copiar emote (Original)',
        icon: '📋',
        shortcut: 'Ctrl+C',
        action: () => handleEmoteCopy(emote)
      },
      {
        label: 'Copiar 48×48 (Emote)',
        icon: '📐',
        action: () => handleEmoteCopyWithSize(emote, 48, 48, '48×48')
      },
      {
        label: 'Copiar 128×128 (Grande)',
        icon: '📐',
        action: () => handleEmoteCopyWithSize(emote, 128, 128, '128×128')
      },
      ...customSizeItems,
      { divider: true, label: '' },
      {
        label: 'Copiar nombre',
        icon: '🔤',
        action: () => handleCopyText(emote.name, 'Nombre')
      },
      {
        label: 'Copiar ruta de archivo',
        icon: '🔗',
        action: () => handleCopyText(emote.path, 'Ruta')
      },
      { divider: true, label: '' },
      {
        label: 'Ver detalles / Editar',
        icon: 'ℹ️',
        action: () => {
          selected = emote;
        }
      },
      {
        label: 'Eliminar emote',
        icon: '🗑️',
        danger: true,
        action: () => handleDeleteEmote(emote)
      }
    ];
  })();
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
    on:contextmenu={(e) => handleContextMenu(e.detail)}
  />
  {#if selected}
    <EmoteDetailModal emote={selected} onClose={closeDetail} />
  {/if}

  <ContextMenu
    bind:open={contextMenuOpen}
    x={contextMenuPos.x}
    y={contextMenuPos.y}
    items={contextMenuItems}
  />

  <Toast />
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
