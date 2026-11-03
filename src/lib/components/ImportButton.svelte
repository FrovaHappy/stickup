<script lang="ts">
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';
  import { formatFileSize } from '$lib/utils/format';
  import type { ResyncStats } from '$lib/types/emote';
  import { Alert, Badge, Button, Card } from '$lib/components/system-ui';
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

    <Button
      variant="success"
      disabled={resyncing}
      on:click={handleResync}
      title="Escanea el directorio activo, convierte PNG/JPG a WebP eliminando originales y sincroniza la base de datos"
    >
      {resyncing ? '🔄 Resincronizando...' : '🔄 Resincronizar'}
    </Button>
  </div>

  {#if statusMsg}
    <div class="status-box">
      <Alert variant="error" dismissible on:close={() => (statusMsg = '')}>
        {statusMsg}
      </Alert>
    </div>
  {/if}

  {#if resyncStats}
    <div class="stats-panel">
      <div class="stats-header">
        <span class="stats-title">📊 Resultado de la Resincronización</span>
        <Button
          variant="icon"
          size="sm"
          ariaLabel="Cerrar estadísticas"
          on:click={() => (resyncStats = null)}
        >
          ✕
        </Button>
      </div>

      <div class="stats-grid">
        <Card padding="sm">
          <span class="stat-val">{resyncStats.scanned_files}</span>
          <span class="stat-label">Archivos escaneados</span>
        </Card>

        <Card padding="sm" highlight="indigo">
          <span class="stat-val">{resyncStats.converted_to_webp}</span>
          <span class="stat-label">
            Convertidos a WebP ({resyncStats.png_converted} PNG, {resyncStats.jpg_converted} JPG)
          </span>
        </Card>

        <Card padding="sm">
          <span class="stat-val">+{resyncStats.added_to_db}</span>
          <span class="stat-label">Nuevos indexados</span>
        </Card>

        <Card padding="sm">
          <span class="stat-val">-{resyncStats.removed_missing_from_db}</span>
          <span class="stat-label">Huérfanos limpiados</span>
        </Card>
      </div>

      {#if resyncStats.converted_to_webp > 0}
        <div class="savings-bar">
          <span>
            Peso antes: <strong>{formatFileSize(resyncStats.bytes_before)}</strong> ➜ Después:
            <strong>{formatFileSize(resyncStats.bytes_after)}</strong>
          </span>
          <Badge variant={resyncStats.saved_bytes >= 0 ? 'green' : 'amber'}>
            {resyncStats.saved_bytes >= 0
              ? `Ahorro: ${formatFileSize(resyncStats.saved_bytes)} (-${savingsPercent}%)`
              : `Diferencia: +${formatFileSize(Math.abs(resyncStats.saved_bytes))}`}
          </Badge>
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
  .status-box {
    width: 100%;
    max-width: 650px;
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
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 10px;
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
</style>
