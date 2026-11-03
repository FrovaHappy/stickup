<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { api } from "$lib/services/api";
  import { loadEmotes } from "$lib/stores/emotes";
  import {
    Alert,
    Button,
    Chip,
    Dropzone,
    Input,
    Modal,
  } from "$lib/components/system-ui";

  let isOpen = false;
  let isDragging = false;
  let isBusy = false;
  let showUrlInput = false;

  let urlValue = "";
  let customName = "";
  let statusMsg = "";
  let isError = false;

  let unlistenTauriDrop: (() => void) | null = null;

  function showFeedback(msg: string, error = false) {
    statusMsg = msg;
    isError = error;
    setTimeout(() => {
      if (statusMsg === msg) {
        statusMsg = "";
      }
    }, 4500);
  }

  onMount(async () => {
    try {
      const webview = getCurrentWebview();
      unlistenTauriDrop = await webview.onDragDropEvent(async (event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") {
          isOpen = true;
          isDragging = true;
        } else if (payload.type === "leave") {
          isDragging = false;
        } else if (payload.type === "drop") {
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
    statusMsg = "Importando archivos...";
    isError = false;

    try {
      const imported = await api.importPaths(paths);
      await loadEmotes();

      if (imported.length > 0) {
        showFeedback(
          `✓ ¡${imported.length} ${imported.length === 1 ? "emote agregado" : "emotes agregados"} exitosamente!`,
        );
      } else {
        showFeedback(
          "No se encontraron imágenes compatibles (.png, .jpg, .jpeg, .gif, .webp).",
          true,
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
        title: "Seleccionar uno o varios emotes",
        filters: [
          {
            name: "Imágenes y GIFs",
            extensions: ["png", "jpg", "jpeg", "gif", "webp"],
          },
        ],
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
        title: "Seleccionar carpeta(s) con imágenes",
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
    statusMsg = "Leyendo portapapeles...";
    isError = false;

    try {
      const created = await api.importFromClipboard(customName);
      customName = "";
      await loadEmotes();
      showFeedback(
        `✓ ¡Emote "${created.name}" (${created.format.toUpperCase()}) agregado desde el portapapeles!`,
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
    statusMsg = "Descargando imagen desde URL...";
    isError = false;

    try {
      const created = await api.importFromUrl(urlValue.trim(), customName);
      urlValue = "";
      customName = "";
      showUrlInput = false;
      await loadEmotes();
      showFeedback(
        `✓ ¡Emote "${created.name}" (${created.format.toUpperCase()}) agregado desde URL!`,
      );
    } catch (err: any) {
      showFeedback(`Error: ${err?.message || err}`, true);
    } finally {
      isBusy = false;
    }
  }

  async function handleHtmlDrop(e: CustomEvent<DragEvent>) {
    const dragEvent = e.detail;
    if (isBusy) return;

    const dt = dragEvent.dataTransfer;
    if (!dt) return;

    if (dt.files && dt.files.length > 0) {
      isBusy = true;
      statusMsg = "Procesando archivos arrastrados...";
      isError = false;
      let count = 0;

      try {
        for (let i = 0; i < dt.files.length; i++) {
          const file = dt.files[i];
          const buf = await file.arrayBuffer();
          const bytes = Array.from(new Uint8Array(buf));
          const stem = file.name.replace(/\.[^/.]+$/, "") || "emote";
          await api.importRawBytes(bytes, stem);
          count++;
        }
        await loadEmotes();
        if (count > 0) {
          showFeedback(
            `✓ ¡${count} ${count === 1 ? "emote agregado" : "emotes agregados"} por Drag & Drop!`,
          );
        }
      } catch (err: any) {
        showFeedback(`Error: ${err?.message || err}`, true);
      } finally {
        isBusy = false;
      }
      return;
    }

    const uri = dt.getData("text/uri-list") || dt.getData("text/plain");
    if (uri && (uri.startsWith("http://") || uri.startsWith("https://"))) {
      urlValue = uri.trim();
      await handleAddFromUrl();
    }
  }

  function handleGlobalPaste(e: ClipboardEvent) {
    const target = e.target as HTMLElement | null;
    if (
      target &&
      (target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable)
    ) {
      return;
    }
    e.preventDefault();
    isOpen = true;
    handlePasteClipboard();
  }
</script>

<svelte:window on:paste={handleGlobalPaste} />

<Button
  variant="primary"
  on:click={() => (isOpen = true)}
  title="Agregar emotes (Archivos, Carpeta, Portapapeles, URL o Drag & Drop)"
>
  <span class="plus-icon">+</span>
  <span>Agregar</span>
</Button>

<Modal bind:open={isOpen} maxWidth="460px" busy={isBusy}>
  <Dropzone
    bind:isDragging
    disabled={isBusy}
    ariaLabel="Zona para arrastrar y soltar emotes"
    on:drop={handleHtmlDrop}
  >
    <div class="primary-actions">
      <Button variant="primary" disabled={isBusy} on:click={pickFiles}>
        🖼️ Elegir Archivo(s)
      </Button>

      <Button variant="secondary" disabled={isBusy} on:click={pickFolder}>
        📁 Elegir Carpeta
      </Button>
    </div>

    <div class="secondary-actions">
      <Chip
        pill
        disabled={isBusy}
        title="También puedes presionar Ctrl+V en cualquier momento"
        on:click={handlePasteClipboard}
      >
        📋 Pegar Portapapeles (Ctrl+V)
      </Chip>

      <Chip
        pill
        active={showUrlInput}
        disabled={isBusy}
        on:click={() => (showUrlInput = !showUrlInput)}
      >
        🔗 Desde URL
      </Chip>
    </div>

    {#if showUrlInput}
      <form class="url-form" on:submit|preventDefault={handleAddFromUrl}>
        <Input
          type="url"
          placeholder="https://ejemplo.com/emote.gif o .webp / .png"
          bind:value={urlValue}
          disabled={isBusy}
          required
        />
        <div class="url-bottom-row">
          <div class="name-col">
            <Input
              type="text"
              placeholder="Nombre opcional"
              bind:value={customName}
              disabled={isBusy}
            />
          </div>
          <Button
            type="submit"
            variant="primary"
            size="sm"
            loading={isBusy}
            disabled={isBusy || !urlValue.trim()}
          >
            ➕ Importar URL
          </Button>
        </div>
      </form>
    {/if}

    {#if statusMsg}
      <Alert variant={isError ? "error" : "success"}>
        {statusMsg}
      </Alert>
    {/if}
  </Dropzone>
</Modal>

<style>
  .plus-icon {
    font-size: 18px;
    line-height: 1;
    font-weight: 800;
  }
  .primary-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    width: 100%;
  }
  .secondary-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    margin-top: 2px;
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
    align-items: center;
    gap: 8px;
  }
  .name-col {
    flex: 1;
    min-width: 0;
  }
</style>
