<script lang="ts">
  import { onMount } from 'svelte';
  import type { Emote } from '$lib/types/emote';
  import emotes, { allTags, isEmoteNewByDate, topHotEmoteIds } from '$lib/stores/emotes';
  import { isAnimatedEmote } from '$lib/utils/format';

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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isOpen = false;
      inputEl?.blur();
    } else if (e.key === 'Enter' && searchQuery.trim()) {
      const type: RecentSearch['type'] =
        searchScope === 'name' ? 'Nombre' : searchScope === 'tag' ? 'Tag' : 'Búsqueda';
      saveRecentSearch(searchQuery.trim(), type);
      isOpen = false;
    }
  }

  // Mapa de conteo de emotes por tag
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

  // Tags sugeridos en el panel según el texto buscado
  $: matchingTags = ($allTags || []).filter((t) => {
    if (!searchQuery.trim()) return true;
    return t.toLowerCase().includes(searchQuery.trim().toLowerCase());
  });

  $: hasActiveFilters =
    searchQuery.trim() !== '' ||
    activePreset !== 'all' ||
    searchScope !== 'all' ||
    selectedTags.length > 0;

  // Filtrado y ordenamiento reactivo de emotes
  $: {
    const q = searchQuery.trim().toLowerCase();
    let result = $emotes.filter((e) => {
      // 1. Filtro por preset establecido
      if (activePreset === 'hot' && !$topHotEmoteIds.has(e.id)) return false;
      if (activePreset === 'new' && !isEmoteNewByDate(e.created_at)) return false;
      if (activePreset === 'animated' && !isAnimatedEmote(e)) return false;
      if (activePreset === 'static' && isAnimatedEmote(e)) return false;

      // 2. Filtro por tags seleccionados explícitamente
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

      // 3. Filtro por texto (nombre y/o tags)
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

    // Si el filtro activo es "Más usados", ordenar por popularidad
    if (activePreset === 'hot') {
      result = [...result].sort((a, b) => {
        const scoreA = (a.copy_count ?? 0) * 3 + (a.interaction_count ?? 0);
        const scoreB = (b.copy_count ?? 0) * 3 + (b.interaction_count ?? 0);
        if (scoreB !== scoreA) return scoreB - scoreA;
        return (b.copy_count ?? 0) - (a.copy_count ?? 0);
      });
    }

    filteredEmotes = result;
  }
</script>

<div class="search-filter-wrapper">
  {#if isOpen}
    <div
      class="search-backdrop"
      on:click={() => (isOpen = false)}
      role="presentation"
    ></div>
  {/if}

  <div class="search-container {isOpen ? 'is-open' : ''}">
    <div class="search-input-bar" on:click={() => inputEl?.focus()} role="presentation">
      <svg
        class="search-icon"
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

      <input
        bind:this={inputEl}
        type="text"
        class="search-input"
        placeholder={searchScope === 'name'
          ? 'Buscar emote por nombre...'
          : searchScope === 'tag'
            ? 'Buscar emote por tag...'
            : 'Buscar por nombre o tags...'}
        bind:value={searchQuery}
        on:focus={() => (isOpen = true)}
        on:keydown={handleKeydown}
      />

      {#if hasActiveFilters}
        <span class="results-counter">
          {filteredEmotes.length} / {$emotes.length}
        </span>
        <button
          type="button"
          class="clear-btn"
          title="Limpiar búsqueda y filtros"
          on:click|stopPropagation={clearAllFilters}
        >
          ✕
        </button>
      {/if}
    </div>

    {#if hasActiveFilters}
      <div class="active-filters-bar">
        {#if activePreset !== 'all'}
          <button
            type="button"
            class="active-chip preset-chip"
            on:click={() => (activePreset = 'all')}
          >
            <span>
              {activePreset === 'hot'
                ? '🔥 Más usados'
                : activePreset === 'new'
                  ? '🍃 Nuevos'
                  : activePreset === 'animated'
                    ? '🎞️ Animados (GIF)'
                    : '🖼️ Imágenes'}
            </span>
            <span class="chip-close">✕</span>
          </button>
        {/if}

        {#if searchScope !== 'all'}
          <button
            type="button"
            class="active-chip scope-chip"
            on:click={() => (searchScope = 'all')}
          >
            <span>{searchScope === 'name' ? '🔤 Solo Nombre' : '🏷️ Solo Tags'}</span>
            <span class="chip-close">✕</span>
          </button>
        {/if}

        {#each selectedTags as tag}
          <button
            type="button"
            class="active-chip tag-chip"
            on:click={() => toggleTag(tag)}
          >
            <span>🏷️ {tag}</span>
            <span class="chip-close">✕</span>
          </button>
        {/each}
      </div>
    {/if}

    {#if isOpen}
      <div class="filter-dropdown">
        <div class="dropdown-section">
          <div class="section-heading">ESTOY BUSCANDO...</div>
          <div class="preset-chips-grid">
            <button
              type="button"
              class="filter-option-chip {activePreset === 'hot' ? 'selected hot' : ''}"
              on:click={() => togglePreset('hot')}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"></path>
              </svg>
              <span>Más usados</span>
            </button>

            <button
              type="button"
              class="filter-option-chip {activePreset === 'new' ? 'selected new' : ''}"
              on:click={() => togglePreset('new')}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"></path>
                <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"></path>
              </svg>
              <span>Nuevos</span>
            </button>

            <button
              type="button"
              class="filter-option-chip {activePreset === 'animated' ? 'selected' : ''}"
              on:click={() => togglePreset('animated')}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"></rect>
                <line x1="7" y1="2" x2="7" y2="22"></line>
                <line x1="17" y1="2" x2="17" y2="22"></line>
                <line x1="2" y1="12" x2="22" y2="12"></line>
              </svg>
              <span>Animados (GIF)</span>
            </button>

            <button
              type="button"
              class="filter-option-chip {activePreset === 'static' ? 'selected' : ''}"
              on:click={() => togglePreset('static')}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <circle cx="8.5" cy="8.5" r="1.5"></circle>
                <polyline points="21 15 16 10 5 21"></polyline>
              </svg>
              <span>Imágenes</span>
            </button>

            <button
              type="button"
              class="filter-option-chip {searchScope === 'name' ? 'selected scope' : ''}"
              on:click={() => toggleScope('name')}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="4 7 4 4 20 4 20 7"></polyline>
                <line x1="9" y1="20" x2="15" y2="20"></line>
                <line x1="12" y1="4" x2="12" y2="20"></line>
              </svg>
              <span>Por Nombre</span>
            </button>

            <button
              type="button"
              class="filter-option-chip {searchScope === 'tag' ? 'selected scope' : ''}"
              on:click={() => toggleScope('tag')}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
                <line x1="7" y1="7" x2="7.01" y2="7"></line>
              </svg>
              <span>Por Tag</span>
            </button>
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
                <button
                  type="button"
                  class="tag-pill-item {isTagSelected ? 'selected' : ''}"
                  on:click={() => toggleTag(tag)}
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
                    <line x1="7" y1="7" x2="7.01" y2="7"></line>
                  </svg>
                  <span>{tag}</span>
                  <span class="tag-pill-count">{tagCounts.get(tag) ?? 0}</span>
                </button>
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
                  <button
                    type="button"
                    class="recent-remove-btn"
                    title="Quitar de recientes"
                    on:click|stopPropagation={() => removeRecentSearch(item)}
                  >
                    ✕
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
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
  .search-backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
  }
  .search-container {
    position: relative;
    z-index: 50;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .search-input-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    background: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 9999px;
    padding: 10px 16px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
    cursor: text;
  }
  .search-container.is-open .search-input-bar,
  .search-input-bar:focus-within {
    border-color: #6366f1;
    box-shadow: 0 4px 16px rgba(99, 102, 241, 0.18);
  }
  @media (prefers-color-scheme: dark) {
    .search-input-bar {
      background: #232327;
      border-color: rgba(255, 255, 255, 0.12);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    }
  }
  .search-icon {
    color: #9ca3af;
    flex-shrink: 0;
  }
  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 14px;
    color: inherit;
    outline: none;
    min-width: 0;
  }
  .search-input::placeholder {
    color: #9ca3af;
  }
  .results-counter {
    font-size: 11px;
    font-weight: 700;
    color: #9ca3af;
    background: rgba(156, 163, 175, 0.15);
    padding: 2px 8px;
    border-radius: 999px;
    white-space: nowrap;
  }
  .clear-btn {
    background: rgba(156, 163, 175, 0.2);
    border: none;
    color: #9ca3af;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    font-size: 11px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
  }
  .clear-btn:hover {
    background: rgba(239, 68, 68, 0.25);
    color: #ef4444;
  }
  .active-filters-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 0 6px;
  }
  .active-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 999px;
    border: 1px solid transparent;
    cursor: pointer;
    transition: transform 0.1s ease, opacity 0.15s ease;
  }
  .active-chip:hover {
    opacity: 0.85;
  }
  .preset-chip {
    background: rgba(245, 158, 11, 0.18);
    border-color: rgba(245, 158, 11, 0.4);
    color: #fbbf24;
  }
  .scope-chip {
    background: rgba(16, 185, 129, 0.18);
    border-color: rgba(16, 185, 129, 0.4);
    color: #34d399;
  }
  .tag-chip {
    background: rgba(99, 102, 241, 0.18);
    border-color: rgba(99, 102, 241, 0.4);
    color: #a5b4fc;
  }
  .chip-close {
    font-size: 10px;
    opacity: 0.8;
  }
  .filter-dropdown {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    right: 0;
    background: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 16px;
    padding: 16px;
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.18);
    display: flex;
    flex-direction: column;
    gap: 16px;
    text-align: left;
    animation: dropIn 0.14s ease-out;
  }
  @media (prefers-color-scheme: dark) {
    .filter-dropdown {
      background: #1f1f23;
      border-color: rgba(255, 255, 255, 0.1);
      box-shadow: 0 20px 40px rgba(0, 0, 0, 0.55);
    }
  }
  @keyframes dropIn {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
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
  .filter-option-chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 7px 12px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 600;
    background: rgba(156, 163, 175, 0.12);
    border: 1px solid rgba(156, 163, 175, 0.2);
    color: inherit;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .filter-option-chip:hover {
    background: rgba(99, 102, 241, 0.12);
    border-color: rgba(99, 102, 241, 0.35);
  }
  .filter-option-chip.selected {
    background: #4f46e5;
    border-color: #6366f1;
    color: #ffffff;
    box-shadow: 0 2px 10px rgba(99, 102, 241, 0.35);
  }
  .filter-option-chip.selected.hot {
    background: linear-gradient(135deg, #f59e0b, #ef4444);
    border-color: #f59e0b;
    box-shadow: 0 2px 10px rgba(245, 158, 11, 0.35);
  }
  .filter-option-chip.selected.new {
    background: #10b981;
    border-color: #34d399;
    box-shadow: 0 2px 10px rgba(16, 185, 129, 0.35);
  }
  .filter-option-chip.selected.scope {
    background: #0284c7;
    border-color: #38bdf8;
    box-shadow: 0 2px 10px rgba(2, 132, 199, 0.35);
  }
  .tags-pills-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    max-height: 120px;
    overflow-y: auto;
  }
  .tag-pill-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: 7px;
    font-size: 12px;
    font-weight: 600;
    background: rgba(99, 102, 241, 0.1);
    border: 1px solid rgba(99, 102, 241, 0.25);
    color: inherit;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .tag-pill-item:hover {
    background: rgba(99, 102, 241, 0.22);
    border-color: rgba(99, 102, 241, 0.45);
  }
  .tag-pill-item.selected {
    background: #4f46e5;
    border-color: #6366f1;
    color: #ffffff;
  }
  .tag-pill-count {
    font-size: 10px;
    opacity: 0.7;
    background: rgba(0, 0, 0, 0.15);
    padding: 1px 5px;
    border-radius: 999px;
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
  .recent-remove-btn {
    background: transparent;
    border: none;
    color: #9ca3af;
    padding: 6px 10px;
    font-size: 11px;
    cursor: pointer;
    border-radius: 6px;
  }
  .recent-remove-btn:hover {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.12);
  }
</style>

