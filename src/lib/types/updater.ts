export interface ReleaseAsset {
  id: number;
  name: string;
  browser_download_url: string;
  size: number;
  content_type: string;
}

export interface GitHubRelease {
  id: number;
  tag_name: string;
  name: string | null;
  body: string | null;
  published_at: string;
  html_url: string;
  prerelease: boolean;
  draft: boolean;
  assets: ReleaseAsset[];
}

export interface ParsedVersion {
  raw: string;
  major: number;
  minor: number;
  patch: number;
  prereleaseTag: string | null;
  prereleaseNum: number | null;
}

export interface UpdateCheckResult {
  hasUpdate: boolean;
  currentVersion: string;
  latestVersion: string;
  release: GitHubRelease | null;
  isHot: boolean;
  hoursSincePublished: number;
  matchedAsset: ReleaseAsset | null;
}

export interface UpdatePreferences {
  autoCheck: boolean;
  includePrereleases: boolean;
  preferredModal: 'app' | 'browser';
  lastCheckTimestamp: number;
}

export interface DownloadProgress {
  downloadedBytes: number;
  totalBytes: number;
  percentage: number;
  finished: boolean;
  savedPath?: string;
  error?: string;
}

