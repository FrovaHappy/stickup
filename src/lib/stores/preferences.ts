import { writable } from 'svelte/store';
import type { Emote } from '$lib/types/emote';
import type { GridSize, SortOption, SortOptionItem } from '$lib/types/preferences';
import { toast } from '$lib/stores/toast';

const GRID_SIZE_KEY = 'stickup_pref_grid_size';
const SORT_OPTION_KEY = 'stickup_pref_sort_option';
const ASK_BEFORE_PASTE_KEY = 'stickup_pref_ask_before_paste';

export const SORT_OPTIONS: SortOptionItem[] = [
  { id: 'date-desc', label: 'Más recientes' },
  { id: 'date-asc', label: 'Más antiguos' },
  { id: 'name-asc', label: 'Nombre (A - Z)' },
  { id: 'name-desc', label: 'Nombre (Z - A)' },
  { id: 'copies-desc', label: 'Más usados / Hot' }
];

function getStoredGridSize(): GridSize {
  if (typeof localStorage === 'undefined') return 'medium';
  try {
    const val = localStorage.getItem(GRID_SIZE_KEY);
    if (val === 'small' || val === 'medium' || val === 'large') {
      return val;
    }
  } catch {
    // Ignore storage errors
  }
  return 'medium';
}

function getStoredSortOption(): SortOption {
  if (typeof localStorage === 'undefined') return 'date-desc';
  try {
    const val = localStorage.getItem(SORT_OPTION_KEY);
    if (
      val === 'date-desc' ||
      val === 'date-asc' ||
      val === 'name-asc' ||
      val === 'name-desc' ||
      val === 'copies-desc'
    ) {
      return val;
    }
  } catch {
    // Ignore storage errors
  }
  return 'date-desc';
}

function getStoredAskBeforePaste(): boolean {
  if (typeof localStorage === 'undefined') return true;
  try {
    const val = localStorage.getItem(ASK_BEFORE_PASTE_KEY);
    if (val !== null) {
      return val === 'true';
    }
  } catch {
    // Ignore storage errors
  }
  return true;
}

export interface CopiedEmoteRecord {
  id: number;
  name: string;
  path: string;
  timestamp: number;
}

export const gridSize = writable<GridSize>(getStoredGridSize());
export const sortOption = writable<SortOption>(getStoredSortOption());
export const askBeforePaste = writable<boolean>(getStoredAskBeforePaste());
export const lastCopiedEmote = writable<CopiedEmoteRecord | null>(null);

export function recordCopiedEmote(emote: Emote) {
  lastCopiedEmote.set({
    id: emote.id,
    name: emote.name,
    path: emote.path,
    timestamp: Date.now()
  });
}

export function setAskBeforePaste(enabled: boolean) {
  askBeforePaste.set(enabled);
  if (enabled) {
    toast.success('Confirmación al pegar activada');
  } else {
    toast.info('Confirmación al pegar desactivada');
  }
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(ASK_BEFORE_PASTE_KEY, String(enabled));
    } catch {
      // Ignore
    }
  }
}

export function setGridSize(size: GridSize) {
  gridSize.set(size);
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(GRID_SIZE_KEY, size);
    } catch {
      // Ignore
    }
  }
}

export function setSortOption(option: SortOption) {
  sortOption.set(option);
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(SORT_OPTION_KEY, option);
    } catch {
      // Ignore
    }
  }
}

export function sortEmotesList(emotes: Emote[], option: SortOption): Emote[] {
  const list = [...emotes];
  switch (option) {
    case 'name-asc':
      return list.sort((a, b) =>
        (a.name || '').localeCompare(b.name || '', undefined, {
          sensitivity: 'base',
          numeric: true
        })
      );
    case 'name-desc':
      return list.sort((a, b) =>
        (b.name || '').localeCompare(a.name || '', undefined, {
          sensitivity: 'base',
          numeric: true
        })
      );
    case 'date-asc':
      return list.sort((a, b) => {
        const timeDiff =
          (Date.parse(a.created_at || '') || 0) - (Date.parse(b.created_at || '') || 0);
        if (timeDiff !== 0) return timeDiff;
        return a.id - b.id;
      });
    case 'copies-desc':
      return list.sort((a, b) => {
        const copiesDiff = (b.copy_count ?? 0) - (a.copy_count ?? 0);
        if (copiesDiff !== 0) return copiesDiff;
        const interDiff = (b.interaction_count ?? 0) - (a.interaction_count ?? 0);
        if (interDiff !== 0) return interDiff;
        return b.id - a.id;
      });
    case 'date-desc':
    default:
      return list.sort((a, b) => {
        const timeDiff =
          (Date.parse(b.created_at || '') || 0) - (Date.parse(a.created_at || '') || 0);
        if (timeDiff !== 0) return timeDiff;
        return b.id - a.id;
      });
  }
}

