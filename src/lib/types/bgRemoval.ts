export interface ModelPresetInfo {
  id: string;
  name: string;
  description: string;
  filename: string;
  download_url: string;
  approx_size_mb: number;
  is_downloaded: boolean;
  file_size_bytes: number;
  is_active: boolean;
}

export interface BgModelStatus {
  is_downloaded: boolean;
  file_size_bytes: number;
  model_path: string;
  active_model_id: string;
}

export interface BgDownloadProgress {
  downloaded_bytes: number;
  total_bytes: number | null;
  percentage: number;
  done: boolean;
}

export interface BgGifProgress {
  current_frame: number;
  total_frames: number;
  percentage: number;
}
