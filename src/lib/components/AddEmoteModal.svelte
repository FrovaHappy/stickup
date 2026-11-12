<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import {
    Plus,
    Image,
    FolderOpen,
    Link,
    Loader2,
    CheckCircle2,
    AlertCircle,
  } from "@lucide/svelte";
  import { api } from "$lib/services/api";
  import { loadEmotes } from "$lib/stores/emotes";
  import { toast } from "$lib/stores/toast";
  import { askBeforePaste, lastCopiedEmote } from "$lib/stores/preferences";
  import { toPascalCase, parseUrlOrPath } from "$lib/utils/format";
  import type { Emote } from "$lib/types/emote";
  import PasteConfirmPopup from "./PasteConfirmPopup.svelte";
  import {
    Button,
    Dropzone,
    Input,
    Modal,
  } from "$lib/components/system-ui";

  export let isOpen = false;
  export let customTrigger = false;
  let isDragging = false;
  let isBusy = false;

  let urlValue = "";
  let statusMsg = "";
  let isError = false;
  let lastPasteTimestamp = 0;
  const PASTE_COOLDOWN_MS = 1200;

  let showPasteConfirm = false;
  let popupPreviewUrl = "";
  let popupSuggestedName = "";
  let popupIsBusy = false;
  let pendingSaveAction: (() => Promise<void>) | null = null;

  async function handleConfirmSave(dontAskAgain: boolean) {
    if (popupIsBusy || !pendingSaveAction) return;
    popupIsBusy = true;
    try {
      await pendingSaveAction();
      showPasteConfirm = false;
      pendingSaveAction = null;
    } catch (err: any) {
      toast.error(`Error al guardar: ${err?.message || err}`);
    } finally {
      popupIsBusy = false;
    }
  }

  function handleCancelSave() {
    showPasteConfirm = false;
    pendingSaveAction = null;
    popupPreviewUrl = "";
    popupSuggestedName = "";
  }

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
    if (isBusy || popupIsBusy) return;

    try {
      // 1. Capa de verificación con inspectClipboard
      const inspect = await api.inspectClipboard();

      if (inspect.is_duplicate) {
        const name = inspect.duplicate_name || "seleccionado";
        toast.info(`El emote "${name}" ya existe en tu colección`);
        return;
      }

      if (
        $lastCopiedEmote &&
        inspect.suggested_name &&
        inspect.suggested_name.toLowerCase() === $lastCopiedEmote.name.toLowerCase()
      ) {
        toast.info(`El emote "${$lastCopiedEmote.name}" ya existe en tu colección`);
        return;
      }

      if (!inspect.has_image || !inspect.preview_data_url) {
        showFeedback("No se encontraron imágenes compatibles (.png, .jpg, .webp, .gif).", true);
        return;
      }

      const normalizedName = inspect.suggested_name || "Clipboard";

      // Si askBeforePaste está desactivado: guardar directamente
      if (!$askBeforePaste) {
        await executeDirectClipboardImport(normalizedName);
        return;
      }

      // Si askBeforePaste está activado: mostrar modal en la derecha inferior
      popupPreviewUrl = inspect.preview_data_url;
      popupSuggestedName = normalizedName;
      pendingSaveAction = async () => {
        await executeDirectClipboardImport(normalizedName);
      };
      showPasteConfirm = true;
    } catch (err: any) {
      if (typeof err === "string" && err.includes("ya está en curso")) {
        return;
      }
      const msg = err?.message || String(err);
      if (msg.includes("ya existe")) {
        toast.info(msg);
      } else {
        showFeedback(`Error: ${msg}`, true);
      }
    }
  }

  async function executeDirectClipboardImport(nameToUse?: string) {
    isBusy = true;
    statusMsg = "Guardando emote...";
    isError = false;

    try {
      const result = await api.importFromClipboard(nameToUse);
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
      }
    } catch (err: any) {
      const msg = err?.message || String(err);
      if (msg.includes("ya existe")) {
        toast.info(msg);
      } else {
        showFeedback(`Error: ${msg}`, true);
      }
    } finally {
      setTimeout(() => {
        isBusy = false;
      }, 400);
    }
  }

  async function handleAddFromUrl() {
    const rawInput = urlValue.trim();
    if (isBusy || !rawInput) return;
    isBusy = true;
    isError = false;

    const parsed = parseUrlOrPath(rawInput);

    if (parsed.type === "local") {
      statusMsg = "Importando desde ruta local...";
      try {
        const imported = await api.importPaths([parsed.value]);
        await loadEmotes();
        urlValue = "";
        if (imported.length === 1) {
          const item = imported[0];
          const fmt = item.format ? ` (${item.format.toUpperCase()})` : "";
          showFeedback(`✓ ¡Emote "${item.name}"${fmt} agregado exitosamente!`);
        } else if (imported.length > 1) {
          showFeedback(
            `✓ ¡${imported.length} emotes agregados exitosamente desde el directorio!`,
          );
        } else {
          showFeedback(
            "No se encontraron imágenes compatibles (.png, .jpg, .jpeg, .gif, .webp) en la ruta.",
            true,
          );
        }
      } catch (err: any) {
        showFeedback(`Error: ${err?.message || err}`, true);
      } finally {
        setTimeout(() => {
          isBusy = false;
        }, 400);
      }
      return;
    }

    // URL web
    const targetUrl = parsed.value;
    const isDirectImage =
      /\.(png|jpe?g|gif|webp|svg|avif|bmp)($|\?|#)/i.test(targetUrl) ||
      targetUrl.includes("cdn.discordapp.com/emojis/");

    if (isDirectImage) {
      statusMsg = "Descargando imagen desde URL...";
      try {
        const created = await api.importFromUrl(targetUrl);
        urlValue = "";
        await loadEmotes();
        const fmt = created?.format ? ` (${created.format.toUpperCase()})` : "";
        const name = created?.name || "emote";
        showFeedback(`✓ ¡Emote "${name}"${fmt} agregado desde URL!`);
      } catch {
        await tryScanWebUrl(targetUrl);
      } finally {
        setTimeout(() => {
          isBusy = false;
        }, 400);
      }
      return;
    }

    // No es extensión de imagen directa: es directorio web o página
    await tryScanWebUrl(targetUrl);
    setTimeout(() => {
      isBusy = false;
    }, 400);
  }

  async function tryScanWebUrl(url: string) {
    statusMsg = "Escaneando URL / directorio web...";
    try {
      const scanned = await api.scanUrl(url);
      if (scanned && scanned.length > 0) {
        statusMsg = `Importando ${scanned.length} ${scanned.length === 1 ? "imagen encontrada" : "imágenes encontradas"}...`;
        const importRequests = scanned.map((s) => ({
          url: s.url,
          name: s.suggested_name,
        }));
        const result = await api.importScannedImages(importRequests);
        await loadEmotes();
        urlValue = "";
        if (result.imported.length === 1) {
          const item = result.imported[0];
          const fmt = item.format ? ` (${item.format.toUpperCase()})` : "";
          showFeedback(`✓ ¡Emote "${item.name}"${fmt} agregado desde URL!`);
        } else if (result.imported.length > 1) {
          showFeedback(
            `✓ ¡${result.imported.length} emotes importados exitosamente desde la URL!`,
          );
        } else {
          showFeedback(
            `No se pudieron importar las imágenes encontradas (${result.failed_count} fallaron).`,
            true,
          );
        }
        return;
      }
    } catch {
      // Si el escaneo falló, intentar importación directa como último recurso
    }

    try {
      const created = await api.importFromUrl(url);
      urlValue = "";
      await loadEmotes();
      const fmt = created?.format ? ` (${created.format.toUpperCase()})` : "";
      const name = created?.name || "emote";
      showFeedback(`✓ ¡Emote "${name}"${fmt} agregado desde URL!`);
    } catch (err: any) {
      if (typeof err === "string" && err.includes("ya está en curso")) {
        return;
      }
      showFeedback(`Error: ${err?.message || err}`, true);
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
          const stem = toPascalCase(file.name.replace(/\.[^/.]+$/, "")) || "Emote";
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
    if (
      uri &&
      (uri.startsWith("http://") ||
        uri.startsWith("https://") ||
        uri.startsWith("file://"))
    ) {
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
        const file = validFiles[0];
        const stem = file.name
          ? toPascalCase(file.name.replace(/\.[^/.]+$/, ""))
          : "Clipboard";

        // Verificación de duplicado
        if (
          $lastCopiedEmote &&
          stem.toLowerCase() === $lastCopiedEmote.name.toLowerCase()
        ) {
          toast.info(`El emote "${$lastCopiedEmote.name}" ya existe en tu colección`);
          return;
        }

        const buf = await file.arrayBuffer();
        const bytes = Array.from(new Uint8Array(buf));

        if (!$askBeforePaste) {
          isBusy = true;
          statusMsg = "Importando imagen pegada...";
          isError = false;
          try {
            await api.importRawBytes(bytes, stem || "Clipboard");
            await loadEmotes();
            showFeedback(`✓ ¡Emote "${stem}" agregado exitosamente!`);
            return;
          } catch (err: any) {
            showFeedback(`Error: ${err?.message || err}`, true);
            return;
          } finally {
            setTimeout(() => {
              isBusy = false;
            }, 400);
          }
        }

        // Si askBeforePaste está activado:
        popupPreviewUrl = URL.createObjectURL(file);
        popupSuggestedName = stem;
        pendingSaveAction = async () => {
          await api.importRawBytes(bytes, stem || "Clipboard");
          await loadEmotes();
          showFeedback(`✓ ¡Emote "${stem}" agregado exitosamente!`);
        };
        showPasteConfirm = true;
        return;
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

<Modal bind:open={isOpen} maxWidth="480px" busy={isBusy}>
  <Dropzone
    bind:isDragging
    disabled={isBusy}
    subtitle="Arrastra imágenes aquí o añade por URL / archivo / carpeta"
    ariaLabel="Zona para arrastrar y soltar emotes"
    on:drop={handleHtmlDrop}
  >
    <form class="url-import-form" on:submit|preventDefault={handleAddFromUrl}>
      <div class="url-input-row">
        <Input
          type="text"
          placeholder="URL o ruta (archivo o directorio)..."
          bind:value={urlValue}
          disabled={isBusy}
          ariaLabel="URL o ruta para importar emote"
        >
          <Link slot="leading" size={15} class="url-icon" />
        </Input>
        <Button
          type="submit"
          variant="primary"
          size="md"
          loading={isBusy}
          disabled={isBusy || !urlValue.trim()}
          title="Añadir inmediatamente desde URL o ruta"
        >
          <Plus size={15} />
          <span>Añadir</span>
        </Button>
      </div>
    </form>

    <div class="divider">
      <span class="divider-line"></span>
      <span class="divider-text">o buscar en tu PC</span>
      <span class="divider-line"></span>
    </div>

    <div class="primary-actions">
      <Button variant="secondary" size="md" disabled={isBusy} on:click={pickFiles}>
        <Image size={15} /> Elegir Archivo(s)
      </Button>

      <Button variant="secondary" size="md" disabled={isBusy} on:click={pickFolder}>
        <FolderOpen size={15} /> Elegir Carpeta
      </Button>
    </div>

    {#if isBusy || statusMsg}
      <div
        class="status-bar"
        class:is-busy={isBusy}
        class:is-error={isError}
        class:is-success={!isBusy && !isError && !!statusMsg}
        role="status"
        aria-live="polite"
      >
        <div class="status-content">
          {#if isBusy}
            <Loader2 size={14} class="spin status-icon" />
          {:else if isError}
            <AlertCircle size={14} class="status-icon" />
          {:else}
            <CheckCircle2 size={14} class="status-icon" />
          {/if}
          <span class="status-text">{statusMsg || "Procesando..."}</span>
        </div>
        {#if isBusy}
          <div class="progress-track">
            <div class="progress-bar-indeterminate"></div>
          </div>
        {/if}
      </div>
    {/if}
  </Dropzone>
</Modal>

<PasteConfirmPopup
  open={showPasteConfirm}
  previewUrl={popupPreviewUrl}
  name={popupSuggestedName}
  isBusy={popupIsBusy}
  onSave={handleConfirmSave}
  onCancel={handleCancelSave}
/>

<style>
  .url-import-form {
    width: 100%;
    margin-top: 2px;
  }
  .url-input-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }
  .url-input-row :global(.ui-input-group) {
    flex: 1;
    min-width: 0;
  }
  :global(.url-icon) {
    color: var(--color-orange-primary, #f97316);
    flex-shrink: 0;
  }
  .divider {
    display: flex;
    align-items: center;
    width: 100%;
    gap: 12px;
    margin: 8px 0 4px;
  }
  .divider-line {
    flex: 1;
    height: 1px;
    background: var(--color-border-subtle, rgba(255, 255, 255, 0.08));
  }
  .divider-text {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text-dim, #71717a);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    white-space: nowrap;
  }
  .primary-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    width: 100%;
  }
  .status-bar {
    position: relative;
    width: 100%;
    margin-top: 8px;
    padding: 7px 12px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    overflow: hidden;
    animation: fadeIn 0.15s ease-out;
    box-sizing: border-box;
  }
  .status-bar.is-busy {
    border-color: rgba(249, 115, 22, 0.3);
    background: rgba(249, 115, 22, 0.06);
  }
  .status-bar.is-success {
    border-color: rgba(34, 197, 94, 0.3);
    background: rgba(34, 197, 94, 0.08);
  }
  .status-bar.is-error {
    border-color: rgba(234, 88, 12, 0.35);
    background: rgba(234, 88, 12, 0.1);
  }
  .status-content {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 500;
  }
  .status-bar.is-busy .status-content {
    color: var(--color-orange-primary, #f97316);
  }
  .status-bar.is-success .status-content {
    color: #4ade80;
  }
  .status-bar.is-error .status-content {
    color: #fb923c;
  }
  .status-text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: left;
  }
  :global(.status-icon) {
    flex-shrink: 0;
  }
  :global(.spin) {
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
  .progress-track {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: rgba(255, 255, 255, 0.06);
    overflow: hidden;
  }
  .progress-bar-indeterminate {
    width: 40%;
    height: 100%;
    background: var(--color-orange-primary, #f97316);
    animation: indeterminate 1.2s infinite ease-in-out;
  }
  @keyframes indeterminate {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(300%);
    }
  }
  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(-2px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
