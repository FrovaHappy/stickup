import { writable, derived } from 'svelte/store';
import type { SizePreset } from '$lib/types/emote';
import { fitDimensions } from '$lib/utils/format';

export const CUSTOM_SIZES_STORAGE_KEY = 'stickup_custom_sizes';
export const DEFAULT_RESOLUTION_STORAGE_KEY = 'stickup_pref_default_resolution';

export const DEFAULT_SIZE_PRESETS: SizePreset[] = [
  { label: 'Original', value: 'original' },
  { label: '48x48', width: 48, height: 48, value: '48' },
  { label: '64x64', width: 64, height: 64, value: '64' },
  { label: '128x128', width: 128, height: 128, value: '128' },
  { label: '256x256', width: 256, height: 256, value: '256' },
  { label: '512x512', width: 512, height: 512, value: '512' }
];

function loadSavedDefaultResolution(): string {
  if (typeof localStorage === 'undefined') return 'original';
  try {
    const val = localStorage.getItem(DEFAULT_RESOLUTION_STORAGE_KEY);
    if (val && typeof val === 'string' && val.trim().length > 0) {
      return val.trim();
    }
  } catch {
    // Ignore
  }
  return 'original';
}

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
export const defaultResolution = writable<string>(loadSavedDefaultResolution());

export const allSizePresets = derived(customSizes, ($custom) => [
  ...DEFAULT_SIZE_PRESETS,
  ...$custom
]);

export function setDefaultResolution(value: string): void {
  const clean = value?.trim() || 'original';
  defaultResolution.set(clean);
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(DEFAULT_RESOLUTION_STORAGE_KEY, clean);
    } catch {
      // Ignore
    }
  }
}

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

  defaultResolution.update((curr) => {
    if (curr === value) {
      if (typeof localStorage !== 'undefined') {
        try {
          localStorage.setItem(DEFAULT_RESOLUTION_STORAGE_KEY, 'original');
        } catch {
          // Ignore
        }
      }
      return 'original';
    }
    return curr;
  });
}

export function resolveDefaultDimensions(
  emote: { original_width: number; original_height: number } | null,
  defaultRes: string,
  customList: SizePreset[] = []
): { width: number; height: number; label: string } {
  if (!emote) return { width: 48, height: 48, label: '48×48' };
  if (!defaultRes || defaultRes === 'original') {
    return {
      width: emote.original_width,
      height: emote.original_height,
      label: 'Original'
    };
  }
  const all = [...DEFAULT_SIZE_PRESETS, ...customList];
  const found = all.find((p) => p.value === defaultRes);
  let boundW = emote.original_width;
  let boundH = emote.original_height;
  let label = 'Original';

  if (found && typeof found.width === 'number') {
    boundW = found.width;
    boundH = found.height ?? found.width;
    label = found.label;
  } else {
    const match = defaultRes.match(/^(?:custom_)?(\d+)x(\d+)$/i);
    if (match) {
      const w = parseInt(match[1], 10);
      const h = parseInt(match[2], 10);
      if (!isNaN(w) && !isNaN(h) && w > 0 && h > 0) {
        boundW = w;
        boundH = h;
        label = `${w}×${h}`;
      }
    } else {
      return {
        width: emote.original_width,
        height: emote.original_height,
        label: 'Original'
      };
    }
  }

  const fitted = fitDimensions(emote.original_width, emote.original_height, boundW, boundH);
  return {
    width: fitted.width,
    height: fitted.height,
    label
  };
}

