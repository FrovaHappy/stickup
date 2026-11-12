import type {
  GitHubRelease,
  ParsedVersion,
  ReleaseAsset,
  UpdateCheckResult
} from '$lib/types/updater';

const SEMVER_REGEX =
  /^v?(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.-]+?)(?:\.(\d+))?)?$/;

/**
 * Parsea una cadena de versión semántica (ej. "0.2.0", "v1.2.3", "v0.2.0-dev.1").
 */
export function parseSemver(version: string): ParsedVersion | null {
  if (!version || typeof version !== 'string') return null;
  const trimmed = version.trim();
  const match = trimmed.match(SEMVER_REGEX);
  if (!match) return null;

  const major = parseInt(match[1], 10);
  const minor = parseInt(match[2], 10);
  const patch = parseInt(match[3], 10);
  const prereleaseTag = match[4] || null;
  const prereleaseNum = match[5] !== undefined ? parseInt(match[5], 10) : null;

  return {
    raw: trimmed,
    major,
    minor,
    patch,
    prereleaseTag,
    prereleaseNum
  };
}

/**
 * Compara dos versiones semánticas.
 * Retorna:
 *  1 si v1 > v2
 * -1 si v1 < v2
 *  0 si v1 == v2
 */
export function compareSemver(v1: string, v2: string): number {
  const p1 = parseSemver(v1);
  const p2 = parseSemver(v2);

  if (!p1 && !p2) return 0;
  if (!p1) return -1;
  if (!p2) return 1;

  if (p1.major !== p2.major) return p1.major > p2.major ? 1 : -1;
  if (p1.minor !== p2.minor) return p1.minor > p2.minor ? 1 : -1;
  if (p1.patch !== p2.patch) return p1.patch > p2.patch ? 1 : -1;

  // Si los números base coinciden:
  // Una versión estable (sin prerelease) es MAYOR que una prerelease
  if (!p1.prereleaseTag && p2.prereleaseTag) return 1;
  if (p1.prereleaseTag && !p2.prereleaseTag) return -1;

  // Ambas tienen prerelease
  if (p1.prereleaseTag && p2.prereleaseTag) {
    if (p1.prereleaseTag !== p2.prereleaseTag) {
      return p1.prereleaseTag.localeCompare(p2.prereleaseTag);
    }
    const n1 = p1.prereleaseNum ?? 0;
    const n2 = p2.prereleaseNum ?? 0;
    if (n1 !== n2) return n1 > n2 ? 1 : -1;
  }

  return 0;
}

/**
 * Detecta si una versión fue lanzada hace menos de `maxHours` (por defecto 24h).
 */
export function isHotRelease(
  publishedAtIso: string,
  maxHours = 24,
  nowMs = Date.now()
): { isHot: boolean; hours: number } {
  try {
    const publishedMs = Date.parse(publishedAtIso);
    if (isNaN(publishedMs)) return { isHot: false, hours: 999 };

    const diffMs = Math.max(0, nowMs - publishedMs);
    const hours = Math.floor(diffMs / (1000 * 60 * 60));
    return {
      isHot: diffMs < maxHours * 60 * 60 * 1000,
      hours
    };
  } catch {
    return { isHot: false, hours: 999 };
  }
}

/**
 * Encuentra el asset instalable más adecuado según la plataforma.
 */
export function findPlatformAsset(
  assets: ReleaseAsset[],
  platform: 'windows' | 'linux' | 'darwin' | 'other'
): ReleaseAsset | null {
  if (!assets || assets.length === 0) return null;

  if (platform === 'windows') {
    // Buscar instalador NSIS setup .exe primero
    const setupExe = assets.find(
      (a) => a.name.toLowerCase().endsWith('.exe') && a.name.toLowerCase().includes('setup')
    );
    if (setupExe) return setupExe;

    // Cualquier .exe
    return assets.find((a) => a.name.toLowerCase().endsWith('.exe')) || null;
  }

  if (platform === 'linux') {
    // Buscar AppImage primero
    const appImage = assets.find((a) => a.name.toLowerCase().endsWith('.appimage'));
    if (appImage) return appImage;

    // Buscar .deb
    const deb = assets.find((a) => a.name.toLowerCase().endsWith('.deb'));
    if (deb) return deb;

    return null;
  }

  return null;
}

/**
 * Busca el manifiesto de sumas SHA-256 adjunto a una release de GitHub.
 */
export function findChecksumAsset(assets: ReleaseAsset[]): ReleaseAsset | null {
  if (!assets || assets.length === 0) return null;
  return (
    assets.find((a) => a.name.toLowerCase() === 'sha256sums.txt') || null
  );
}

/**
 * Detecta la plataforma del sistema operativo en el navegador / WebView.
 */
export function detectPlatform(): 'windows' | 'linux' | 'darwin' | 'other' {
  if (typeof navigator === 'undefined') return 'windows';
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes('win')) return 'windows';
  if (ua.includes('linux')) return 'linux';
  if (ua.includes('mac')) return 'darwin';
  return 'other';
}

/**
 * Evalúa las releases de GitHub contra la versión local y devuelve el resultado de comprobación.
 */
export function checkUpdateFromReleases(
  currentVersion: string,
  releases: GitHubRelease[],
  options?: {
    includePrereleases?: boolean;
    platform?: 'windows' | 'linux' | 'darwin' | 'other';
    nowMs?: number;
  }
): UpdateCheckResult {
  const includePrereleases = options?.includePrereleases ?? false;
  const platform = options?.platform ?? detectPlatform();
  const nowMs = options?.nowMs ?? Date.now();

  const validReleases = (releases || []).filter((r) => {
    if (r.draft) return false;
    if (!includePrereleases && r.prerelease) return false;
    return true;
  });

  if (validReleases.length === 0) {
    return {
      hasUpdate: false,
      currentVersion,
      latestVersion: currentVersion,
      release: null,
      isHot: false,
      hoursSincePublished: 0,
      matchedAsset: null
    };
  }

  // Ordenar releases por versión semántica descendente
  const sorted = [...validReleases].sort((a, b) =>
    compareSemver(b.tag_name, a.tag_name)
  );

  const topRelease = sorted[0];
  const topCleanTag = topRelease.tag_name.replace(/^v/, '');
  const hasUpdate = compareSemver(topCleanTag, currentVersion) > 0;

  if (!hasUpdate) {
    return {
      hasUpdate: false,
      currentVersion,
      latestVersion: topCleanTag,
      release: null,
      isHot: false,
      hoursSincePublished: 0,
      matchedAsset: null
    };
  }

  const hotInfo = isHotRelease(topRelease.published_at, 24, nowMs);
  const matchedAsset = findPlatformAsset(topRelease.assets, platform);
  const checksumAsset = findChecksumAsset(topRelease.assets);

  return {
    hasUpdate: true,
    currentVersion,
    latestVersion: topCleanTag,
    release: topRelease,
    isHot: hotInfo.isHot,
    hoursSincePublished: hotInfo.hours,
    matchedAsset,
    checksumUrl: checksumAsset?.browser_download_url ?? null
  };
}

/**
 * Consulta la API pública de GitHub para obtener las releases de StickUp.
 */
export async function fetchGitHubReleases(
  repo = 'FrovaHappy/stickup'
): Promise<GitHubRelease[]> {
  const res = await fetch(`https://api.github.com/repos/${repo}/releases`, {
    headers: {
      Accept: 'application/vnd.github.v3+json',
      'User-Agent': 'StickUp-App'
    }
  });

  if (!res.ok) {
    throw new Error(`GitHub API error: ${res.status} ${res.statusText}`);
  }

  return (await res.json()) as GitHubRelease[];
}

