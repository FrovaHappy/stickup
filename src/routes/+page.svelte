<script lang="ts">
  import { onMount } from 'svelte';
  import { Copy, Scaling, Type, Link, Info, Trash2 } from '@lucide/svelte';
  import EmoteGrid from '$lib/components/EmoteGrid.svelte';
  import MainHeader from '$lib/components/MainHeader.svelte';
  import EmoteDetailModal from '$lib/components/detail/EmoteDetailModal.svelte';
  import { ContextMenu } from '$lib/components/system-ui';
  import { api } from '$lib/services/api';
  import emotes, { loadEmotes, type Emote } from '$lib/stores/emotes';
  import { customSizes } from '$lib/stores/sizes';
  import { toast } from '$lib/stores/toast';
  import { sortOption, sortEmotesList } from '$lib/stores/preferences';

  let selected: Emote | null = null;
  let searchQuery = '';
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

  $: filteredAndSortedEmotes = (() => {
    const q = searchQuery.trim().toLowerCase();
    let result = $emotes;

    if (q) {
      result = result.filter((e) => {
        const nameMatch = (e.name ?? '').toLowerCase().includes(q);
        const tags = (e.tag ?? '')
          .split(',')
          .map((t) => t.trim().toLowerCase());
        const tagMatch = tags.some((t) => t.includes(q));
        return nameMatch || tagMatch;
      });
    }

    return sortEmotesList(result, $sortOption);
  })();

  $: contextMenuItems = (() => {
    if (!contextMenuEmote) return [];
    const emote = contextMenuEmote;

    const customSizeItems = $customSizes.map((cs) => ({
      label: `Copiar ${cs.label}`,
      icon: Scaling,
      action: () =>
        cs.width &&
        cs.height &&
        handleEmoteCopyWithSize(emote, cs.width, cs.height, cs.label)
    }));

    return [
      {
        label: 'Copiar emote (Original)',
        icon: Copy,
        shortcut: 'Ctrl+C',
        action: () => handleEmoteCopy(emote)
      },
      {
        label: 'Copiar 48×48 (Emote)',
        icon: Scaling,
        action: () => handleEmoteCopyWithSize(emote, 48, 48, '48×48')
      },
      {
        label: 'Copiar 128×128 (Grande)',
        icon: Scaling,
        action: () => handleEmoteCopyWithSize(emote, 128, 128, '128×128')
      },
      ...customSizeItems,
      { divider: true, label: '' },
      {
        label: 'Copiar nombre',
        icon: Type,
        action: () => handleCopyText(emote.name, 'Nombre')
      },
      {
        label: 'Copiar ruta de archivo',
        icon: Link,
        action: () => handleCopyText(emote.path, 'Ruta')
      },
      { divider: true, label: '' },
      {
        label: 'Ver detalles / Editar',
        icon: Info,
        action: () => {
          selected = emote;
        }
      },
      {
        label: 'Eliminar emote',
        icon: Trash2,
        danger: true,
        action: () => handleDeleteEmote(emote)
      }
    ];
  })();
</script>

<div class="home-view">
  <MainHeader bind:searchQuery />

  <div class="grid-container">
    <EmoteGrid
      emotes={filteredAndSortedEmotes}
      disabled={selected !== null}
      on:select={(e) => (selected = e.detail)}
      on:copy={(e) => handleEmoteCopy(e.detail)}
      on:contextmenu={(e) => handleContextMenu(e.detail)}
    />
  </div>

  {#if selected}
    <EmoteDetailModal emote={selected} onClose={closeDetail} />
  {/if}

  <ContextMenu
    bind:open={contextMenuOpen}
    x={contextMenuPos.x}
    y={contextMenuPos.y}
    items={contextMenuItems}
  />
</div>

<style>
  .home-view {
    display: contents;
  }

  .grid-container {
    width: 100%;
  }
</style>
