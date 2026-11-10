<script lang="ts">
  import { onMount } from 'svelte';
  import {
    ArrowLeft,
    Database,
    Sliders,
    Info,
    ShieldCheck,
    Monitor,
    Keyboard,
    RotateCcw,
    Star,
    RefreshCw,
    Sparkles,
    Flame,
    ExternalLink,
    Wand2,
    Trash2,
    Download,
    Check,
    Cpu,
    ZapOff
  } from '@lucide/svelte';
  import StorageSettings from '$lib/components/StorageSettings.svelte';
  import ImportButton from '$lib/components/ImportButton.svelte';
  import { Button, Badge, Logo } from '$lib/components/system-ui';
  import { api } from '$lib/services/api';
  import {
    currentAppVersion,
    loadCurrentAppVersion,
    autoCheckUpdates,
    setAutoCheck,
    includePrereleases,
    setIncludePrereleases,
    isCheckingUpdates,
    availableUpdate,
    checkForUpdates,
    openUpdateModal
  } from '$lib/stores/updater';
  import {
    customSizes,
    addCustomSize,
    removeCustomSize,
    DEFAULT_SIZE_PRESETS,
    allSizePresets,
    defaultResolution,
    setDefaultResolution
  } from '$lib/stores/sizes';
  import { askBeforePaste, setAskBeforePaste } from '$lib/stores/preferences';
  import { toast } from '$lib/stores/toast';
  import {
    autostartEnabled,
    minimizeToTrayEnabled,
    minimizeOnCopyEnabled,
    forceGifLoopEnabled,
    globalShortcut,
    DEFAULT_GLOBAL_SHORTCUT,
    systemSettingsLoading,
    loadSystemSettings,
    setAutostartSetting,
    setMinimizeToTraySetting,
    setMinimizeOnCopySetting,
    setForceGifLoopSetting,
    setGlobalShortcutSetting,
    formatShortcutKeys
  } from '$lib/stores/systemSettings';
  import {
    bgModelStatus,
    modelPresets,
    activeModelId,
    isDownloadingModel,
    downloadProgress,
    checkBgModelStatus,
    loadModelPresets,
    setActiveModel,
    downloadBgModel,
    deleteBgModel,
    formatBytes,
    idleTimeoutSecs,
    isModelLoadedInRam,
    loadIdleTimeout,
    setIdleTimeout,
    checkModelLoadedInRam,
    unloadModelSession
  } from '$lib/stores/bgRemoval';

  let newSizeWidth: number | null = null;
  let newSizeHeight: number | null = null;
  let isRecordingShortcut = false;

  const GITHUB_REPO_URL = 'https://github.com/FrovaHappy/stickup';

  function openGitHubRepo() {
    api.openExternalUrl(GITHUB_REPO_URL).catch(() => {
      window.open(GITHUB_REPO_URL, '_blank');
    });
  }

  onMount(() => {
    loadSystemSettings();
    loadCurrentAppVersion();
    checkBgModelStatus();
    loadModelPresets();
    loadIdleTimeout();
    checkModelLoadedInRam();
    window.addEventListener('keydown', handleKeydown, true);
    return () => {
      window.removeEventListener('keydown', handleKeydown, true);
    };
  });

  function startRecordingShortcut() {
    isRecordingShortcut = true;
  }

  function stopRecordingShortcut() {
    isRecordingShortcut = false;
  }

  async function resetDefaultShortcut() {
    await setGlobalShortcutSetting(DEFAULT_GLOBAL_SHORTCUT);
  }

  async function handleKeydown(e: KeyboardEvent) {
    if (!isRecordingShortcut) return;

    e.preventDefault();
    e.stopPropagation();

    if (e.key === 'Escape') {
      stopRecordingShortcut();
      return;
    }

    // Tecla modificadora sola
    if (['Control', 'Shift', 'Alt', 'Meta', 'OS'].includes(e.key)) {
      return;
    }

    const parts: string[] = [];
    if (e.metaKey) parts.push('Super');
    if (e.ctrlKey) parts.push('Control');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');

    if (parts.length === 0) {
      toast.error('El atajo debe incluir al menos una tecla modificadora (Win, Ctrl, Alt o Shift)');
      return;
    }

    let codeName = '';
    if (e.code === 'Period' || e.key === '.') {
      codeName = 'Period';
    } else if (e.code === 'Comma' || e.key === ',') {
      codeName = 'Comma';
    } else if (e.code === 'Space' || e.key === ' ') {
      codeName = 'Space';
    } else if (/^Key[A-Z]$/.test(e.code)) {
      codeName = e.code;
    } else if (/^Digit[0-9]$/.test(e.code)) {
      codeName = e.code;
    } else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(e.code)) {
      codeName = e.code;
    } else if (
      [
        'ArrowUp',
        'ArrowDown',
        'ArrowLeft',
        'ArrowRight',
        'Enter',
        'Tab',
        'Backspace',
        'Delete',
        'Home',
        'End',
        'PageUp',
        'PageDown'
      ].includes(e.code)
    ) {
      codeName = e.code;
    } else if (e.key.length === 1 && /[a-zA-Z0-9]/.test(e.key)) {
      codeName = `Key${e.key.toUpperCase()}`;
    } else {
      toast.error(`Tecla no admitida (${e.key || e.code})`);
      return;
    }

    parts.push(codeName);
    const candidate = parts.join('+');
    stopRecordingShortcut();
    await setGlobalShortcutSetting(candidate);
  }

  function handleAddSize() {
    if (!newSizeWidth || !newSizeHeight) {
      toast.error('Especifica un ancho y alto válidos en píxeles');
      return;
    }
    const added = addCustomSize(newSizeWidth, newSizeHeight);
    if (added) {
      toast.success(`✓ Tamaño ${added.label} agregado`);
      newSizeWidth = null;
      newSizeHeight = null;
    } else {
      toast.error('El tamaño ingresado no es válido o ya existe');
    }
  }

  function handleRemoveSize(value: string, label: string) {
    removeCustomSize(value);
    toast.success(`Tamaño ${label} eliminado`);
  }

  function handleSetFavoriteResolution(value: string, label: string) {
    setDefaultResolution(value);
    toast.success(`✓ Resolución favorita establecida en ${label} (Ctrl+C)`);
  }
</script>

<div class="settings-page">
  <!-- Header fijo con estilo translúcido idéntico al patrón de Inicio -->
  <header class="settings-header">
    <div class="header-inner">
      <div class="header-nav">
        <a href="/" class="back-link" title="Volver a Emotes">
          <ArrowLeft size={16} />
          <span>Volver a Emotes</span>
        </a>
      </div>
      <div class="header-title-block">
        <h1 class="page-title">Configuración</h1>
        <p class="page-subtitle">Administra el almacenamiento, copias de seguridad y preferencias de StickUp.</p>
      </div>
    </div>
  </header>

  <!-- Contenido con ancho unificado y sin background en las tarjetas -->
  <div class="settings-content">
    <!-- Sección de Almacenamiento y Respaldos (Información expandida con copias hechas) -->
    <section class="settings-section">
      <div class="section-title-row">
        <Database size={18} class="section-icon" />
        <h2>Almacenamiento y Respaldos</h2>
      </div>
      <StorageSettings />
    </section>

    <!-- Sección de Comportamiento del Sistema (Sin background de tarjeta) -->
    <section class="settings-section">
      <div class="section-title-row">
        <Monitor size={18} class="section-icon" />
        <h2>Comportamiento del Sistema</h2>
      </div>
      <div class="section-card">
        <div class="system-toggles">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-title">Iniciar al arrancar el sistema</span>
              <p class="toggle-desc">
                Inicia StickUp automáticamente al encender tu equipo o iniciar sesión en segundo plano.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={$autostartEnabled}
              class="toggle-switch"
              class:active={$autostartEnabled}
              disabled={$systemSettingsLoading}
              title={$autostartEnabled ? 'Desactivar inicio con el sistema' : 'Activar inicio con el sistema'}
              on:click={() => setAutostartSetting(!$autostartEnabled)}
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          <div class="toggle-separator"></div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-title">Minimizar a la bandeja del sistema</span>
              <p class="toggle-desc">
                Al hacer clic en cerrar (×), mantiene StickUp ejecutándose en segundo plano en la bandeja de iconos del sistema.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={$minimizeToTrayEnabled}
              class="toggle-switch"
              class:active={$minimizeToTrayEnabled}
              disabled={$systemSettingsLoading}
              title={$minimizeToTrayEnabled ? 'Desactivar minimizar en la bandeja' : 'Activar minimizar en la bandeja'}
              on:click={() => setMinimizeToTraySetting(!$minimizeToTrayEnabled)}
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          <div class="toggle-separator"></div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-title">Minimizar al copiar emote</span>
              <p class="toggle-desc">
                Minimiza automáticamente la ventana de StickUp tras copiar un emote con éxito al portapapeles.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={$minimizeOnCopyEnabled}
              class="toggle-switch"
              class:active={$minimizeOnCopyEnabled}
              disabled={$systemSettingsLoading}
              title={$minimizeOnCopyEnabled ? 'Desactivar minimizar al copiar' : 'Activar minimizar al copiar'}
              on:click={() => setMinimizeOnCopySetting(!$minimizeOnCopyEnabled)}
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          <div class="toggle-separator"></div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-title">Confirmar al pegar imagen</span>
              <p class="toggle-desc">
                Muestra una ventana en la esquina inferior derecha para previsualizar y confirmar antes de guardar al presionar Ctrl+V.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={$askBeforePaste}
              class="toggle-switch"
              class:active={$askBeforePaste}
              title={$askBeforePaste ? 'Desactivar confirmación al pegar' : 'Activar confirmación al pegar'}
              on:click={() => setAskBeforePaste(!$askBeforePaste)}
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          <div class="toggle-separator"></div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-title">Forzar bucle infinito en GIFs al importar</span>
              <p class="toggle-desc">
                Normaliza los GIFs importados para que nunca se congelen tras reproducirse y mantengan animación continua en la app y al pegarlos.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={$forceGifLoopEnabled}
              class="toggle-switch"
              class:active={$forceGifLoopEnabled}
              disabled={$systemSettingsLoading}
              title={$forceGifLoopEnabled ? 'Desactivar bucle infinito automático' : 'Activar bucle infinito automático'}
              on:click={() => setForceGifLoopSetting(!$forceGifLoopEnabled)}
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          <div class="toggle-separator"></div>

          <div class="shortcut-setting-row">
            <div class="toggle-info">
              <span class="toggle-title">Acceso rápido global</span>
              <p class="toggle-desc">
                Atajo de teclado global para abrir o alternar la ventana de StickUp desde cualquier aplicación.
              </p>
            </div>
            <div class="shortcut-action-area">
              {#if isRecordingShortcut}
                <div class="recording-badge" role="status" aria-live="polite">
                  <span class="recording-dot"></span>
                  <span>Presiona la combinación de teclas...</span>
                  <button
                    type="button"
                    class="cancel-recording-btn"
                    on:click={stopRecordingShortcut}
                    title="Cancelar grabación"
                  >
                    Cancelar (Esc)
                  </button>
                </div>
              {:else}
                <div class="shortcut-keys-display" title="Atajo actual">
                  {#each formatShortcutKeys($globalShortcut) as key}
                    <kbd class="shortcut-key">{key}</kbd>
                  {/each}
                </div>
                <div class="shortcut-buttons">
                  <Button
                    variant="secondary"
                    size="sm"
                    disabled={$systemSettingsLoading}
                    on:click={startRecordingShortcut}
                    title="Configurar una nueva combinación de teclas"
                  >
                    <Keyboard size={14} />
                    <span>Cambiar</span>
                  </Button>
                  {#if $globalShortcut !== DEFAULT_GLOBAL_SHORTCUT}
                    <Button
                      variant="ghost"
                      size="sm"
                      disabled={$systemSettingsLoading}
                      on:click={resetDefaultShortcut}
                      title="Restablecer al atajo por defecto (Win + Ctrl + .)"
                    >
                      <RotateCcw size={14} />
                      <span>Por defecto</span>
                    </Button>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Sección de Mantenimiento y Sincronización (Sin background de tarjeta) -->
    <section class="settings-section">
      <div class="section-title-row">
        <ShieldCheck size={18} class="section-icon" />
        <h2>Sincronización e Importación</h2>
      </div>
      <div class="section-card">
        <div class="sync-row">
          <div class="sync-info">
            <span class="sync-title">Sincronizar y optimizar emotes</span>
            <p class="section-desc">
              Agrega nuevos emotes o sincroniza el directorio con la base de datos convirtiendo imágenes automáticamente a formato WebP optimizado.
            </p>
          </div>
          <div class="sync-action">
            <ImportButton />
          </div>
        </div>
      </div>
    </section>

    <!-- Sección de Inteligencia Artificial / Recorte de Fondo -->
    <section class="settings-section">
      <div class="section-title-row">
        <Wand2 size={18} class="section-icon" />
        <h2>Inteligencia Artificial y Recorte de Fondo</h2>
      </div>
      <div class="section-card">
        <p class="section-desc">
          Elige el modelo neuronal utilizado para eliminar fondos de imágenes estáticas 100% de forma local en tu equipo.
        </p>

        {#if $isDownloadingModel}
          <div class="download-progress-container mb-4">
            <div class="progress-info-row">
              <span class="progress-text">Descargando modelo desde Hugging Face...</span>
              <span class="progress-percent">
                {$downloadProgress?.percentage ? `${Math.round($downloadProgress.percentage)}%` : '0%'}
              </span>
            </div>
            <div class="progress-bar-bg">
              <div
                class="progress-bar-fill"
                style="width: {$downloadProgress?.percentage || 0}%;"
              ></div>
            </div>
            <div class="progress-bytes-row">
              <span class="bytes-text">
                {formatBytes($downloadProgress?.downloaded_bytes || 0)} /
                {$downloadProgress?.total_bytes ? formatBytes($downloadProgress.total_bytes) : 'Procesando...'}
              </span>
            </div>
          </div>
        {/if}

        <div class="model-presets-list">
          {#each $modelPresets as preset}
            {@const isActive = preset.id === $activeModelId}
            <div class="preset-card" class:active={isActive}>
              <div class="preset-card-header">
                <div class="preset-title-wrap">
                  <span class="preset-name">{preset.name}</span>
                  <div class="preset-badges">
                    {#if isActive}
                      <Badge variant="amber">Activo</Badge>
                    {/if}
                    {#if preset.is_downloaded}
                      <Badge variant="green">Instalado ({formatBytes(preset.file_size_bytes)})</Badge>
                    {:else}
                      <Badge variant="gray">~{preset.approx_size_mb} MB</Badge>
                    {/if}
                  </div>
                </div>

                <div class="preset-actions">
                  {#if !isActive}
                    <Button
                      variant="outline"
                      size="sm"
                      on:click={() => setActiveModel(preset.id)}
                      title="Establecer este modelo como el predeterminado para recortes"
                    >
                      <Check size={13} />
                      <span>Seleccionar</span>
                    </Button>
                  {/if}

                  {#if !preset.is_downloaded}
                    <Button
                      variant={isActive ? 'primary' : 'secondary'}
                      size="sm"
                      disabled={$isDownloadingModel}
                      on:click={() => downloadBgModel(preset.id)}
                      title={`Descargar ${preset.name} (~${preset.approx_size_mb} MB)`}
                    >
                      <Download size={13} />
                      <span>Descargar</span>
                    </Button>
                  {:else}
                    <Button
                      variant="outline"
                      size="sm"
                      disabled={$isDownloadingModel}
                      on:click={() => deleteBgModel(preset.id)}
                      title="Eliminar este archivo para liberar espacio en disco"
                    >
                      <Trash2 size={13} />
                      <span>Liberar</span>
                    </Button>
                  {/if}
                </div>
              </div>

              <p class="preset-desc">{preset.description}</p>
            </div>
          {/each}
        </div>

        <!-- Gestión de Memoria RAM e Inactividad -->
        <div class="ai-ram-management-box">
          <div class="ram-management-header">
            <div class="ram-title-group">
              <Cpu size={16} class="text-orange" />
              <span class="ram-management-title">Memoria RAM y Tiempo de Espera</span>
              {#if $isModelLoadedInRam}
                <Badge variant="amber">En RAM activa</Badge>
              {:else}
                <Badge variant="gray">Inactivo (0 MB en RAM)</Badge>
              {/if}
            </div>

            {#if $isModelLoadedInRam}
              <Button
                variant="secondary"
                size="sm"
                on:click={unloadModelSession}
                title="Descargar el modelo de la memoria RAM inmediatamente"
              >
                <ZapOff size={13} />
                <span>Liberar memoria ahora</span>
              </Button>
            {/if}
          </div>

          <p class="ram-management-desc">
            Define cuánto tiempo se mantiene el modelo cargado en memoria RAM tras un recorte para acelerar futuros recortes, o selecciona <strong>Instantáneo</strong> para liberar toda la memoria inmediatamente al terminar la inferencia.
          </p>

          <div class="timeout-options-row">
            <button
              type="button"
              class="timeout-btn"
              class:active={$idleTimeoutSecs === 0}
              on:click={() => setIdleTimeout(0)}
              title="Descargar de la memoria RAM inmediatamente tras cada recorte (0 segundos)"
            >
              Instantáneo
            </button>
            <button
              type="button"
              class="timeout-btn"
              class:active={$idleTimeoutSecs === 60}
              on:click={() => setIdleTimeout(60)}
              title="Descargar tras 1 minuto de inactividad (Por defecto)"
            >
              1 minuto (Por defecto)
            </button>
            <button
              type="button"
              class="timeout-btn"
              class:active={$idleTimeoutSecs === 300}
              on:click={() => setIdleTimeout(300)}
              title="Descargar tras 5 minutos de inactividad"
            >
              5 minutos
            </button>
            <button
              type="button"
              class="timeout-btn"
              class:active={$idleTimeoutSecs === 600}
              on:click={() => setIdleTimeout(600)}
              title="Descargar tras 10 minutos de inactividad"
            >
              10 minutos
            </button>
            <button
              type="button"
              class="timeout-btn"
              class:active={$idleTimeoutSecs === 1800}
              on:click={() => setIdleTimeout(1800)}
              title="Descargar tras 30 minutos de inactividad"
            >
              30 minutos
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Sección de Tamaños de Portapapeles Personalizados (Sin background de tarjeta) -->
    <section class="settings-section">
      <div class="section-title-row">
        <Sliders size={18} class="section-icon" />
        <h2>Tamaños de Portapapeles Personalizados</h2>
      </div>
      <div class="section-card">
        <p class="section-desc">
          Configura dimensiones personalizadas para copiar emotes con tamaños específicos desde el menú contextual.
        </p>

        <div class="favorite-resolution-box">
          <div class="favorite-header-row">
            <div class="favorite-info">
              <span class="favorite-title">Resolución favorita por defecto (Atajo Ctrl+C)</span>
              <p class="section-desc">
                Selecciona la resolución predeterminada al copiar cualquier emote con el atajo de teclado <code>Ctrl+C</code> o desde la acción principal del menú contextual.
              </p>
            </div>
          </div>
          <div class="favorite-chips-grid">
            {#each $allSizePresets as preset}
              {@const isFav = $defaultResolution === preset.value}
              <button
                type="button"
                class="favorite-chip-btn"
                class:active={isFav}
                aria-pressed={isFav}
                on:click={() => handleSetFavoriteResolution(preset.value, preset.label)}
                title={isFav ? `Resolución favorita actual: ${preset.label}` : `Elegir ${preset.label} como resolución favorita`}
              >
                <Star
                  size={14}
                  class={isFav ? 'star-icon star-active' : 'star-icon'}
                  fill={isFav ? 'currentColor' : 'none'}
                />
                <span>{preset.label}</span>
                {#if isFav}
                  <span class="favorite-tag">Favorita</span>
                {/if}
              </button>
            {/each}
          </div>
        </div>

        <div class="sizes-list">
          <div class="sizes-group">
            <span class="sizes-group-title">Tamaños predeterminados del sistema:</span>
            <div class="chips-row">
              {#each DEFAULT_SIZE_PRESETS as preset}
                <Badge variant="gray" shape="pill">
                  {preset.label}
                </Badge>
              {/each}
            </div>
          </div>

          <div class="sizes-group">
            <span class="sizes-group-title">Tamaños personalizados:</span>
            {#if $customSizes.length === 0}
              <p class="empty-hint">No hay tamaños personalizados creados aún.</p>
            {:else}
              <div class="chips-row">
                {#each $customSizes as custom}
                  <div class="custom-size-item">
                    <Badge variant="blue" shape="pill">
                      {custom.label}
                    </Badge>
                    <button
                      type="button"
                      class="remove-size-btn"
                      title={`Eliminar ${custom.label}`}
                      aria-label={`Eliminar ${custom.label}`}
                      on:click={() => handleRemoveSize(custom.value, custom.label)}
                    >
                      ×
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <div class="add-size-form">
            <span class="form-label">Añadir nuevo tamaño (px):</span>
            <div class="inputs-inline">
              <input
                type="number"
                min="16"
                max="4096"
                placeholder="Ancho (px)"
                class="num-input"
                bind:value={newSizeWidth}
              />
              <span class="x-separator">×</span>
              <input
                type="number"
                min="16"
                max="4096"
                placeholder="Alto (px)"
                class="num-input"
                bind:value={newSizeHeight}
              />
              <Button variant="primary" size="sm" on:click={handleAddSize}>
                Añadir tamaño
              </Button>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Acerca de StickUp y Gestión de Versiones -->
    <section class="settings-section">
      <div class="section-title-row">
        <Info size={18} class="section-icon" />
        <h2>Acerca de StickUp y Actualizaciones</h2>
      </div>
      <div class="section-card">
        <div class="about-card">
          <div class="about-header">
            <Logo size={44} interactive />
            <div class="about-info-col">
              <div class="about-title-row">
                <strong>StickUp - Gestor de Emotes</strong>
                <Badge variant="green" shape="pill">v{$currentAppVersion}</Badge>
              </div>
              <p class="about-text">
                Diseñado para copiar rápidamente stickers y emotes optimizados a cualquier aplicación de mensajería (Discord, Telegram, WhatsApp, Slack).
              </p>
              <div class="about-actions-row">
                <Button
                  variant="outline"
                  size="sm"
                  on:click={openGitHubRepo}
                  title="Abrir repositorio de StickUp en GitHub"
                >
                  <ExternalLink size={14} />
                  <span>Repositorio GitHub</span>
                </Button>
              </div>
            </div>
          </div>
        </div>

        <!-- Panel de Actualizaciones -->
        <div class="updater-box">
          <div class="updater-status-row">
            <div class="updater-status-info">
              {#if $availableUpdate?.hasUpdate}
                <div class="status-alert-row">
                  <span class="status-title text-orange">
                    <Sparkles size={16} /> ¡Nueva versión v{$availableUpdate.latestVersion} disponible!
                  </span>
                  {#if $availableUpdate.isHot}
                    <Badge variant="hot" shape="pill">
                      <Flame size={12} /> Caliente (&lt; 24h)
                    </Badge>
                  {/if}
                </div>
                <p class="updater-status-desc">
                  Hay una versión superior lista para instalar. Puedes actualizar en la app o descargar desde el navegador.
                </p>
              {:else}
                <span class="status-title">StickUp está actualizado</span>
                <p class="updater-status-desc">
                  Tienes instalada la versión más reciente (v{$currentAppVersion}).
                </p>
              {/if}
            </div>

            <div class="updater-actions">
              {#if $availableUpdate?.hasUpdate}
                <Button
                  variant="primary"
                  size="sm"
                  on:click={openUpdateModal}
                >
                  <Sparkles size={14} /> Ver actualización
                </Button>
              {/if}

              <Button
                variant={$availableUpdate?.hasUpdate ? 'outline' : 'secondary'}
                size="sm"
                loading={$isCheckingUpdates}
                disabled={$isCheckingUpdates}
                on:click={() => checkForUpdates(true)}
              >
                <RefreshCw size={14} class={$isCheckingUpdates ? 'spin-icon' : ''} />
                {$isCheckingUpdates ? 'Buscando...' : 'Buscar actualizaciones'}
              </Button>
            </div>
          </div>

          <div class="updater-toggles">
            <div class="toggle-row">
              <div class="toggle-info">
                <span class="toggle-title">Comprobar automáticamente al iniciar</span>
                <p class="toggle-desc">
                  Verifica una vez cada 24 horas si hay nuevas versiones publicadas en GitHub.
                </p>
              </div>
              <label class="switch">
                <input
                  type="checkbox"
                  checked={$autoCheckUpdates}
                  on:change={(e) => setAutoCheck(e.currentTarget.checked)}
                />
                <span class="slider"></span>
              </label>
            </div>

            <div class="toggle-row">
              <div class="toggle-info">
                <span class="toggle-title">Incluir versiones preliminares (Pre-releases)</span>
                <p class="toggle-desc">
                  Propone versiones en desarrollo antes del lanzamiento público general.
                </p>
              </div>
              <label class="switch">
                <input
                  type="checkbox"
                  checked={$includePrereleases}
                  on:change={(e) => setIncludePrereleases(e.currentTarget.checked)}
                />
                <span class="slider"></span>
              </label>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  .settings-page {
    width: 100%;
    min-height: 100%;
    display: flex;
    flex-direction: column;
  }

  /* Header sticky idéntico al patrón de Inicio */
  .settings-header {
    width: 100%;
    position: sticky;
    top: 0;
    z-index: 30;
    background: rgba(26, 26, 29, 0.82);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    padding: 16px 28px;
    box-sizing: border-box;
  }

  .header-inner {
    max-width: 960px;
    width: 100%;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .header-nav {
    display: flex;
    align-items: center;
  }

  .back-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: #a1a1aa;
    text-decoration: none;
    font-size: 13px;
    font-weight: 500;
    transition: color 0.15s ease;
  }

  .back-link:hover {
    color: var(--color-orange-primary, #f97316);
  }

  .header-title-block {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .page-title {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    color: #f4f4f5;
  }

  .page-subtitle {
    margin: 0;
    font-size: 13px;
    color: #a1a1aa;
  }

  /* Contenido principal con ancho unificado y padding coherente */
  .settings-content {
    max-width: 960px;
    width: 100%;
    margin: 0 auto;
    padding: 24px 28px 64px 28px;
    display: flex;
    flex-direction: column;
    gap: 28px;
    box-sizing: border-box;
  }

  .settings-section {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #e4e4e7;
  }

  .section-title-row :global(.section-icon) {
    color: var(--color-orange-primary, #f97316);
  }

  .section-title-row h2 {
    margin: 0;
    font-size: 15.5px;
    font-weight: 600;
  }

  /* Tarjeta sin fondo (background transparente) con borde sutil */
  .section-card {
    width: 100%;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 16px 18px;
    box-sizing: border-box;
  }

  .section-desc {
    margin: 0 0 14px 0;
    font-size: 13px;
    color: #a1a1aa;
    line-height: 1.5;
  }

  /* Sincronización e importación */
  .sync-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    flex-wrap: wrap;
    width: 100%;
  }

  .sync-info {
    flex: 1;
    min-width: 260px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .sync-title {
    font-size: 14px;
    font-weight: 600;
    color: #f4f4f5;
  }

  .sync-action {
    flex-shrink: 0;
  }

  /* Resolución favorita por defecto */
  .favorite-resolution-box {
    padding: 16px;
    background: transparent;
    border: 1px solid var(--color-border-subtle);
    border-radius: 8px;
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .favorite-title {
    font-size: 14px;
    font-weight: 600;
    color: #f4f4f5;
    display: block;
    margin-bottom: 4px;
  }

  .favorite-chips-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 4px;
  }

  .favorite-chip-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 9999px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    background: transparent;
    border: 1px solid var(--color-border-subtle);
    color: #d4d4d8;
    transition: all 0.15s ease;
  }

  .favorite-chip-btn:hover {
    border-color: rgba(249, 115, 22, 0.4);
    color: #ffffff;
    background: rgba(249, 115, 22, 0.06);
  }

  .favorite-chip-btn:focus-visible {
    outline: 2px solid var(--color-orange-primary);
    outline-offset: 2px;
  }

  .favorite-chip-btn.active {
    border-color: var(--color-orange-primary);
    background: rgba(249, 115, 22, 0.12);
    color: #fed7aa;
  }

  .favorite-chip-btn :global(.star-active) {
    color: var(--color-orange-primary);
  }

  .favorite-tag {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    background: var(--color-orange-primary);
    color: #ffffff;
    padding: 1px 6px;
    border-radius: 9999px;
    margin-left: 2px;
  }

  /* Tamaños personalizados */
  .sizes-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .sizes-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sizes-group-title {
    font-size: 11.5px;
    font-weight: 600;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .chips-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }

  .empty-hint {
    margin: 0;
    font-size: 13px;
    color: #71717a;
    font-style: italic;
  }

  .custom-size-item {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(59, 130, 246, 0.15);
    border-radius: 9999px;
    padding-right: 8px;
  }

  .remove-size-btn {
    background: transparent;
    border: none;
    color: #93c5fd;
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .remove-size-btn:hover {
    color: #ef4444;
  }

  .add-size-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .form-label {
    font-size: 12px;
    font-weight: 500;
    color: #a1a1aa;
  }

  .inputs-inline {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .num-input {
    width: 100px;
    padding: 6px 10px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(0, 0, 0, 0.25);
    color: #fff;
    font-size: 13px;
  }

  .num-input:focus {
    outline: none;
    border-color: var(--color-orange-primary, #f97316);
  }

  .x-separator {
    color: #71717a;
    font-weight: bold;
  }

  /* Acerca de */
  .about-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .about-header {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    font-size: 14px;
  }

  .about-info-col {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .about-title-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .about-text {
    margin: 0;
    font-size: 13px;
    color: #a1a1aa;
    line-height: 1.5;
  }

  .about-actions-row {
    margin-top: 4px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  /* Updater Box */
  .updater-box {
    margin-top: 18px;
    padding-top: 16px;
    border-top: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .updater-status-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.06));
    border-radius: 8px;
    padding: 12px 14px;
  }

  .updater-status-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-width: 240px;
  }

  .status-alert-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .status-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary, #f3f4f6);
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .status-title.text-orange {
    color: var(--color-orange-primary, #f97316);
  }

  .updater-status-desc {
    margin: 0;
    font-size: 12px;
    color: #a1a1aa;
    line-height: 1.4;
  }

  .updater-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .updater-toggles {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  :global(.spin-icon) {
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

  /* Toggles del sistema */
  .system-toggles {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 24px;
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }

  .toggle-title {
    font-size: 14px;
    font-weight: 600;
    color: #f4f4f5;
  }

  .toggle-desc {
    margin: 0;
    font-size: 13px;
    color: #a1a1aa;
    line-height: 1.4;
  }

  .toggle-separator {
    height: 1px;
    background: rgba(255, 255, 255, 0.06);
  }

  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 9999px;
    border: none;
    cursor: pointer;
    padding: 2px;
    transition: background-color 0.2s ease;
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .toggle-switch:focus-visible {
    outline: 2px solid var(--color-orange-primary, #f97316);
    outline-offset: 2px;
  }

  .toggle-switch.active {
    background: var(--color-orange-primary, #f97316);
  }

  .toggle-thumb {
    width: 20px;
    height: 20px;
    background: #ffffff;
    border-radius: 9999px;
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    display: block;
  }

  .toggle-switch.active .toggle-thumb {
    transform: translateX(20px);
    background: #18181b;
  }

  .toggle-switch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .shortcut-setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
    flex-wrap: wrap;
  }

  .shortcut-action-area {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .shortcut-keys-display {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .shortcut-key {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 28px;
    height: 28px;
    padding: 0 8px;
    background: #242427;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-bottom: 2px solid rgba(255, 255, 255, 0.22);
    border-radius: 6px;
    font-family: inherit;
    font-size: 12px;
    font-weight: 600;
    color: #f4f4f5;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.25);
  }

  .shortcut-buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .recording-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    background: rgba(249, 115, 22, 0.12);
    border: 1px dashed var(--color-orange-primary, #f97316);
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-orange-400, #fb923c);
  }

  .recording-dot {
    width: 8px;
    height: 8px;
    background-color: var(--color-orange-primary, #f97316);
    border-radius: 50%;
    animation: pulse-recording 1.2s infinite ease-in-out;
  }

  .cancel-recording-btn {
    background: transparent;
    border: none;
    color: #a1a1aa;
    cursor: pointer;
    font-size: 12px;
    padding: 2px 6px;
    border-radius: 4px;
    transition: color 0.15s ease;
  }

  .cancel-recording-btn:hover {
    color: #ffffff;
    text-decoration: underline;
  }

  @keyframes pulse-recording {
    0%,
    100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.3;
      transform: scale(0.8);
    }
  }

  .model-presets-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 14px;
  }

  .preset-card {
    background: transparent;
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: border-color 0.15s ease;
  }

  .preset-card.active {
    border-color: rgba(249, 115, 22, 0.4);
    background: rgba(249, 115, 22, 0.03);
  }

  .preset-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .preset-title-wrap {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .preset-name {
    font-size: 0.95rem;
    font-weight: 600;
    color: #ffffff;
  }

  .preset-badges {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .preset-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .preset-desc {
    margin: 0;
    font-size: 0.82rem;
    color: rgba(255, 255, 255, 0.65);
    line-height: 1.4;
  }

  .download-progress-container {
    margin-top: 8px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    padding: 12px;
    border-radius: 8px;
  }

  .download-progress-container.mb-4 {
    margin-bottom: 14px;
  }

  .progress-info-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    margin-bottom: 6px;
    color: rgba(255, 255, 255, 0.85);
  }

  .progress-percent {
    font-weight: 600;
    color: var(--color-orange-primary, #f97316);
  }

  .progress-bar-bg {
    width: 100%;
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--color-orange-primary, #f97316);
    transition: width 0.2s ease;
  }

  .progress-bytes-row {
    margin-top: 4px;
    text-align: right;
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.5);
  }

  .ai-ram-management-box {
    margin-top: 14px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .ram-management-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .ram-title-group {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .ram-management-title {
    font-size: 0.88rem;
    font-weight: 600;
    color: #ffffff;
  }

  .ram-management-desc {
    margin: 0;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
    line-height: 1.4;
  }

  .timeout-options-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    background: rgba(0, 0, 0, 0.25);
    padding: 4px;
    border-radius: 8px;
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    width: fit-content;
  }

  .timeout-btn {
    padding: 5px 12px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.65);
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .timeout-btn:hover {
    color: #ffffff;
  }

  .timeout-btn.active {
    background: rgba(249, 115, 22, 0.2);
    color: #ffffff;
    font-weight: 500;
  }
</style>
