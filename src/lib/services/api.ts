import { invoke } from '@tauri-apps/api/core';
import type { Emote, ResyncStats } from '$lib/types/emote';

export const api = {
  listEmotes: () => invoke<Emote[]>('list_emotes'),
  getStoragePath: () => invoke<string>('get_storage_path'),
  setStoragePath: (path: string, migrate = true) =>
    invoke<string>('set_storage_path', { path, migrate }),
  getDbPath: () => invoke<string>('get_db_path'),
  setDbPath: (path: string, migrate = true) => invoke<string>('set_db_path', { path, migrate }),
  importPaths: (paths: string[]) => invoke<Emote[]>('import_paths', { paths }),
  importRawBytes: (bytes: number[], name?: string) =>
    invoke<Emote>('import_raw_bytes', { bytes, name: name?.trim() || null }),
  importFromUrl: (url: string, name?: string) =>
    invoke<Emote>('import_from_url', { url, name: name?.trim() || null }),
  importFromClipboard: (name?: string) =>
    invoke<Emote>('import_from_clipboard', { name: name?.trim() || null }),
  deleteEmote: (id: number) => invoke<boolean>('delete_emote', { id }),
  updateTag: (id: number, name: string, tag: string) =>
    invoke<boolean>('update_tag', { id, name, tag }),
  copyToClipboard: (
    id: number,
    path: string,
    format: string,
    targetWidth: number,
    targetHeight: number
  ) => invoke<boolean>('copy_to_clipboard', { id, path, format, targetWidth, targetHeight }),
  recordInteraction: (id: number) => invoke<boolean>('record_interaction', { id }),
  resyncEmotes: () => invoke<ResyncStats>('resync_emotes')
};
