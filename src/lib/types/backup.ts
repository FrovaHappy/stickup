export interface BackupInfo {
  file_name: string;
  path: string;
  size_bytes: number;
  emote_count: number;
  created_at: string;
}

export interface BackupResult {
  success: boolean;
  backup: BackupInfo;
  message: string;
}

export interface RestoreResult {
  success: boolean;
  restored_emote_count: number;
  safety_backup_path?: string | null;
  message: string;
}

