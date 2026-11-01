<script lang="ts">
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';
  import { formatFileSize } from '$lib/utils/format';
  import type { ResyncStats } from '$lib/types/emote';
  import AddEmoteModal from './AddEmoteModal.svelte';

  let resyncing = false;
  let statusMsg = '';
  let resyncStats: ResyncStats | null = null;

  async function handleResync() {
    if (resyncing) return;
    statusMsg = '';
    resyncStats = null;
    resyncing = true;

    try {
      const result = await api.resyncEmotes();
      resyncStats = result;
      await loadEmotes();
    } catch (err: any) {
      console.error('Error al resincronizar directorio:', err);
      statusMsg = `Error al resincronizar: ${err?.message || err}`;
    } finally {
      resyncing = false;
    }
  }

  $: savingsPercent = (() => {
    if (!resyncStats || resyncStats.bytes_before <= 0) return 0;
    return Math.round((resyncStats.saved_bytes / resyncStats.bytes_before) * 100);
  })();
</script>

<div class="actions-wrapper">
  <div class="buttons-row">
    <AddEmoteModal />

    <button
      type="button"
      class="action-btn secondary"
      on:click={handleResync}
      disabled={resyncing}
      title="Escanea el directorio activo, convierte PNG/JPG a WebP eliminando originales y sincroniza la base de datos"
    >
      {resyncing ? '🔄 Resincronizando...' : '🔄 Resincronizar'}
    </button>
  </div>

  {#if statusMsg}
    <div class="status-text">{statusMsg}</div>
  {/if}

  {#if resyncStats}
    <div class="stats-panel">
      <div class="stats-header">
        <span class="stats-title">📊 Resultado de la Resincronización</span>
        <button
          type="button"
          class="stats-close"
          on:click={() => (resyncStats = null)}
          aria-label="Cerrar estadísticas"
        >
          ✕
        </button>
      </div>

      <div class="stats-grid">
        <div class="stat-item">
          <span class="stat-val">{resyncStats.scanned_files}</span>
          <span class="stat-label">Archivos escaneados</span>
        </div>

        <div class="stat-item highlight">
          <span class="stat-val">{resyncStats.converted_to_webp}</span>
          <span class="stat-label">
            Convertidos a WebP ({resyncStats.png_converted} PNG, {resyncStats.jpg_converted} JPG)
          </span>
        </div>

        <div class="stat-item">
          <span class="stat-val">+{resyncStats.added_to_db}</span>
          <span class="stat-label">Nuevos indexados</span>
        </div>

        <div class="stat-item">
          <span class="stat-val">-{resyncStats.removed_missing_from_db}</span>
          <span class="stat-label">Huérfanos limpiados</span>
        </div>
      </div>

      {#if resyncStats.converted_to_webp > 0}
        <div class="savings-bar">
          <span>
            Peso antes: <strong>{formatFileSize(resyncStats.bytes_before)}</strong> ➜ Después:
            <strong>{formatFileSize(resyncStats.bytes_after)}</strong>
          </span>
          <span class="savings-badge {resyncStats.saved_bytes >= 0 ? 'positive' : 'negative'}">
            {resyncStats.saved_bytes >= 0
              ? `Ahorro: ${formatFileSize(resyncStats.saved_bytes)} (-${savingsPercent}%)`
              : `Diferencia: +${formatFileSize(Math.abs(resyncStats.saved_bytes))}`}
          </span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .actions-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    margin: 12px 0 20px 0;
    width: 100%;
  }
  .buttons-row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    justify-content: center;
    align-items: center;
  }
  .action-btn.secondary {
    background: rgba(16, 185, 129, 0.14);
    color: #059669;
    border: 1px solid rgba(16, 185, 129, 0.35);
    border-radius: 10px;
    padding: 9px 16px;
    font-size: 13.5px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }
  .action-btn.secondary:hover:not(:disabled) {
    background: rgba(16, 185, 129, 0.24);
    transform: translateY(-1px);
  }
  @media (prefers-color-scheme: dark) {
    .action-btn.secondary {
      color: #10b981;
    }
  }
  .action-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .status-text {
    font-size: 13px;
    color: #ef4444;
    min-height: 20px;
  }
  .stats-panel {
    background: rgba(30, 30, 34, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 14px 18px;
    width: 100%;
    max-width: 650px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.25);
    color: #f3f4f6;
  }
  .stats-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 8px;
  }
  .stats-title {
    font-size: 13px;
    font-weight: 700;
    color: #e5e7eb;
  }
  .stats-close {
    background: transparent;
    border: none;
    color: #9ca3af;
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .stats-close:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.1);
  }
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 10px;
  }
  .stat-item {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .stat-item.highlight {
    background: rgba(99, 102, 241, 0.14);
    border-color: rgba(99, 102, 241, 0.35);
  }
  .stat-val {
    font-size: 18px;
    font-weight: 800;
    color: #ffffff;
  }
  .stat-label {
    font-size: 11px;
    color: #9ca3af;
    text-align: center;
  }
  .savings-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 8px;
    background: rgba(0, 0, 0, 0.25);
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 12px;
    color: #d1d5db;
  }
  .savings-badge {
    padding: 3px 8px;
    border-radius: 6px;
    font-weight: 700;
    font-size: 11px;
  }
  .savings-badge.positive {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }
  .savings-badge.negative {
    background: rgba(245, 158, 11, 0.2);
    color: #fbbf24;
  }
</style>
