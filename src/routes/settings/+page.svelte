<script lang="ts">
  import { ArrowLeft, Database, Sliders, Info, ShieldCheck } from '@lucide/svelte';
  import StorageSettings from '$lib/components/StorageSettings.svelte';
  import ImportButton from '$lib/components/ImportButton.svelte';
  import { Button, Card, Badge, Logo } from '$lib/components/system-ui';
  import { customSizes, addCustomSize, removeCustomSize, DEFAULT_SIZE_PRESETS } from '$lib/stores/sizes';
  import { toast } from '$lib/stores/toast';

  let newSizeWidth: number | null = null;
  let newSizeHeight: number | null = null;

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
  <header class="settings-header">
    <div class="header-left">
      <a href="/" class="back-link" title="Volver a Emotes">
        <ArrowLeft size={18} />
        <span>Volver a Emotes</span>
      </a>
      <h1 class="page-title">Configuración</h1>
      <p class="page-subtitle">Administra el almacenamiento, copias de seguridad y preferencias de StickUp.</p>
    </div>
  </header>

  <div class="settings-content">
    <!-- Sección de Almacenamiento y Base de Datos -->
    <section class="settings-section">
      <div class="section-title-row">
        <Database size={18} class="section-icon" />
        <h2>Almacenamiento y Respaldos</h2>
      </div>
      <StorageSettings />
    </section>

    <!-- Sección de Mantenimiento y Sincronización -->
    <section class="settings-section">
      <div class="section-title-row">
        <ShieldCheck size={18} class="section-icon" />
        <h2>Sincronización e Importación</h2>
      </div>
      <Card padding="md">
        <div class="import-card-body">
          <p class="section-desc">
            Agrega nuevos emotes o sincroniza el directorio con la base de datos convirtiendo imágenes automáticamente a formato WebP optimizado.
          </p>
          <ImportButton />
        </div>
      </Card>
    </section>

    <!-- Sección de Tamaños de Copia Personalizados -->
    <section class="settings-section">
      <div class="section-title-row">
        <Sliders size={18} class="section-icon" />
        <h2>Tamaños de Portapapeles Personalizados</h2>
      </div>
      <Card padding="md">
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
              <Button variant="secondary" size="sm" on:click={handleAddSize}>
                Añadir tamaño
              </Button>
            </div>
          </div>
        </div>
      </Card>
    </section>

    <!-- Acerca de StickUp -->
    <section class="settings-section">
      <div class="section-title-row">
        <Info size={18} class="section-icon" />
        <h2>Acerca de StickUp</h2>
      </div>
      <Card padding="md">
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
      </Card>
    </section>
  </div>
</div>

<style>
  .settings-page {
    width: 100%;
    max-width: 900px;
    margin: 0 auto;
    padding: 32px 24px 64px 24px;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 18px;
  }

  .back-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: #a1a1aa;
    text-decoration: none;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 10px;
    transition: color 0.15s ease;
  }

  .back-link:hover {
    color: #facc15;
  }

  .page-title {
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    color: #f4f4f5;
  }

  .page-subtitle {
    margin: 4px 0 0 0;
    font-size: 13px;
    color: #a1a1aa;
  }

  .settings-content {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .settings-section {
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
    color: #facc15;
  }

  .section-title-row h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .section-desc {
    margin: 0 0 14px 0;
    font-size: 13px;
    color: #a1a1aa;
    line-height: 1.5;
  }

  .import-card-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

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
    font-size: 12px;
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
    border-color: #facc15;
  }

  .x-separator {
    color: #71717a;
    font-weight: bold;
  }

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
</style>

