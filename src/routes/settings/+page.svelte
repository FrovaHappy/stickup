<script lang="ts">
  import { onMount } from 'svelte';
  import { ArrowLeft, Database, Sliders, Info, ShieldCheck, Monitor } from '@lucide/svelte';
  import StorageSettings from '$lib/components/StorageSettings.svelte';
  import ImportButton from '$lib/components/ImportButton.svelte';
  import { Button, Badge, Logo } from '$lib/components/system-ui';
  import { customSizes, addCustomSize, removeCustomSize, DEFAULT_SIZE_PRESETS } from '$lib/stores/sizes';
  import { toast } from '$lib/stores/toast';
  import {
    autostartEnabled,
    minimizeToTrayEnabled,
    systemSettingsLoading,
    loadSystemSettings,
    setAutostartSetting,
    setMinimizeToTraySetting
  } from '$lib/stores/systemSettings';

  let newSizeWidth: number | null = null;
  let newSizeHeight: number | null = null;

  onMount(() => {
    loadSystemSettings();
  });

  function handleAddSize() {
    if (!newSizeWidth || !newSizeHeight) {
      toast.error('Especifica un ancho y alto válidos en píxeles');
      return;
    }
    const added = addCustomSize(newSizeWidth, newSizeHeight);
    if (added) {
      toast.success(`✓ Tamaño ${added.label} agregado`);
      newSizeWidth = null;
      newSizeHeight = null;
    } else {
      toast.error('El tamaño ingresado no es válido o ya existe');
    }
  }

  function handleRemoveSize(value: string, label: string) {
    removeCustomSize(value);
    toast.success(`Tamaño ${label} eliminado`);
  }
</script>

<div class="settings-page">
  <!-- Header fijo con estilo translúcido idéntico al patrón de Inicio -->
  <header class="settings-header">
    <div class="header-inner">
      <div class="header-nav">
        <a href="/" class="back-link" title="Volver a Emotes">
          <ArrowLeft size={16} />
          <span>Volver a Emotes</span>
        </a>
      </div>
      <div class="header-title-block">
        <h1 class="page-title">Configuración</h1>
        <p class="page-subtitle">Administra el almacenamiento, copias de seguridad y preferencias de StickUp.</p>
      </div>
    </div>
  </header>

  <!-- Contenido con ancho unificado y sin background en las tarjetas -->
  <div class="settings-content">
    <!-- Sección de Almacenamiento y Respaldos (Información expandida con copias hechas) -->
    <section class="settings-section">
      <div class="section-title-row">
        <Database size={18} class="section-icon" />
        <h2>Almacenamiento y Respaldos</h2>
      </div>
      <StorageSettings />
    </section>

    <!-- Sección de Comportamiento del Sistema (Sin background de tarjeta) -->
    <section class="settings-section">
      <div class="section-title-row">
        <Monitor size={18} class="section-icon" />
        <h2>Comportamiento del Sistema</h2>
      </div>
      <div class="section-card">
        <div class="system-toggles">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-title">Iniciar al arrancar el sistema</span>
              <p class="toggle-desc">
                Inicia StickUp automáticamente al encender tu equipo o iniciar sesión en segundo plano.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={$autostartEnabled}
              class="toggle-switch"
              class:active={$autostartEnabled}
              disabled={$systemSettingsLoading}
              title={$autostartEnabled ? 'Desactivar inicio con el sistema' : 'Activar inicio con el sistema'}
              on:click={() => setAutostartSetting(!$autostartEnabled)}
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          <div class="toggle-separator"></div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-title">Minimizar a la bandeja del sistema</span>
              <p class="toggle-desc">
                Al hacer clic en cerrar (×), mantiene StickUp ejecutándose en segundo plano en la bandeja de iconos del sistema.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={$minimizeToTrayEnabled}
              class="toggle-switch"
              class:active={$minimizeToTrayEnabled}
              disabled={$systemSettingsLoading}
              title={$minimizeToTrayEnabled ? 'Desactivar minimizar en la bandeja' : 'Activar minimizar en la bandeja'}
              on:click={() => setMinimizeToTraySetting(!$minimizeToTrayEnabled)}
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Sección de Mantenimiento y Sincronización (Sin background de tarjeta) -->
    <section class="settings-section">
      <div class="section-title-row">
        <ShieldCheck size={18} class="section-icon" />
        <h2>Sincronización e Importación</h2>
      </div>
      <div class="section-card">
        <div class="sync-row">
          <div class="sync-info">
            <span class="sync-title">Sincronizar y optimizar emotes</span>
            <p class="section-desc">
              Agrega nuevos emotes o sincroniza el directorio con la base de datos convirtiendo imágenes automáticamente a formato WebP optimizado.
            </p>
          </div>
          <div class="sync-action">
            <ImportButton />
          </div>
        </div>
      </div>
    </section>

    <!-- Sección de Tamaños de Portapapeles Personalizados (Sin background de tarjeta) -->
    <section class="settings-section">
      <div class="section-title-row">
        <Sliders size={18} class="section-icon" />
        <h2>Tamaños de Portapapeles Personalizados</h2>
      </div>
      <div class="section-card">
        <p class="section-desc">
          Configura dimensiones personalizadas para copiar emotes con tamaños específicos desde el menú contextual.
        </p>

        <div class="sizes-list">
          <div class="sizes-group">
            <span class="sizes-group-title">Tamaños predeterminados del sistema:</span>
            <div class="chips-row">
              {#each DEFAULT_SIZE_PRESETS as preset}
                <Badge variant="gray" shape="pill">
                  {preset.label}
                </Badge>
              {/each}
            </div>
          </div>

          <div class="sizes-group">
            <span class="sizes-group-title">Tamaños personalizados:</span>
            {#if $customSizes.length === 0}
              <p class="empty-hint">No hay tamaños personalizados creados aún.</p>
            {:else}
              <div class="chips-row">
                {#each $customSizes as custom}
                  <div class="custom-size-item">
                    <Badge variant="blue" shape="pill">
                      {custom.label}
                    </Badge>
                    <button
                      type="button"
                      class="remove-size-btn"
                      title={`Eliminar ${custom.label}`}
                      aria-label={`Eliminar ${custom.label}`}
                      on:click={() => handleRemoveSize(custom.value, custom.label)}
                    >
                      ×
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <div class="add-size-form">
            <span class="form-label">Añadir nuevo tamaño (px):</span>
            <div class="inputs-inline">
              <input
                type="number"
                min="16"
                max="4096"
                placeholder="Ancho (px)"
                class="num-input"
                bind:value={newSizeWidth}
              />
              <span class="x-separator">×</span>
              <input
                type="number"
                min="16"
                max="4096"
                placeholder="Alto (px)"
                class="num-input"
                bind:value={newSizeHeight}
              />
              <Button variant="primary" size="sm" on:click={handleAddSize}>
                Añadir tamaño
              </Button>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Acerca de StickUp (Sin background de tarjeta) -->
    <section class="settings-section">
      <div class="section-title-row">
        <Info size={18} class="section-icon" />
        <h2>Acerca de StickUp</h2>
      </div>
      <div class="section-card">
        <div class="about-card">
          <div class="about-header">
            <Logo size={44} interactive />
            <div class="about-info-col">
              <div class="about-title-row">
                <strong>StickUp - Gestor de Emotes</strong>
                <Badge variant="green" shape="pill">v0.1.1</Badge>
              </div>
              <p class="about-text">
                Diseñado para copiar rápidamente stickers y emotes optimizados a cualquier aplicación de mensajería (Discord, Telegram, WhatsApp, Slack).
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  .settings-page {
    width: 100%;
    min-height: 100%;
    display: flex;
    flex-direction: column;
  }

  /* Header sticky idéntico al patrón de Inicio */
  .settings-header {
    width: 100%;
    position: sticky;
    top: 0;
    z-index: 30;
    background: rgba(26, 26, 29, 0.82);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    padding: 16px 28px;
    box-sizing: border-box;
  }

  .header-inner {
    max-width: 960px;
    width: 100%;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .header-nav {
    display: flex;
    align-items: center;
  }

  .back-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: #a1a1aa;
    text-decoration: none;
    font-size: 13px;
    font-weight: 500;
    transition: color 0.15s ease;
  }

  .back-link:hover {
    color: var(--color-orange-primary, #f97316);
  }

  .header-title-block {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .page-title {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    color: #f4f4f5;
  }

  .page-subtitle {
    margin: 0;
    font-size: 13px;
    color: #a1a1aa;
  }

  /* Contenido principal con ancho unificado y padding coherente */
  .settings-content {
    max-width: 960px;
    width: 100%;
    margin: 0 auto;
    padding: 24px 28px 64px 28px;
    display: flex;
    flex-direction: column;
    gap: 28px;
    box-sizing: border-box;
  }

  .settings-section {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #e4e4e7;
  }

  .section-title-row :global(.section-icon) {
    color: var(--color-orange-primary, #f97316);
  }

  .section-title-row h2 {
    margin: 0;
    font-size: 15.5px;
    font-weight: 600;
  }

  /* Tarjeta sin fondo (background transparente) con borde sutil */
  .section-card {
    width: 100%;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 16px 18px;
    box-sizing: border-box;
  }

  .section-desc {
    margin: 0 0 14px 0;
    font-size: 13px;
    color: #a1a1aa;
    line-height: 1.5;
  }

  /* Sincronización e importación */
  .sync-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    flex-wrap: wrap;
    width: 100%;
  }

  .sync-info {
    flex: 1;
    min-width: 260px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .sync-title {
    font-size: 14px;
    font-weight: 600;
    color: #f4f4f5;
  }

  .sync-action {
    flex-shrink: 0;
  }

  /* Tamaños personalizados */
  .sizes-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .sizes-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sizes-group-title {
    font-size: 11.5px;
    font-weight: 600;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .chips-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }

  .empty-hint {
    margin: 0;
    font-size: 13px;
    color: #71717a;
    font-style: italic;
  }

  .custom-size-item {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(59, 130, 246, 0.15);
    border-radius: 9999px;
    padding-right: 8px;
  }

  .remove-size-btn {
    background: transparent;
    border: none;
    color: #93c5fd;
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .remove-size-btn:hover {
    color: #ef4444;
  }

  .add-size-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .form-label {
    font-size: 12px;
    font-weight: 500;
    color: #a1a1aa;
  }

  .inputs-inline {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .num-input {
    width: 100px;
    padding: 6px 10px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(0, 0, 0, 0.25);
    color: #fff;
    font-size: 13px;
  }

  .num-input:focus {
    outline: none;
    border-color: var(--color-orange-primary, #f97316);
  }

  .x-separator {
    color: #71717a;
    font-weight: bold;
  }

  /* Acerca de */
  .about-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .about-header {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    font-size: 14px;
  }

  .about-info-col {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .about-title-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .about-text {
    margin: 0;
    font-size: 13px;
    color: #a1a1aa;
    line-height: 1.5;
  }

  /* Toggles del sistema */
  .system-toggles {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 24px;
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }

  .toggle-title {
    font-size: 14px;
    font-weight: 600;
    color: #f4f4f5;
  }

  .toggle-desc {
    margin: 0;
    font-size: 13px;
    color: #a1a1aa;
    line-height: 1.4;
  }

  .toggle-separator {
    height: 1px;
    background: rgba(255, 255, 255, 0.06);
  }

  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 9999px;
    border: none;
    cursor: pointer;
    padding: 2px;
    transition: background-color 0.2s ease;
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .toggle-switch:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 2px;
  }

  .toggle-switch.active {
    background: var(--color-orange-primary, #f97316);
  }

  .toggle-thumb {
    width: 20px;
    height: 20px;
    background: #ffffff;
    border-radius: 9999px;
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    display: block;
  }

  .toggle-switch.active .toggle-thumb {
    transform: translateX(20px);
    background: #18181b;
  }

  .toggle-switch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
