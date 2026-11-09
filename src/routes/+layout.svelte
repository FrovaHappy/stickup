<script lang="ts">
  import "../global.css";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { House, Settings, Plus, ScanSearch } from "@lucide/svelte";
  import { Logo, Toast } from "$lib/components/system-ui";
  import AddEmoteModal from "$lib/components/AddEmoteModal.svelte";
  import UpdateModal from "$lib/components/UpdateModal.svelte";
  import { checkForUpdates, availableUpdate } from "$lib/stores/updater";

  let addModalOpen = false;

  onMount(() => {
    // Comprobación periódica no invasiva al inicio (respetando margen estricto de 24h)
    checkForUpdates(false);
  });
</script>

<div class="app-layout">
  <aside class="sidebar">
    <div class="sidebar-top">
      <a
        href="/"
        class="nav-btn"
        class:active={$page.url.pathname === "/"}
        title="Inicio"
        aria-label="Inicio"
      >
        <House size={22} strokeWidth={2.2} />
      </a>
      <a
        href="/scanner"
        class="nav-btn"
        class:active={$page.url.pathname.startsWith("/scanner")}
        title="Escáner de Imágenes"
        aria-label="Escáner de Imágenes"
      >
        <ScanSearch size={22} strokeWidth={2.2} />
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
        class="nav-btn nav-btn-relative"
        class:active={$page.url.pathname.startsWith("/settings")}
        title={$availableUpdate?.hasUpdate ? "Configuración (Actualización disponible)" : "Configuración"}
        aria-label="Configuración"
      >
        <Settings size={22} strokeWidth={2.2} />
        {#if $availableUpdate?.hasUpdate}
          <span class="update-indicator-dot" class:hot={$availableUpdate.isHot}></span>
        {/if}
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
  <UpdateModal />
  <Toast />
</div>

<style>
  .app-layout {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: var(--color-bg-app);
  }

  .sidebar {
    width: 58px;
    min-width: 58px;
    height: 100vh;
    background-color: var(--color-bg-sidebar);
    border-right: 1px solid var(--color-border-light);
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
    color: var(--color-text-dim);
    text-decoration: none;
    transition: all 0.2s ease;
    background: transparent;
    border: none;
    cursor: pointer;
  }

  .nav-btn:hover {
    color: var(--color-text-main);
    background: var(--color-bg-hover);
  }

  .nav-btn-relative {
    position: relative;
  }

  .update-indicator-dot {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--color-orange-primary, #f97316);
    box-shadow: 0 0 6px var(--color-orange-primary, #f97316);
  }

  .update-indicator-dot.hot {
    background-color: #ef4444;
    box-shadow: 0 0 8px #ef4444;
    animation: pulse-hot 1.5s infinite;
  }

  @keyframes pulse-hot {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.75;
      transform: scale(1.25);
    }
  }

  .nav-btn.active {
    color: var(--color-orange-primary);
    background: rgba(var(--color-orange-rgb), 0.12);
  }

  .nav-btn.add-btn:hover {
    color: var(--color-orange-primary);
    background: rgba(var(--color-orange-rgb), 0.1);
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
    background-color: var(--color-bg-surface);
  }
</style>
