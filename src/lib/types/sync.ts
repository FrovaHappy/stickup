export type CloudProviderType = 'none' | 'mega' | 'googledrive' | 'onedrive' | 'localfolder';

export interface SyncConfig {
  provider: string;
  cloud_path: string | null;
  auto_sync: boolean;
  last_sync_at: number | null;
}

export interface SyncStatus {
  is_syncing: boolean;
  last_sync_at: number | null;
  provider: string | null;
  progress: string | null;
  error: string | null;
  uploaded_count: number;
  downloaded_count: number;
}

