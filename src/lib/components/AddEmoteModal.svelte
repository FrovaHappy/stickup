<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { api } from '$lib/services/api';
  import { loadEmotes } from '$lib/stores/emotes';

  let isOpen = false;
  let isDragging = false;
  let isBusy = false;
  let showUrlInput = false;

  let urlValue = '';
  let customName = '';
  let statusMsg = '';
  let isError = false;

  let unlistenTauriDrop: (() => void) | null = null;

  function showFeedback(msg: string, error = false) {
    statusMsg = msg;
    isError = error;
    setTimeout(() => {
      if (statusMsg === msg) {
        statusMsg = '';
      }
    }, 4500);
  }

  onMount(async () => {
    try {
      const webview = getCurrentWebview();
      unlistenTauriDrop = await webview.onDragDropEvent(async (event) => {
        const payload = event.payload;
        if (payload.type === 'enter' || payload.type === 'over') {
          isOpen = true;
          isDragging = true;
        } else if (payload.type === 'leave') {
          isDragging = false;
        } else if (payload.type === 'drop') {
          isDragging = false;
          if (payload.paths && payload.paths.length > 0) {
            isOpen = true;
            await importFromPathsList(payload.paths);
          }
        }
      });
    } catch {
      // Fallback HTML5 drag & drop sigue activo si Tauri webview event no está disponible
    }
  });

  onDestroy(() => {
    if (unlistenTauriDrop) {
      unlistenTauriDrop();
    }
  });

  async function importFromPathsList(paths: string[]) {
    if (isBusy || paths.length === 0) return;
    isBusy = true;
    statusMsg = 'Importando archivos...';
    isError = false;

    try {
      const imported = await api.importPaths(paths);
      await loadEmotes();

      if (imported.length > 0) {
        showFeedback(
          `✓ ¡${imported.length} ${imported.length === 1 ? 'emote agregado' : 'emotes agregados'} exitosamente!`
        );
      } else {
        showFeedback(
          'No se encontraron imágenes compatibles (.png, .jpg, .jpeg, .gif, .webp).',
          true
        );
      }
    } catch (err: any) {
      showFeedback(`Error: ${err?.message || err}`, true);
    } finally {
      isBusy = false;
    }
  }

  async function pickFiles() {
    if (isBusy) return;
    try {
      const selected = await open({
        multiple: true,
        directory: false,
        title: 'Seleccionar uno o varios emotes',
        filters: [
          {
            name: 'Imágenes y GIFs',
            extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp']
          }
        ]
      });

      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      await importFromPathsList(paths);
    } catch (err: any) {
      showFeedback(`Error: ${err?.message || err}`, true);
    }
  }

  async function pickFolder() {
    if (isBusy) return;
    try {
      const selected = await open({
        directory: true,
        multiple: true,
        title: 'Seleccionar carpeta(s) con imágenes'
      });

      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      await importFromPathsList(paths);
    } catch (err: any) {
      showFeedback(`Error: ${err?.message || err}`, true);
    }
  }

  async function handlePasteClipboard() {
    if (isBusy) return;
    isBusy = true;
    statusMsg = 'Leyendo portapapeles...';
    isError = false;

    try {
      const created = await api.importFromClipboard(customName);
      customName = '';
      await loadEmotes();
      showFeedback(
        `✓ ¡Emote "${created.name}" (${created.format.toUpperCase()}) agregado desde el portapapeles!`
      );
    } catch (err: any) {
      showFeedback(`Error: ${err?.message || err}`, true);
    } finally {
      isBusy = false;
    }
  }

  async function handleAddFromUrl() {
    if (isBusy || !urlValue.trim()) return;
    isBusy = true;
    statusMsg = 'Descargando imagen desde URL...';
    isError = false;

    try {
      const created = await api.importFromUrl(urlValue.trim(), customName);
      urlValue = '';
      customName = '';
      showUrlInput = false;
      await loadEmotes();
      showFeedback(
        `✓ ¡Emote "${created.name}" (${created.format.toUpperCase()}) agregado desde URL!`
      );
    } catch (err: any) {
      showFeedback(`Error: ${err?.message || err}`, true);
    } finally {
      isBusy = false;
    }
  }

  async function handleHtmlDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    if (isBusy) return;

    const dt = e.dataTransfer;
    if (!dt) return;

    // 1. Si se arrastraron archivos desde el navegador o sistema
    if (dt.files && dt.files.length > 0) {
      isBusy = true;
      statusMsg = 'Procesando archivos arrastrados...';
      isError = false;
      let count = 0;

      try {
        for (let i = 0; i < dt.files.length; i++) {
          const file = dt.files[i];
          const buf = await file.arrayBuffer();
          const bytes = Array.from(new Uint8Array(buf));
          const stem = file.name.replace(/\.[^/.]+$/, '') || 'emote';
          await api.importRawBytes(bytes, stem);
          count++;
        }
        await loadEmotes();
        if (count > 0) {
          showFeedback(
            `✓ ¡${count} ${count === 1 ? 'emote agregado' : 'emotes agregados'} por Drag & Drop!`
          );
        }
      } catch (err: any) {
        showFeedback(`Error: ${err?.message || err}`, true);
      } finally {
        isBusy = false;
      }
      return;
    }

    // 2. Si se arrastró una URL de imagen desde el navegador
    const uri = dt.getData('text/uri-list') || dt.getData('text/plain');
    if (uri && (uri.startsWith('http://') || uri.startsWith('https://'))) {
      urlValue = uri.trim();
      await handleAddFromUrl();
    }
  }

  function handleGlobalPaste(e: ClipboardEvent) {
    const target = e.target as HTMLElement | null;
    if (
      target &&
      (target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable)
    ) {
      return;
    }
    e.preventDefault();
    isOpen = true;
    handlePasteClipboard();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen && !isBusy) {
      isOpen = false;
    }
  }
</script>

<svelte:window on:paste={handleGlobalPaste} on:keydown={handleKeydown} />

<button
  type="button"
  class="add-trigger-btn"
  on:click={() => (isOpen = true)}
  title="Agregar emotes (Archivos, Carpeta, Portapapeles, URL o Drag & Drop)"
>
  <span class="plus-icon">+</span>
  <span>Agregar</span>
</button>

{#if isOpen}
  <div
    class="modal-backdrop"
    on:click={(e) => {
      if (e.target === e.currentTarget && !isBusy) isOpen = false;
    }}
    role="presentation"
  >
    <div class="upload-card">
      <button
        type="button"
        class="close-modal-btn"
        on:click={() => (isOpen = false)}
        disabled={isBusy}
        aria-label="Cerrar ventana"
      >
        ✕
      </button>

      <div
        class="dropzone-box {isDragging ? 'dragging' : ''}"
        on:dragover|preventDefault={() => (isDragging = true)}
        on:dragleave|preventDefault={() => (isDragging = false)}
        on:drop={handleHtmlDrop}
        role="region"
        aria-label="Zona para arrastrar y soltar emotes"
      >
        <!-- Icono de carpeta azul estilo ilustración -->
        <div class="folder-illustration">
          <svg width="68" height="58" viewBox="0 0 68 58" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
              d="M6 10C6 6.68629 8.68629 4 12 4H25.5147C27.106 4 28.6321 4.63214 29.7574 5.75736L34 10H54C57.3137 10 60 12.6863 60 16V48C60 51.3137 57.3137 54 54 54H12C8.68629 54 6 51.3137 6 48V10Z"
              fill="#3B5BFF"
            />
            <path
              d="M9 14H56V46H9V14Z"
              fill="#0F172A"
            />
            <path
              d="M6 22C6 19.2386 8.23858 17 11 17H61.5C64.5376 17 66.8548 19.7263 66.3556 22.7219L61.6889 50.7219C61.2909 53.1098 59.2248 54.8571 56.8037 54.8571H11C8.23858 54.8571 6 52.6186 6 49.8571V22Z"
              fill="#4C6EF5"
            />
            <rect x="44" y="25" width="16" height="4" rx="2" fill="#1E3A8A" />
            <rect x="42" y="33" width="16" height="4" rx="2" fill="#1E3A8A" />
          </svg>
        </div>

        <p class="dropzone-title">
          {#if isDragging}
            ¡Suelta tus archivos o carpetas aquí para importarlos!
          {:else}
            Arrastra y suelta tus archivos o carpetas aquí, o usa los botones de abajo.
          {/if}
        </p>
        <span class="dropzone-subtitle">
          Soporta 1 o varios archivos (.png, .jpg, .webp, .gif), carpetas completas, portapapeles y URL
        </span>

        <div class="or-divider">
          <span class="or-line"></span>
          <span class="or-text">OR</span>
          <span class="or-line"></span>
        </div>

        <div class="primary-actions">
          <button
            type="button"
            class="choose-btn primary"
            on:click={pickFiles}
            disabled={isBusy}
          >
            🖼️ Elegir Archivo(s)
          </button>

          <button
            type="button"
            class="choose-btn secondary"
            on:click={pickFolder}
            disabled={isBusy}
          >
            📁 Elegir Carpeta
          </button>
        </div>

        <div class="secondary-actions">
          <button
            type="button"
            class="pill-action-btn"
            on:click={handlePasteClipboard}
            disabled={isBusy}
            title="También puedes presionar Ctrl+V en cualquier momento"
          >
            📋 Pegar Portapapeles (Ctrl+V)
          </button>

          <button
            type="button"
            class="pill-action-btn {showUrlInput ? 'active' : ''}"
            on:click={() => (showUrlInput = !showUrlInput)}
            disabled={isBusy}
          >
            🔗 Desde URL
          </button>
        </div>

        {#if showUrlInput}
          <form class="url-form" on:submit|preventDefault={handleAddFromUrl}>
            <input
              type="url"
              class="url-field"
              placeholder="https://ejemplo.com/emote.gif o .webp / .png"
              bind:value={urlValue}
              disabled={isBusy}
              required
            />
            <div class="url-bottom-row">
              <input
                type="text"
                class="url-field name-field"
                placeholder="Nombre opcional"
                bind:value={customName}
                disabled={isBusy}
              />
              <button
                type="submit"
                class="choose-btn primary compact"
                disabled={isBusy || !urlValue.trim()}
              >
                {isBusy ? '⏳...' : '➕ Importar URL'}
              </button>
            </div>
          </form>
        {/if}

        {#if statusMsg}
          <div class="feedback-banner {isError ? 'error' : 'success'}">
            {statusMsg}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .add-trigger-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: #3b5bff;
    color: #ffffff;
    border: none;
    border-radius: 10px;
    padding: 9px 18px;
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(59, 91, 255, 0.35);
    transition: background 0.15s ease, transform 0.12s ease, box-shadow 0.15s ease;
  }
  .add-trigger-btn:hover {
    background: #2f49d1;
    transform: translateY(-1px);
    box-shadow: 0 6px 18px rgba(59, 91, 255, 0.45);
  }
  .add-trigger-btn:active {
    transform: scale(0.98);
  }
  .plus-icon {
    font-size: 18px;
    line-height: 1;
    font-weight: 800;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.65);
    backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 16px;
    animation: fadeIn 0.15s ease-out;
  }
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .upload-card {
    position: relative;
    background: #ffffff;
    border-radius: 22px;
    padding: 20px;
    width: 100%;
    max-width: 460px;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.22);
    box-sizing: border-box;
  }
  @media (prefers-color-scheme: dark) {
    .upload-card {
      background: #1e1f26;
      box-shadow: 0 24px 48px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(255, 255, 255, 0.08);
    }
  }

  .close-modal-btn {
    position: absolute;
    top: 12px;
    right: 14px;
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 18px;
    font-weight: 700;
    cursor: pointer;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
    z-index: 2;
  }
  .close-modal-btn:hover:not(:disabled) {
    background: rgba(148, 163, 184, 0.15);
    color: #ef4444;
  }

  .dropzone-box {
    background: #f4f7fb;
    border: 1.5px dashed #cbd5e1;
    border-radius: 16px;
    padding: 32px 24px 26px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 12px;
    transition: border-color 0.15s ease, background 0.15s ease, transform 0.15s ease;
  }
  .dropzone-box.dragging {
    border-color: #3b5bff;
    background: rgba(59, 91, 255, 0.08);
    transform: scale(1.01);
  }
  @media (prefers-color-scheme: dark) {
    .dropzone-box {
      background: #262830;
      border-color: rgba(255, 255, 255, 0.16);
    }
    .dropzone-box.dragging {
      border-color: #4c6ef5;
      background: rgba(76, 110, 245, 0.14);
    }
  }

  .folder-illustration {
    margin-bottom: 4px;
    filter: drop-shadow(0 6px 12px rgba(59, 91, 255, 0.22));
  }

  .dropzone-title {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    line-height: 1.45;
    color: #334155;
    max-width: 290px;
  }
  @media (prefers-color-scheme: dark) {
    .dropzone-title {
      color: #f1f5f9;
    }
  }

  .dropzone-subtitle {
    font-size: 11.5px;
    color: #64748b;
    max-width: 320px;
    line-height: 1.35;
  }
  @media (prefers-color-scheme: dark) {
    .dropzone-subtitle {
      color: #94a3b8;
    }
  }

  .or-divider {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    max-width: 220px;
    margin: 4px 0;
  }
  .or-line {
    flex: 1;
    height: 1px;
    background: #cbd5e1;
  }
  @media (prefers-color-scheme: dark) {
    .or-line {
      background: rgba(255, 255, 255, 0.15);
    }
  }
  .or-text {
    font-size: 12px;
    font-weight: 600;
    color: #64748b;
    letter-spacing: 0.5px;
  }

  .primary-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    width: 100%;
  }

  .choose-btn {
    border: none;
    border-radius: 9px;
    padding: 11px 20px;
    font-size: 13.5px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
  }
  .choose-btn.primary {
    background: #3b5bff;
    color: #ffffff;
    box-shadow: 0 4px 12px rgba(59, 91, 255, 0.3);
  }
  .choose-btn.primary:hover:not(:disabled) {
    background: #2f49d1;
    transform: translateY(-1px);
  }
  .choose-btn.secondary {
    background: rgba(59, 91, 255, 0.1);
    color: #3b5bff;
    border: 1px solid rgba(59, 91, 255, 0.25);
  }
  .choose-btn.secondary:hover:not(:disabled) {
    background: rgba(59, 91, 255, 0.18);
    transform: translateY(-1px);
  }
  @media (prefers-color-scheme: dark) {
    .choose-btn.secondary {
      background: rgba(76, 110, 245, 0.16);
      color: #93c5fd;
      border-color: rgba(76, 110, 245, 0.35);
    }
  }
  .choose-btn.compact {
    padding: 8px 14px;
    font-size: 12.5px;
  }
  .choose-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .secondary-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    margin-top: 2px;
  }

  .pill-action-btn {
    background: rgba(100, 116, 139, 0.12);
    border: 1px solid rgba(100, 116, 139, 0.22);
    color: #475569;
    border-radius: 999px;
    padding: 6px 13px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .pill-action-btn:hover:not(:disabled),
  .pill-action-btn.active {
    background: rgba(59, 91, 255, 0.14);
    border-color: #3b5bff;
    color: #3b5bff;
  }
  @media (prefers-color-scheme: dark) {
    .pill-action-btn {
      background: rgba(255, 255, 255, 0.06);
      border-color: rgba(255, 255, 255, 0.12);
      color: #cbd5e1;
    }
    .pill-action-btn:hover:not(:disabled),
    .pill-action-btn.active {
      background: rgba(76, 110, 245, 0.22);
      border-color: #60a5fa;
      color: #ffffff;
    }
  }

  .url-form {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 6px;
    padding-top: 10px;
    border-top: 1px solid rgba(148, 163, 184, 0.2);
  }
  .url-bottom-row {
    display: flex;
    gap: 8px;
  }
  .url-field {
    width: 100%;
    box-sizing: border-box;
    padding: 8px 11px;
    border-radius: 8px;
    border: 1px solid #cbd5e1;
    background: #ffffff;
    color: inherit;
    font-size: 12.5px;
    outline: none;
  }
  .url-field:focus {
    border-color: #3b5bff;
  }
  .url-field.name-field {
    flex: 1;
  }
  @media (prefers-color-scheme: dark) {
    .url-field {
      background: #1b1c22;
      border-color: rgba(255, 255, 255, 0.15);
    }
  }

  .feedback-banner {
    width: 100%;
    box-sizing: border-box;
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    margin-top: 4px;
  }
  .feedback-banner.success {
    background: rgba(16, 185, 129, 0.15);
    color: #059669;
  }
  .feedback-banner.error {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }
  @media (prefers-color-scheme: dark) {
    .feedback-banner.success {
      color: #34d399;
    }
    .feedback-banner.error {
      color: #f87171;
    }
  }
</style>

