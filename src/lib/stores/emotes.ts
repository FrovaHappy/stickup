import { writable, derived } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import type { Emote } from '$lib/types/emote';
import { api } from '$lib/services/api';
import { toPascalCase, sanitizeEmoteName } from '$lib/utils/format';

export type { Emote };

const emotes: Writable<Emote[]> = writable([]);

// Ventana de tiempo para considerar un emote como "Nuevo" por su fecha de creación (10 minutos)
const NEW_THRESHOLD_MS = 10 * 60 * 1000;

export function isEmoteNewByDate(createdAt?: string | null): boolean {
  if (!createdAt) return false;
  const createdMs = Date.parse(createdAt);
  if (Number.isNaN(createdMs)) return false;
  const ageMs = Date.now() - createdMs;
  return ageMs >= -60_000 && ageMs <= NEW_THRESHOLD_MS;
}

function parseTimestamp(iso?: string | null): number {
  if (!iso) return 0;
  const ms = Date.parse(iso);
  return Number.isNaN(ms) ? 0 : ms;
}

/**
 * Ordena exclusivamente por cantidad de copias (`copy_count`):
 * 1. Mayor `copy_count` va primero (#1, #2, ...).
 * 2. En caso de empate en copias, el copiado más recientemente (`last_used_at`).
 * 3. Desempate determinista por `id` (no depende de abrir el modal).
 */
export function compareEmotesByPopularity(a: Emote, b: Emote): number {
  const copiesA = a.copy_count ?? 0;
  const copiesB = b.copy_count ?? 0;
  if (copiesB !== copiesA) return copiesB - copiesA;

  const lastCopiedDiff = parseTimestamp(b.last_used_at) - parseTimestamp(a.last_used_at);
  if (lastCopiedDiff !== 0) return lastCopiedDiff;

  return a.id - b.id;
}

// Mapa del Top 20: emote.id -> posición en el ranking (1..20)
// Solo entran al Top 20 los emotes que han sido copiados al menos 1 vez (copy_count > 0)
export const topHotRanks: Readable<Map<number, number>> = derived(emotes, ($emotes) => {
  const copiedEmotes = $emotes.filter((e) => (e.copy_count ?? 0) > 0);

  copiedEmotes.sort(compareEmotesByPopularity);

  const rankMap = new Map<number, number>();
  copiedEmotes.slice(0, 20).forEach((e, idx) => {
    rankMap.set(e.id, idx + 1);
  });
  return rankMap;
});

export const allTags: Readable<string[]> = derived(emotes, ($emotes) => {
  const set = new Set<string>();
  for (const e of $emotes) {
    if (e.tag) {
      const parts = e.tag
        .split(',')
        .map((t) => toPascalCase(t.trim()))
        .filter(Boolean);
      for (const t of parts) {
        set.add(t);
      }
    }
  }
  return Array.from(set).sort((a, b) => a.localeCompare(b));
});

export async function loadEmotes() {
  try {
    const list = await api.listEmotes();
    const normalized = (list ?? []).map((e) => ({
      ...e,
      name: sanitizeEmoteName(e.name) || e.name,
      tag: e.tag
        ? e.tag
            .split(',')
            .map((t) => toPascalCase(t.trim()))
            .filter(Boolean)
            .join(', ')
        : e.tag
    }));
    emotes.set(normalized);
  } catch (e) {
    console.error('Failed to load emotes', e);
  }
}

export default emotes;
