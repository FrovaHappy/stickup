<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/services/api';

  let storagePath = '';
  let message = '';

  onMount(async () => {
    try {
      storagePath = await api.getStoragePath();
    } catch (e) {
      console.error('Error al obtener la ruta de almacenamiento:', e);
    }
  });

  async function changeStoragePath() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Seleccionar carpeta para guardar emotes'
      });

      if (!selected) return;

      const newPath = Array.isArray(selected) ? selected[0] : selected;
      if (newPath) {
        storagePath = await api.setStoragePath(newPath);
        message = '✓ Ruta de almacenamiento actualizada';
        setTimeout(() => {
          message = '';
        }, 3000);
      }
    } catch (e: any) {
      console.error('Error al cambiar la ruta de almacenamiento:', e);
      message = `Error: ${e?.message || e}`;
    }
  }
</script>

<div class="storage-box">
  <div class="info-row">
    <span class="path-label">💾 Guardar emotes en:</span>
    <span class="path-val" title={storagePath}>{storagePath || 'Cargando...'}</span>
    <button type="button" class="change-btn" on:click={changeStoragePath}>
      ✏️ Cambiar ruta
    </button>
  </div>
  {#if message}
    <div class="msg">{message}</div>
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
    transition: background 0.15s, border-color 0.15s;
  }
  .change-btn:hover {
    background: #f3f4f6;
    border-color: #9ca3af;
  }
  @media (prefers-color-scheme: dark) {
    .change-btn {
      background: #374151;
      color: #f3f4f6;
      border-color: #4b5563;
    }
    .change-btn:hover {
      background: #4b5563;
    }
  }
  .msg {
    font-size: 12px;
    color: #059669;
  }
</style>
