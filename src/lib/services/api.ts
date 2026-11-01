import { invoke } from '@tauri-apps/api/core';
import type { Emote, ResyncStats } from '$lib/types/emote';

export const api = {
  listEmotes: () => invoke<Emote[]>('list_emotes'),
  getStoragePath: () => invoke<string>('get_storage_path'),
  setStoragePath: (path: string, migrate = true) =>
    invoke<string>('set_storage_path', { path, migrate }),
  getDbPath: () => invoke<string>('get_db_path'),
  setDbPath: (path: string, migrate = true) => invoke<string>('set_db_path', { path, migrate }),
  importFolder: (folderPath: string) => invoke<Emote[]>('import_folder', { folderPath }),
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
