import type { Emote, SizePreset } from '$lib/types/emote';
import { convertFileSrc } from '@tauri-apps/api/core';

export const SIZE_PRESETS: SizePreset[] = [
  { label: 'Original', value: 'original' },
  { label: '48x48', width: 48, height: 48, value: '48' },
  { label: '64x64', width: 64, height: 64, value: '64' },
  { label: '128x128', width: 128, height: 128, value: '128' },
  { label: '256x256', width: 256, height: 256, value: '256' },
  { label: '512x512', width: 512, height: 512, value: '512' }
];

export const STATIC_FORMAT_PRESETS = ['webp', 'png', 'jpg'] as const;
export const ANIMATED_FORMAT_PRESETS = ['gif'] as const;

export function isAnimatedEmote(emote: Emote | null): boolean {
  if (!emote) return false;
  return (
    emote.format?.toLowerCase() === 'gif' ||
    emote.path?.toLowerCase().endsWith('.gif')
  );
}

export function getAvailableFormats(emote: Emote | null): readonly string[] {
  return isAnimatedEmote(emote) ? ANIMATED_FORMAT_PRESETS : STATIC_FORMAT_PRESETS;
}

export function getMimeType(format: string): string {
  switch (format.toLowerCase()) {
    case 'webp':
      return 'image/webp';
    case 'jpg':
    case 'jpeg':
      return 'image/jpeg';
    default:
      return 'image/png';
  }
}

export async function renderEmoteToCanvas(
  path: string,
  width: number,
  height: number
): Promise<HTMLCanvasElement> {
  const img = new Image();
  img.crossOrigin = 'anonymous';
  img.src = convertFileSrc(path);

  await new Promise<void>((resolve, reject) => {
    if (img.complete) resolve();
    else {
      img.onload = () => resolve();
      img.onerror = () => reject(new Error('No se pudo cargar la imagen'));
    }
  });

  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('No se pudo inicializar canvas');
  ctx.drawImage(img, 0, 0, width, height);
  return canvas;
}

export function formatFileSize(bytes?: number | null): string {
  if (!bytes || bytes <= 0) return 'Desconocido';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function resolveDimensions(
  emote: Emote | null,
  selectedSize: string,
  extraPresets?: SizePreset[]
): { width: number; height: number } {
  if (!emote) return { width: 48, height: 48 };
  if (selectedSize === 'original') {
    return { width: emote.original_width, height: emote.original_height };
  }
  const allPresets = extraPresets ? [...SIZE_PRESETS, ...extraPresets] : SIZE_PRESETS;
  const preset = allPresets.find((p) => p.value === selectedSize);
  if (preset && typeof preset.width === 'number') {
    return { width: preset.width, height: preset.height ?? preset.width };
  }
  const match = selectedSize.match(/^(?:custom_)?(\d+)x(\d+)$/i);
  if (match) {
    const w = parseInt(match[1], 10);
    const h = parseInt(match[2], 10);
    if (!isNaN(w) && !isNaN(h) && w > 0 && h > 0) {
      return { width: w, height: h };
    }
  }
  return { width: emote.original_width, height: emote.original_height };
}

/**
 * Normaliza cualquier texto a convención PascalCase:
 * - "pepe dance" -> "PepeDance"
 * - "pepe_dance" -> "PepeDance"
 * - "pepe-dance" -> "PepeDance"
 * - "catJam" -> "CatJam"
 * - "PogChamp" -> "PogChamp"
 * - "OMEGALUL" -> "Omegalul"
 */
export function toPascalCase(str?: string | null): string {
  if (!str) return '';
  const trimmed = str.trim();
  if (!trimmed) return '';

  // Separar transiciones de camelCase / PascalCase / acrónimos
  const separated = trimmed
    .replace(/([a-z\d\u00C0-\u024F\u1E00-\u1EFF])([A-Z])/g, '$1 $2')
    .replace(/([A-Z]+)([A-Z][a-z\d\u00C0-\u024F\u1E00-\u1EFF]+)/g, '$1 $2')
    .replace(/[^a-zA-Z0-9\u00C0-\u024F\u1E00-\u1EFF]+/g, ' ');

  const words = separated.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return '';

  return words
    .map((word) => {
      if (word.length === 0) return '';
      return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase();
    })
    .join('');
}

/**
 * Parsea el texto del buscador en tiempo real a PascalCase:
 * Preserva un espacio al final si el usuario acaba de escribir un separador,
 * y en el siguiente carácter lo compacta en el nuevo término en mayúscula.
 */
export function formatSearchQueryInput(input: string): string {
  if (!input) return '';

  if (!/[a-zA-Z0-9\u00C0-\u024F\u1E00-\u1EFF]/.test(input)) {
    return '';
  }

  const hasTrailingSeparator = /[\s_\-]+$/.test(input);
  const parsed = toPascalCase(input);
  if (!parsed) return '';

  return hasTrailingSeparator ? `${parsed} ` : parsed;
}

/**
 * Sanitiza el nombre de un emote permitiendo letras, números, espacios y caracteres seguros,
 * eliminando caracteres ilegales del sistema de archivos y colapsando espacios consecutivos.
 */
export function sanitizeEmoteName(str?: string | null): string {
  if (!str) return '';
  const trimmed = str.trim();
  if (!trimmed) return '';

  return trimmed
    .replace(/[/\\:*?"<>|\x00-\x1f\x7f-\x9f]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

export interface ParsedUrlOrPath {
  type: 'local' | 'web';
  value: string;
}

/**
 * Normaliza y clasifica una entrada de texto como ruta local (archivo o directorio)
 * o URL web (archivo, directorio o página). Soporta file://, paths Windows y Unix.
 */
export function parseUrlOrPath(raw: string): ParsedUrlOrPath {
  let trimmed = (raw || '').trim();
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    trimmed = trimmed.slice(1, -1).trim();
  }

  // Comprobar esquema file://
  if (trimmed.toLowerCase().startsWith('file://')) {
    let clean = trimmed.replace(/^file:\/\/(localhost)?/i, '');
    try {
      clean = decodeURIComponent(clean);
    } catch {
      // conservar sin decodificar si falla
    }
    // En Windows, si queda con letra de unidad como /C:/ o C:/
    if (/^\/?[a-zA-Z]:/.test(clean)) {
      if (clean.startsWith('/')) {
        clean = clean.slice(1);
      }
      clean = clean.replace(/\//g, '\\');
    } else if (!clean.startsWith('/') && !clean.startsWith('\\')) {
      clean = '/' + clean;
    }
    return { type: 'local', value: clean };
  }

  // Rutas locales de Windows (C:\..., C:/..., \\server\share), Unix (/home/...), o relativas (~, ./, ../)
  if (
    /^[a-zA-Z]:[/\\]/.test(trimmed) ||
    trimmed.startsWith('\\\\') ||
    (trimmed.startsWith('/') && !trimmed.startsWith('//')) ||
    trimmed.startsWith('~/') ||
    trimmed === '~' ||
    trimmed.startsWith('./') ||
    trimmed.startsWith('../')
  ) {
    return { type: 'local', value: trimmed };
  }

  // URL web (http, https o dominio directo)
  let webUrl = trimmed;
  if (!webUrl.startsWith('http://') && !webUrl.startsWith('https://')) {
    webUrl = 'https://' + webUrl;
  }
  return { type: 'web', value: webUrl };
}


