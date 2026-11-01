<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { loadEmotes } from '../stores/emotes';

  let importing = false;
  let statusMsg = '';

  async function pickFolderAndImport() {
    if (importing) return;
    statusMsg = '';
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Seleccionar carpeta con imágenes para importar'
      });

      if (!selected) return;

      const folderPath = Array.isArray(selected) ? selected[0] : selected;
      if (!folderPath) return;

      importing = true;
      statusMsg = 'Importando y convirtiendo imágenes a WebP...';

      const imported = await invoke<any[]>('import_folder', { folderPath });
      await loadEmotes();

      if (imported && imported.length > 0) {
        statusMsg = `¡${imported.length} emote(s) importados y convertidos a WebP exitosamente!`;
      } else {
        statusMsg = 'No se encontraron imágenes compatibles (.png, .jpg, .gif, .webp) en la carpeta.';
      }

      setTimeout(() => {
        statusMsg = '';
      }, 4000);
    } catch (err: any) {
      console.error('Error importando carpeta:', err);
      statusMsg = `Error: ${err?.message || err}`;
    } finally {
      importing = false;
    }
  }
</script>

<style>
  .import-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin: 16px 0;
  }
  .import-btn {
    background: #6366f1;
    color: white;
    border: none;
    border-radius: 8px;
    padding: 10px 20px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 4px 6px -1px rgba(99, 102, 241, 0.2);
    transition: background 0.15s ease, transform 0.1s ease;
  }
  .import-btn:hover:not(:disabled) {
    background: #4f46e5;
    transform: translateY(-1px);
  }
  .import-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .status-text {
    font-size: 13px;
    color: #4b5563;
    min-height: 20px;
  }
  @media (prefers-color-scheme: dark) {
    .status-text {
      color: #9ca3af;
    }
  }
</style>

<div class="import-container">
  <button type="button" class="import-btn" on:click={pickFolderAndImport} disabled={importing}>
    {importing ? 'Importando a WebP...' : '📁 Importar Carpeta'}
  </button>
  {#if statusMsg}
    <div class="status-text">{statusMsg}</div>
  {/if}
</div>
