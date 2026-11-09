import type { Emote } from './emote';

export interface ScannedImage {
  url: string;
  alt?: string | null;
  suggested_name: string;
  format_hint?: string | null;
}

export interface RawScannedItem {
  url: string;
  alt?: string | null;
}

export interface ScannedImageImportRequest {
  url: string;
  name?: string | null;
}

export interface BatchImportResult {
  imported: Emote[];
  failed_count: number;
  errors: string[];
}

