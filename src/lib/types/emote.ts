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
  copy_count?: number;
  interaction_count?: number;
  last_used_at?: string | null;
}

export interface SizePreset {
  label: string;
  value: string;
  width?: number;
  height?: number;
}

export interface ResyncStats {
  scanned_files: number;
  converted_to_webp: number;
  png_converted: number;
  jpg_converted: number;
  added_to_db: number;
  removed_missing_from_db: number;
  bytes_before: number;
  bytes_after: number;
  saved_bytes: number;
}
