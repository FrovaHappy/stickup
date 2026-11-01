import { writable, derived } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Emote {
  id: number;
  name: string;
  tag?: string | null;
  path: string;
  original_width: number;
  original_height: number;
  format: string;
  created_at: string;
  file_size?: number | null;
}

const emotes: Writable<Emote[]> = writable([]);

// Lista global de todas las etiquetas existentes en los emotes
export const allTags: Readable<string[]> = derived(emotes, ($emotes) => {
  const set = new Set<string>();
  for (const e of $emotes) {
    if (e.tag) {
      const parts = e.tag.split(',').map((t) => t.trim()).filter(Boolean);
      for (const t of parts) {
        set.add(t);
      }
    }
  }
  return Array.from(set).sort((a, b) => a.localeCompare(b));
});

export async function loadEmotes() {
  try {
    const list = await invoke<Emote[]>('list_emotes');
    emotes.set(list ?? []);
  } catch (e) {
    console.error('Failed to load emotes', e);
  }
}

export default emotes;
