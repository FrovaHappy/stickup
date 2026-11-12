export type GridSize = 'small' | 'medium' | 'large';

export type SortOption =
  | 'name-asc'
  | 'name-desc'
  | 'date-desc'
  | 'date-asc'
  | 'copies-desc';

export interface SortOptionItem {
  id: SortOption;
  label: string;
}

export type EmoteFormatFilter = 'all' | 'static' | 'gif';

export interface FormatFilterItem {
  id: EmoteFormatFilter;
  label: string;
  desc?: string;
}


