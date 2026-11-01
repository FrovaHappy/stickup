import { writable, derived } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import type { Emote } from '$lib/types/emote';
import { api } from '$lib/services/api';

export type { Emote };

const emotes: Writable<Emote[]> = writable([]);

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
