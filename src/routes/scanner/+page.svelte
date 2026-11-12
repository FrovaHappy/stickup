<script lang="ts">
  import {
    ScanSearch,
    Globe,
    FileCode,
    Search,
    Download,
    Check,
    Square,
    SquareCheck,
    RotateCcw,
    Copy,
    Upload,
    Trash2,
    ExternalLink,
    Image as ImageIcon,
    Radio,
    ClipboardPaste,
    Code,
    Bookmark
  } from "@lucide/svelte";
  import { Button, Input, Badge, Modal, Toast } from "$lib/components/system-ui";
  import { api } from "$lib/services/api";
  import { loadEmotes } from "$lib/stores/emotes";
  import { toast } from "$lib/stores/toast";
  import type { ScannedImage, ScannedImageImportRequest } from "$lib/types/scanner";
  import {
    generateCollectorScript,
    generateBookmarklet,
    parseCollectorInput,
    isSafeHttpUrl,
  } from "$lib/utils/domCollector";
  import { cleanUrlInput } from "$lib/services/urlScanner";

  const GITHUB_COLLECTOR_FILE_URL = "https://github.com/FrovaHappy/stickup/blob/main/src/lib/utils/domCollector.ts";

  type ScanTab = "live" | "url" | "html";

  let activeTab: ScanTab = "live";
  let targetUrl = "";
  let rawHtml = "";
  let htmlBaseUrl = "";
  let liveInput = "";
  let liveBaseUrl = "";
  let showCodeModal = false;
  let isScanning = false;

  $: collectorScriptCode = generateCollectorScript();

  function openCollectorFileInGitHub() {
    api.openExternalUrl(GITHUB_COLLECTOR_FILE_URL).catch(() => {
      window.open(GITHUB_COLLECTOR_FILE_URL, "_blank");
    });
  }
  let isImporting = false;
  let scanError: string | null = null;

  // Lista de imágenes extraídas
  let scannedImages: (ScannedImage & { selected: boolean; saved: boolean })[] = [];
  let filterQuery = "";
  let fileInputElement: HTMLInputElement | null = null;

  // Filtrado reactivo de imágenes
  $: filteredImages = scannedImages.filter((img) => {
    if (!filterQuery.trim()) return true;
    const q = filterQuery.toLowerCase();
    return (
      img.suggested_name.toLowerCase().includes(q) ||
      (img.format_hint && img.format_hint.toLowerCase().includes(q)) ||
      img.url.toLowerCase().includes(q) ||
      (img.alt && img.alt.toLowerCase().includes(q))
    );
  });

  $: selectedCount = scannedImages.filter((i) => i.selected).length;
  $: allSelected = scannedImages.length > 0 && selectedCount === scannedImages.length;

  async function handleScanUrl() {
    const trimmed = cleanUrlInput(targetUrl);
    if (!trimmed) {
      scanError = "Por favor, ingresa una URL válida que comience con http:// o https://";
      return;
    }
    if (!trimmed.startsWith("http://") && !trimmed.startsWith("https://")) {
      scanError = "La URL debe comenzar con http:// o https://";
      return;
    }

    scanError = null;
    isScanning = true;
    try {
      const results = await api.scanUrl(trimmed);
      scannedImages = (results || []).map((img) => ({
        ...img,
        selected: true,
        saved: false
      }));
      if (scannedImages.length === 0) {
        toast.info("No se encontraron imágenes en la URL indicada.");
      } else {
        toast.success(`Se encontraron ${scannedImages.length} imágenes.`);
      }
    } catch (err: any) {
      console.error("Scan URL failed", err);
      scanError = typeof err === "string" ? err : err?.message || "Error al escanear la URL.";
      scannedImages = [];
    } finally {
      isScanning = false;
    }
  }

  async function handleScanHtml() {
    const trimmedHtml = rawHtml.trim();
    if (!trimmedHtml) {
      scanError = "Por favor, pega el código HTML o carga un archivo .html";
      return;
    }

    scanError = null;
    isScanning = true;
    try {
      const results = await api.scanHtml(trimmedHtml, htmlBaseUrl.trim() || undefined);
      scannedImages = (results || []).map((img) => ({
        ...img,
        selected: true,
        saved: false
      }));
      if (scannedImages.length === 0) {
        toast.info("No se encontraron imágenes en el código HTML.");
      } else {
        toast.success(`Se encontraron ${scannedImages.length} imágenes.`);
      }
    } catch (err: any) {
      console.error("Scan HTML failed", err);
      scanError = typeof err === "string" ? err : err?.message || "Error al analizar el HTML.";
      scannedImages = [];
    } finally {
      isScanning = false;
    }
  }

  async function handleCopyCollector() {
    try {
      const code = generateCollectorScript();
      await navigator.clipboard.writeText(code);
      toast.success("Script colector copiado. Pégalo en la consola (F12) de tu navegador.");
    } catch (_) {
      toast.error("No se pudo copiar el script.");
    }
  }

  async function handleCopyBookmarklet() {
    try {
      const code = generateBookmarklet();
      await navigator.clipboard.writeText(code);
      toast.success("Bookmarklet copiado. Crea un marcador con esta URL en tu navegador.");
    } catch (_) {
      toast.error("No se pudo copiar el bookmarklet.");
    }
  }

  async function handlePasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (!text.trim()) {
        toast.info("El portapapeles está vacío.");
        return;
      }
      liveInput = text;
      await handleProcessLive(text);
    } catch (err: any) {
      toast.error("No se pudo leer el portapapeles: " + (err?.message || err));
    }
  }

  async function handleProcessLive(customText?: string) {
    const raw = (customText !== undefined ? customText : liveInput).trim();
    if (!raw) {
      scanError = "Por favor, pega el JSON de captura o la lista de URLs recolectadas.";
      return;
    }

    const { items, source } = parseCollectorInput(raw);
    if (items.length === 0) {
      scanError = "No se encontraron URLs válidas en el contenido pegado.";
      return;
    }

    scanError = null;
    isScanning = true;
    try {
      const baseUrl = source || liveBaseUrl.trim() || undefined;
      const results = await api.processScannedUrls(items, baseUrl);
      scannedImages = (results || []).map((img) => ({
        ...img,
        selected: true,
        saved: false
      }));

      if (scannedImages.length === 0) {
        toast.info("No se pudieron procesar las imágenes.");
      } else {
        toast.success(`Se capturaron y procesaron ${scannedImages.length} imágenes.`);
      }
    } catch (err: any) {
      console.error("Process live URLs failed", err);
      scanError = typeof err === "string" ? err : err?.message || "Error al procesar las imágenes.";
      scannedImages = [];
    } finally {
      isScanning = false;
    }
  }

  function handleSelectAll() {
    scannedImages = scannedImages.map((img) => ({ ...img, selected: true }));
  }

  function handleDeselectAll() {
    scannedImages = scannedImages.map((img) => ({ ...img, selected: false }));
  }

  function handleInvertSelection() {
    scannedImages = scannedImages.map((img) => ({ ...img, selected: !img.selected }));
  }

  function toggleItemSelection(index: number) {
    scannedImages[index].selected = !scannedImages[index].selected;
    scannedImages = [...scannedImages];
  }

  function handleDeleteSingle(index: number) {
    if (index >= 0 && index < scannedImages.length) {
      const removed = scannedImages.splice(index, 1);
      scannedImages = [...scannedImages];
      toast.info(`Imagen "${removed[0]?.suggested_name || 'seleccionada'}" eliminada.`);
    }
  }

  function handleDeleteSelected() {
    const toDeleteCount = selectedCount;
    if (toDeleteCount === 0) return;
    scannedImages = scannedImages.filter((img) => !img.selected);
    toast.info(`Se eliminaron ${toDeleteCount} imágenes de la lista.`);
  }

  function handleClearAll() {
    if (scannedImages.length === 0) return;
    const total = scannedImages.length;
    scannedImages = [];
    toast.info(`Se limpiaron ${total} imágenes de la lista.`);
  }

  async function handleSaveSelected() {
    const itemsToSave: ScannedImageImportRequest[] = scannedImages
      .filter((i) => i.selected && !i.saved)
      .map((i) => ({
        url: i.url,
        name: i.suggested_name.trim() || null
      }));

    if (itemsToSave.length === 0) {
      toast.info("No hay imágenes seleccionadas pendientes de guardar.");
      return;
    }

    isImporting = true;
    try {
      const res = await api.importScannedImages(itemsToSave);
      await loadEmotes();

      // Marcar como guardadas
      const savedUrls = new Set(itemsToSave.map((it) => it.url));
      scannedImages = scannedImages.map((img) => {
        if (savedUrls.has(img.url)) {
          return { ...img, saved: true, selected: false };
        }
        return img;
      });

      if (res.imported.length > 0) {
        toast.success(
          `¡${res.imported.length} ${
            res.imported.length === 1 ? "imagen guardada" : "imágenes guardadas"
          } en tu biblioteca!`
        );
      }
      if (res.failed_count > 0) {
        toast.error(`${res.failed_count} imágenes no pudieron descargarse o procesarse.`);
      }
    } catch (err: any) {
      console.error("Import failed", err);
      toast.error(typeof err === "string" ? err : "Error al guardar imágenes.");
    } finally {
      isImporting = false;
    }
  }

  async function handleSaveSingle(index: number) {
    const target = scannedImages[index];
    if (target.saved) return;

    isImporting = true;
    try {
      const res = await api.importScannedImages([
        {
          url: target.url,
          name: target.suggested_name.trim() || null
        }
      ]);
      await loadEmotes();

      if (res.imported.length > 0) {
        scannedImages[index].saved = true;
        scannedImages[index].selected = false;
        scannedImages = [...scannedImages];
        toast.success(`"${target.suggested_name}" guardado exitosamente.`);
      } else {
        toast.error(`No se pudo guardar la imagen: ${res.errors[0] || "formato no soportado"}`);
      }
    } catch (err: any) {
      console.error("Single save failed", err);
      toast.error(typeof err === "string" ? err : "Error al guardar imagen.");
    } finally {
      isImporting = false;
    }
  }

  async function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      rawHtml = text;
      toast.success(`Archivo "${file.name}" cargado (${Math.round(file.size / 1024)} KB)`);
      handleScanHtml();
    } catch (err) {
      toast.error("Error al leer el archivo HTML seleccionado.");
    } finally {
      input.value = "";
    }
  }

  function handleHtmlDrop(event: DragEvent) {
    event.preventDefault();
    const file = event.dataTransfer?.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        rawHtml = (e.target?.result as string) || "";
        toast.success(`Archivo "${file.name}" cargado.`);
        handleScanHtml();
      };
      reader.readAsText(file);
    }
  }

  function copyUrlToClipboard(url: string) {
    navigator.clipboard.writeText(url);
    toast.success("URL copiada al portapapeles");
  }
</script>

<svelte:head>
  <title>StickUp - Escáner de Imágenes</title>
</svelte:head>

<div class="scanner-container">
  <!-- Encabezado Principal -->
  <header class="scanner-header">
    <div class="header-icon-wrapper">
      <ScanSearch size={28} class="header-icon" />
    </div>
    <div class="header-text">
      <h1 class="header-title">Escáner Web & HTML</h1>
      <p class="header-subtitle">
        Extrae imágenes desde cualquier página web o estructura HTML, previsualízalas y selecciona
        cuáles conservar en tu colección de StickUp.
      </p>
    </div>
  </header>

  <!-- Pestañas de Modo -->
  <div class="mode-tabs" role="tablist">
    <button
      type="button"
      class="tab-btn"
      class:active={activeTab === "live"}
      role="tab"
      aria-selected={activeTab === "live"}
      on:click={() => (activeTab = "live")}
    >
      <Radio size={18} class={activeTab === "live" ? "tab-icon-active" : ""} />
      <span>Captura en vivo</span>
    </button>
    <button
      type="button"
      class="tab-btn"
      class:active={activeTab === "url"}
      role="tab"
      aria-selected={activeTab === "url"}
      on:click={() => (activeTab = "url")}
    >
      <Globe size={18} />
      <span>Escanear por URL</span>
    </button>
    <button
      type="button"
      class="tab-btn"
      class:active={activeTab === "html"}
      role="tab"
      aria-selected={activeTab === "html"}
      on:click={() => (activeTab = "html")}
    >
      <FileCode size={18} />
      <span>Cargar HTML manual</span>
    </button>
  </div>

  <!-- Formulario de Entrada -->
  <div class="input-panel">
    {#if activeTab === "live"}
      <div class="live-capture-section">
        <div class="live-unified-card">
          <!-- 1. Barra de Acciones Principales Arriba -->
          <div class="live-actions-top-bar">
            <div class="primary-action-wrap">
              <Button
                variant="primary"
                loading={isScanning}
                on:click={handlePasteFromClipboard}
                title="Pegar y cargar capturas del portapapeles"
              >
                <ClipboardPaste size={16} />
                <span>Pegar desde Portapapeles</span>
              </Button>
            </div>

            <div class="secondary-actions-wrap">
              <Button
                variant="secondary"
                on:click={handleCopyBookmarklet}
                title="Copiar código del bookmarklet para tu barra de favoritos"
              >
                <Copy size={15} />
                <span>Copiar Bookmarklet</span>
              </Button>

              <Button
                variant="outline"
                on:click={handleCopyCollector}
                title="Copiar script para la consola de desarrollo (F12)"
              >
                <Code size={15} />
                <span>Copiar Script (F12)</span>
              </Button>

              <Button
                variant="ghost"
                on:click={() => (showCodeModal = true)}
                title="Ver e inspeccionar el código JavaScript"
              >
                <Code size={15} />
                <span>Ver código</span>
              </Button>

              <Button
                variant="ghost"
                on:click={openCollectorFileInGitHub}
                title="Abrir domCollector.ts en GitHub"
              >
                <ExternalLink size={15} />
                <span>GitHub</span>
              </Button>
            </div>
          </div>

          <!-- 2. Información y Guía didáctica Bookmarklet -->
          <div class="live-banner-content">
            <div class="banner-badge">
              <Radio size={15} class="pulse-icon" />
              <span>Colector DOM en vivo</span>
            </div>
            <p class="banner-description">
              Navega libremente por Twitch, 7TV, Discord, etc. El colector consulta el DOM cada 1 segundo y acumula los emotes en un <code>Set</code> para importarlos sin perder transparencia ni animación.
            </p>

            <div class="bookmarklet-info-box">
              <div class="bookmarklet-info-header">
                <Bookmark size={15} class="text-orange" />
                <strong>¿Cómo usar el Bookmarklet?</strong>
              </div>
              <p class="bookmarklet-info-desc">
                Un <strong>Bookmarklet</strong> es un marcador inteligente que ejecuta una función JavaScript en lugar de abrir una URL. Te permite activar el colector con un clic desde tu barra de favoritos:
              </p>
              <div class="bookmarklet-steps-grid">
                <div class="guide-step-card">
                  <span class="guide-step-badge">1</span>
                  <div class="guide-step-content">
                    <span class="guide-step-title">Copiar Bookmarklet</span>
                    <p class="guide-step-text">Pulsa <em>"Copiar Bookmarklet"</em> en las acciones superiores.</p>
                  </div>
                </div>
                <div class="guide-step-card">
                  <span class="guide-step-badge">2</span>
                  <div class="guide-step-content">
                    <span class="guide-step-title">Crear Marcador</span>
                    <p class="guide-step-text">Añade un favorito en tu navegador (Ctrl+D) y pega el código en <em>URL</em>.</p>
                  </div>
                </div>
                <div class="guide-step-card">
                  <span class="guide-step-badge">3</span>
                  <div class="guide-step-content">
                    <span class="guide-step-title">Navegar y Capturar</span>
                    <p class="guide-step-text">En cualquier web, haz clic en el marcador para <strong>Iniciar</strong> y <strong>Parar</strong>.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 3. Opción manual / Avanzada -->
          <div class="live-manual-details">
            <details class="manual-collapsible">
              <summary class="collapsible-summary">
                O pegar manualmente JSON o lista de URLs
              </summary>
              <div class="collapsible-body">
                <div class="base-url-input">
                  <Input
                    bind:value={liveBaseUrl}
                    type="url"
                    placeholder="URL de origen opcional (ej: https://twitch.tv/ o https://7tv.app/)"
                    disabled={isScanning}
                  >
                    <svelte:fragment slot="leading">
                      <Globe size={16} />
                    </svelte:fragment>
                  </Input>
                </div>
                <textarea
                  bind:value={liveInput}
                  placeholder="Pega aquí el JSON del colector o enlaces directos (uno por línea)..."
                  rows="4"
                  class="html-textarea"
                  disabled={isScanning}
                ></textarea>
                <div class="live-manual-actions">
                  <Button
                    variant="secondary"
                    disabled={!liveInput.trim() || isScanning}
                    loading={isScanning}
                    on:click={() => handleProcessLive()}
                  >
                    <ScanSearch size={16} />
                    <span>Procesar URLs</span>
                  </Button>
                </div>
              </div>
            </details>
          </div>
        </div>
      </div>
    {:else if activeTab === "url"}
      <form class="scan-form" on:submit|preventDefault={handleScanUrl}>
        <div class="url-input-row">
          <div class="input-field-wrapper">
            <Input
              bind:value={targetUrl}
              type="url"
              placeholder="https://ejemplo.com/galeria-de-emotes..."
              disabled={isScanning}
              ariaLabel="Dirección web para escanear"
            >
              <svelte:fragment slot="leading">
                <Globe size={16} />
              </svelte:fragment>
            </Input>
          </div>
          <Button
            type="submit"
            variant="primary"
            loading={isScanning}
            disabled={!targetUrl.trim() || isScanning}
          >
            <Search size={16} />
            <span>Escanear URL</span>
          </Button>
        </div>
        <p class="input-hint">
          Soporta páginas web completas, galerías y links directos de Discord (detectando automáticamente
          si son estáticos en WebP o animados en GIF según las reglas de CDN y parámetros como <code>animated=true</code>).
        </p>
      </form>
    {:else if activeTab === "html"}
      <div class="html-manual-section">
        <div class="html-controls-row">
          <div class="base-url-input">
            <Input
              bind:value={htmlBaseUrl}
              type="url"
              placeholder="URL base (opcional, ej: https://sitio.com/ para rutas relativas)"
              disabled={isScanning}
            >
              <svelte:fragment slot="leading">
                <Globe size={16} />
              </svelte:fragment>
            </Input>
          </div>
          <div class="file-upload-actions">
            <input
              type="file"
              accept=".html,.htm"
              bind:this={fileInputElement}
              on:change={handleFileUpload}
              style="display: none;"
            />
            <Button
              variant="secondary"
              on:click={() => fileInputElement?.click()}
              disabled={isScanning}
            >
              <Upload size={16} />
              <span>Cargar .html</span>
            </Button>
            {#if rawHtml.trim()}
              <Button
                variant="ghost"
                title="Limpiar código HTML"
                on:click={() => (rawHtml = "")}
                disabled={isScanning}
              >
                <Trash2 size={16} />
              </Button>
            {/if}
          </div>
        </div>

        <div
          class="html-textarea-wrapper"
          on:dragover|preventDefault
          on:drop={handleHtmlDrop}
          role="region"
          aria-label="Zona de pegado o soltado de HTML"
        >
          <textarea
            bind:value={rawHtml}
            placeholder="Pega aquí el código HTML o arrastra un archivo .html..."
            rows="5"
            class="html-textarea"
            disabled={isScanning}
          ></textarea>
        </div>

        <div class="html-actions-row">
          <span class="html-length-hint">
            {#if rawHtml.trim()}
              {rawHtml.length} caracteres ingresados
            {:else}
              Soporta código HTML, páginas guardadas o sintaxis nativa de Discord (&lt;a:nombre:id&gt; animado y &lt;:nombre:id&gt; estático).
            {/if}
          </span>
          <Button
            variant="primary"
            loading={isScanning}
            disabled={!rawHtml.trim() || isScanning}
            on:click={handleScanHtml}
          >
            <ScanSearch size={16} />
            <span>Extraer imágenes</span>
          </Button>
        </div>
      </div>
    {/if}

    {#if scanError}
      <div class="error-banner">
        <Toast
          variant="warning"
          title="Aviso de escaneo"
          message={scanError}
          dismissible
          on:dismiss={() => (scanError = null)}
        />
      </div>
    {/if}
  </div>

  <!-- Barra de Herramientas de Resultados -->
  {#if scannedImages.length > 0}
    <section class="results-section">
      <div class="results-toolbar">
        <div class="toolbar-left">
          <div class="results-count">
            <span class="count-badge">
              {scannedImages.length}
            </span>
            <span class="count-label">
              {scannedImages.length === 1 ? "imagen encontrada" : "imágenes encontradas"}
            </span>
          </div>

          <div class="selection-pill">
            <span class="selected-number">{selectedCount}</span>
            <span>seleccionadas</span>
          </div>

          <div class="quick-select-actions">
            <button
              type="button"
              class="action-pill-btn"
              title="Seleccionar todas"
              on:click={handleSelectAll}
            >
              <SquareCheck size={14} />
              <span>Todas</span>
            </button>
            <button
              type="button"
              class="action-pill-btn"
              title="Deseleccionar todas"
              on:click={handleDeselectAll}
            >
              <Square size={14} />
              <span>Ninguna</span>
            </button>
            <button
              type="button"
              class="action-pill-btn"
              title="Invertir selección"
              on:click={handleInvertSelection}
            >
              <RotateCcw size={14} />
              <span>Invertir</span>
            </button>
            <button
              type="button"
              class="action-pill-btn danger-hover"
              title="Eliminar imágenes seleccionadas de la lista"
              disabled={selectedCount === 0}
              on:click={handleDeleteSelected}
            >
              <Trash2 size={13} />
              <span>Eliminar ({selectedCount})</span>
            </button>
            <button
              type="button"
              class="action-pill-btn danger-hover"
              title="Limpiar todas las imágenes"
              on:click={handleClearAll}
            >
              <Trash2 size={13} />
              <span>Limpiar todo</span>
            </button>
          </div>
        </div>

        <div class="toolbar-right">
          <div class="search-filter-input">
            <Input
              bind:value={filterQuery}
              type="search"
              placeholder="Filtrar por nombre o formato..."
            >
              <svelte:fragment slot="leading">
                <Search size={14} />
              </svelte:fragment>
            </Input>
          </div>

          <Button
            variant="primary"
            loading={isImporting}
            disabled={selectedCount === 0 || isImporting}
            on:click={handleSaveSelected}
          >
            <Download size={16} />
            <span>Guardar seleccionadas ({selectedCount})</span>
          </Button>
        </div>
      </div>

      <!-- Grilla de Imágenes Encontradas -->
      {#if filteredImages.length === 0}
        <div class="empty-filter">
          <p>No hay imágenes que coincidan con el filtro "{filterQuery}".</p>
        </div>
      {:else}
        <div class="images-grid">
          {#each filteredImages as img, idx (img.url)}
            <div
              class="image-card"
              class:is-selected={img.selected}
              class:is-saved={img.saved}
              on:click={() => toggleItemSelection(scannedImages.indexOf(img))}
              role="button"
              tabindex="0"
              on:keydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  toggleItemSelection(scannedImages.indexOf(img));
                }
              }}
            >
              <!-- Selector / Checkbox Superior -->
              <div class="card-header">
                <div class="selection-checkbox" class:checked={img.selected}>
                  {#if img.selected}
                    <Check size={14} strokeWidth={3} />
                  {/if}
                </div>

                <div class="card-badges">
                  {#if img.saved}
                    <Badge variant="green">Guardado</Badge>
                  {/if}
                  {#if img.format_hint}
                    <Badge variant="gray">{img.format_hint.toUpperCase()}</Badge>
                  {/if}
                </div>
              </div>

              <!-- Vista Previa de la Imagen -->
              <div class="card-preview">
                <img
                  src={img.url}
                  alt={img.alt || img.suggested_name}
                  loading="lazy"
                  referrerpolicy="no-referrer"
                  on:error={(e) => {
                    const target = e.currentTarget as HTMLImageElement;
                    target.style.display = "none";
                    const fallback = target.nextElementSibling as HTMLElement;
                    if (fallback) fallback.style.display = "flex";
                  }}
                />
                <div class="preview-fallback" style="display: none;">
                  <ImageIcon size={28} />
                  <span>Sin preview</span>
                </div>
              </div>

              <!-- Metadatos y Acciones de la Card -->
              <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
              <div
                class="card-footer"
                role="presentation"
                on:click|stopPropagation
                on:keydown|stopPropagation
              >
                <div class="card-name-input">
                  <input
                    type="text"
                    bind:value={img.suggested_name}
                    class="name-input"
                    placeholder="Nombre del emote"
                    title="Editar nombre antes de guardar"
                    disabled={img.saved}
                  />
                </div>

                <div class="card-actions">
                  <button
                    type="button"
                    class="card-action-btn"
                    title="Copiar URL de la imagen"
                    on:click={() => copyUrlToClipboard(img.url)}
                  >
                    <Copy size={14} />
                  </button>

                  {#if isSafeHttpUrl(img.url)}
                    <a
                      href={img.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="card-action-btn"
                      title="Abrir imagen en navegador"
                    >
                      <ExternalLink size={14} />
                    </a>
                  {/if}

                  <button
                    type="button"
                    class="card-action-btn delete-btn"
                    title="Eliminar de la lista"
                    on:click={() => handleDeleteSingle(scannedImages.indexOf(img))}
                  >
                    <Trash2 size={14} />
                  </button>

                  {#if !img.saved}
                    <button
                      type="button"
                      class="card-save-btn"
                      title="Guardar este emote en tu colección"
                      disabled={isImporting}
                      on:click={() => handleSaveSingle(scannedImages.indexOf(img))}
                    >
                      <Download size={14} />
                      <span>Guardar</span>
                    </button>
                  {:else}
                    <span class="card-saved-indicator">
                      <Check size={14} strokeWidth={2.5} />
                      <span>Listo</span>
                    </span>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  {/if}

  {#if showCodeModal}
    <Modal
      open={showCodeModal}
      title="Código del Script Colector"
      maxWidth="720px"
      onClose={() => (showCodeModal = false)}
    >
      <div class="code-modal-content">
        <p class="code-modal-intro">
          Este es el script JavaScript que puedes inyectar en la consola (F12) o mediante Bookmarklet para recolectar emotes del DOM cada 1 segundo. Puedes auditarlo, copiarlo o inspeccionar el código fuente en GitHub:
        </p>

        <div class="code-viewer-container">
          <pre class="code-viewer-block"><code>{collectorScriptCode}</code></pre>
        </div>

        <div class="code-modal-footer">
          <Button variant="outline" on:click={openCollectorFileInGitHub} title="Abrir fichero domCollector.ts en GitHub">
            <ExternalLink size={15} />
            <span>Ver domCollector.ts en GitHub</span>
          </Button>
          <div class="modal-footer-actions">
            <Button variant="secondary" on:click={handleCopyCollector}>
              <Copy size={15} />
              <span>Copiar script</span>
            </Button>
            <Button variant="primary" on:click={() => (showCodeModal = false)}>
              <span>Cerrar</span>
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  {/if}
</div>

<style>
  .scanner-container {
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
    padding: 24px 32px 48px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  /* Encabezado */
  .scanner-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding-bottom: 8px;
  }

  .header-icon-wrapper {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    background: rgba(var(--color-orange-rgb), 0.12);
    border: 1px solid rgba(var(--color-orange-rgb), 0.3);
    color: var(--color-orange-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .header-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .header-title {
    font-size: 22px;
    font-weight: 700;
    color: var(--color-text-main);
    margin: 0;
  }

  .header-subtitle {
    font-size: 13.5px;
    color: var(--color-text-dim);
    margin: 0;
    max-width: 760px;
    line-height: 1.45;
  }

  /* Pestañas */
  .mode-tabs {
    display: flex;
    gap: 8px;
    border-bottom: 1px solid var(--color-border-subtle);
    padding-bottom: 4px;
  }

  .tab-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: 8px 8px 0 0;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--color-text-dim);
    font-size: 13.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .tab-btn:hover {
    color: var(--color-text-main);
    background: var(--color-bg-hover);
  }

  .tab-btn.active {
    color: var(--color-orange-primary);
    border-bottom-color: var(--color-orange-primary);
    background: rgba(var(--color-orange-rgb), 0.08);
  }

  /* Panel de Entrada */
  .input-panel {
    background: transparent;
    border: 1px solid var(--color-border-subtle);
    border-radius: 12px;
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .scan-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .url-input-row {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .input-field-wrapper {
    flex: 1;
  }

  .input-hint {
    font-size: 12px;
    color: var(--color-text-dim);
    margin: 0;
    line-height: 1.4;
  }

  /* HTML Manual */
  .html-manual-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .html-controls-row {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .base-url-input {
    flex: 1;
  }

  .file-upload-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .html-textarea-wrapper {
    width: 100%;
  }

  .html-textarea {
    width: 100%;
    box-sizing: border-box;
    background: var(--color-bg-sidebar);
    border: 1px solid var(--color-border-light);
    border-radius: 8px;
    padding: 12px 14px;
    color: var(--color-text-main);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12.5px;
    line-height: 1.5;
    resize: vertical;
    transition: border-color 0.2s ease;
  }

  .html-textarea:focus {
    outline: none;
    border-color: var(--color-orange-primary);
    box-shadow: 0 0 0 2px rgba(var(--color-orange-rgb), 0.2);
  }

  .html-actions-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .html-length-hint {
    font-size: 12px;
    color: var(--color-text-dim);
  }

  .error-banner {
    margin-top: 4px;
  }

  /* Sección de Resultados */
  .results-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .results-toolbar {
    position: sticky;
    top: 12px;
    z-index: 30;
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 14px;
    padding: 12px 18px;
    background: rgba(28, 28, 31, 0.94);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid var(--color-border-subtle);
    border-radius: 12px;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.45);
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .results-count {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .count-badge {
    background: rgba(var(--color-orange-rgb), 0.15);
    color: var(--color-orange-primary);
    font-weight: 700;
    font-size: 13px;
    padding: 2px 8px;
    border-radius: 6px;
    border: 1px solid rgba(var(--color-orange-rgb), 0.3);
  }

  .count-label {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--color-text-main);
  }

  .selection-pill {
    font-size: 12.5px;
    color: var(--color-text-dim);
    background: var(--color-bg-sidebar);
    padding: 3px 10px;
    border-radius: 20px;
    border: 1px solid var(--color-border-subtle);
    display: flex;
    gap: 5px;
  }

  .selected-number {
    font-weight: 700;
    color: var(--color-orange-primary);
  }

  .quick-select-actions {
    display: flex;
    gap: 6px;
  }

  .action-pill-btn {
    background: transparent;
    border: 1px solid var(--color-border-light);
    border-radius: 6px;
    color: var(--color-text-dim);
    font-size: 12px;
    padding: 4px 10px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    transition: all 0.2s ease;
  }

  .action-pill-btn:hover {
    color: var(--color-text-main);
    border-color: var(--color-orange-primary);
    background: rgba(var(--color-orange-rgb), 0.08);
  }

  .action-pill-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .action-pill-btn.danger-hover:hover:not(:disabled) {
    color: #fb7185;
    border-color: rgba(244, 63, 94, 0.5);
    background: rgba(244, 63, 94, 0.12);
  }

  .search-filter-input {
    min-width: 220px;
  }

  .empty-filter {
    padding: 32px;
    text-align: center;
    color: var(--color-text-dim);
    border: 1px dashed var(--color-border-subtle);
    border-radius: 12px;
    font-size: 13.5px;
  }

  /* Grilla de Imágenes */
  .images-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 16px;
  }

  .image-card {
    background: transparent;
    border: 1px solid var(--color-border-subtle);
    border-radius: 12px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    cursor: pointer;
    position: relative;
    user-select: none;
  }

  .image-card:hover {
    transform: translateY(-2px);
    border-color: rgba(var(--color-orange-rgb), 0.5);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  }

  .image-card.is-selected {
    border-color: var(--color-orange-primary);
    box-shadow: 0 0 0 1px var(--color-orange-primary), 0 4px 18px rgba(var(--color-orange-rgb), 0.2);
  }

  .image-card.is-saved {
    opacity: 0.75;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    z-index: 2;
  }

  .selection-checkbox {
    width: 20px;
    height: 20px;
    border-radius: 5px;
    border: 1.5px solid var(--color-border-light);
    background: var(--color-bg-surface);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    transition: all 0.15s ease;
  }

  .selection-checkbox.checked {
    background: var(--color-orange-primary);
    border-color: var(--color-orange-primary);
  }

  .card-badges {
    display: flex;
    gap: 6px;
  }

  .card-preview {
    height: 130px;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px 12px;
    box-sizing: border-box;
    background: rgba(0, 0, 0, 0.15);
  }

  .card-preview img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
    image-rendering: -webkit-optimize-contrast;
  }

  .preview-fallback {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    color: var(--color-text-dim);
    font-size: 11px;
  }

  .card-footer {
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    border-top: 1px solid var(--color-border-subtle);
    background: rgba(0, 0, 0, 0.1);
  }

  .card-name-input {
    width: 100%;
  }

  .name-input {
    width: 100%;
    box-sizing: border-box;
    background: var(--color-bg-sidebar);
    border: 1px solid var(--color-border-subtle);
    border-radius: 6px;
    padding: 4px 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-main);
    transition: border-color 0.2s ease;
  }

  .name-input:focus {
    outline: none;
    border-color: var(--color-orange-primary);
  }

  .card-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }

  .card-action-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    background: transparent;
    border: 1px solid var(--color-border-light);
    color: var(--color-text-dim);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    text-decoration: none;
    transition: all 0.2s ease;
  }

  .card-action-btn:hover {
    color: var(--color-text-main);
    border-color: var(--color-orange-primary);
    background: rgba(var(--color-orange-rgb), 0.1);
  }

  .card-action-btn.delete-btn:hover {
    color: #fb7185;
    border-color: rgba(244, 63, 94, 0.5);
    background: rgba(244, 63, 94, 0.12);
  }

  .card-save-btn {
    flex: 1;
    height: 28px;
    border-radius: 6px;
    background: rgba(var(--color-orange-rgb), 0.14);
    border: 1px solid var(--color-orange-primary);
    color: #fb923c;
    font-size: 11.5px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .card-save-btn:hover {
    background: var(--color-orange-primary);
    color: #ffffff;
  }

  .card-saved-indicator {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    font-weight: 700;
    color: #10b981;
    padding: 4px 8px;
  }

  /* Captura en Vivo */
  .live-capture-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .banner-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 700;
    color: var(--color-orange-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  :global(.pulse-icon) {
    animation: pulse-live 1.8s infinite ease-in-out;
  }

  @keyframes pulse-live {
    0% { transform: scale(0.95); opacity: 0.8; }
    50% { transform: scale(1.15); opacity: 1; }
    100% { transform: scale(0.95); opacity: 0.8; }
  }

  .banner-description {
    font-size: 12.5px;
    color: var(--color-text-dim);
    margin: 0;
    line-height: 1.45;
  }

  .banner-description code {
    background: rgba(255, 255, 255, 0.06);
    color: var(--color-orange-hover);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11.5px;
  }

  /* Bookmarklet Info Box */
  .bookmarklet-info-box {
    margin-top: 6px;
    padding-top: 10px;
    border-top: 1px dashed rgba(var(--color-orange-rgb), 0.25);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .bookmarklet-info-header {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    color: var(--color-orange-primary);
  }

  .bookmarklet-info-desc {
    font-size: 12px;
    color: var(--color-text-dim);
    line-height: 1.45;
    margin: 0;
  }

  .bookmarklet-steps-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 10px;
    margin-top: 4px;
  }

  .guide-step-card {
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    padding: 10px 12px;
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }

  .guide-step-badge {
    background: rgba(var(--color-orange-rgb), 0.18);
    color: var(--color-orange-primary);
    border: 1px solid rgba(var(--color-orange-rgb), 0.4);
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: 700;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .guide-step-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .guide-step-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-main);
  }

  .guide-step-text {
    font-size: 11.5px;
    color: var(--color-text-dim);
    margin: 0;
    line-height: 1.35;
  }

  /* Live Unified Card */
  .live-unified-card {
    background: transparent;
    border: 1px solid var(--color-border-subtle);
    border-radius: 12px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .live-actions-top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
    padding: 14px 18px;
    background: rgba(var(--color-orange-rgb), 0.05);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .primary-action-wrap {
    display: flex;
    align-items: center;
  }

  .secondary-actions-wrap {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }

  .live-banner-content {
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .live-manual-details {
    border-top: 1px dashed var(--color-border-subtle);
    padding-top: 12px;
  }

  .manual-collapsible {
    font-size: 12.5px;
    color: var(--color-text-dim);
  }

  .collapsible-summary {
    cursor: pointer;
    user-select: none;
    font-weight: 500;
    color: var(--color-text-dim);
    transition: color 0.15s ease;
  }

  .collapsible-summary:hover {
    color: var(--color-orange-primary);
  }

  .collapsible-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 12px;
  }

  .live-manual-actions {
    display: flex;
    justify-content: flex-end;
  }

  :global(.tab-icon-active) {
    color: var(--color-orange-primary);
  }

  /* Modal de Código */
  .code-modal-content {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .code-modal-intro {
    font-size: 13px;
    color: var(--color-text-dim);
    line-height: 1.45;
    margin: 0;
  }

  .code-viewer-container {
    background: #141416;
    border: 1px solid var(--color-border-subtle);
    border-radius: 8px;
    overflow: hidden;
  }

  .code-viewer-block {
    margin: 0;
    padding: 14px 16px;
    max-height: 320px;
    overflow-y: auto;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 11.5px;
    color: #e4e4e7;
    line-height: 1.5;
    user-select: text;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .code-viewer-block code {
    font-family: inherit;
  }

  .code-modal-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 10px;
    border-top: 1px solid var(--color-border-subtle);
    flex-wrap: wrap;
    gap: 10px;
  }

  .modal-footer-actions {
    display: flex;
    gap: 8px;
  }
</style>

