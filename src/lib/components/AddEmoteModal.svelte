<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { Plus, Image, FolderOpen, Link } from "@lucide/svelte";
  import { api } from "$lib/services/api";
  import { loadEmotes } from "$lib/stores/emotes";
  import { toast } from "$lib/stores/toast";
  import type { Emote } from "$lib/types/emote";
  import {
    Alert,
    Button,
    Chip,
    Dropzone,
    Input,
    Modal,
  } from "$lib/components/system-ui";

  export let isOpen = false;
  export let customTrigger = false;
  let isDragging = false;
  let isBusy = false;
  let showUrlInput = false;

  let urlValue = "";
  let customName = "";
  let statusMsg = "";
  let isError = false;
  let lastPasteTimestamp = 0;
  const PASTE_COOLDOWN_MS = 1200;

  let unlistenTauriDrop: (() => void) | null = null;

  function showFeedback(msg: string, error = false) {
    statusMsg = msg;
    isError = error;
    if (error) {
      toast.error(msg);
    } else {
      toast.success(msg);
    }
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
      if (typeof err === "string" && err.includes("ya está en curso")) {
        return;
      }
      showFeedback(`Error: ${err?.message || err}`, true);
    } finally {
      setTimeout(() => {
        isBusy = false;
      }, 400);
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
    statusMsg = "Identificando contenido del portapapeles...";
    isError = false;

    try {
      const result = await api.importFromClipboard(customName);
      customName = "";
      await loadEmotes();

      const list: Emote[] = Array.isArray(result)
        ? result
        : result && typeof result === "object" && ("id" in result || "name" in result)
        ? [result as Emote]
        : [];

      if (list.length === 1) {
        const item = list[0];
        const fmt = item?.format ? ` (${item.format.toUpperCase()})` : "";
        const name = item?.name || "emote";
        showFeedback(
          `✓ ¡Emote "${name}"${fmt} agregado exitosamente!`,
        );
      } else if (list.length > 1) {
        showFeedback(
          `✓ ¡${list.length} emotes agregados exitosamente!`,
        );
      } else {
        showFeedback(
          "No se encontraron imágenes compatibles (.png, .jpg, .webp, .gif).",
          true,
        );
      }
    } catch (err: any) {
      if (typeof err === "string" && err.includes("ya está en curso")) {
        return;
      }
      showFeedback(`Error: ${err?.message || err}`, true);
    } finally {
      setTimeout(() => {
        isBusy = false;
      }, 400);
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
      const fmt = created?.format ? ` (${created.format.toUpperCase()})` : "";
      const name = created?.name || "emote";
      showFeedback(
        `✓ ¡Emote "${name}"${fmt} agregado desde URL!`,
      );
    } catch (err: any) {
      if (typeof err === "string" && err.includes("ya está en curso")) {
        return;
      }
      showFeedback(`Error: ${err?.message || err}`, true);
    } finally {
      setTimeout(() => {
        isBusy = false;
      }, 400);
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

  function handleGlobalKeyDown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && (e.key === "v" || e.key === "V")) {
      if (e.repeat) {
        e.preventDefault();
        return;
      }
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable)
      ) {
        return;
      }
      const now = Date.now();
      if (now - lastPasteTimestamp < PASTE_COOLDOWN_MS || isBusy) {
        e.preventDefault();
        return;
      }
      lastPasteTimestamp = now;
      e.preventDefault();
      handlePasteClipboard();
    }
  }

  async function handleGlobalPaste(e: ClipboardEvent) {
    const target = e.target as HTMLElement | null;
    if (
      target &&
      (target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable)
    ) {
      return;
    }
    const now = Date.now();
    if (now - lastPasteTimestamp < PASTE_COOLDOWN_MS || isBusy) {
      e.preventDefault();
      return;
    }
    lastPasteTimestamp = now;
    e.preventDefault();

    // Discriminar si el navegador entrega directamente archivos / blobs en el evento de paste
    const dt = e.clipboardData;
    if (dt && dt.files && dt.files.length > 0) {
      const validFiles: File[] = [];
      let hadUnsupported = false;
      for (let i = 0; i < dt.files.length; i++) {
        const file = dt.files[i];
        const isSupported =
          file.type.startsWith("image/") ||
          /\.(png|jpe?g|gif|webp)$/i.test(file.name);
        if (isSupported) {
          validFiles.push(file);
        } else {
          hadUnsupported = true;
        }
      }

      if (validFiles.length > 0) {
        isBusy = true;
        statusMsg = "Importando imagen pegada...";
        isError = false;
        try {
          let count = 0;
          for (const file of validFiles) {
            const buf = await file.arrayBuffer();
            const bytes = Array.from(new Uint8Array(buf));
            const stem = file.name
              ? file.name.replace(/\.[^/.]+$/, "")
              : "clipboard";
            await api.importRawBytes(bytes, stem || "clipboard");
            count++;
          }
          await loadEmotes();
          showFeedback(
            `✓ ¡${count} ${count === 1 ? "emote agregado" : "emotes agregados"} desde el portapapeles!`,
          );
          return;
        } catch (err: any) {
          showFeedback(`Error: ${err?.message || err}`, true);
          return;
        } finally {
          setTimeout(() => {
            isBusy = false;
          }, 400);
        }
      } else if (hadUnsupported) {
        showFeedback(
          "El archivo pegado no es compatible. Solo se admiten imágenes PNG, JPG, JPEG, GIF o WebP.",
          true,
        );
        return;
      }
    }

    await handlePasteClipboard();
  }
</script>

<svelte:window on:paste={handleGlobalPaste} on:keydown={handleGlobalKeyDown} />

{#if !customTrigger && !$$slots.trigger}
  <Button
    variant="primary"
    on:click={() => (isOpen = true)}
    title="Agregar emotes (Archivos, Carpeta, Portapapeles con Ctrl+V, URL o Drag & Drop)"
  >
    <Plus size={16} />
    <span>Agregar</span>
  </Button>
{:else if $$slots.trigger}
  <slot name="trigger" open={() => (isOpen = true)} />
{/if}

<Modal bind:open={isOpen} maxWidth="460px" busy={isBusy}>
  <Dropzone
    bind:isDragging
    disabled={isBusy}
    subtitle="Soporta archivos (.png, .jpg, .webp, .gif), carpetas, URL o portapapeles con Ctrl+V"
    ariaLabel="Zona para arrastrar y soltar emotes"
    on:drop={handleHtmlDrop}
  >
    <div class="primary-actions">
      <Button variant="primary" disabled={isBusy} on:click={pickFiles}>
        <Image size={15} /> Elegir Archivo(s)
      </Button>

      <Button variant="secondary" disabled={isBusy} on:click={pickFolder}>
        <FolderOpen size={15} /> Elegir Carpeta
      </Button>
    </div>

    <div class="secondary-actions">
      <Chip
        pill
        active={showUrlInput}
        disabled={isBusy}
        on:click={() => (showUrlInput = !showUrlInput)}
      >
        <Link size={13} /> Desde URL
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
            <Plus size={14} /> Importar URL
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
