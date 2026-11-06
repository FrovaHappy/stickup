<script lang="ts">
  import {
    Search,
    X,
    ArrowDownAZ,
    ArrowUpZA,
    CalendarArrowDown,
    CalendarArrowUp,
    Flame,
    Grid2x2,
    Check
  } from "@lucide/svelte";
  import {
    gridSize,
    setGridSize,
    sortOption,
    setSortOption,
  } from "$lib/stores/preferences";
  import type { GridSize, SortOption } from "$lib/types/preferences";

  export let searchQuery = "";

  let sortMenuOpen = false;
  let gridMenuOpen = false;

  const SORT_ITEMS = [
    {
      id: "date-desc" as SortOption,
      label: "Más recientes",
      icon: CalendarArrowDown,
      desc: "Fecha descendente",
    },
    {
      id: "date-asc" as SortOption,
      label: "Más antiguos",
      icon: CalendarArrowUp,
      desc: "Fecha ascendente",
    },
    {
      id: "name-asc" as SortOption,
      label: "Nombre (A - Z)",
      icon: ArrowDownAZ,
      desc: "Alfabético ascendente",
    },
    {
      id: "name-desc" as SortOption,
      label: "Nombre (Z - A)",
      icon: ArrowUpZA,
      desc: "Alfabético descendente",
    },
    {
      id: "copies-desc" as SortOption,
      label: "Más usados / Hot",
      icon: Flame,
      desc: "Mayor cantidad de copias",
    },
  ];

  $: currentSortItem =
    SORT_ITEMS.find((s) => s.id === $sortOption) ?? SORT_ITEMS[0];

  const GRID_SIZES: { id: GridSize; label: string; desc: string }[] = [
    { id: "small", label: "Pequeño", desc: "Vista compacta" },
    { id: "medium", label: "Mediano", desc: "Tamaño estándar" },
    { id: "large", label: "Grande", desc: "Vista detallada" },
  ];

  function toggleSortMenu(e: MouseEvent) {
    e.stopPropagation();
    sortMenuOpen = !sortMenuOpen;
    gridMenuOpen = false;
  }

  function toggleGridMenu(e: MouseEvent) {
    e.stopPropagation();
    gridMenuOpen = !gridMenuOpen;
    sortMenuOpen = false;
  }

  function selectSort(opt: SortOption) {
    setSortOption(opt);
    sortMenuOpen = false;
  }

  function selectGrid(size: GridSize) {
    setGridSize(size);
    gridMenuOpen = false;
  }

  function closeAllMenus() {
    sortMenuOpen = false;
    gridMenuOpen = false;
  }
</script>

<svelte:window on:click={closeAllMenus} />

<header class="main-header">
  <div class="header-left-spacer"></div>

  <!-- Barra de búsqueda central en formato píldora -->
  <div class="search-wrapper">
    <div class="search-pill">
      <Search size={16} class="search-icon" />
      <input
        type="text"
        class="search-input"
        placeholder="Busca por nombre o por tags..."
        bind:value={searchQuery}
      />
      {#if searchQuery.trim()}
        <button
          type="button"
          class="clear-btn"
          title="Borrar búsqueda"
          aria-label="Borrar búsqueda"
          on:click={() => (searchQuery = "")}
        >
          <X size={14} />
        </button>
      {/if}
    </div>
  </div>

  <!-- Botones de acción a la derecha: Ordenar y Tamaño de Grid -->
  <div class="header-actions">
    <!-- Menú de Ordenamiento -->
    <div class="action-dropdown-container">
      <button
        type="button"
        class="action-btn"
        class:active={sortMenuOpen}
        title={`Ordenar: ${currentSortItem.label}`}
        aria-label={`Ordenar: ${currentSortItem.label}`}
        on:click={toggleSortMenu}
      >
        <svelte:component this={currentSortItem.icon} size={20} strokeWidth={2} />
      </button>

      {#if sortMenuOpen}
        <div class="dropdown-popover sort-popover" on:click|stopPropagation role="menu">
          <div class="popover-header">Ordenar por</div>
          {#each SORT_ITEMS as opt}
            <button
              type="button"
              class="popover-item"
              class:selected={$sortOption === opt.id}
              role="menuitem"
              on:click={() => selectSort(opt.id)}
            >
              <div class="popover-item-left">
                <svelte:component
                  this={opt.icon}
                  size={18}
                  strokeWidth={2}
                  class={`sort-option-icon ${$sortOption === opt.id ? "is-active" : ""}`}
                />
                <div class="item-text">
                  <span class="item-title">{opt.label}</span>
                  <span class="item-desc">{opt.desc}</span>
                </div>
              </div>
              {#if $sortOption === opt.id}
                <Check size={16} class="check-icon" />
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Menú de Tamaño de Grid -->
    <div class="action-dropdown-container">
      <button
        type="button"
        class="action-btn"
        class:active={gridMenuOpen}
        title="Cambiar tamaño de grid"
        aria-label="Cambiar tamaño de grid"
        on:click={toggleGridMenu}
      >
        <Grid2x2 size={20} strokeWidth={2} />
      </button>

      {#if gridMenuOpen}
        <div
          class="dropdown-popover right-aligned"
          on:click|stopPropagation
          role="menu"
        >
          <div class="popover-header">Tamaño del grid</div>
          {#each GRID_SIZES as size}
            <button
              type="button"
              class="popover-item"
              class:selected={$gridSize === size.id}
              role="menuitem"
              on:click={() => selectGrid(size.id)}
            >
              <div class="item-text">
                <span class="item-title">{size.label}</span>
                <span class="item-desc">{size.desc}</span>
              </div>
              {#if $gridSize === size.id}
                <Check size={16} class="check-icon" />
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</header>

<style>
  .main-header {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 24px;
    background: rgba(26, 26, 29, 0.82);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    position: sticky;
    top: 0;
    z-index: 30;
  }

  .header-left-spacer {
    width: 80px;
    display: none;
  }

  @media (min-width: 640px) {
    .header-left-spacer {
      display: block;
    }
  }

  .search-wrapper {
    flex: 1;
    max-width: 440px;
    margin: 0 auto;
    display: flex;
    justify-content: center;
  }

  .search-pill {
    width: 100%;
    height: 38px;
    background: #242427;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 9999px;
    display: flex;
    align-items: center;
    padding: 0 14px;
    gap: 10px;
    transition: all 0.2s ease;
  }

  .search-pill:focus-within {
    border-color: #facc15;
    background: #29292d;
    box-shadow: 0 0 0 2px rgba(250, 204, 21, 0.15);
  }

  .search-pill :global(.search-icon) {
    color: #a1a1aa;
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: #f4f4f5;
    font-size: 13.5px;
    outline: none;
    padding: 0;
  }

  .search-input::placeholder {
    color: #71717a;
  }

  .clear-btn {
    background: transparent;
    border: none;
    color: #a1a1aa;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border-radius: 50%;
  }

  .clear-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .action-dropdown-container {
    position: relative;
  }

  .action-btn {
    width: 38px;
    height: 38px;
    border-radius: 8px;
    background: transparent;
    border: 1px solid transparent;
    color: #a1a1aa;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    color: #f4f4f5;
    background: rgba(255, 255, 255, 0.07);
  }

  .action-btn.active {
    color: #facc15;
    background: rgba(250, 204, 21, 0.12);
    border-color: rgba(250, 204, 21, 0.25);
  }

  .dropdown-popover {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    min-width: 190px;
    background: #232326;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    padding: 6px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .dropdown-popover.sort-popover {
    min-width: 240px;
  }

  .popover-header {
    padding: 8px 10px 4px 10px;
    font-size: 11px;
    font-weight: 600;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .popover-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 8px 10px;
    border-radius: 6px;
    background: transparent;
    border: none;
    color: #e4e4e7;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s ease;
  }

  .popover-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .popover-item.selected {
    background: rgba(250, 204, 21, 0.08);
    color: #facc15;
    font-weight: 500;
  }

  .popover-item-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .popover-item :global(.sort-option-icon) {
    color: #a1a1aa;
    flex-shrink: 0;
    transition: color 0.15s ease;
  }

  .popover-item:hover :global(.sort-option-icon) {
    color: #f4f4f5;
  }

  .popover-item.selected :global(.sort-option-icon.is-active) {
    color: #facc15;
  }

  .popover-item :global(.check-icon) {
    color: #facc15;
    margin-left: 8px;
    flex-shrink: 0;
  }

  .item-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item-title {
    font-weight: 500;
    color: inherit;
  }

  .item-desc {
    font-size: 11px;
    color: #71717a;
  }
</style>
