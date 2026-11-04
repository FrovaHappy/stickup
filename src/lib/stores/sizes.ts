import { writable, derived } from 'svelte/store';
import type { SizePreset } from '$lib/types/emote';

export const CUSTOM_SIZES_STORAGE_KEY = 'stickup_custom_sizes';

export const DEFAULT_SIZE_PRESETS: SizePreset[] = [
  { label: 'Original', value: 'original' },
  { label: '48x48', width: 48, height: 48, value: '48' },
  { label: '64x64', width: 64, height: 64, value: '64' },
  { label: '128x128', width: 128, height: 128, value: '128' },
  { label: '256x256', width: 256, height: 256, value: '256' },
  { label: '512x512', width: 512, height: 512, value: '512' }
];

function loadSavedCustomSizes(): SizePreset[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const raw = localStorage.getItem(CUSTOM_SIZES_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      return parsed.filter(
        (p): p is SizePreset =>
          p &&
          typeof p.value === 'string' &&
          typeof p.width === 'number' &&
          typeof p.height === 'number' &&
          p.width > 0 &&
          p.height > 0
      );
    }
  } catch {
    // Error al cargar tamaños personalizados
  }
  return [];
}

function saveCustomSizes(items: SizePreset[]): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(CUSTOM_SIZES_STORAGE_KEY, JSON.stringify(items));
  } catch {
    // Error al guardar tamaños personalizados
  }
}

export const customSizes = writable<SizePreset[]>(loadSavedCustomSizes());

export const allSizePresets = derived(customSizes, ($custom) => [
  ...DEFAULT_SIZE_PRESETS,
  ...$custom
]);

export function addCustomSize(width: number, height: number): SizePreset | null {
  const w = Math.round(width);
  const h = Math.round(height);

  if (isNaN(w) || isNaN(h) || w < 8 || h < 8 || w > 4096 || h > 4096) {
    return null;
  }

  const value = `custom_${w}x${h}`;
  const label = `${w}x${h}`;
  const newPreset: SizePreset = { label, value, width: w, height: h };

  customSizes.update((list) => {
    // Si ya existe por value o por dimensiones idénticas, no duplicar
    const exists = list.some(
      (item) => item.value === value || (item.width === w && item.height === h)
    );
    if (exists) return list;
    const updated = [...list, newPreset];
    saveCustomSizes(updated);
    return updated;
  });

  return newPreset;
}

export function removeCustomSize(value: string): void {
  customSizes.update((list) => {
    const updated = list.filter((p) => p.value !== value);
    saveCustomSizes(updated);
    return updated;
  });
}

