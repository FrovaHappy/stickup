import { invoke } from '@tauri-apps/api/core';
import type { ClipboardInspectResult, Emote, ResyncStats } from '$lib/types/emote';
import type { BackupInfo, BackupResult, RestoreResult } from '$lib/types/backup';
import type { ScannedImage, ScannedImageImportRequest, BatchImportResult } from '$lib/types/scanner';

export const api = {
  listEmotes: () => invoke<Emote[]>('list_emotes'),
  getStoragePath: () => invoke<string>('get_storage_path'),
  setStoragePath: (path: string, migrate = true, overwriteDb?: boolean) =>
    invoke<string>('set_storage_path', { path, migrate, overwriteDb: overwriteDb ?? null }),
  importPaths: (paths: string[]) => invoke<Emote[]>('import_paths', { paths }),
  importRawBytes: (bytes: number[], name?: string) =>
    invoke<Emote>('import_raw_bytes', { bytes, name: name?.trim() || null }),
  importFromUrl: (url: string, name?: string) =>
    invoke<Emote>('import_from_url', { url, name: name?.trim() || null }),
  importFromClipboard: (name?: string) =>
    invoke<Emote | Emote[]>('import_from_clipboard', { name: name?.trim() || null }),
  inspectClipboard: () => invoke<ClipboardInspectResult>('inspect_clipboard'),
  deleteEmote: (id: number) => invoke<boolean>('delete_emote', { id }),
  updateTag: (id: number, name: string, tag: string) =>
    invoke<string>('update_tag', { id, name, tag }),
  copyToClipboard: (
    id: number,
    path: string,
    format: string,
    targetWidth: number,
    targetHeight: number
  ) => invoke<boolean>('copy_to_clipboard', { id, path, format, targetWidth, targetHeight }),
  recordInteraction: (id: number) => invoke<boolean>('record_interaction', { id }),
  resyncEmotes: () => invoke<ResyncStats>('resync_emotes'),
  createBackup: (destinationPath?: string) =>
    invoke<BackupResult>('create_backup', { destinationPath: destinationPath || null }),
  restoreBackup: (backupPath: string) =>
    invoke<RestoreResult>('restore_backup', { backupPath }),
  deleteBackup: (backupPath: string) =>
    invoke<boolean>('delete_backup', { backupPath }),
  listBackups: () => invoke<BackupInfo[]>('list_backups'),
  inspectBackup: (backupPath: string) =>
    invoke<BackupInfo>('inspect_backup', { backupPath }),
  getAppVersion: () => invoke<string>('get_app_version'),
  getMinimizeToTray: () => invoke<boolean>('get_minimize_to_tray'),
  setMinimizeToTray: (enabled: boolean) =>
    invoke<void>('set_minimize_to_tray', { enabled }),
  getMinimizeOnCopy: () => invoke<boolean>('get_minimize_on_copy'),
  setMinimizeOnCopy: (enabled: boolean) =>
    invoke<void>('set_minimize_on_copy', { enabled }),
  minimizeWindow: () => invoke<void>('minimize_window'),
  getGlobalShortcut: () => invoke<string>('get_global_shortcut'),
  setGlobalShortcut: (shortcut: string) =>
    invoke<string>('set_global_shortcut', { shortcut }),
  scanUrl: (url: string) => invoke<ScannedImage[]>('scan_url', { url }),
  scanHtml: (html: string, baseUrl?: string) =>
    invoke<ScannedImage[]>('scan_html', { html, baseUrl: baseUrl?.trim() || null }),
  importScannedImages: (items: ScannedImageImportRequest[]) =>
    invoke<BatchImportResult>('import_scanned_images', { items }),
  openExternalUrl: (url: string) =>
    invoke<void>('open_external_url', { url }),
  downloadUpdateAsset: (url: string, filename: string) =>
    invoke<string>('download_update_asset', { url, filename }),
  launchInstaller: (filePath: string) =>
    invoke<void>('launch_installer', { filePath })
};
