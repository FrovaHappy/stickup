<script lang="ts">
  import { createEventDispatcher, onMount, tick } from 'svelte';
  import type { ContextMenuItem } from '$lib/types/system-ui';

  export let open = false;
  export let x = 0;
  export let y = 0;
  export let items: ContextMenuItem[] = [];

  const dispatch = createEventDispatcher<{
    close: void;
    select: ContextMenuItem;
  }>();

  let menuEl: HTMLDivElement | null = null;
  let posX = 0;
  let posY = 0;

  async function adjustPosition() {
    if (!open) return;
    await tick();
    if (!menuEl) return;

    const rect = menuEl.getBoundingClientRect();
    const margin = 8;
    const winW = window.innerWidth;
    const winH = window.innerHeight;

    let targetX = x;
    let targetY = y;

    if (targetX + rect.width > winW - margin) {
      targetX = Math.max(margin, winW - rect.width - margin);
    }
    if (targetY + rect.height > winH - margin) {
      targetY = Math.max(margin, winH - rect.height - margin);
    }

    posX = targetX;
    posY = targetY;
  }

  $: if (open && (x || y)) {
    posX = x;
    posY = y;
    adjustPosition();
  }

  function handleWindowClick(e: MouseEvent) {
    if (!open) return;
    if (menuEl && !menuEl.contains(e.target as Node)) {
      close();
    }
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    if (open && e.key === 'Escape') {
      close();
    }
  }

  function close() {
    open = false;
    dispatch('close');
  }

  function handleItemClick(item: ContextMenuItem) {
    if (item.disabled || item.divider) return;
    close();
    if (item.action) {
      item.action();
    }
    dispatch('select', item);
  }
</script>

<svelte:window on:click={handleWindowClick} on:keydown={handleWindowKeydown} />

{#if open}
  <div
    bind:this={menuEl}
    class="context-menu"
    style="left: {posX}px; top: {posY}px;"
    role="menu"
    tabindex="-1"
    on:contextmenu|preventDefault
  >
    <slot>
      {#each items as item}
        {#if item.divider}
          <div class="menu-divider" role="separator"></div>
        {:else}
          <button
            type="button"
            class="menu-item"
            class:danger={item.danger}
            disabled={item.disabled}
            role="menuitem"
            on:click={() => handleItemClick(item)}
          >
            {#if item.icon}
              <span class="menu-icon">{item.icon}</span>
            {/if}
            <span class="menu-label">{item.label}</span>
            {#if item.shortcut}
              <span class="menu-shortcut">{item.shortcut}</span>
            {/if}
          </button>
        {/if}
      {/each}
    </slot>
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    z-index: 10000;
    min-width: 190px;
    max-width: 320px;
    background: rgba(30, 30, 34, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.14);
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.5), 0 2px 8px rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(12px);
    border-radius: 10px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    color: #f3f4f6;
    user-select: none;
    animation: menu-pop 0.15s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 7px 10px;
    border: none;
    background: transparent;
    color: inherit;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition: background 0.12s ease, color 0.12s ease;
  }

  .menu-item:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
  }

  .menu-item:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .menu-item.danger {
    color: #f87171;
  }

  .menu-item.danger:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.2);
    color: #fca5a5;
  }

  .menu-icon {
    font-size: 14px;
    width: 16px;
    display: inline-flex;
    justify-content: center;
    flex-shrink: 0;
  }

  .menu-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .menu-shortcut {
    font-size: 11px;
    color: #9ca3af;
    background: rgba(255, 255, 255, 0.08);
    padding: 2px 6px;
    border-radius: 4px;
    margin-left: 8px;
    font-family: monospace;
    flex-shrink: 0;
  }

  .menu-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.08);
    margin: 4px 2px;
  }

  @keyframes menu-pop {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(-4px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  @media (prefers-color-scheme: light) {
    .context-menu {
      background: rgba(255, 255, 255, 0.98);
      border-color: rgba(0, 0, 0, 0.12);
      color: #1f2937;
      box-shadow: 0 10px 28px rgba(0, 0, 0, 0.18), 0 2px 8px rgba(0, 0, 0, 0.08);
    }
    .menu-item:hover:not(:disabled) {
      background: rgba(0, 0, 0, 0.06);
    }
    .menu-item.danger {
      color: #dc2626;
    }
    .menu-item.danger:hover:not(:disabled) {
      background: rgba(239, 68, 68, 0.12);
      color: #b91c1c;
    }
    .menu-shortcut {
      background: rgba(0, 0, 0, 0.06);
      color: #6b7280;
    }
    .menu-divider {
      background: rgba(0, 0, 0, 0.08);
    }
  }
</style>

