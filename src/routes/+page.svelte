<script lang="ts">
  import { onMount } from 'svelte';
  import { Copy, Scaling, Type, Link, Info, Trash2 } from '@lucide/svelte';
  import EmoteGrid from '$lib/components/EmoteGrid.svelte';
  import MainHeader from '$lib/components/MainHeader.svelte';
  import EmoteDetailModal from '$lib/components/detail/EmoteDetailModal.svelte';
  import DeleteEmoteModal from '$lib/components/DeleteEmoteModal.svelte';
  import { ContextMenu } from '$lib/components/system-ui';
  import { api } from '$lib/services/api';
  import emotes, { loadEmotes, type Emote } from '$lib/stores/emotes';
  import { customSizes, defaultResolution, resolveDefaultDimensions } from '$lib/stores/sizes';
  import { minimizeOnCopyEnabled } from '$lib/stores/systemSettings';
  import { toast } from '$lib/stores/toast';
  import {
    sortOption,
    sortEmotesList,
    recordCopiedEmote,
    formatFilter,
    filterEmotesByFormat
  } from '$lib/stores/preferences';
  import { toPascalCase, isAnimatedEmote, fitDimensions } from '$lib/utils/format';

  let selected: Emote | null = null;
  let emoteToDelete: Emote | null = null;
  let searchQuery = '';
  let selectedTags: string[] = [];
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
    const isGif = isAnimatedEmote(emote);
    let resizingToastId: string | null = null;
    if (isGif) {
      resizingToastId = toast.info('Redimensionando...', 0);
      await new Promise((r) => setTimeout(r, 20));
    }
    try {
      const resolved = resolveDefaultDimensions(emote, $defaultResolution, $customSizes);
      await api.copyToClipboard(
        emote.id,
        emote.path,
        emote.format,
        resolved.width,
        resolved.height
      );
      if (resizingToastId) {
        toast.dismiss(resizingToastId);
        resizingToastId = null;
      }
      toast.success(`¡Emote "${emote.name}" copiado (${resolved.label})!`);
      recordCopiedEmote(emote);
      await loadEmotes();
      if ($minimizeOnCopyEnabled) {
        await api.minimizeWindow();
      }
    } catch (err: any) {
      if (resizingToastId) {
        toast.dismiss(resizingToastId);
        resizingToastId = null;
      }
      toast.error(`Error al copiar: ${err?.message || err}`);
    }
  }

  async function handleEmoteCopyWithSize(
    emote: Emote,
    width: number,
    height: number,
    sizeLabel: string
  ) {
    const isGif = isAnimatedEmote(emote);
    let resizingToastId: string | null = null;
    if (isGif) {
      resizingToastId = toast.info('Redimensionando...', 0);
      await new Promise((r) => setTimeout(r, 20));
    }
    try {
      const fitted = fitDimensions(
        emote.original_width,
        emote.original_height,
        width,
        height
      );
      await api.copyToClipboard(
        emote.id,
        emote.path,
        emote.format,
        fitted.width,
        fitted.height
      );
      if (resizingToastId) {
        toast.dismiss(resizingToastId);
        resizingToastId = null;
      }
      toast.success(`¡"${emote.name}" copiado (${sizeLabel})!`);
      recordCopiedEmote(emote);
      await loadEmotes();
      if ($minimizeOnCopyEnabled) {
        await api.minimizeWindow();
      }
    } catch (err: any) {
      if (resizingToastId) {
        toast.dismiss(resizingToastId);
        resizingToastId = null;
      }
      toast.error(`Error al copiar: ${err?.message || err}`);
    }
  }

  async function handleCopyText(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`${label} copiado al portapapeles`);
    } catch {
      toast.error(`Error al copiar ${label.toLowerCase()}`);
    }
  }

  function handleDeleteEmote(emote: Emote) {
    emoteToDelete = emote;
  }

  function handleContextMenu(detail: { emote: Emote; event: MouseEvent }) {
    contextMenuEmote = detail.emote;
    contextMenuPos = { x: detail.event.clientX, y: detail.event.clientY };
    contextMenuOpen = true;
  }

  $: filteredAndSortedEmotes = (() => {
    const rawQ = searchQuery.trim();
    const q = rawQ.toLowerCase();
    const pascalQ = toPascalCase(rawQ).toLowerCase();
    let result = filterEmotesByFormat($emotes, $formatFilter);

    if (selectedTags.length > 0) {
      const isUntaggedSelected = selectedTags.some(
        (t) => t === '__untagged__' || t.toLowerCase() === 'sin tag'
      );
      const regularTags = selectedTags
        .filter((t) => t !== '__untagged__' && t.toLowerCase() !== 'sin tag')
        .map((t) => t.toLowerCase());

      result = result.filter((e) => {
        const rawTags = (e.tag ?? '').trim();
        const hasNoTags = !rawTags;
        if (hasNoTags && isUntaggedSelected) {
          return true;
        }
        if (regularTags.length > 0 && !hasNoTags) {
          const emoteTags = rawTags
            .split(',')
            .map((t) => t.trim().toLowerCase());
          return regularTags.some((selected) => emoteTags.includes(selected));
        }
        return false;
      });
    }

    if (q) {
      result = result.filter((e) => {
        const name = (e.name ?? '').toLowerCase();
        const nameMatch = name.includes(q) || (pascalQ ? name.includes(pascalQ) : false);

        const tags = (e.tag ?? '')
          .split(',')
          .map((t) => t.trim().toLowerCase());
        const tagMatch = tags.some(
          (t) => t.includes(q) || (pascalQ ? t.includes(pascalQ) : false)
        );

        return nameMatch || tagMatch;
      });
    }

    return sortEmotesList(result, $sortOption);
  })();

  $: contextMenuItems = (() => {
    if (!contextMenuEmote) return [];
    const emote = contextMenuEmote;
    const defaultRes = resolveDefaultDimensions(emote, $defaultResolution, $customSizes);

    const customSizeItems = $customSizes
      .filter((cs) => cs.value !== $defaultResolution)
      .map((cs) => ({
        label: `Copiar ${cs.label}`,
        icon: Scaling,
        action: () =>
          cs.width &&
          cs.height &&
          handleEmoteCopyWithSize(emote, cs.width, cs.height, cs.label)
      }));

    const standardSizes = [
      { value: '48', label: '48×48 (Emote)', w: 48, h: 48 },
      { value: '128', label: '128×128 (Grande)', w: 128, h: 128 }
    ];

    const standardSizeItems = standardSizes
      .filter((s) => s.value !== $defaultResolution)
      .map((s) => ({
        label: `Copiar ${s.label}`,
        icon: Scaling,
        action: () => handleEmoteCopyWithSize(emote, s.w, s.h, s.label)
      }));

    const originalItem =
      $defaultResolution !== 'original'
        ? [
            {
              label: 'Copiar Original',
              icon: Scaling,
              action: () =>
                handleEmoteCopyWithSize(
                  emote,
                  emote.original_width,
                  emote.original_height,
                  'Original'
                )
            }
          ]
        : [];

    return [
      {
        label: `Copiar emote (${defaultRes.label})`,
        icon: Copy,
        shortcut: 'Ctrl+C',
        action: () => handleEmoteCopy(emote)
      },
      ...originalItem,
      ...standardSizeItems,
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
  <MainHeader bind:searchQuery bind:selectedTags />

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

  {#if emoteToDelete}
    <DeleteEmoteModal
      emote={emoteToDelete}
      open={true}
      onClose={() => (emoteToDelete = null)}
      onDeleted={() => {
        if (selected && selected.id === emoteToDelete?.id) {
          selected = null;
        }
        emoteToDelete = null;
      }}
    />
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
