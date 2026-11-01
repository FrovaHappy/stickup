import { writable, derived } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import type { Emote } from '$lib/types/emote';
import { api } from '$lib/services/api';

export type { Emote };

const emotes: Writable<Emote[]> = writable([]);

// Ventana de tiempo para considerar un emote como "Nuevo" por su fecha de creación (24 horas)
const NEW_THRESHOLD_MS = 24 * 60 * 60 * 1000;

export function isEmoteNewByDate(createdAt?: string | null): boolean {
  if (!createdAt) return false;
  const createdMs = Date.parse(createdAt);
  if (Number.isNaN(createdMs)) return false;
  const ageMs = Date.now() - createdMs;
  return ageMs >= -60_000 && ageMs <= NEW_THRESHOLD_MS;
}

// Top 20 de emotes con más copias / interacciones (al menos 1 copia o interacción)
export const topHotEmoteIds: Readable<Set<number>> = derived(emotes, ($emotes) => {
  const active = $emotes.filter(
    (e) => (e.copy_count ?? 0) > 0 || (e.interaction_count ?? 0) > 0
  );

  active.sort((a, b) => {
    const scoreA = (a.copy_count ?? 0) * 3 + (a.interaction_count ?? 0);
    const scoreB = (b.copy_count ?? 0) * 3 + (b.interaction_count ?? 0);
    if (scoreB !== scoreA) return scoreB - scoreA;
    return (b.copy_count ?? 0) - (a.copy_count ?? 0);
  });

  return new Set(active.slice(0, 20).map((e) => e.id));
});

export const allTags: Readable<string[]> = derived(emotes, ($emotes) => {
  const set = new Set<string>();
  for (const e of $emotes) {
    if (e.tag) {
      const parts = e.tag
        .split(',')
        .map((t) => t.trim())
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
    emotes.set(list ?? []);
  } catch (e) {
    console.error('Failed to load emotes', e);
  }
}

export default emotes;
