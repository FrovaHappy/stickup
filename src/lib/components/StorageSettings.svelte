<script lang="ts">
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';
  import { toast } from '$lib/stores/toast';
  import { formatFileSize } from '$lib/utils/format';
  import { Alert, Badge, Button, Card, Modal } from '$lib/components/system-ui';
  import type { BackupInfo } from '$lib/types/backup';

  let storagePath = '';
  let appVersion = '';
  let message = '';
  let isError = false;
  let isBusy = false;

  let pendingTargetPath: string | null = null;
  let askDbConflict = false;
  let currentDbCount = 0;
  let targetDbCount = 0;

  let showHistoryModal = false;
  let showRestoreConfirmModal = false;
  let showDeleteConfirmModal = false;
  let localBackups: BackupInfo[] = [];
  let backupToRestore: BackupInfo | null = null;
  let backupToDelete: BackupInfo | null = null;
  let loadingBackups = false;
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
    } catch (e) {
      console.error('Error al inicializar StorageSettings:', e);
    }
  });

  async function pickStoragePath() {
    if (isBusy) return;
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Seleccionar carpeta para emotes y base de datos (.store.db)'
      });

      if (!selected) return;

      const newPath = Array.isArray(selected) ? selected[0] : selected;
      if (newPath && newPath.toLowerCase() !== storagePath.toLowerCase()) {
        pendingTargetPath = newPath;
        askDbConflict = false;
      }
    } catch (e: any) {
      console.error('Error al seleccionar la ruta de almacenamiento:', e);
      showStatus(`Error: ${e?.message || e}`, true);
    }
  }

  function cancelPending() {
    pendingTargetPath = null;
    askDbConflict = false;
  }

  async function applyPendingChange(migrate: boolean, overwriteDb?: boolean) {
    if (!pendingTargetPath || isBusy) return;
    const targetPath = pendingTargetPath;
    isBusy = true;

    try {
      storagePath = await api.setStoragePath(targetPath, migrate, overwriteDb);
      pendingTargetPath = null;
      askDbConflict = false;
      await loadEmotes();
      showStatus(
        migrate
          ? overwriteDb === false
            ? '✓ Imágenes migradas conservando la base de datos del destino'
            : '✓ Emotes y base de datos migrados a la nueva carpeta'
          : '✓ Directorio de emotes y base de datos actualizado'
      );
    } catch (e: any) {
      const errText = String(e?.message || e || '');
      if (errText.includes('DB_EXISTS_IN_TARGET')) {
        const parts = errText.split(':');
        currentDbCount = Number(parts[1]) || 0;
        targetDbCount = Number(parts[2]) || 0;
        askDbConflict = true;
      } else {
        pendingTargetPath = null;
        askDbConflict = false;
        console.error('Error al aplicar cambio de directorio:', e);
        showStatus(`Error: ${errText}`, true);
      }
    } finally {
      isBusy = false;
    }
  }

  async function handleQuickBackup() {
    if (isBusy) return;
    isBusy = true;
    try {
      const res = await api.createBackup();
      toast.success(`Copia creada: ${res.backup.file_name} (${res.backup.emote_count} emotes)`);
      showStatus(`✓ Copia de seguridad guardada: ${res.backup.file_name}`);
      if (showHistoryModal) {
        await refreshBackupsList();
      }
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

  async function handleOpenHistory() {
    loadingBackups = true;
    showHistoryModal = true;
    await refreshBackupsList();
    loadingBackups = false;
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
    showHistoryModal = false;

    try {
      const res = await api.restoreBackup(target.path);
      await loadEmotes();
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

<div class="storage-wrapper">
  <Card padding="md">
    <div class="info-row">
      <span class="path-label">💾 Directorio:</span>
      <span class="path-val" title={storagePath ? `${storagePath} (incluye .store.db)` : ''}>
        {storagePath ? `${storagePath} (.store.db)` : 'Cargando...'}
      </span>
      {#if appVersion}
        <Badge variant="gray" title={`Versión de StickUp registrada: v${appVersion}`}>v{appVersion}</Badge>
      {/if}
      <Button variant="outline" size="sm" disabled={isBusy} on:click={pickStoragePath}>
        ✏️ Cambiar ruta
      </Button>
    </div>

    <div class="backup-row">
      <span class="backup-label">🛡️ Copias de seguridad:</span>
      <div class="backup-actions">
        <Button variant="secondary" size="sm" disabled={isBusy} on:click={handleQuickBackup}>
          💾 Copia rápida
        </Button>
        <Button variant="outline" size="sm" disabled={isBusy} on:click={handleExportBackup}>
          📤 Exportar...
        </Button>
        <Button variant="outline" size="sm" disabled={isBusy} on:click={handleSelectBackupToRestore}>
          📥 Restaurar...
        </Button>
        <Button variant="ghost" size="sm" disabled={isBusy} on:click={handleOpenHistory}>
          📜 Historial
        </Button>
      </div>
    </div>

    {#if pendingTargetPath}
      <div class="prompt-container">
        {#if askDbConflict}
          <Alert
            variant="info"
            title="⚠️ Ya existe una base de datos (.store.db) en el destino. ¿Cuál deseas conservar?"
          >
            <div class="prompt-dest" title={pendingTargetPath}>
              Destino: <code>{pendingTargetPath}</code>
            </div>
            <p class="prompt-desc">
              Se moverán todas tus imágenes a la nueva carpeta. Elige qué archivo
              <code>.store.db</code> tendrá prioridad para tus tags, nombres y estadísticas:
            </p>
            <svelte:fragment slot="actions">
              <Button
                variant="primary"
                size="sm"
                disabled={isBusy}
                on:click={() => applyPendingChange(true, true)}
              >
                🗂️ Conservar BD actual ({currentDbCount} emotes)
              </Button>
              <Button
                variant="success"
                size="sm"
                disabled={isBusy}
                on:click={() => applyPendingChange(true, false)}
              >
                📥 Conservar BD del destino ({targetDbCount} emotes)
              </Button>
              <Button variant="ghost" size="sm" disabled={isBusy} on:click={cancelPending}>
                Cancelar
              </Button>
            </svelte:fragment>
          </Alert>
        {:else}
          <Alert
            variant="info"
            title="📦 ¿Deseas migrar tus emotes y base de datos a la nueva carpeta?"
          >
            <div class="prompt-dest" title={pendingTargetPath}>
              Destino: <code>{pendingTargetPath}</code>
            </div>
            <p class="prompt-desc">
              <strong>Migrar todo</strong> moverá todas las imágenes actuales, actualizará sus rutas y
              trasladará <code>.store.db</code> junto con tus tags y estadísticas a la nueva carpeta.
            </p>
            <svelte:fragment slot="actions">
              <Button
                variant="primary"
                size="sm"
                disabled={isBusy}
                on:click={() => applyPendingChange(true)}
              >
                📦 Migrar todo (Imágenes + BD)
              </Button>
              <Button
                variant="success"
                size="sm"
                disabled={isBusy}
                on:click={() => applyPendingChange(false)}
              >
                📁 Solo cambiar ruta
              </Button>
              <Button variant="ghost" size="sm" disabled={isBusy} on:click={cancelPending}>
                Cancelar
              </Button>
            </svelte:fragment>
          </Alert>
        {/if}
      </div>
    {/if}

    {#if isBusy}
      <div class="status-row">
        <Alert variant="info">⏳ Procesando solicitud...</Alert>
      </div>
    {:else if message}
      <div class="status-row">
        <Alert variant={isError ? 'error' : 'success'}>{message}</Alert>
      </div>
    {/if}
  </Card>

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
        <Alert variant="warning" title="⚠️ Reemplazo de base de datos">
          Esta acción reemplazará los datos actuales por los del archivo seleccionado. StickUp creará
          automáticamente un respaldo de seguridad preventivo antes de continuar.
        </Alert>

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
        🔄 Confirmar y Restaurar
      </Button>
    </svelte:fragment>
  </Modal>

  <!-- Modal de Historial de Copias Locales -->
  <Modal
    open={showHistoryModal}
    title={`Historial de copias de seguridad locales (${localBackups.length}/${MAX_BACKUPS})`}
    maxWidth="580px"
    busy={isBusy}
    onClose={() => (showHistoryModal = false)}
  >
    <div class="history-content">
      <div class="history-header">
        <span class="history-quota">
          Capacidad: <strong>{localBackups.length} de {MAX_BACKUPS}</strong> copias guardadas
        </span>
        <span class="history-hint">Rotación automática activa (máx. 15)</span>
      </div>

      {#if loadingBackups}
        <div class="empty-backups">⏳ Cargando copias disponibles...</div>
      {:else if localBackups.length === 0}
        <div class="empty-backups">
          <p>No se encontraron copias de seguridad en la carpeta local (<code>backups/</code>).</p>
          <Button variant="primary" size="sm" disabled={isBusy} on:click={handleQuickBackup}>
            💾 Crear copia ahora
          </Button>
        </div>
      {:else}
        <div class="backups-list">
          {#each localBackups as item (item.path)}
            <div class="backup-item">
              <div class="backup-item-info">
                <div class="backup-item-name" title={item.path}>
                  📁 {item.file_name}
                </div>
                <div class="backup-item-meta">
                  <span>📅 {item.created_at}</span>
                  <span>•</span>
                  <span>📦 {formatFileSize(item.size_bytes)}</span>
                  <span>•</span>
                  <Badge variant="blue">{item.emote_count} emotes</Badge>
                </div>
              </div>
              <div class="backup-item-actions">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={isBusy}
                  on:click={() => handleChooseHistoryItem(item)}
                >
                  📥 Restaurar
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={isBusy}
                  title="Eliminar copia de seguridad"
                  on:click={() => requestDeleteBackup(item)}
                >
                  🗑️
                </Button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <svelte:fragment slot="footer">
      <Button variant="ghost" size="sm" on:click={() => (showHistoryModal = false)}>
        Cerrar
      </Button>
      <Button variant="secondary" size="sm" disabled={isBusy} on:click={handleQuickBackup}>
        ➕ Nueva copia rápida
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
        <Alert variant="error" title="🗑️ Eliminar permanentemente">
          ¿Estás seguro de que deseas eliminar esta copia de seguridad? Esta acción no se puede deshacer.
        </Alert>

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
        🗑️ Eliminar copia
      </Button>
    </svelte:fragment>
  </Modal>
</div>

<style>
  .storage-wrapper {
    max-width: 650px;
    width: 100%;
    margin: 0 auto 20px auto;
    font-size: 13px;
  }
  .info-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    gap: 12px;
  }
  .backup-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    gap: 12px;
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid rgba(0, 0, 0, 0.08);
    flex-wrap: wrap;
  }
  @media (prefers-color-scheme: dark) {
    .backup-row {
      border-top-color: rgba(255, 255, 255, 0.08);
    }
  }
  .backup-label {
    font-weight: 600;
    color: #4b5563;
    white-space: nowrap;
    text-align: left;
  }
  .backup-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .path-label {
    font-weight: 600;
    color: #4b5563;
    white-space: nowrap;
    text-align: left;
  }
  .path-val {
    font-family: monospace;
    font-size: 12px;
    color: #1f2937;
    background: rgba(0, 0, 0, 0.05);
    padding: 3px 8px;
    border-radius: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    text-align: left;
  }
  @media (prefers-color-scheme: dark) {
    .path-label,
    .backup-label {
      color: #9ca3af;
    }
    .path-val {
      color: #f3f4f6;
      background: rgba(255, 255, 255, 0.08);
    }
  }
  .prompt-container,
  .status-row {
    width: 100%;
    margin-top: 8px;
  }
  .prompt-dest {
    font-size: 12px;
    color: #6b7280;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .prompt-dest code {
    font-family: monospace;
    background: rgba(0, 0, 0, 0.1);
    padding: 2px 6px;
    border-radius: 4px;
    color: inherit;
  }
  @media (prefers-color-scheme: dark) {
    .prompt-dest {
      color: #d1d5db;
    }
    .prompt-dest code {
      background: rgba(255, 255, 255, 0.1);
    }
  }
  .prompt-desc {
    margin: 0;
    font-size: 12px;
    line-height: 1.45;
    color: #4b5563;
  }
  @media (prefers-color-scheme: dark) {
    .prompt-desc {
      color: #9ca3af;
    }
  }
  .restore-details {
    display: flex;
    flex-direction: column;
    gap: 12px;
    text-align: left;
  }
  .detail-box {
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 6px;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 13px;
  }
  @media (prefers-color-scheme: dark) {
    .detail-box {
      background: rgba(255, 255, 255, 0.04);
      border-color: rgba(255, 255, 255, 0.08);
    }
  }
  .detail-line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .detail-label {
    font-weight: 600;
    color: #6b7280;
  }
  .detail-val {
    font-family: monospace;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 280px;
  }
  .history-content {
    max-height: 380px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
  }
  .empty-backups {
    text-align: center;
    padding: 24px 16px;
    color: #6b7280;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .backups-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .backup-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.02);
    border: 1px solid rgba(0, 0, 0, 0.06);
    border-radius: 6px;
    transition: background 0.15s ease;
  }
  .backup-item:hover {
    background: rgba(0, 0, 0, 0.04);
  }
  @media (prefers-color-scheme: dark) {
    .backup-item {
      background: rgba(255, 255, 255, 0.03);
      border-color: rgba(255, 255, 255, 0.06);
    }
    .backup-item:hover {
      background: rgba(255, 255, 255, 0.06);
    }
  }
  .backup-item-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow: hidden;
  }
  .backup-item-name {
    font-weight: 600;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .backup-item-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: #6b7280;
    flex-wrap: wrap;
  }
  @media (prefers-color-scheme: dark) {
    .backup-item-meta {
      color: #9ca3af;
    }
  }
  .backup-item-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: rgba(0, 0, 0, 0.03);
    border-radius: 6px;
    font-size: 12px;
    color: #4b5563;
    gap: 8px;
    flex-wrap: wrap;
  }
  .history-quota strong {
    color: #1f2937;
  }
  .history-hint {
    font-size: 11px;
    color: #6b7280;
  }
  @media (prefers-color-scheme: dark) {
    .history-header {
      background: rgba(255, 255, 255, 0.04);
      color: #9ca3af;
    }
    .history-quota strong {
      color: #f3f4f6;
    }
    .history-hint {
      color: #9ca3af;
    }
  }
</style>
