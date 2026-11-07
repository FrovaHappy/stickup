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
    Image as ImageIcon
  } from "@lucide/svelte";
  import { Button, Input, Badge, Alert } from "$lib/components/system-ui";
  import { api } from "$lib/services/api";
  import { loadEmotes } from "$lib/stores/emotes";
  import { toast } from "$lib/stores/toast";
  import type { ScannedImage, ScannedImageImportRequest } from "$lib/types/scanner";

  type ScanTab = "url" | "html";

  let activeTab: ScanTab = "url";
  let targetUrl = "";
  let rawHtml = "";
  let htmlBaseUrl = "";
  let isScanning = false;
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
    const trimmed = targetUrl.trim();
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
    {#if activeTab === "url"}
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
    {:else}
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
        <Alert variant="warning" title="Aviso de escaneo" dismissible on:close={() => (scanError = null)}>
          {scanError}
        </Alert>
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
                    <Badge variant="success">Guardado</Badge>
                  {/if}
                  {#if img.format_hint}
                    <Badge variant="neutral">{img.format_hint.toUpperCase()}</Badge>
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
              <div class="card-footer" on:click|stopPropagation>
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

                  <a
                    href={img.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="card-action-btn"
                    title="Abrir imagen en navegador"
                  >
                    <ExternalLink size={14} />
                  </a>

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
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 14px;
    padding: 14px 18px;
    background: transparent;
    border: 1px solid var(--color-border-subtle);
    border-radius: 12px;
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
</style>

