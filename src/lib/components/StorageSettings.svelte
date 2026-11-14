<script lang="ts">
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import {
    Save,
    Upload,
    Download,
    Database,
    RotateCcw,
    Trash2,
    HardDrive,
    FolderOpen
  } from '@lucide/svelte';
  import { api } from '$lib/services/api';
  import emotes, { loadEmotes } from '$lib/stores/emotes';
  import { toast } from '$lib/stores/toast';
  import { formatFileSize } from '$lib/utils/format';
  import { Badge, Button, Modal, Toast } from '$lib/components/system-ui';
  import type { BackupInfo } from '$lib/types/backup';

  let storagePath = '';
  let appVersion = '';
  let message = '';
  let isError = false;
  let isBusy = false;

  let showRestoreConfirmModal = false;
  let showDeleteConfirmModal = false;
  let localBackups: BackupInfo[] = [];
  let backupToRestore: BackupInfo | null = null;
  let backupToDelete: BackupInfo | null = null;
  let loadingBackups = true;
  const MAX_BACKUPS = 15;

  function showStatus(msg: string, error = false) {
    message = msg;
    isError = error;
    setTimeout(() => {
      message = '';
    }, 5000);
  }

  onMount(async () => {
    try {
      storagePath = await api.getStoragePath();
      appVersion = await api.getAppVersion();
      await refreshBackupsList();
    } catch (e) {
      console.error('Error al inicializar StorageSettings:', e);
    } finally {
      loadingBackups = false;
    }
  });

  async function handleOpenStorageDirectory() {
    try {
      await api.openStorageDirectory();
    } catch (e: any) {
      console.error('Error al abrir la carpeta de almacenamiento:', e);
      const errMsg = e?.message || e || 'Error al abrir la carpeta';
      toast.error(errMsg);
    }
  }

  async function handleQuickBackup() {
    if (isBusy) return;
    isBusy = true;
    try {
      const res = await api.createBackup();
      toast.success(`Copia creada: ${res.backup.file_name} (${res.backup.emote_count} emotes)`);
      showStatus(`✓ Copia de seguridad guardada: ${res.backup.file_name}`);
      await refreshBackupsList();
    } catch (e: any) {
      const msg = e?.message || e || 'Error al crear copia';
      toast.error(`Error al crear copia: ${msg}`);
      showStatus(`Error: ${msg}`, true);
    } finally {
      isBusy = false;
    }
  }

  async function handleExportBackup() {
    if (isBusy) return;
    try {
      const dateStr = new Date().toISOString().slice(0, 10);
      const selected = await save({
        title: 'Guardar copia de seguridad de la base de datos',
        defaultPath: `stickup_backup_${dateStr}.db`,
        filters: [{ name: 'Base de datos SQLite (.db)', extensions: ['db'] }]
      });

      if (!selected) return;

      isBusy = true;
      const res = await api.createBackup(selected);
      toast.success(`Copia exportada: ${res.backup.file_name} (${res.backup.emote_count} emotes)`);
      showStatus(`✓ Copia exportada a: ${res.backup.path}`);
      await refreshBackupsList();
    } catch (e: any) {
      const msg = e?.message || e || 'Error al exportar copia';
      toast.error(`Error: ${msg}`);
      showStatus(`Error: ${msg}`, true);
    } finally {
      isBusy = false;
    }
  }

  async function handleSelectBackupToRestore() {
    if (isBusy) return;
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: 'Seleccionar archivo de copia de seguridad (.db)',
        filters: [{ name: 'Base de datos SQLite', extensions: ['db', 'sqlite'] }]
      });

      if (!selected) return;
      const filePath = Array.isArray(selected) ? selected[0] : selected;
      if (!filePath) return;

      isBusy = true;
      const info = await api.inspectBackup(filePath);
      backupToRestore = info;
      showRestoreConfirmModal = true;
    } catch (e: any) {
      const msg = e?.message || e || 'Archivo no válido';
      toast.error(`Copia no válida: ${msg}`);
      showStatus(`Error: ${msg}`, true);
    } finally {
      isBusy = false;
    }
  }

  async function refreshBackupsList() {
    try {
      localBackups = await api.listBackups();
    } catch (e) {
      console.error('Error al listar backups:', e);
      localBackups = [];
    }
  }

  function handleChooseHistoryItem(item: BackupInfo) {
    backupToRestore = item;
    showRestoreConfirmModal = true;
  }

  async function confirmRestore() {
    if (!backupToRestore || isBusy) return;
    const target = backupToRestore;
    isBusy = true;
    showRestoreConfirmModal = false;

    try {
      const res = await api.restoreBackup(target.path);
      await loadEmotes();
      await refreshBackupsList();
      toast.success(`Base de datos restaurada (${res.restored_emote_count} emotes)`);
      showStatus(
        res.safety_backup_path
          ? `✓ BD restaurada (${res.restored_emote_count} emotes). Copia previa en: ${res.safety_backup_path}`
          : `✓ Base de datos restaurada correctamente`
      );
    } catch (e: any) {
      const msg = e?.message || e || 'Error al restaurar';
      toast.error(`Error al restaurar: ${msg}`);
      showStatus(`Error al restaurar: ${msg}`, true);
    } finally {
      isBusy = false;
      backupToRestore = null;
    }
  }

  function cancelRestore() {
    showRestoreConfirmModal = false;
    backupToRestore = null;
  }

  function requestDeleteBackup(item: BackupInfo) {
    backupToDelete = item;
    showDeleteConfirmModal = true;
  }

  function cancelDeleteBackup() {
    showDeleteConfirmModal = false;
    backupToDelete = null;
  }

  async function confirmDeleteBackup() {
    if (!backupToDelete || isBusy) return;
    const target = backupToDelete;
    isBusy = true;
    showDeleteConfirmModal = false;

    try {
      await api.deleteBackup(target.path);
      toast.success(`Copia eliminada: ${target.file_name}`);
      await refreshBackupsList();
    } catch (e: any) {
      const msg = e?.message || e || 'Error al eliminar copia';
      toast.error(`Error al eliminar: ${msg}`);
      showStatus(`Error: ${msg}`, true);
    } finally {
      isBusy = false;
      backupToDelete = null;
    }
  }
</script>

<div class="storage-container">
  <!-- Información expandida de almacenamiento y directorio (sin bg de tarjeta) -->
  <div class="storage-panel">
    <div class="panel-main-row">
      <div class="path-details">
        <div class="path-title-line">
          <HardDrive size={15} class="path-icon" />
          <span class="path-title">Directorio de datos y base de datos:</span>
        </div>
        <div class="path-display-box" title={storagePath ? `${storagePath} (.store.db)` : ''}>
          <span class="path-text">{storagePath || 'Cargando directorio...'}</span>
          <Badge variant="blue" shape="pill">.store.db</Badge>
        </div>
      </div>

      <div class="path-action">
        <Button variant="secondary" size="sm" on:click={handleOpenStorageDirectory}>
          <FolderOpen size={14} /> Abrir carpeta
        </Button>
      </div>
    </div>

    <div class="storage-stats-bar">
      <div class="stat-item">
        <span class="status-dot"></span>
        <span class="stat-label">SQLite Conectado</span>
      </div>
      <span class="stat-sep">•</span>
      <div class="stat-item">
        <span class="stat-val">{$emotes.length}</span>
        <span class="stat-label">emotes registrados</span>
      </div>
      {#if appVersion}
        <span class="stat-sep">•</span>
        <div class="stat-item">
          <span class="stat-label">Versión de esquema:</span>
          <Badge variant="gray" shape="pill">v{appVersion}</Badge>
        </div>
      {/if}
    </div>
  </div>

  <!-- Mensajes de estado -->
  {#if isBusy}
    <div class="status-row">
      <Toast variant="info" message="⏳ Procesando solicitud..." />
    </div>
  {:else if message}
    <div class="status-row">
      <Toast variant={isError ? 'error' : 'success'} message={message} />
    </div>
  {/if}

  <!-- Sección Expandida de Copias de Seguridad Hechas -->
  <div class="backups-panel">
    <div class="backups-toolbar">
      <div class="backups-summary">
        <div class="backups-title-row">
          <Database size={15} class="backups-icon" />
          <span class="backups-title">Copias de seguridad locales</span>
          <Badge variant="gray" shape="pill">{localBackups.length} / {MAX_BACKUPS}</Badge>
        </div>
        <span class="backups-hint">
          Rotación automática activa (máx. {MAX_BACKUPS} copias almacenadas)
        </span>
      </div>

      <div class="backups-actions">
        <Button variant="secondary" size="sm" disabled={isBusy} on:click={handleQuickBackup}>
          <Save size={14} /> Copia rápida
        </Button>
        <Button variant="outline" size="sm" disabled={isBusy} on:click={handleExportBackup}>
          <Upload size={14} /> Exportar...
        </Button>
        <Button variant="outline" size="sm" disabled={isBusy} on:click={handleSelectBackupToRestore}>
          <Download size={14} /> Restaurar...
        </Button>
      </div>
    </div>

    <!-- Lista visible de copias de seguridad hechas -->
    <div class="backups-list-box">
      {#if loadingBackups}
        <div class="empty-state">
          <span>⏳ Cargando historial de copias...</span>
        </div>
      {:else if localBackups.length === 0}
        <div class="empty-state">
          <Database size={22} class="empty-icon" />
          <p class="empty-title">No hay copias de seguridad locales registradas aún</p>
          <p class="empty-desc">
            Crea una copia rápida para respaldar la base de datos de tus emotes y etiquetas.
          </p>
          <Button variant="secondary" size="sm" disabled={isBusy} on:click={handleQuickBackup}>
            <Save size={14} /> Crear primera copia
          </Button>
        </div>
      {:else}
        <div class="backups-table">
          <div class="table-header">
            <span class="th col-file">Archivo de copia</span>
            <span class="th col-date">Fecha</span>
            <span class="th col-emotes">Emotes</span>
            <span class="th col-size">Tamaño</span>
            <span class="th col-actions">Acciones</span>
          </div>

          <div class="table-rows">
            {#each localBackups as item (item.path)}
              <div class="table-row">
                <div class="td col-file" title={item.path}>
                  <Database size={14} class="file-icon" />
                  <span class="file-name">{item.file_name}</span>
                </div>
                <div class="td col-date">
                  <span class="date-val">{item.created_at}</span>
                </div>
                <div class="td col-emotes">
                  <Badge variant="blue" shape="pill">{item.emote_count} emotes</Badge>
                </div>
                <div class="td col-size">
                  <span class="size-val">{formatFileSize(item.size_bytes)}</span>
                </div>
                <div class="td col-actions">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={isBusy}
                    title="Restaurar base de datos a este punto"
                    on:click={() => handleChooseHistoryItem(item)}
                  >
                    <RotateCcw size={13} /> Restaurar
                  </Button>
                  <Button
                    variant="danger"
                    size="sm"
                    disabled={isBusy}
                    title="Eliminar esta copia de seguridad"
                    ariaLabel="Eliminar copia de seguridad"
                    on:click={() => requestDeleteBackup(item)}
                  >
                    <Trash2 size={13} />
                  </Button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Modal de confirmación de restauración -->
  <Modal
    open={showRestoreConfirmModal}
    title="Confirmar restauración de base de datos"
    maxWidth="480px"
    busy={isBusy}
    onClose={cancelRestore}
  >
    {#if backupToRestore}
      <div class="restore-details">
        <Toast
          variant="warning"
          title="Reemplazo de base de datos"
          message="Esta acción reemplazará los datos actuales por los del archivo seleccionado. StickUp creará automáticamente un respaldo de seguridad preventivo antes de continuar."
        />

        <div class="detail-box">
          <div class="detail-line">
            <span class="detail-label">Archivo:</span>
            <span class="detail-val" title={backupToRestore.path}>{backupToRestore.file_name}</span>
          </div>
          <div class="detail-line">
            <span class="detail-label">Emotes:</span>
            <Badge variant="blue">{backupToRestore.emote_count} emotes</Badge>
          </div>
          <div class="detail-line">
            <span class="detail-label">Tamaño:</span>
            <span>{formatFileSize(backupToRestore.size_bytes)}</span>
          </div>
          <div class="detail-line">
            <span class="detail-label">Fecha:</span>
            <span>{backupToRestore.created_at}</span>
          </div>
        </div>
      </div>
    {/if}

    <svelte:fragment slot="footer">
      <Button variant="ghost" size="sm" disabled={isBusy} on:click={cancelRestore}>
        Cancelar
      </Button>
      <Button variant="primary" size="sm" disabled={isBusy} on:click={confirmRestore}>
        <RotateCcw size={14} /> Confirmar y Restaurar
      </Button>
    </svelte:fragment>
  </Modal>

  <!-- Modal de confirmación de eliminación -->
  <Modal
    open={showDeleteConfirmModal}
    title="Eliminar copia de seguridad"
    maxWidth="440px"
    busy={isBusy}
    onClose={cancelDeleteBackup}
  >
    {#if backupToDelete}
      <div class="restore-details">
        <Toast
          variant="error"
          title="Eliminar permanentemente"
          message="¿Estás seguro de que deseas eliminar esta copia de seguridad? Esta acción no se puede deshacer."
        />

        <div class="detail-box">
          <div class="detail-line">
            <span class="detail-label">Archivo:</span>
            <span class="detail-val" title={backupToDelete.path}>{backupToDelete.file_name}</span>
          </div>
          <div class="detail-line">
            <span class="detail-label">Emotes:</span>
            <Badge variant="blue">{backupToDelete.emote_count} emotes</Badge>
          </div>
          <div class="detail-line">
            <span class="detail-label">Tamaño:</span>
            <span>{formatFileSize(backupToDelete.size_bytes)}</span>
          </div>
          <div class="detail-line">
            <span class="detail-label">Fecha:</span>
            <span>{backupToDelete.created_at}</span>
          </div>
        </div>
      </div>
    {/if}

    <svelte:fragment slot="footer">
      <Button variant="ghost" size="sm" disabled={isBusy} on:click={cancelDeleteBackup}>
        Cancelar
      </Button>
      <Button variant="danger" size="sm" disabled={isBusy} on:click={confirmDeleteBackup}>
        <Trash2 size={14} /> Eliminar copia
      </Button>
    </svelte:fragment>
  </Modal>
</div>

<style>
  .storage-container {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-sizing: border-box;
  }

  /* Panel de Directorio - Sin background de tarjeta */
  .storage-panel {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 18px;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    box-sizing: border-box;
  }

  .panel-main-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    width: 100%;
  }

  .path-details {
    flex: 1;
    min-width: 260px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .path-title-line {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #a1a1aa;
    font-size: 12px;
    font-weight: 500;
  }

  .path-title-line :global(.path-icon) {
    color: var(--color-orange-primary, #f97316);
  }

  .path-title {
    font-size: 12.5px;
  }

  .path-display-box {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 6px 12px;
    overflow: hidden;
  }

  .path-text {
    flex: 1;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12px;
    color: #f4f4f5;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .path-action {
    flex-shrink: 0;
  }

  .storage-stats-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    color: #a1a1aa;
    flex-wrap: wrap;
    padding-top: 4px;
    border-top: 1px solid rgba(255, 255, 255, 0.04);
  }

  .stat-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #10b981;
    box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
  }

  .stat-val {
    font-weight: 600;
    color: #f4f4f5;
  }

  .stat-sep {
    color: #52525b;
  }

  /* Panel de Copias de Seguridad - Sin background de tarjeta */
  .backups-panel {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 18px;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    box-sizing: border-box;
  }

  .backups-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    width: 100%;
  }

  .backups-summary {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .backups-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .backups-title-row :global(.backups-icon) {
    color: var(--color-orange-primary, #f97316);
  }

  .backups-title {
    font-size: 13.5px;
    font-weight: 600;
    color: #f4f4f5;
  }

  .backups-hint {
    font-size: 11.5px;
    color: #71717a;
  }

  .backups-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  /* Tabla y Lista de Backups */
  .backups-list-box {
    width: 100%;
    display: flex;
    flex-direction: column;
  }

  .backups-table {
    width: 100%;
    display: flex;
    flex-direction: column;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .table-header {
    display: flex;
    align-items: center;
    padding: 8px 10px;
    font-size: 11px;
    font-weight: 600;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .table-rows {
    display: flex;
    flex-direction: column;
  }

  .table-row {
    display: flex;
    align-items: center;
    padding: 10px 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    transition: background 0.15s ease;
  }

  .table-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .table-row:last-child {
    border-bottom: none;
  }

  .col-file {
    flex: 2;
    min-width: 180px;
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .col-file :global(.file-icon) {
    color: #a1a1aa;
    flex-shrink: 0;
  }

  .file-name {
    font-size: 12.5px;
    font-weight: 500;
    color: #f4f4f5;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .col-date {
    flex: 1.4;
    min-width: 130px;
    font-size: 12px;
    color: #a1a1aa;
  }

  .col-emotes {
    flex: 1;
    min-width: 90px;
  }

  .col-size {
    flex: 0.8;
    min-width: 70px;
    font-size: 12px;
    color: #a1a1aa;
  }

  .col-actions {
    flex: 1.2;
    min-width: 130px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
  }



  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px 16px;
    text-align: center;
    color: #71717a;
    gap: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .empty-state :global(.empty-icon) {
    color: #52525b;
    margin-bottom: 2px;
  }

  .empty-title {
    margin: 0;
    font-size: 13.5px;
    font-weight: 500;
    color: #e4e4e7;
  }

  .empty-desc {
    margin: 0 0 8px 0;
    font-size: 12px;
    color: #71717a;
    max-width: 360px;
  }

  /* Contenedores de estado */
  .status-row {
    width: 100%;
  }

  .restore-details {
    display: flex;
    flex-direction: column;
    gap: 12px;
    text-align: left;
  }

  .detail-box {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 13px;
  }

  .detail-line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .detail-label {
    font-weight: 600;
    color: #9ca3af;
  }

  .detail-val {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12px;
    color: #f4f4f5;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 280px;
  }
</style>
