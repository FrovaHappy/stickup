<script lang="ts">
  import { page } from '$app/stores';
  import { House, Settings, Plus } from '@lucide/svelte';
  import { Logo, Toast } from '$lib/components/system-ui';
  import AddEmoteModal from '$lib/components/AddEmoteModal.svelte';

  let addModalOpen = false;
</script>

<div class="app-layout">
  <aside class="sidebar">
    <div class="sidebar-top">
      <a
        href="/"
        class="nav-btn"
        class:active={$page.url.pathname === '/'}
        title="Inicio"
        aria-label="Inicio"
      >
        <House size={22} strokeWidth={2.2} />
      </a>
    </div>

    <div class="sidebar-bottom">
      <!-- Icono + para cargar nuevo elemento (por encima del icono de Settings) -->
      <button
        type="button"
        class="nav-btn add-btn"
        class:active={addModalOpen}
        title="Cargar nuevo emote"
        aria-label="Cargar nuevo emote"
        on:click={() => (addModalOpen = true)}
      >
        <Plus size={22} strokeWidth={2.4} />
      </button>

      <a
        href="/settings"
        class="nav-btn"
        class:active={$page.url.pathname.startsWith('/settings')}
        title="Configuración"
        aria-label="Configuración"
      >
        <Settings size={22} strokeWidth={2.2} />
      </a>

      <div class="brand-badge" title="StickUp - Gestor de Emotes">
        <Logo size={28} interactive />
      </div>
    </div>
  </aside>

  <main class="main-content">
    <slot />
  </main>

  <AddEmoteModal bind:isOpen={addModalOpen} customTrigger />
  <Toast />
</div>

<style>
  :global(:root) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
    font-size: 15px;
    line-height: 1.5;
    color: #e4e4e7;
    background-color: #1a1a1c;
    color-scheme: dark;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    box-sizing: border-box;
  }

  :global(*),
  :global(*::before),
  :global(*::after) {
    box-sizing: inherit;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background-color: #1a1a1c;
    user-select: none;
  }

  /* Custom Yellow Scrollbar as in reference */
  :global(::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: #eab308;
    border-radius: 4px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: #facc15;
  }

  .app-layout {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: #1a1a1c;
  }

  .sidebar {
    width: 58px;
    min-width: 58px;
    height: 100vh;
    background-color: #141416;
    border-right: 1px solid rgba(255, 255, 255, 0.05);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    padding: 16px 0;
    z-index: 20;
    user-select: none;
  }

  .sidebar-top,
  .sidebar-bottom {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .nav-btn {
    width: 42px;
    height: 42px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #71717a;
    text-decoration: none;
    transition: all 0.2s ease;
    background: transparent;
    border: none;
    cursor: pointer;
  }

  .nav-btn:hover {
    color: #f4f4f5;
    background: rgba(255, 255, 255, 0.06);
  }

  .nav-btn.active {
    color: #facc15;
    background: rgba(250, 204, 21, 0.12);
  }

  .nav-btn.add-btn:hover {
    color: #facc15;
    background: rgba(250, 204, 21, 0.1);
  }

  .brand-badge {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: transparent;
    cursor: default;
    transition: transform 0.2s ease;
  }

  .brand-badge:hover {
    transform: scale(1.08);
  }

  .main-content {
    flex: 1;
    height: 100vh;
    overflow-y: auto;
    overflow-x: hidden;
    position: relative;
    background-color: #1c1c1f;
  }
</style>
