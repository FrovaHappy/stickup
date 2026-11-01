import type { Emote, SizePreset } from '$lib/types/emote';
import { convertFileSrc } from '@tauri-apps/api/core';

export const SIZE_PRESETS: SizePreset[] = [
  { label: 'Original', value: 'original' },
  { label: '25x25', width: 25, height: 25, value: '25' },
  { label: '32x32', width: 32, height: 32, value: '32' },
  { label: '48x48', width: 48, height: 48, value: '48' },
  { label: '64x64', width: 64, height: 64, value: '64' },
  { label: '112x112', width: 112, height: 112, value: '112' },
  { label: '128x128', width: 128, height: 128, value: '128' }
];

export const FORMAT_PRESETS = ['webp', 'png', 'gif', 'jpg'] as const;

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
  selectedSize: string
): { width: number; height: number } {
  if (!emote) return { width: 48, height: 48 };
  if (selectedSize === 'original') {
    return { width: emote.original_width, height: emote.original_height };
  }
  const preset = SIZE_PRESETS.find((p) => p.value === selectedSize);
  if (preset && typeof preset.width === 'number') {
    return { width: preset.width, height: preset.height ?? preset.width };
  }
  return { width: emote.original_width, height: emote.original_height };
}
