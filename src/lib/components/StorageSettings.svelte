<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';
  import { Alert, Button, Card } from '$lib/components/system-ui';

  let storagePath = '';
  let message = '';
  let isError = false;
  let isBusy = false;

  let pendingTargetPath: string | null = null;
  let askDbConflict = false;
  let currentDbCount = 0;
  let targetDbCount = 0;

  function showStatus(msg: string, error = false) {
    message = msg;
    isError = error;
    setTimeout(() => {
      message = '';
    }, 4000);
  }

  onMount(async () => {
    try {
      storagePath = await api.getStoragePath();
    } catch (e) {
      console.error('Error al obtener la ruta de almacenamiento:', e);
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
</script>

<div class="storage-wrapper">
  <Card padding="md">
    <div class="info-row">
      <span class="path-label">💾 Directorio (Emotes y BD):</span>
      <span class="path-val" title={storagePath ? `${storagePath} (incluye .store.db)` : ''}>
        {storagePath ? `${storagePath} (.store.db)` : 'Cargando...'}
      </span>
      <Button variant="outline" size="sm" disabled={isBusy} on:click={pickStoragePath}>
        ✏️ Cambiar ruta
      </Button>
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
        <Alert variant="info">⏳ Procesando cambio de directorio...</Alert>
      </div>
    {:else if message}
      <div class="status-row">
        <Alert variant={isError ? 'error' : 'success'}>{message}</Alert>
      </div>
    {/if}
  </Card>
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
    .path-label {
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
</style>
