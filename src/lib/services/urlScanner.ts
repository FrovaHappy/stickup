import { api } from './api';
import type { Emote } from '$lib/types/emote';
import type { ScannedImage } from '$lib/types/scanner';
import { parseUrlOrPath } from '$lib/utils/format';

/**
 * Comprueba si una URL pertenece a la CDN de Discord (emojis, avatares, stickers).
 */
export function isDiscordUrl(url: string): boolean {
  const lower = url.toLowerCase();
  return (
    lower.includes('cdn.discordapp.com') ||
    lower.includes('media.discordapp.net') ||
    lower.includes('.discordapp.net') ||
    lower.includes('.discordapp.com')
  );
}

/**
 * Comprueba si una URL de Discord es animada según sus parámetros o extensión.
 */
export function isDiscordAnimatedUrl(url: string): boolean {
  if (!isDiscordUrl(url)) return false;
  const lower = url.toLowerCase();
  if (lower.includes('animated=true')) return true;
  const withoutQuery = lower.split('?')[0].split('#')[0];
  if (withoutQuery.endsWith('.gif')) return true;
  const lastSeg = withoutQuery.split('/').pop() || '';
  return lastSeg.startsWith('a_');
}

/**
 * Detecta si una cadena apunta directamente a un archivo de imagen o endpoint de imagen conocido.
 */
export function isDirectImageUrl(url: string): boolean {
  const clean = url.trim();
  if (!clean.startsWith('http://') && !clean.startsWith('https://')) {
    return false;
  }

  if (isDiscordUrl(clean)) {
    const withoutQuery = clean.split('?')[0].split('#')[0];
    if (
      withoutQuery.includes('/emojis/') ||
      withoutQuery.includes('/avatars/') ||
      withoutQuery.includes('/stickers/')
    ) {
      return true;
    }
  }

  const withoutFragment = clean.split('#')[0];
  const withoutQuery = withoutFragment.split('?')[0];
  return /\.(png|jpe?g|gif|webp|svg|avif|bmp)$/i.test(withoutQuery);
}

/**
 * Limpia y normaliza una URL ingresada por el usuario o extraída del portapapeles.
 */
export function cleanUrlInput(raw: string): string {
  let cleaned = raw.trim().replace(/^["']+|["']+$/g, '').trim();
  // Limpiar entidades HTML residuales como &amp;
  cleaned = cleaned.replace(/&amp;/g, '&');
  return cleaned;
}

export interface UrlImportResult {
  type: 'local' | 'direct' | 'scanned';
  localEmotes?: Emote[];
  directEmote?: Emote;
  scannedImages?: ScannedImage[];
}

/**
 * Flujo unificado para procesar cualquier entrada de URL o ruta local:
 * 1. Si es ruta local: importa archivos de la ruta.
 * 2. Si es URL de imagen directa (o Discord emote): intenta importación directa. Si falla, hace fallback a escáner.
 * 3. Si es URL de página web: ejecuta el escáner web.
 */
export async function processUrlOrPath(
  input: string,
  callbacks?: {
    onStatus?: (msg: string) => void;
  }
): Promise<UrlImportResult> {
  const parsed = parseUrlOrPath(cleanUrlInput(input));

  if (parsed.type === 'local') {
    callbacks?.onStatus?.('Importando desde ruta local...');
    const imported = await api.importPaths([parsed.value]);
    return { type: 'local', localEmotes: imported };
  }

  const targetUrl = parsed.value;

  if (isDirectImageUrl(targetUrl)) {
    callbacks?.onStatus?.('Descargando imagen desde URL...');
    try {
      const created = await api.importFromUrl(targetUrl);
      return { type: 'direct', directEmote: created };
    } catch (err: any) {
      const msg = String(err?.message || err);
      if (msg.includes('ya existe') || msg.includes('bloqueado') || msg.includes('no compatible')) {
        throw err;
      }
      callbacks?.onStatus?.('Escaneando contenido de la URL...');
      try {
        const scanned = await api.scanUrl(targetUrl);
        if (scanned && scanned.length > 0) {
          return { type: 'scanned', scannedImages: scanned };
        }
      } catch {
        // Fallback no produjo resultados
      }
      throw err;
    }
  }

  // Página web o directorio
  callbacks?.onStatus?.('Escaneando URL / directorio web...');
  const scanned = await api.scanUrl(targetUrl);
  return { type: 'scanned', scannedImages: scanned || [] };
}

