<script lang="ts">
  import { onMount } from 'svelte';
  import type { Emote } from '$lib/types/emote';
  import emotes, {
    allTags,
    isEmoteNewByDate,
    topHotRanks
  } from '$lib/stores/emotes';
  import { isAnimatedEmote } from '$lib/utils/format';
  import { Badge, Button, Chip, Dropdown, Input } from '$lib/components/system-ui';

  export let filteredEmotes: Emote[] = [];

  type PresetFilter = 'all' | 'hot' | 'new' | 'animated' | 'static';
  type SearchScope = 'all' | 'name' | 'tag';

  interface RecentSearch {
    query: string;
    type: 'Búsqueda' | 'Nombre' | 'Tag';
  }

  const RECENT_STORAGE_KEY = 'stickup_recent_searches';

  let searchQuery = '';
  let activePreset: PresetFilter = 'all';
  let searchScope: SearchScope = 'all';
  let selectedTags: string[] = [];
  let isOpen = false;
  let recentSearches: RecentSearch[] = [];
  let inputEl: HTMLInputElement | null = null;

  onMount(() => {
    try {
      const saved = localStorage.getItem(RECENT_STORAGE_KEY);
      if (saved) {
        recentSearches = JSON.parse(saved);
      }
    } catch {
      recentSearches = [];
    }
  });

  function saveRecentSearch(query: string, type: RecentSearch['type']) {
    const clean = query.trim();
    if (!clean) return;
    const filtered = recentSearches.filter(
      (r) => !(r.query.toLowerCase() === clean.toLowerCase() && r.type === type)
    );
    recentSearches = [{ query: clean, type }, ...filtered].slice(0, 5);
    try {
      localStorage.setItem(RECENT_STORAGE_KEY, JSON.stringify(recentSearches));
    } catch {
      // Ignorar errores de almacenamiento local
    }
  }

  function removeRecentSearch(item: RecentSearch) {
    recentSearches = recentSearches.filter(
      (r) => !(r.query === item.query && r.type === item.type)
    );
    try {
      localStorage.setItem(RECENT_STORAGE_KEY, JSON.stringify(recentSearches));
    } catch {
      // Ignorar errores de almacenamiento local
    }
  }

  function togglePreset(preset: PresetFilter) {
    activePreset = activePreset === preset ? 'all' : preset;
  }

  function toggleScope(scope: SearchScope) {
    searchScope = searchScope === scope ? 'all' : scope;
    inputEl?.focus();
  }

  function toggleTag(tag: string) {
    const exists = selectedTags.some((t) => t.toLowerCase() === tag.toLowerCase());
    if (exists) {
      selectedTags = selectedTags.filter((t) => t.toLowerCase() !== tag.toLowerCase());
    } else {
      selectedTags = [...selectedTags, tag];
      saveRecentSearch(tag, 'Tag');
    }
  }

  function applyRecent(item: RecentSearch) {
    if (item.type === 'Tag') {
      if (!selectedTags.some((t) => t.toLowerCase() === item.query.toLowerCase())) {
        selectedTags = [...selectedTags, item.query];
      }
    } else {
      searchQuery = item.query;
      searchScope = item.type === 'Nombre' ? 'name' : 'all';
    }
    isOpen = false;
  }

  function clearAllFilters() {
    searchQuery = '';
    activePreset = 'all';
    searchScope = 'all';
    selectedTags = [];
  }

  function handleKeydown(e: CustomEvent<KeyboardEvent>) {
    const keyEvent = e.detail;
    if (keyEvent.key === 'Escape') {
      isOpen = false;
      inputEl?.blur();
    } else if (keyEvent.key === 'Enter' && searchQuery.trim()) {
      const type: RecentSearch['type'] =
        searchScope === 'name' ? 'Nombre' : searchScope === 'tag' ? 'Tag' : 'Búsqueda';
      saveRecentSearch(searchQuery.trim(), type);
      isOpen = false;
    }
  }

  $: tagCounts = (() => {
    const counts = new Map<string, number>();
    for (const e of $emotes) {
      if (!e.tag) continue;
      const parts = e.tag
        .split(',')
        .map((t) => t.trim())
        .filter(Boolean);
      for (const p of parts) {
        counts.set(p, (counts.get(p) ?? 0) + 1);
      }
    }
    return counts;
  })();

  $: matchingTags = ($allTags || []).filter((t) => {
    if (!searchQuery.trim()) return true;
    return t.toLowerCase().includes(searchQuery.trim().toLowerCase());
  });

  $: hasActiveFilters =
    searchQuery.trim() !== '' ||
    activePreset !== 'all' ||
    searchScope !== 'all' ||
    selectedTags.length > 0;

  $: {
    const q = searchQuery.trim().toLowerCase();
    const hotRanks = $topHotRanks;

    let result = $emotes.filter((e) => {
      if (activePreset === 'hot' && !hotRanks.has(e.id)) return false;
      if (activePreset === 'new' && !isEmoteNewByDate(e.created_at)) return false;
      if (activePreset === 'animated' && !isAnimatedEmote(e)) return false;
      if (activePreset === 'static' && isAnimatedEmote(e)) return false;

      if (selectedTags.length > 0) {
        const emoteTags = (e.tag ?? '')
          .split(',')
          .map((t) => t.trim().toLowerCase())
          .filter(Boolean);
        const hasAllSelected = selectedTags.every((sel) =>
          emoteTags.includes(sel.toLowerCase())
        );
        if (!hasAllSelected) return false;
      }

      if (q) {
        const nameMatch = (e.name ?? '').toLowerCase().includes(q);
        const emoteTags = (e.tag ?? '')
          .split(',')
          .map((t) => t.trim().toLowerCase())
          .filter(Boolean);
        const tagMatch = emoteTags.some((t) => t.includes(q));

        if (searchScope === 'name' && !nameMatch) return false;
        if (searchScope === 'tag' && !tagMatch) return false;
        if (searchScope === 'all' && !nameMatch && !tagMatch) return false;
      }

      return true;
    });

    if (activePreset === 'hot') {
      result = [...result].sort(
        (a, b) => (hotRanks.get(a.id) ?? 999) - (hotRanks.get(b.id) ?? 999)
      );
    } else {
      result = [...result].sort((a, b) => {
        const timeDiff = (Date.parse(b.created_at) || 0) - (Date.parse(a.created_at) || 0);
        if (timeDiff !== 0) return timeDiff;
        return b.id - a.id;
      });
    }

    filteredEmotes = result;
  }
</script>

<div class="search-filter-wrapper">
  <div class="search-container">
    <Input
      variant="pill"
      active={isOpen}
      bind:inputElement={inputEl}
      bind:value={searchQuery}
      placeholder={searchScope === 'name'
        ? 'Buscar emote por nombre...'
        : searchScope === 'tag'
          ? 'Buscar emote por tag...'
          : 'Buscar por nombre o tags...'}
      on:focus={() => (isOpen = true)}
      on:keydown={handleKeydown}
    >
      <svg
        slot="leading"
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>

      <svelte:fragment slot="trailing">
        {#if hasActiveFilters}
          <Badge variant="gray" shape="pill">
            {filteredEmotes.length} / {$emotes.length}
          </Badge>
          <Button
            variant="icon"
            size="sm"
            title="Limpiar búsqueda y filtros"
            ariaLabel="Limpiar búsqueda y filtros"
            on:click={(e) => {
              e.detail.stopPropagation();
              clearAllFilters();
            }}
          >
            ✕
          </Button>
        {/if}
      </svelte:fragment>
    </Input>

    {#if hasActiveFilters}
      <div class="active-filters-bar">
        {#if activePreset !== 'all'}
          <Chip
            variant="amber"
            pill
            removable
            on:click={() => (activePreset = 'all')}
            on:remove={() => (activePreset = 'all')}
          >
            {activePreset === 'hot'
              ? '🔥 Más usados'
              : activePreset === 'new'
                ? '🍃 Nuevos'
                : activePreset === 'animated'
                  ? '🎞️ Animados (GIF)'
                  : '🖼️ Imágenes'}
          </Chip>
        {/if}

        {#if searchScope !== 'all'}
          <Chip
            variant="green"
            pill
            removable
            on:click={() => (searchScope = 'all')}
            on:remove={() => (searchScope = 'all')}
          >
            {searchScope === 'name' ? '🔤 Solo Nombre' : '🏷️ Solo Tags'}
          </Chip>
        {/if}

        {#each selectedTags as tag}
          <Chip
            variant="indigo"
            pill
            removable
            on:click={() => toggleTag(tag)}
            on:remove={() => toggleTag(tag)}
          >
            🏷️ {tag}
          </Chip>
        {/each}
      </div>
    {/if}

    <Dropdown bind:open={isOpen} maxHeight="420px">
      <div class="dropdown-section">
        <div class="section-heading">ESTOY BUSCANDO...</div>
        <div class="preset-chips-grid">
          <Chip
            variant="hot"
            active={activePreset === 'hot'}
            on:click={() => togglePreset('hot')}
          >
            <svg
              slot="icon"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"
              ></path>
            </svg>
            Más usados
          </Chip>

          <Chip
            variant="green"
            active={activePreset === 'new'}
            on:click={() => togglePreset('new')}
          >
            <svg
              slot="icon"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"
              ></path>
              <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"></path>
            </svg>
            Nuevos
          </Chip>

          <Chip
            active={activePreset === 'animated'}
            on:click={() => togglePreset('animated')}
          >
            <svg
              slot="icon"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"></rect>
              <line x1="7" y1="2" x2="7" y2="22"></line>
              <line x1="17" y1="2" x2="17" y2="22"></line>
              <line x1="2" y1="12" x2="22" y2="12"></line>
            </svg>
            Animados (GIF)
          </Chip>

          <Chip
            active={activePreset === 'static'}
            on:click={() => togglePreset('static')}
          >
            <svg
              slot="icon"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
              <circle cx="8.5" cy="8.5" r="1.5"></circle>
              <polyline points="21 15 16 10 5 21"></polyline>
            </svg>
            Imágenes
          </Chip>

          <Chip
            variant="sky"
            active={searchScope === 'name'}
            on:click={() => toggleScope('name')}
          >
            <svg
              slot="icon"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="4 7 4 4 20 4 20 7"></polyline>
              <line x1="9" y1="20" x2="15" y2="20"></line>
              <line x1="12" y1="4" x2="12" y2="20"></line>
            </svg>
            Por Nombre
          </Chip>

          <Chip
            variant="sky"
            active={searchScope === 'tag'}
            on:click={() => toggleScope('tag')}
          >
            <svg
              slot="icon"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"
              ></path>
              <line x1="7" y1="7" x2="7.01" y2="7"></line>
            </svg>
            Por Tag
          </Chip>
        </div>
      </div>

      {#if matchingTags.length > 0}
        <div class="dropdown-section">
          <div class="section-heading">TAGS DISPONIBLES</div>
          <div class="tags-pills-list">
            {#each matchingTags.slice(0, 14) as tag}
              {@const isTagSelected = selectedTags.some(
                (t) => t.toLowerCase() === tag.toLowerCase()
              )}
              <Chip
                variant="indigo"
                active={isTagSelected}
                count={tagCounts.get(tag) ?? 0}
                on:click={() => toggleTag(tag)}
              >
                <svg
                  slot="icon"
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path
                    d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"
                  ></path>
                  <line x1="7" y1="7" x2="7.01" y2="7"></line>
                </svg>
                {tag}
              </Chip>
            {/each}
          </div>
        </div>
      {/if}

      {#if recentSearches.length > 0}
        <div class="dropdown-section">
          <div class="section-heading">BÚSQUEDAS RECIENTES</div>
          <div class="recent-list">
            {#each recentSearches as item}
              <div class="recent-row">
                <button
                  type="button"
                  class="recent-main-btn"
                  on:click={() => applyRecent(item)}
                >
                  <span class="recent-icon">
                    {#if item.type === 'Tag'}
                      🏷️
                    {:else if item.type === 'Nombre'}
                      🔤
                    {:else}
                      🕒
                    {/if}
                  </span>
                  <span class="recent-query">{item.query}</span>
                  <span class="recent-type">{item.type}</span>
                </button>
                <Button
                  variant="icon"
                  size="sm"
                  title="Quitar de recientes"
                  ariaLabel="Quitar de recientes"
                  on:click={(e) => {
                    e.detail.stopPropagation();
                    removeRecentSearch(item);
                  }}
                >
                  ✕
                </Button>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </Dropdown>
  </div>
</div>

<style>
  .search-filter-wrapper {
    position: relative;
    width: 100%;
    max-width: 650px;
    margin: 0 auto 18px auto;
    z-index: 50;
  }
  .search-container {
    position: relative;
    z-index: 50;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .active-filters-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 0 6px;
  }
  .dropdown-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .section-heading {
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.6px;
    color: #9ca3af;
    text-transform: uppercase;
  }
  .preset-chips-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .tags-pills-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    max-height: 120px;
    overflow-y: auto;
  }
  .recent-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .recent-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-radius: 8px;
    transition: background 0.12s ease;
  }
  .recent-row:hover {
    background: rgba(156, 163, 175, 0.1);
  }
  .recent-main-btn {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    background: transparent;
    border: none;
    color: inherit;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
  }
  .recent-icon {
    font-size: 14px;
    line-height: 1;
  }
  .recent-query {
    flex: 1;
    font-weight: 500;
  }
  .recent-type {
    font-size: 12px;
    color: #9ca3af;
  }
</style>
