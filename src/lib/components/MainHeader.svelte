<script lang="ts">
  import { onMount } from "svelte";
  import {
    Search,
    X,
    ArrowDownAZ,
    ArrowUpZA,
    CalendarArrowDown,
    CalendarArrowUp,
    Flame,
    Grid2x2,
    Check,
    Bell,
    BellOff,
    CheckCheck,
    Trash2,
    Info,
    CircleCheck,
    TriangleAlert,
    CircleAlert,
    Images,
    Image,
    Film,
    Tag,
    TagX,
    ChevronLeft,
    ChevronRight,
  } from "@lucide/svelte";
  import {
    gridSize,
    setGridSize,
    sortOption,
    setSortOption,
    formatFilter,
    setFormatFilter,
  } from "$lib/stores/preferences";
  import type {
    GridSize,
    SortOption,
    EmoteFormatFilter,
  } from "$lib/types/preferences";
  import { formatSearchQueryInput } from "$lib/utils/format";
  import {
    notifications,
    unreadCount,
    markAllAsRead,
    markAsRead,
    deleteNotification,
    clearNotifications,
    formatTimeAgo,
  } from "$lib/stores/notifications";
  import emotes, { allTags } from "$lib/stores/emotes";

  export let searchQuery = "";
  export let selectedTags: string[] = [];

  function handleSearchInput(e: Event) {
    const target = e.currentTarget as HTMLInputElement;
    const original = target.value;
    const formatted = formatSearchQueryInput(original);
    searchQuery = formatted;
    target.value = formatted;
  }

  function handleSearchBlur() {
    searchQuery = searchQuery.trim();
  }

  $: tagCounts = (() => {
    const counts = new Map<string, number>();
    for (const e of $emotes) {
      if (e.tag) {
        const parts = e.tag
          .split(",")
          .map((t) => t.trim())
          .filter(Boolean);
        for (const t of parts) {
          const lower = t.toLowerCase();
          counts.set(lower, (counts.get(lower) ?? 0) + 1);
        }
      }
    }
    return counts;
  })();

  $: untaggedCount = ($emotes || []).filter(
    (e) => !e.tag || !e.tag.trim(),
  ).length;

  interface TagItem {
    id: string;
    label: string;
    count: number;
    isUntagged?: boolean;
  }

  $: displayedTags = (() => {
    const items: TagItem[] = ($allTags || []).map((t) => ({
      id: t,
      label: t,
      count: tagCounts.get(t.toLowerCase()) ?? 0,
      isUntagged: false,
    }));

    if (untaggedCount > 0) {
      items.push({
        id: "__untagged__",
        label: "Sin tag",
        count: untaggedCount,
        isUntagged: true,
      });
    }

    return items.sort((a, b) => {
      const aLower = a.id.toLowerCase();
      const bLower = b.id.toLowerCase();
      const aIdx = selectedTags.findIndex((t) => t.toLowerCase() === aLower);
      const bIdx = selectedTags.findIndex((t) => t.toLowerCase() === bLower);
      const aActive = aIdx !== -1;
      const bActive = bIdx !== -1;

      // 1. Elementos seleccionados se mueven al principio de la fila
      if (aActive && !bActive) return -1;
      if (!aActive && bActive) return 1;

      // 2. Si ambos están seleccionados, mantener el orden de selección (más reciente primero)
      if (aActive && bActive) {
        return aIdx - bIdx;
      }

      // 3. Para elementos no seleccionados, "Sin tag" va al final
      if (a.isUntagged) return 1;
      if (b.isUntagged) return -1;

      // 4. Elementos regulares no seleccionados: mayor cantidad a menor cantidad
      if (b.count !== a.count) {
        return b.count - a.count;
      }
      return a.label.localeCompare(b.label);
    });
  })();

  let tagsScrollEl: HTMLElement | null = null;
  let canScrollLeft = false;
  let canScrollRight = false;
  let isDraggingTags = false;
  let dragStartX = 0;
  let dragScrollLeft = 0;
  let hasDragged = false;

  function updateScrollButtons() {
    if (!tagsScrollEl) return;
    const { scrollLeft, scrollWidth, clientWidth } = tagsScrollEl;
    canScrollLeft = scrollLeft > 2;
    canScrollRight = scrollLeft + clientWidth < scrollWidth - 2;
  }

  function scrollTags(offset: number) {
    if (!tagsScrollEl) return;
    tagsScrollEl.scrollBy({ left: offset, behavior: "smooth" });
    setTimeout(updateScrollButtons, 220);
  }

  function isTagActive(tag: string): boolean {
    const lower = tag.toLowerCase();
    return selectedTags.some((t) => t.toLowerCase() === lower);
  }

  function toggleTag(tag: string) {
    if (hasDragged) return;
    const lower = tag.toLowerCase();
    const existingIdx = selectedTags.findIndex(
      (t) => t.toLowerCase() === lower,
    );
    if (existingIdx !== -1) {
      selectedTags = selectedTags.filter((_, i) => i !== existingIdx);
    } else {
      selectedTags = [tag, ...selectedTags];
      if (tagsScrollEl) {
        tagsScrollEl.scrollTo({ left: 0, behavior: "smooth" });
        setTimeout(updateScrollButtons, 220);
      }
    }
  }

  function onTagsMouseUp() {
    isDraggingTags = false;
    setTimeout(() => {
      hasDragged = false;
    }, 50);
  }

  function onTagsMouseDown(e: MouseEvent) {
    if (e.button !== 0 || !tagsScrollEl) return;
    isDraggingTags = true;
    hasDragged = false;
    dragStartX = e.clientX;
    dragScrollLeft = tagsScrollEl.scrollLeft;

    const onMove = (ev: MouseEvent) => {
      if (!isDraggingTags || !tagsScrollEl) return;
      const walk = ev.clientX - dragStartX;
      if (Math.abs(walk) > 5) {
        hasDragged = true;
      }
      if (hasDragged) {
        tagsScrollEl.scrollLeft = dragScrollLeft - walk;
        updateScrollButtons();
      }
    };

    const onUp = () => {
      onTagsMouseUp();
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  function tagsWheelAction(node: HTMLElement) {
    const onWheel = (e: WheelEvent) => {
      if (e.deltaY !== 0 || e.deltaX !== 0) {
        e.preventDefault();
        e.stopPropagation();
        node.scrollLeft += e.deltaY !== 0 ? e.deltaY : e.deltaX;
        updateScrollButtons();
      }
    };
    node.addEventListener("wheel", onWheel, { passive: false });
    return {
      destroy() {
        node.removeEventListener("wheel", onWheel);
      },
    };
  }

  onMount(() => {
    updateScrollButtons();
  });

  $: if (tagsScrollEl && displayedTags.length > 0) {
    setTimeout(updateScrollButtons, 50);
  }

  let filterMenuOpen = false;
  let sortMenuOpen = false;
  let gridMenuOpen = false;
  let notificationsMenuOpen = false;

  const FILTER_ITEMS = [
    {
      id: "all" as EmoteFormatFilter,
      label: "Todos",
      icon: Images,
      desc: "Mostrar GIFs y estáticos",
    },
    {
      id: "static" as EmoteFormatFilter,
      label: "Estáticos",
      icon: Image,
      desc: "Solo imágenes estáticas",
    },
    {
      id: "gif" as EmoteFormatFilter,
      label: "GIFs / Animados",
      icon: Film,
      desc: "Solo emotes animados",
    },
  ];

  $: currentFilterItem =
    FILTER_ITEMS.find((f) => f.id === $formatFilter) ?? FILTER_ITEMS[0];
  $: CurrentFilterIcon = currentFilterItem.icon;

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
  $: CurrentSortIcon = currentSortItem.icon;

  const GRID_SIZES: { id: GridSize; label: string; desc: string }[] = [
    { id: "small", label: "Pequeño", desc: "Vista compacta" },
    { id: "medium", label: "Mediano", desc: "Tamaño estándar" },
    { id: "large", label: "Grande", desc: "Vista detallada" },
  ];

  function toggleFilterMenu(e: MouseEvent) {
    e.stopPropagation();
    filterMenuOpen = !filterMenuOpen;
    sortMenuOpen = false;
    gridMenuOpen = false;
    notificationsMenuOpen = false;
  }

  function toggleSortMenu(e: MouseEvent) {
    e.stopPropagation();
    sortMenuOpen = !sortMenuOpen;
    filterMenuOpen = false;
    gridMenuOpen = false;
    notificationsMenuOpen = false;
  }

  function toggleGridMenu(e: MouseEvent) {
    e.stopPropagation();
    gridMenuOpen = !gridMenuOpen;
    filterMenuOpen = false;
    sortMenuOpen = false;
    notificationsMenuOpen = false;
  }

  function toggleNotificationsMenu(e: MouseEvent) {
    e.stopPropagation();
    notificationsMenuOpen = !notificationsMenuOpen;
    filterMenuOpen = false;
    sortMenuOpen = false;
    gridMenuOpen = false;
  }

  function selectFilter(opt: EmoteFormatFilter) {
    setFormatFilter(opt);
    filterMenuOpen = false;
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
    filterMenuOpen = false;
    sortMenuOpen = false;
    gridMenuOpen = false;
    notificationsMenuOpen = false;
  }
</script>

<svelte:window
  on:click={closeAllMenus}
  on:mouseup={onTagsMouseUp}
  on:resize={updateScrollButtons}
/>

<header class="main-header">
  <div class="header-main-row">
    <div class="header-left-spacer"></div>

    <!-- Barra de búsqueda central en formato píldora -->
    <div class="search-wrapper">
      <div class="search-pill">
        <Search size={16} class="search-icon" />
        <input
          type="text"
          class="search-input"
          placeholder="Busca por nombre o por tags..."
          value={searchQuery}
          on:input={handleSearchInput}
          on:blur={handleSearchBlur}
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

    <!-- Botones de acción a la derecha: Filtro, Ordenar y Tamaño de Grid -->
    <div class="header-actions">
      <!-- Menú de Filtro de Formato (Todos / Estáticos / GIFs) -->
      <div class="action-dropdown-container">
        <button
          type="button"
          class="action-btn"
          class:active={filterMenuOpen || $formatFilter !== "all"}
          title={`Filtro: ${currentFilterItem.label}`}
          aria-label={`Filtro: ${currentFilterItem.label}`}
          on:click={toggleFilterMenu}
        >
          <CurrentFilterIcon size={20} strokeWidth={2} />
          {#if $formatFilter !== "all"}
            <span class="active-filter-dot" aria-hidden="true"></span>
          {/if}
        </button>

        {#if filterMenuOpen}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
          <div
            class="dropdown-popover filter-popover"
            on:click|stopPropagation
            role="menu"
            tabindex="-1"
          >
            <div class="popover-header">Filtrar por formato</div>
            {#each FILTER_ITEMS as opt}
              {@const OptionIcon = opt.icon}
              <button
                type="button"
                class="popover-item"
                class:selected={$formatFilter === opt.id}
                role="menuitem"
                on:click={() => selectFilter(opt.id)}
              >
                <div class="popover-item-left">
                  <OptionIcon
                    size={18}
                    strokeWidth={2}
                    class={`sort-option-icon ${$formatFilter === opt.id ? "is-active" : ""}`}
                  />
                  <div class="item-text">
                    <span class="item-title">{opt.label}</span>
                    <span class="item-desc">{opt.desc}</span>
                  </div>
                </div>
                {#if $formatFilter === opt.id}
                  <Check size={16} class="check-icon" />
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>

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
          <CurrentSortIcon size={20} strokeWidth={2} />
        </button>

        {#if sortMenuOpen}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
          <div
            class="dropdown-popover sort-popover"
            on:click|stopPropagation
            role="menu"
            tabindex="-1"
          >
            <div class="popover-header">Ordenar por</div>
            {#each SORT_ITEMS as opt}
              {@const OptionIcon = opt.icon}
              <button
                type="button"
                class="popover-item"
                class:selected={$sortOption === opt.id}
                role="menuitem"
                on:click={() => selectSort(opt.id)}
              >
                <div class="popover-item-left">
                  <OptionIcon
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
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
          <div
            class="dropdown-popover right-aligned"
            on:click|stopPropagation
            role="menu"
            tabindex="-1"
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

      <!-- Menú de Registro de Notificaciones -->
      <div class="action-dropdown-container">
        <button
          type="button"
          class="action-btn"
          class:active={notificationsMenuOpen}
          title="Registro de notificaciones"
          aria-label="Registro de notificaciones"
          on:click={toggleNotificationsMenu}
        >
          <Bell size={20} strokeWidth={2} />
          {#if $unreadCount > 0}
            <span class="unread-badge"
              >{$unreadCount > 9 ? "9+" : $unreadCount}</span
            >
          {/if}
        </button>

        {#if notificationsMenuOpen}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
          <div
            class="dropdown-popover right-aligned notifications-popover"
            on:click|stopPropagation
            role="menu"
            tabindex="-1"
          >
            <div class="notif-header">
              <div class="notif-header-left">
                <span class="notif-header-title">Notificaciones</span>
                {#if $notifications.length > 0}
                  <span class="notif-count-pill">{$notifications.length}</span>
                {/if}
              </div>
              {#if $notifications.length > 0}
                <div class="notif-header-actions">
                  {#if $unreadCount > 0}
                    <button
                      type="button"
                      class="notif-action-icon-btn"
                      title="Marcar todas como leídas"
                      aria-label="Marcar todas como leídas"
                      on:click={markAllAsRead}
                    >
                      <CheckCheck size={16} strokeWidth={2} />
                    </button>
                  {/if}
                  <button
                    type="button"
                    class="notif-action-icon-btn danger"
                    title="Vaciar historial"
                    aria-label="Vaciar historial"
                    on:click={clearNotifications}
                  >
                    <Trash2 size={15} strokeWidth={2} />
                  </button>
                </div>
              {/if}
            </div>

            <div class="notif-list">
              {#if $notifications.length === 0}
                <div class="notif-empty-state">
                  <BellOff
                    size={30}
                    strokeWidth={1.8}
                    class="notif-empty-icon"
                  />
                  <span class="notif-empty-title">Sin notificaciones</span>
                  <span class="notif-empty-subtitle"
                    >El historial de avisos y copias aparecerá aquí.</span
                  >
                </div>
              {:else}
                {#each $notifications as notif (notif.id)}
                  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
                  <div
                    class="notif-item"
                    class:unread={!notif.read}
                    role="button"
                    tabindex="0"
                    on:click={() => markAsRead(notif.id)}
                  >
                    <div class="notif-item-icon-box {notif.variant}">
                      {#if notif.variant === "error"}
                        <CircleAlert size={16} strokeWidth={2} />
                      {:else if notif.variant === "warning"}
                        <TriangleAlert size={16} strokeWidth={2} />
                      {:else if notif.variant === "info"}
                        <Info size={16} strokeWidth={2} />
                      {:else}
                        <CircleCheck size={16} strokeWidth={2} />
                      {/if}
                    </div>

                    <div class="notif-item-body">
                      <div class="notif-item-top">
                        <span class="notif-item-title">{notif.title}</span>
                        <span class="notif-item-time"
                          >{formatTimeAgo(notif.timestamp)}</span
                        >
                      </div>
                      <p class="notif-item-message">{notif.message}</p>
                    </div>

                    <button
                      type="button"
                      class="notif-item-delete"
                      title="Eliminar notificación"
                      aria-label="Eliminar notificación"
                      on:click|stopPropagation={() =>
                        deleteNotification(notif.id)}
                    >
                      <X size={13} strokeWidth={2} />
                    </button>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>

  {#if displayedTags.length > 0}
    <div class="tags-bar-row">
      <button
        type="button"
        class="tag-nav-btn left"
        class:is-hidden={!canScrollLeft}
        disabled={!canScrollLeft}
        aria-label="Desplazar tags hacia la izquierda"
        title="Desplazar tags a la izquierda"
        on:click={() => scrollTags(-220)}
      >
        <ChevronLeft size={14} />
      </button>

      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tags-bar-scroll"
        class:is-dragging={isDraggingTags}
        bind:this={tagsScrollEl}
        use:tagsWheelAction
        on:mousedown={onTagsMouseDown}
        on:scroll={updateScrollButtons}
      >
        {#each displayedTags as item (item.id)}
          {@const isActive = isTagActive(item.id)}
          <button
            type="button"
            class="tag-pill"
            class:active={isActive}
            class:untagged-pill={item.isUntagged}
            on:click={() => toggleTag(item.id)}
            title={item.isUntagged
              ? isActive
                ? "Quitar filtro sin tag"
                : "Filtrar emotes sin tag"
              : isActive
                ? `Quitar filtro "${item.label}"`
                : `Filtrar por tag "${item.label}"`}
          >
            {#if isActive}
              <Check
                size={11}
                strokeWidth={2.6}
                class="tag-pill-icon tag-pill-check"
              />
            {:else if item.isUntagged}
              <TagX size={11} class="tag-pill-icon" />
            {:else}
              <Tag size={11} class="tag-pill-icon" />
            {/if}
            <span class="tag-pill-name">{item.label}</span>
            {#if item.count > 0}
              <span class="tag-pill-count">{item.count}</span>
            {/if}
          </button>
        {/each}
      </div>

      <button
        type="button"
        class="tag-nav-btn right"
        class:is-hidden={!canScrollRight}
        disabled={!canScrollRight}
        aria-label="Desplazar tags hacia la derecha"
        title="Desplazar tags a la derecha"
        on:click={() => scrollTags(220)}
      >
        <ChevronRight size={14} />
      </button>
    </div>
  {/if}
</header>

<style>
  .main-header {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px 24px 12px;
    background: rgba(26, 26, 29, 0.82);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    position: sticky;
    top: 0;
    z-index: 30;
  }

  .header-main-row {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .tags-bar-row {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    position: relative;
  }

  .tag-nav-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: #242427;
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--color-text-muted, #9ca3af);
    cursor: pointer;
    flex-shrink: 0;
    transition: all 0.2s ease;
  }

  .tag-nav-btn:hover:not(:disabled) {
    background: #2d2d31;
    color: var(--color-text-main, #f3f4f6);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .tag-nav-btn:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 2px;
  }

  .tag-nav-btn.is-hidden {
    opacity: 0;
    pointer-events: none;
    visibility: hidden;
  }

  .tags-bar-scroll {
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: calc(100% - 70px);
    overflow-x: auto;
    scrollbar-width: none;
    -ms-overflow-style: none;
    padding: 2px 4px;
    margin: 0;
    cursor: grab;
    user-select: none;
  }

  .tags-bar-scroll.is-dragging {
    cursor: grabbing;
    scroll-behavior: auto;
  }

  .tags-bar-scroll::-webkit-scrollbar {
    display: none;
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    color: var(--color-text-muted, #a1a1aa);
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
    transition:
      background 0.16s ease,
      border-color 0.16s ease,
      color 0.16s ease,
      box-shadow 0.16s ease,
      transform 0.1s ease;
    flex-shrink: 0;
  }

  .tag-pill:hover {
    background: rgba(255, 255, 255, 0.09);
    border-color: rgba(255, 255, 255, 0.18);
    color: var(--color-text-main, #f4f4f5);
  }

  .tag-pill.active {
    background: var(--color-orange-primary, #f97316);
    border-color: var(--color-orange-primary, #f97316);
    color: #ffffff;
    font-weight: 700;
    box-shadow: 0 2px 10px rgba(249, 115, 22, 0.38);
  }

  .tag-pill.active:hover {
    background: var(--color-orange-hover, #ea580c);
    border-color: var(--color-orange-hover, #ea580c);
    color: #ffffff;
  }

  .tag-pill:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 2px;
  }

  :global(.tag-pill-icon) {
    opacity: 0.8;
    flex-shrink: 0;
  }

  .tag-pill.active :global(.tag-pill-icon) {
    opacity: 1;
    color: #ffffff;
  }

  .tag-pill-name {
    line-height: 1.2;
  }

  .tag-pill-count {
    font-size: 10px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    color: inherit;
    opacity: 0.85;
    transition:
      background 0.16s ease,
      color 0.16s ease;
  }

  .tag-pill.active .tag-pill-count {
    background: rgba(0, 0, 0, 0.24);
    color: #ffffff;
    opacity: 1;
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
    border-color: var(--color-orange-primary, #f97316);
    background: #29292d;
    box-shadow: 0 0 0 2px rgba(var(--color-orange-rgb, 249, 115, 22), 0.18);
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
    position: relative;
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
    color: var(--color-orange-primary, #f97316);
    background: rgba(var(--color-orange-rgb, 249, 115, 22), 0.12);
    border-color: rgba(var(--color-orange-rgb, 249, 115, 22), 0.28);
  }

  .active-filter-dot {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--color-orange-primary, #f97316);
    box-shadow: 0 0 6px rgba(var(--color-orange-rgb, 249, 115, 22), 0.7);
    pointer-events: none;
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

  .dropdown-popover.sort-popover,
  .dropdown-popover.filter-popover {
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
    background: rgba(var(--color-orange-rgb, 249, 115, 22), 0.08);
    color: var(--color-orange-primary, #f97316);
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
    color: var(--color-orange-primary, #f97316);
  }

  .popover-item :global(.check-icon) {
    color: var(--color-orange-primary, #f97316);
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

  /* Notificaciones */
  .unread-badge {
    position: absolute;
    top: 4px;
    right: 4px;
    background-color: var(--color-orange-primary, #f97316);
    color: #ffffff;
    font-size: 10px;
    font-weight: 700;
    line-height: 1;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 999px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 0 0 2px var(--color-bg-app, #1a1a1c);
    pointer-events: none;
  }

  .notifications-popover {
    width: 360px;
    max-width: calc(100vw - 32px);
    max-height: 440px;
    display: flex;
    flex-direction: column;
    padding: 0;
    overflow: hidden;
  }

  .notif-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid
      var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    background: rgba(255, 255, 255, 0.02);
  }

  .notif-header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .notif-header-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-main, #f4f4f5);
  }

  .notif-count-pill {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-muted, #a1a1aa);
  }

  .notif-header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .notif-action-icon-btn {
    background: transparent;
    border: none;
    color: var(--color-text-dim, #71717a);
    cursor: pointer;
    padding: 5px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .notif-action-icon-btn:hover {
    color: var(--color-text-main, #f4f4f5);
    background: rgba(255, 255, 255, 0.08);
  }

  .notif-action-icon-btn.danger:hover {
    color: #f87171;
    background: rgba(239, 68, 68, 0.14);
  }

  .notif-list {
    overflow-y: auto;
    max-height: 380px;
    display: flex;
    flex-direction: column;
  }

  .notif-item {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 14px;
    border-bottom: 1px solid
      var(--color-border-light, rgba(255, 255, 255, 0.06));
    cursor: pointer;
    transition: background-color 0.15s ease;
    text-align: left;
    outline: none;
  }

  .notif-item:last-child {
    border-bottom: none;
  }

  .notif-item:hover {
    background-color: rgba(255, 255, 255, 0.04);
  }

  .notif-item.unread {
    background-color: rgba(var(--color-orange-rgb, 249, 115, 22), 0.05);
  }

  .notif-item.unread::before {
    content: "";
    position: absolute;
    left: 4px;
    top: 50%;
    transform: translateY(-50%);
    width: 3.5px;
    height: 18px;
    border-radius: 2px;
    background-color: var(--color-orange-primary, #f97316);
  }

  .notif-item-icon-box {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .notif-item-icon-box.success {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .notif-item-icon-box.error {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
  }

  .notif-item-icon-box.warning {
    background: rgba(245, 158, 11, 0.15);
    color: #fbbf24;
  }

  .notif-item-icon-box.info {
    background: rgba(14, 165, 233, 0.15);
    color: #38bdf8;
  }

  .notif-item-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .notif-item-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .notif-item-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-main, #f4f4f5);
  }

  .notif-item-time {
    font-size: 11px;
    color: var(--color-text-dim, #71717a);
    white-space: nowrap;
  }

  .notif-item-message {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted, #a1a1aa);
    line-height: 1.4;
    word-break: break-word;
  }

  .notif-item-delete {
    opacity: 0;
    background: transparent;
    border: none;
    color: var(--color-text-dim, #71717a);
    cursor: pointer;
    padding: 3px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      opacity 0.15s ease,
      color 0.15s ease;
  }

  .notif-item:hover .notif-item-delete {
    opacity: 1;
  }

  .notif-item-delete:hover {
    color: #f87171;
    background: rgba(239, 68, 68, 0.14);
  }

  .notif-empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 36px 20px;
    text-align: center;
    gap: 6px;
  }

  :global(.notif-empty-icon) {
    color: var(--color-text-dim, #71717a);
    margin-bottom: 4px;
  }

  .notif-empty-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary, #e4e4e7);
  }

  .notif-empty-subtitle {
    font-size: 12px;
    color: var(--color-text-dim, #71717a);
    max-width: 240px;
    line-height: 1.4;
  }
</style>
