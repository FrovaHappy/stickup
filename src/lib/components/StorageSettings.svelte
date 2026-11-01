<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';

  let storagePath = '';
  let dbPath = '';
  let message = '';
  let isError = false;
  let isBusy = false;

  let pendingChange: {
    kind: 'storage' | 'db';
    targetPath: string;
  } | null = null;

  function showStatus(msg: string, error = false) {
    message = msg;
    isError = error;
    setTimeout(() => {
      message = '';
    }, 4000);
  }

  onMount(async () => {
    try {
      const [sPath, dPath] = await Promise.all([api.getStoragePath(), api.getDbPath()]);
      storagePath = sPath;
      dbPath = dPath;
    } catch (e) {
      console.error('Error al obtener las rutas de almacenamiento:', e);
    }
  });

  async function pickStoragePath() {
    if (isBusy) return;
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Seleccionar carpeta para guardar emotes'
      });

      if (!selected) return;

      const newPath = Array.isArray(selected) ? selected[0] : selected;
      if (newPath && newPath.toLowerCase() !== storagePath.toLowerCase()) {
        pendingChange = { kind: 'storage', targetPath: newPath };
      }
    } catch (e: any) {
      console.error('Error al seleccionar la ruta de almacenamiento:', e);
      showStatus(`Error: ${e?.message || e}`, true);
    }
  }

  async function pickDbPath() {
    if (isBusy) return;
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Seleccionar carpeta para la base de datos (emotes.db)'
      });

      if (!selected) return;

      const newPath = Array.isArray(selected) ? selected[0] : selected;
      if (newPath && newPath.toLowerCase() !== dbPath.toLowerCase()) {
        pendingChange = { kind: 'db', targetPath: newPath };
      }
    } catch (e: any) {
      console.error('Error al seleccionar el directorio de la base de datos:', e);
      showStatus(`Error: ${e?.message || e}`, true);
    }
  }

  async function applyPendingChange(migrate: boolean) {
    if (!pendingChange || isBusy) return;
    const { kind, targetPath } = pendingChange;
    pendingChange = null;
    isBusy = true;

    try {
      if (kind === 'storage') {
        storagePath = await api.setStoragePath(targetPath, migrate);
        dbPath = await api.getDbPath();
        await loadEmotes();
        showStatus(
          migrate
            ? '✓ Todo migrado exitosamente a la nueva carpeta'
            : '✓ Ruta de emotes actualizada (sin migrar archivos)'
        );
      } else {
        dbPath = await api.setDbPath(targetPath, migrate);
        storagePath = await api.getStoragePath();
        await loadEmotes();
        showStatus(
          migrate
            ? '✓ Base de datos migrada al nuevo directorio'
            : '✓ Directorio de base de datos actualizado'
        );
      }
    } catch (e: any) {
      console.error('Error al aplicar cambio de directorio:', e);
      showStatus(`Error: ${e?.message || e}`, true);
    } finally {
      isBusy = false;
    }
  }
</script>

<div class="storage-box">
  <div class="info-row">
    <span class="path-label">💾 Guardar emotes en:</span>
    <span class="path-val" title={storagePath}>{storagePath || 'Cargando...'}</span>
    <button type="button" class="change-btn" disabled={isBusy} on:click={pickStoragePath}>
      ✏️ Cambiar ruta
    </button>
  </div>

  <div class="info-row">
    <span class="path-label">🗄️ Directorio de BD:</span>
    <span class="path-val" title="{dbPath}/emotes.db">
      {dbPath ? `${dbPath} (emotes.db)` : 'Cargando...'}
    </span>
    <button type="button" class="change-btn" disabled={isBusy} on:click={pickDbPath}>
      ✏️ Cambiar BD
    </button>
  </div>

  {#if pendingChange}
    <div class="migration-prompt">
      <div class="prompt-header">
        <span class="prompt-title">
          {pendingChange.kind === 'storage'
            ? '📦 ¿Deseas migrar todo a la nueva carpeta?'
            : '🗄️ ¿Deseas migrar la base de datos actual al nuevo directorio?'}
        </span>
      </div>
      <div class="prompt-dest" title={pendingChange.targetPath}>
        Destino: <code>{pendingChange.targetPath}</code>
      </div>
      <p class="prompt-desc">
        {#if pendingChange.kind === 'storage'}
          <strong>Migrar todo</strong> moverá todas las imágenes actuales, actualizará sus rutas en la
          base de datos y trasladará <code>emotes.db</code> junto con ellas.
        {:else}
          <strong>Migrar BD</strong> moverá el archivo <code>emotes.db</code> actual con todos tus tags
          y estadísticas al nuevo directorio.
        {/if}
      </p>
      <div class="prompt-actions">
        <button
          type="button"
          class="prompt-btn migrate"
          on:click={() => applyPendingChange(true)}
        >
          {pendingChange.kind === 'storage' ? '📦 Migrar todo (Imágenes + BD)' : '📦 Migrar BD actual'}
        </button>
        <button
          type="button"
          class="prompt-btn switch-only"
          on:click={() => applyPendingChange(false)}
        >
          📁 Solo cambiar ruta
        </button>
        <button
          type="button"
          class="prompt-btn cancel"
          on:click={() => (pendingChange = null)}
        >
          Cancelar
        </button>
      </div>
    </div>
  {/if}

  {#if isBusy}
    <div class="msg">⏳ Procesando cambio de directorio...</div>
  {:else if message}
    <div class="msg {isError ? 'error' : ''}">{message}</div>
  {/if}
</div>

<style>
  .storage-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 8px;
    padding: 10px 16px;
    margin: 0 auto 20px auto;
    max-width: 650px;
    width: 100%;
    box-sizing: border-box;
    font-size: 13px;
    gap: 8px;
  }
  @media (prefers-color-scheme: dark) {
    .storage-box {
      background: rgba(255, 255, 255, 0.05);
      border-color: rgba(255, 255, 255, 0.12);
    }
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
    min-width: 145px;
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
  .change-btn {
    padding: 5px 12px;
    font-size: 12px;
    font-weight: 500;
    border-radius: 6px;
    border: 1px solid #d1d5db;
    background: #ffffff;
    cursor: pointer;
    white-space: nowrap;
    min-width: 112px;
    transition: background 0.15s, border-color 0.15s;
  }
  .change-btn:hover:not(:disabled) {
    background: #f3f4f6;
    border-color: #9ca3af;
  }
  .change-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  @media (prefers-color-scheme: dark) {
    .change-btn {
      background: #374151;
      color: #f3f4f6;
      border-color: #4b5563;
    }
    .change-btn:hover:not(:disabled) {
      background: #4b5563;
    }
  }
  .migration-prompt {
    width: 100%;
    margin-top: 6px;
    padding: 12px 14px;
    border-radius: 8px;
    background: rgba(99, 102, 241, 0.08);
    border: 1px solid rgba(99, 102, 241, 0.35);
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
    box-sizing: border-box;
  }
  .prompt-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .prompt-title {
    font-weight: 700;
    font-size: 13px;
    color: #4f46e5;
  }
  @media (prefers-color-scheme: dark) {
    .prompt-title {
      color: #a5b4fc;
    }
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
  .prompt-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 4px;
  }
  .prompt-btn {
    border: none;
    border-radius: 6px;
    padding: 7px 12px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s ease, transform 0.1s ease;
  }
  .prompt-btn:hover {
    opacity: 0.92;
    transform: translateY(-1px);
  }
  .prompt-btn.migrate {
    background: #6366f1;
    color: #ffffff;
  }
  .prompt-btn.switch-only {
    background: rgba(16, 185, 129, 0.18);
    border: 1px solid rgba(16, 185, 129, 0.4);
    color: #059669;
  }
  @media (prefers-color-scheme: dark) {
    .prompt-btn.switch-only {
      color: #34d399;
    }
  }
  .prompt-btn.cancel {
    background: rgba(156, 163, 175, 0.2);
    color: inherit;
  }
  .msg {
    font-size: 12px;
    color: #059669;
  }
  .msg.error {
    color: #ef4444;
  }
</style>
