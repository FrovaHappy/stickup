<script lang="ts">
  import {
    Sparkles,
    Flame,
    Download,
    ExternalLink,
    Play,
    CheckCircle2,
    Clock,
    ArrowRight
  } from '@lucide/svelte';
  import { Modal, Button, Badge } from '$lib/components/system-ui';
  import {
    availableUpdate,
    updateModalOpen,
    isDownloadingAsset,
    downloadedFilePath,
    closeUpdateModal,
    downloadAndInstallAsset,
    launchDownloadedInstaller,
    openInBrowser
  } from '$lib/stores/updater';
  import { formatFileSize } from '$lib/utils/format';

  $: info = $availableUpdate;
  $: release = info?.release;
  $: matchedAsset = info?.matchedAsset;
  $: isHot = info?.isHot ?? false;
  $: hours = info?.hoursSincePublished ?? 0;

  async function handleDownloadInApp() {
    if (!matchedAsset) return;
    try {
      const saved = await downloadAndInstallAsset(
        matchedAsset.browser_download_url,
        matchedAsset.name
      );
      if (saved) {
        // Ejecutar inmediatamente tras descargar
        await launchDownloadedInstaller(saved);
      }
    } catch {
      // Error manejado en el store con toast
    }
  }

  async function handleLaunchInstalled() {
    if ($downloadedFilePath) {
      await launchDownloadedInstaller($downloadedFilePath);
    }
  }

  async function handleOpenBrowser() {
    if (release?.html_url) {
      await openInBrowser(release.html_url);
    }
  }

  async function handleDirectAssetDownloadBrowser() {
    if (matchedAsset?.browser_download_url) {
      await openInBrowser(matchedAsset.browser_download_url);
    } else if (release?.html_url) {
      await openInBrowser(release.html_url);
    }
  }
</script>

<Modal
  open={$updateModalOpen}
  maxWidth="540px"
  onClose={closeUpdateModal}
>
  <div slot="header" class="update-header">
    <div class="update-title-col">
      <div class="title-with-badge">
        <Sparkles size={20} class="text-orange" />
        <h2 class="update-modal-title">Actualización disponible</h2>
      </div>

      {#if info}
        <div class="version-diff-row">
          <Badge variant="gray" shape="pill">v{info.currentVersion}</Badge>
          <ArrowRight size={14} class="diff-arrow" />
          <Badge variant="new" shape="pill">v{info.latestVersion}</Badge>
        </div>
      {/if}
    </div>
  </div>

  <div class="update-body">
    {#if isHot}
      <div class="hot-release-banner">
        <div class="hot-badge-row">
          <Badge variant="hot" shape="pill">
            <Flame size={12} class="hot-icon" />
            Versión Caliente / Reciente
          </Badge>
          <span class="hot-time">
            <Clock size={12} /> Publicada hace {hours === 0 ? 'menos de 1' : hours} hora{hours === 1 ? '' : 's'}
          </span>
        </div>
        <p class="hot-desc">
          Esta versión es muy reciente. Puedes actualizar ahora para disfrutar de las novedades o esperar si prefieres mayor estabilidad.
        </p>
      </div>
    {/if}

    {#if release?.body}
      <div class="changelog-container">
        <span class="changelog-title">Novedades de la versión:</span>
        <div class="changelog-content">
          <pre>{release.body}</pre>
        </div>
      </div>
    {/if}

    {#if matchedAsset}
      <div class="asset-info-box">
        <span class="asset-label">Instalador para tu sistema:</span>
        <div class="asset-details">
          <span class="asset-name">{matchedAsset.name}</span>
          <span class="asset-size">({formatFileSize(matchedAsset.size)})</span>
        </div>
      </div>
    {/if}

    <div class="modalities-section">
      <span class="section-subtitle">Elige cómo deseas actualizar:</span>

      <div class="modality-grid">
        <!-- Modalidad 1: Descarga directa en la app -->
        <div class="modality-card">
          <div class="modality-header">
            <div class="modality-icon">
              <Download size={18} />
            </div>
            <div class="modality-texts">
              <strong>Descargar en la app</strong>
              <p>Descarga el instalador a tu equipo y lo ejecuta automáticamente.</p>
            </div>
          </div>

          {#if $downloadedFilePath}
            <div class="downloaded-action">
              <span class="downloaded-success">
                <CheckCircle2 size={16} /> Descargado
              </span>
              <Button
                variant="primary"
                size="sm"
                on:click={handleLaunchInstalled}
              >
                <Play size={14} /> Ejecutar instalador
              </Button>
            </div>
          {:else}
            <Button
              variant="primary"
              size="sm"
              loading={$isDownloadingAsset}
              disabled={!matchedAsset || $isDownloadingAsset}
              on:click={handleDownloadInApp}
            >
              <Download size={14} />
              {$isDownloadingAsset ? 'Descargando...' : 'Descargar e instalar'}
            </Button>
          {/if}
        </div>

        <!-- Modalidad 2: Asistida en navegador -->
        <div class="modality-card">
          <div class="modality-header">
            <div class="modality-icon">
              <ExternalLink size={18} />
            </div>
            <div class="modality-texts">
              <strong>En el navegador</strong>
              <p>Abre la descarga directa o la página del release en GitHub.</p>
            </div>
          </div>

          <div class="browser-actions-row">
            {#if matchedAsset}
              <Button
                variant="secondary"
                size="sm"
                on:click={handleDirectAssetDownloadBrowser}
              >
                <Download size={14} /> Descargar archivo
              </Button>
            {/if}
            <Button
              variant="outline"
              size="sm"
              on:click={handleOpenBrowser}
            >
              <ExternalLink size={14} /> Ver en GitHub
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div slot="footer" class="update-footer">
    <Button
      variant="ghost"
      size="sm"
      on:click={closeUpdateModal}
    >
      Recordar más tarde
    </Button>
  </div>
</Modal>

<style>
  .update-header {
    display: flex;
    align-items: center;
    width: 100%;
  }

  .update-title-col {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .title-with-badge {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  :global(.text-orange) {
    color: var(--color-orange-primary, #f97316);
  }

  .update-modal-title {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    color: var(--color-text-primary, #f3f4f6);
  }

  .version-diff-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  :global(.diff-arrow) {
    color: var(--color-text-secondary, #9ca3af);
  }

  .update-body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px 0;
  }

  /* Hot Release Alert */
  .hot-release-banner {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: rgba(234, 88, 12, 0.12);
    border: 1px solid rgba(234, 88, 12, 0.35);
    border-radius: 8px;
    padding: 10px 14px;
  }

  .hot-badge-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }

  :global(.hot-icon) {
    margin-right: 4px;
  }

  .hot-time {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 0.8rem;
    color: var(--color-orange-hover, #ea580c);
  }

  .hot-desc {
    margin: 0;
    font-size: 0.82rem;
    color: var(--color-text-secondary, #d1d5db);
    line-height: 1.4;
  }

  /* Changelog */
  .changelog-container {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .changelog-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-secondary, #9ca3af);
  }

  .changelog-content {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    padding: 10px 12px;
    max-height: 160px;
    overflow-y: auto;
  }

  .changelog-content pre {
    margin: 0;
    font-family: inherit;
    font-size: 0.82rem;
    color: var(--color-text-primary, #e5e7eb);
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.45;
  }

  /* Asset info */
  .asset-info-box {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.06));
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 0.82rem;
  }

  .asset-label {
    color: var(--color-text-secondary, #9ca3af);
  }

  .asset-details {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .asset-name {
    font-family: monospace;
    color: var(--color-text-primary, #f3f4f6);
    font-weight: 500;
  }

  .asset-size {
    color: var(--color-text-secondary, #9ca3af);
  }

  /* Modalities */
  .modalities-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .section-subtitle {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-secondary, #9ca3af);
  }

  .modality-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .modality-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: transparent;
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    padding: 12px;
  }

  .modality-header {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }

  .modality-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    background: rgba(249, 115, 22, 0.12);
    color: var(--color-orange-primary, #f97316);
    flex-shrink: 0;
  }

  .modality-texts {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .modality-texts strong {
    font-size: 0.9rem;
    color: var(--color-text-primary, #f3f4f6);
  }

  .modality-texts p {
    margin: 0;
    font-size: 0.8rem;
    color: var(--color-text-secondary, #9ca3af);
    line-height: 1.35;
  }

  .downloaded-action {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .downloaded-success {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 0.82rem;
    color: #22c55e;
    font-weight: 500;
  }

  .browser-actions-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .update-footer {
    display: flex;
    justify-content: flex-end;
    width: 100%;
  }
</style>
