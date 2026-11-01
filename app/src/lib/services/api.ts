import { invoke } from '@tauri-apps/api/core';
import type { Emote } from '$lib/types/emote';

export const api = {
  listEmotes: () => invoke<Emote[]>('list_emotes'),
  getStoragePath: () => invoke<string>('get_storage_path'),
  setStoragePath: (path: string) => invoke<string>('set_storage_path', { path }),
  importFolder: (folderPath: string) => invoke<Emote[]>('import_folder', { folderPath }),
  deleteEmote: (id: number) => invoke<boolean>('delete_emote', { id }),
  updateTag: (id: number, name: string, tag: string) =>
    invoke<boolean>('update_tag', { id, name, tag }),
  copyToClipboard: (path: string, format: string, targetWidth: number, targetHeight: number) =>
    invoke<boolean>('copy_to_clipboard', { path, format, targetWidth, targetHeight })
};

