import type { RawScannedItem } from '$lib/types/scanner';

export interface CollectorPayload {
  stickup_collector_v1: boolean;
  source?: string;
  items: RawScannedItem[];
}

/**
 * Genera el script ejecutable para inyectar en la consola o usar como Bookmarklet en cualquier navegador.
 * Dispone de una interfaz flotante con Iniciar / Parar, sondeo cada 1s del DOM, desduplicación en Set/Map
 * y copia automática de resultados al portapapeles en formato JSON compatible con StickUp.
 */
export function generateCollectorScript(): string {
  return `(function() {
  if (window.__STICKUP_COLLECTOR_ACTIVE__) {
    alert('StickUp Collector ya está activo en esta pestaña.');
    return;
  }
  window.__STICKUP_COLLECTOR_ACTIVE__ = true;

  const collectedMap = new Map();
  let timer = null;
  let startTime = null;

  const panel = document.createElement('div');
  panel.id = '__stickup_panel__';
  Object.assign(panel.style, {
    position: 'fixed',
    bottom: '24px',
    right: '24px',
    zIndex: '2147483647',
    backgroundColor: 'rgb(26, 26, 28)',
    color: 'rgb(255, 255, 255)',
    padding: '14px 18px',
    borderRadius: '12px',
    boxShadow: '0 10px 35px rgba(0,0,0,0.7), 0 0 0 1px rgb(249, 115, 22)',
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    fontSize: '13px',
    lineHeight: '1.4',
    display: 'flex',
    flexDirection: 'column',
    gap: '10px',
    minWidth: '240px',
    maxWidth: '320px',
    backdropFilter: 'blur(8px)',
    cursor: 'default',
    userSelect: 'none'
  });

  const header = document.createElement('div');
  Object.assign(header.style, {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    fontWeight: '600',
    color: 'rgb(249, 115, 22)'
  });
  header.innerHTML = '<span>⚡ StickUp Live Collector</span>';

  const closeBtn = document.createElement('button');
  closeBtn.textContent = '✕';
  Object.assign(closeBtn.style, {
    background: 'none',
    border: 'none',
    color: 'rgb(161, 161, 170)',
    cursor: 'pointer',
    fontSize: '14px',
    padding: '0 4px'
  });
  closeBtn.onclick = () => cleanup();
  header.appendChild(closeBtn);

  const statusText = document.createElement('div');
  statusText.style.color = 'rgb(212, 212, 216)';
  statusText.style.fontSize = '12px';
  statusText.textContent = 'Listo para capturar mientras navegas.';

  const statsRow = document.createElement('div');
  Object.assign(statsRow.style, {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '12px',
    background: 'rgba(255, 255, 255, 0.05)',
    padding: '6px 10px',
    borderRadius: '6px'
  });
  statsRow.innerHTML = '<span>Emotes detectados:</span><strong id="__su_count__" style="color: rgb(251, 146, 60);">0</strong>';

  const actionsRow = document.createElement('div');
  Object.assign(actionsRow.style, {
    display: 'flex',
    gap: '8px'
  });

  const toggleBtn = document.createElement('button');
  toggleBtn.textContent = '🔴 Iniciar Captura';
  Object.assign(toggleBtn.style, {
    flex: '1',
    backgroundColor: 'rgb(249, 115, 22)',
    color: 'rgb(255, 255, 255)',
    border: 'none',
    borderRadius: '6px',
    padding: '8px 12px',
    fontWeight: '600',
    cursor: 'pointer',
    transition: 'background 0.15s ease'
  });

  function scanDom() {
    for (const img of document.images) {
      const src = img.currentSrc || img.src || img.getAttribute('data-src');
      if (!src || src.startsWith('data:image/svg')) continue;

      const w = img.naturalWidth || img.width;
      const h = img.naturalHeight || img.height;
      if (w >= 16 && w <= 512 && h >= 16 && h <= 512) {
        try {
          const absUrl = new URL(src, window.location.href).href;
          if (!collectedMap.has(absUrl)) {
            const alt = (img.alt || img.title || '').trim();
            collectedMap.set(absUrl, alt || null);
          }
        } catch (_) {}
      }
    }

    const countEl = document.getElementById('__su_count__');
    if (countEl) countEl.textContent = collectedMap.size.toString();
  }

  function start() {
    timer = setInterval(scanDom, 1000);
    scanDom();
    toggleBtn.textContent = '⏹️ Parar y Copiar';
    toggleBtn.style.backgroundColor = 'rgb(234, 88, 12)';
    statusText.textContent = '● Capturando cada 1s...';
    statusText.style.color = 'rgb(74, 222, 128)';
  }

  async function stopAndExport() {
    if (timer) clearInterval(timer);
    timer = null;

    const items = Array.from(collectedMap.entries()).map(([url, alt]) => ({ url, alt }));
    const payload = {
      stickup_collector_v1: true,
      source: window.location.href,
      items
    };

    const jsonStr = JSON.stringify(payload, null, 2);
    try {
      await navigator.clipboard.writeText(jsonStr);
      statusText.textContent = '✓ ¡Copiado! Abre StickUp para importar.';
      statusText.style.color = 'rgb(56, 189, 248)';
      toggleBtn.textContent = 'Copiado al Portapapeles';
      toggleBtn.disabled = true;
      setTimeout(cleanup, 2200);
    } catch (e) {
      prompt('Copia este JSON para StickUp:', jsonStr);
      cleanup();
    }
  }

  function cleanup() {
    if (timer) clearInterval(timer);
    window.__STICKUP_COLLECTOR_ACTIVE__ = false;
    panel.remove();
  }

  toggleBtn.onclick = () => {
    if (!timer) {
      start();
    } else {
      stopAndExport();
    }
  };

  actionsRow.appendChild(toggleBtn);
  panel.appendChild(header);
  panel.appendChild(statusText);
  panel.appendChild(statsRow);
  panel.appendChild(actionsRow);
  document.body.appendChild(panel);
})();`;
}

/**
 * Genera la versión Bookmarklet (javascript:...) limpia para guardar en la barra de marcadores del navegador.
 * Se formatea en una sola línea válida sin codificaciones pesadas (%20, %7B, %0A), permitiendo copiarla
 * y pegarla de forma legible directamente como URL de marcador en cualquier navegador.
 */
export function generateBookmarklet(): string {
  const code = generateCollectorScript();
  const singleLine = code
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .join(' ');

  return `javascript:${singleLine}`;
}

/**
 * Parsea datos provenientes de la captura interactiva o el portapapeles.
 * Acepta JSON de StickUp Collector, listas de URLs o cadenas de texto.
 */
export function parseCollectorInput(input: string): { items: RawScannedItem[]; source?: string } {
  const trimmed = input.trim();
  if (!trimmed) {
    return { items: [] };
  }

  // 1. Intento de parseo JSON estructurado
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    try {
      const parsed = JSON.parse(trimmed);
      if (parsed && typeof parsed === 'object') {
        if (Array.isArray(parsed)) {
          const items: RawScannedItem[] = [];
          const seen = new Set<string>();
          for (const item of parsed) {
            const rawUrl = typeof item === 'string' ? item : String(item?.url || '');
            const url = rawUrl.trim();
            if (!url || seen.has(url)) continue;
            seen.add(url);
            const alt = typeof item === 'object' && item?.alt ? String(item.alt).trim() || null : null;
            items.push({ url, alt });
          }
          return { items };
        }

        if (parsed.items && Array.isArray(parsed.items)) {
          const items: RawScannedItem[] = [];
          const seen = new Set<string>();
          for (const item of parsed.items) {
            const rawUrl = typeof item === 'string' ? item : String(item?.url || '');
            const url = rawUrl.trim();
            if (!url || seen.has(url)) continue;
            seen.add(url);
            const alt = typeof item === 'object' && item?.alt ? String(item.alt).trim() || null : null;
            items.push({ url, alt });
          }
          return { items, source: parsed.source || undefined };
        }
      }
    } catch (_) {
      // Continuar a fallback de líneas de texto
    }
  }

  // 2. Fallback: Parseo línea a línea de URLs
  const lines = trimmed.split(/[\r\n]+/);
  const items: RawScannedItem[] = [];
  const seen = new Set<string>();

  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (!line) continue;
    const tokens = line.split(/\s+/);
    const candidateUrl = tokens[0] || '';
    if (candidateUrl.startsWith('http://') || candidateUrl.startsWith('https://') || candidateUrl.startsWith('data:image/')) {
      if (!seen.has(candidateUrl)) {
        seen.add(candidateUrl);
        const remaining = tokens.slice(1).join(' ').replace(/^[#;/\s]+/, '').trim();
        items.push({ url: candidateUrl, alt: remaining || null });
      }
    }
  }

  return { items };
}

