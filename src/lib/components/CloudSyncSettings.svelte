<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    Cloud,
    RefreshCw,
    FolderOpen,
    FolderCheck,
    CheckCircle2,
    AlertCircle,
    ArrowUpCircle,
    ArrowDownCircle
  } from '@lucide/svelte';
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';
  import { toast } from '$lib/stores/toast';
  import { Badge, Button, Toast } from '$lib/components/system-ui';
  import type { SyncConfig, SyncStatus } from '$lib/types/sync';

  let config: SyncConfig = {
    provider: 'none',
    cloud_path: null,
    auto_sync: false,
    last_sync_at: null
  };

  let status: SyncStatus = {
    is_syncing: false,
    last_sync_at: null,
    provider: null,
    progress: null,
    error: null,
    uploaded_count: 0,
    downloaded_count: 0
  };

  let detectedFolders: Record<string, string> = {};
  let isBusy = false;
  let statusMessage = '';
  let statusIsError = false;

  const PROVIDERS = [
    { id: 'none', label: 'Desactivada', desc: 'Sin sincronización en la nube' },
    { id: 'mega', label: 'MEGA', desc: 'Carpeta MEGAsync (20 GB gratis, E2EE)' },
    { id: 'googledrive', label: 'Google Drive', desc: 'Google Drive para escritorio (15 GB gratis)' },
    { id: 'onedrive', label: 'OneDrive', desc: 'Carpeta OneDrive de Windows (5 GB gratis)' },
    { id: 'localfolder', label: 'Carpeta Personalizada', desc: 'Cualquier carpeta de nube, disco o red' }
  ];

  function showStatus(msg: string, isError = false) {
    statusMessage = msg;
    statusIsError = isError;
    setTimeout(() => {
      statusMessage = '';
    }, 5000);
  }

  onMount(async () => {
    try {
      config = await api.getCloudSyncConfig();
      status = await api.getCloudSyncStatus();
      detectedFolders = await api.detectCloudFolders();

      // Si no hay ruta configurada pero hay una carpeta detectada para el proveedor activo, sugerirla
      if (!config.cloud_path && config.provider && detectedFolders[config.provider]) {
        config.cloud_path = detectedFolders[config.provider];
        await api.saveCloudSyncConfig(config);
      }
    } catch (e) {
      console.error('Error al inicializar CloudSyncSettings:', e);
    }
  });

  async function handleProviderSelect(providerId: string) {
    if (isBusy || status.is_syncing) return;
    config.provider = providerId;

    // Si seleccionó un proveedor con carpeta detectada y no tenía ruta, asignarla automáticamente
    if (providerId !== 'none' && !config.cloud_path && detectedFolders[providerId]) {
      config.cloud_path = detectedFolders[providerId];
    }

    try {
      await api.saveCloudSyncConfig(config);
      showStatus(`✓ Proveedor cambiado a: ${providerId}`);
    } catch (e: any) {
      toast.error(`Error al guardar configuración: ${e?.message || e}`);
    }
  }

  async function handleSelectFolder() {
    if (isBusy || status.is_syncing) return;
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Seleccionar carpeta de sincronización en la nube'
      });

      if (!selected) return;
      const folderPath = Array.isArray(selected) ? selected[0] : selected;
      if (!folderPath) return;

      config.cloud_path = folderPath;
      await api.saveCloudSyncConfig(config);
      toast.success('Carpeta de sincronización actualizada');
      showStatus(`✓ Carpeta de sincronización: ${folderPath}`);
    } catch (e: any) {
      toast.error(`Error al seleccionar carpeta: ${e?.message || e}`);
    }
  }

  async function handleUseDetectedPath(path: string) {
    config.cloud_path = path;
    try {
      await api.saveCloudSyncConfig(config);
      toast.success('Ruta detectada asignada correctamente');
      showStatus(`✓ Carpeta asignada: ${path}`);
    } catch (e: any) {
      toast.error(`Error al guardar configuración: ${e?.message || e}`);
    }
  }

  async function handleToggleAutoSync() {
    if (isBusy || status.is_syncing) return;
    config.auto_sync = !config.auto_sync;
    try {
      await api.saveCloudSyncConfig(config);
      toast.success(config.auto_sync ? 'Sincronización automática activada' : 'Sincronización automática desactivada');
    } catch (e: any) {
      toast.error(`Error al actualizar sincronización automática: ${e?.message || e}`);
    }
  }

  async function handleSyncNow() {
    if (isBusy || status.is_syncing) return;
    if (config.provider === 'none') {
      toast.error('Selecciona un proveedor de nube antes de sincronizar');
      return;
    }
    if (!config.cloud_path) {
      toast.error('Configura primero una carpeta de sincronización');
      return;
    }

    isBusy = true;
    status.is_syncing = true;
    status.progress = 'Sincronizando emotes con la nube...';

    try {
      const result = await api.startCloudSync();
      status = result;
      config.last_sync_at = result.last_sync_at;
      await loadEmotes();
      toast.success(`Sincronización completada: ${result.uploaded_count} subidos, ${result.downloaded_count} descargados`);
      showStatus(`✓ Sincronización exitosa (${result.uploaded_count} subidos, ${result.downloaded_count} descargados)`);
    } catch (e: any) {
      const msg = e?.message || e || 'Error de sincronización';
      status.error = msg;
      status.is_syncing = false;
      toast.error(`Error de sincronización: ${msg}`);
      showStatus(`Error: ${msg}`, true);
    } finally {
      isBusy = false;
      status.is_syncing = false;
    }
  }

  function formatSyncDate(timestamp: number | null): string {
    if (!timestamp) return 'Nunca';
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch {
      return 'Fecha desconocida';
    }
  }
</script>

<div class="cloud-sync-panel">
  <div class="panel-header">
    <div class="header-info">
      <div class="title-row">
        <Cloud size={16} class="cloud-icon" />
        <span class="panel-title">Sincronización en la Nube (Cloud Sync)</span>
        {#if config.provider !== 'none'}
          <Badge variant="blue" shape="pill">{config.provider.toUpperCase()}</Badge>
        {:else}
          <Badge variant="gray" shape="pill">DESACTIVADA</Badge>
        {/if}
      </div>
      <p class="panel-desc">
        Sincroniza tus emotes e imágenes automáticamente entre computadoras mediante tu nube personal (MEGA, Google Drive o OneDrive) sin límites de espacio.
      </p>
    </div>

    <div class="header-action">
      <Button
        variant="primary"
        size="sm"
        disabled={isBusy || status.is_syncing || config.provider === 'none' || !config.cloud_path}
        on:click={handleSyncNow}
      >
        <span class:spin={status.is_syncing} class="btn-icon">
          <RefreshCw size={14} />
        </span>
        {status.is_syncing ? 'Sincronizando...' : 'Sincronizar ahora'}
      </Button>
    </div>
  </div>

  <!-- Selector de proveedor -->
  <div class="provider-selector-grid">
    {#each PROVIDERS as p}
      <button
        type="button"
        class="provider-card"
        class:selected={config.provider === p.id}
        on:click={() => handleProviderSelect(p.id)}
      >
        <div class="provider-card-header">
          <span class="provider-radio">
            {#if config.provider === p.id}
              <CheckCircle2 size={14} class="check-icon" />
            {/if}
          </span>
          <span class="provider-name">{p.label}</span>
        </div>
        <p class="provider-desc">{p.desc}</p>
      </button>
    {/each}
  </div>

  <!-- Configuración de carpeta cuando no está desactivada -->
  {#if config.provider !== 'none'}
    <div class="config-body">
      <!-- Detección de carpeta sugerida -->
      {#if detectedFolders[config.provider] && config.cloud_path !== detectedFolders[config.provider]}
        <div class="detected-alert">
          <FolderCheck size={16} class="detected-icon" />
          <div class="detected-text">
            <span>Se detectó una carpeta local de {config.provider.toUpperCase()}:</span>
            <code>{detectedFolders[config.provider]}</code>
          </div>
          <Button
            variant="secondary"
            size="sm"
            on:click={() => handleUseDetectedPath(detectedFolders[config.provider])}
          >
            Usar detectada
          </Button>
        </div>
      {/if}

      <div class="folder-config-row">
        <div class="folder-field">
          <label for="sync-path-input" class="folder-label">
            Carpeta sincronizada en tu nube:
          </label>
          <div class="folder-input-box">
            <span class="folder-path-display">
              {config.cloud_path || 'Ninguna carpeta seleccionada. Elige una para comenzar.'}
            </span>
          </div>
        </div>
        <div class="folder-action">
          <Button variant="outline" size="sm" on:click={handleSelectFolder}>
            <FolderOpen size={14} /> Elegir carpeta...
          </Button>
        </div>
      </div>

      <!-- Barra de opciones e información de estado -->
      <div class="sync-footer-bar">
        <div class="auto-sync-toggle">
          <button
            type="button"
            role="switch"
            aria-checked={config.auto_sync}
            aria-label="Sincronizar automáticamente en segundo plano"
            class="toggle-switch"
            class:active={config.auto_sync}
            on:click={handleToggleAutoSync}
          >
            <span class="toggle-thumb"></span>
          </button>
          <span class="toggle-label">Sincronizar automáticamente en segundo plano</span>
        </div>

        <div class="sync-stats-meta">
          <span class="meta-item">
            Última sinc.: <strong>{formatSyncDate(config.last_sync_at)}</strong>
          </span>
          {#if status.uploaded_count > 0 || status.downloaded_count > 0}
            <span class="meta-sep">•</span>
            <span class="meta-item sync-counts">
              <span class="stat-pill upload" title="Subidos en la última sincronización">
                <ArrowUpCircle size={12} /> {status.uploaded_count}
              </span>
              <span class="stat-pill download" title="Descargados en la última sincronización">
                <ArrowDownCircle size={12} /> {status.downloaded_count}
              </span>
            </span>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- Mensajes de estado -->
  {#if statusMessage}
    <div class="status-box">
      <Toast variant={statusIsError ? 'error' : 'success'} message={statusMessage} />
    </div>
  {/if}
  {#if status.error}
    <div class="error-banner">
      <AlertCircle size={15} class="error-icon" />
      <span>{status.error}</span>
    </div>
  {/if}
</div>

<style>
  .cloud-sync-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    background: transparent;
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    margin-top: 16px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
  }

  .header-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  :global(.cloud-icon) {
    color: var(--color-orange-primary, #f97316);
  }

  .panel-title {
    font-size: 14px;
    font-weight: 600;
    color: #f4f4f5;
  }

  .panel-desc {
    font-size: 12px;
    color: #a1a1aa;
    line-height: 1.4;
    margin: 0;
  }

  .header-action {
    flex-shrink: 0;
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  /* Grid de selección de proveedor */
  .provider-selector-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 10px;
  }

  .provider-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease-in-out;
  }

  .provider-card:hover {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(249, 115, 22, 0.35);
  }

  .provider-card.selected {
    border-color: var(--color-orange-primary, #f97316);
    background: rgba(249, 115, 22, 0.08);
  }

  .provider-card-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .provider-radio {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.2));
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.2);
  }

  .provider-card.selected .provider-radio {
    border-color: var(--color-orange-primary, #f97316);
    background: var(--color-orange-primary, #f97316);
  }

  :global(.check-icon) {
    color: #ffffff;
  }

  .provider-name {
    font-size: 13px;
    font-weight: 500;
    color: #f4f4f5;
  }

  .provider-desc {
    font-size: 11px;
    color: #71717a;
    line-height: 1.3;
    margin: 0;
  }

  /* Config body */
  .config-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-top: 8px;
    border-top: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.06));
  }

  .detected-alert {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    background: rgba(249, 115, 22, 0.08);
    border: 1px solid rgba(249, 115, 22, 0.25);
    border-radius: 6px;
  }

  :global(.detected-icon) {
    color: var(--color-orange-primary, #f97316);
    flex-shrink: 0;
  }

  .detected-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12px;
    color: #fb923c;
    overflow: hidden;
  }

  .detected-text code {
    font-size: 11px;
    color: #f4f4f5;
    background: rgba(0, 0, 0, 0.25);
    padding: 2px 6px;
    border-radius: 4px;
    text-overflow: ellipsis;
    overflow: hidden;
    white-space: nowrap;
  }

  .folder-config-row {
    display: flex;
    align-items: flex-end;
    gap: 12px;
  }

  .folder-field {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .folder-label {
    font-size: 12px;
    font-weight: 500;
    color: #a1a1aa;
  }

  .folder-input-box {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    min-height: 36px;
  }

  .folder-path-display {
    font-size: 12px;
    color: #f4f4f5;
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .folder-action {
    flex-shrink: 0;
  }

  /* Footer bar */
  .sync-footer-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
    padding-top: 8px;
  }

  .auto-sync-toggle {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .toggle-switch {
    width: 36px;
    height: 20px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.12));
    border-radius: 20px;
    position: relative;
    cursor: pointer;
    transition: all 0.2s ease;
    padding: 2px;
  }

  .toggle-switch.active {
    background: var(--color-orange-primary, #f97316);
    border-color: var(--color-orange-primary, #f97316);
  }

  .toggle-thumb {
    display: block;
    width: 14px;
    height: 14px;
    background: #ffffff;
    border-radius: 50%;
    transition: transform 0.2s ease;
  }

  .toggle-switch.active .toggle-thumb {
    transform: translateX(16px);
  }

  .toggle-label {
    font-size: 12px;
    color: #d4d4d8;
  }

  .sync-stats-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #71717a;
  }

  .meta-item strong {
    color: #e4e4e7;
    font-weight: 500;
  }

  .meta-sep {
    color: #3f3f46;
  }

  .sync-counts {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .stat-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
  }

  .stat-pill.upload {
    background: rgba(249, 115, 22, 0.12);
    color: #fb923c;
  }

  .stat-pill.download {
    background: rgba(34, 197, 94, 0.12);
    color: #4ade80;
  }

  .status-box {
    margin-top: 4px;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.25);
    border-radius: 6px;
    font-size: 12px;
    color: #f87171;
  }

  :global(.error-icon) {
    color: #f87171;
    flex-shrink: 0;
  }
</style>
