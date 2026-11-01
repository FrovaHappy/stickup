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

export interface SizePreset {
  label: string;
  value: string;
  width?: number;
  height?: number;
}

