import { writable, get } from 'svelte/store';
import { api } from '$lib/services/api';
import {
  fetchGitHubReleases,
  checkUpdateFromReleases,
  detectPlatform
} from '$lib/services/updater';
import type { GitHubRelease, UpdateCheckResult } from '$lib/types/updater';
import { toast } from '$lib/stores/toast';

export const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000; // 24 horas exactas

const AUTO_CHECK_KEY = 'stickup_update_auto_check';
const INCLUDE_PRERELEASE_KEY = 'stickup_update_include_prerelease';
const PREFERRED_MODAL_KEY = 'stickup_update_preferred_modal';
const LAST_CHECK_KEY = 'stickup_update_last_check_ts';

function getStoredBoolean(key: string, defaultVal: boolean): boolean {
  if (typeof localStorage === 'undefined') return defaultVal;
  try {
    const val = localStorage.getItem(key);
    if (val !== null) return val === 'true';
  } catch {
    // Ignore
  }
  return defaultVal;
}

function getStoredPreferredModal(): 'app' | 'browser' {
  if (typeof localStorage === 'undefined') return 'app';
  try {
    const val = localStorage.getItem(PREFERRED_MODAL_KEY);
    if (val === 'app' || val === 'browser') return val;
  } catch {
    // Ignore
  }
  return 'app';
}

export const currentAppVersion = writable<string>('0.2.0');
export const autoCheckUpdates = writable<boolean>(getStoredBoolean(AUTO_CHECK_KEY, true));
export const includePrereleases = writable<boolean>(
  getStoredBoolean(INCLUDE_PRERELEASE_KEY, false)
);
export const preferredModal = writable<'app' | 'browser'>(getStoredPreferredModal());

export const isCheckingUpdates = writable<boolean>(false);
export const availableUpdate = writable<UpdateCheckResult | null>(null);
export const updateModalOpen = writable<boolean>(false);

export const isDownloadingAsset = writable<boolean>(false);
export const downloadedFilePath = writable<string | null>(null);

export async function loadCurrentAppVersion(): Promise<string> {
  try {
    const ver = await api.getAppVersion();
    if (ver && typeof ver === 'string') {
      currentAppVersion.set(ver);
      return ver;
    }
  } catch {
    // Fallback en entorno web / tests
  }
  return get(currentAppVersion);
}

export function setAutoCheck(enabled: boolean) {
  autoCheckUpdates.set(enabled);
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(AUTO_CHECK_KEY, String(enabled));
    } catch {
      // Ignore
    }
  }
}

export function setIncludePrereleases(enabled: boolean) {
  includePrereleases.set(enabled);
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(INCLUDE_PRERELEASE_KEY, String(enabled));
    } catch {
      // Ignore
    }
  }
}

export function setPreferredModal(mode: 'app' | 'browser') {
  preferredModal.set(mode);
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(PREFERRED_MODAL_KEY, mode);
    } catch {
      // Ignore
    }
  }
}

export function openUpdateModal() {
  updateModalOpen.set(true);
}

export function closeUpdateModal() {
  updateModalOpen.set(false);
}

export async function checkForUpdates(
  manual = false,
  fetcher: () => Promise<GitHubRelease[]> = fetchGitHubReleases,
  injectedCurrentVersion?: string
): Promise<UpdateCheckResult | null> {
  // Verificación no manual: respetar auto-check y margen estricto de 24 horas
  if (!manual) {
    if (!get(autoCheckUpdates)) return null;

    if (typeof localStorage !== 'undefined') {
      try {
        const lastTs = parseInt(localStorage.getItem(LAST_CHECK_KEY) || '0', 10);
        if (Date.now() - lastTs < CHECK_INTERVAL_MS) {
          return null;
        }
      } catch {
        // Ignore
      }
    }
  }

  isCheckingUpdates.set(true);

  try {
    const version = injectedCurrentVersion || (await loadCurrentAppVersion());
    const releases = await fetcher();
    const result = checkUpdateFromReleases(version, releases, {
      includePrereleases: get(includePrereleases),
      platform: detectPlatform()
    });

    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(LAST_CHECK_KEY, String(Date.now()));
      } catch {
        // Ignore
      }
    }

    if (result.hasUpdate) {
      availableUpdate.set(result);
      updateModalOpen.set(true);
    } else {
      availableUpdate.set(null);
      if (manual) {
        toast.info(`StickUp está al día (v${version})`);
      }
    }

    return result;
  } catch (err) {
    if (manual) {
      const msg = err instanceof Error ? err.message : String(err);
      toast.error(`Error al comprobar actualizaciones: ${msg}`);
    }
    return null;
  } finally {
    isCheckingUpdates.set(false);
  }
}

/**
 * Descarga el instalador mediante el comando Tauri y ofrece ejecutarlo.
 */
export async function downloadAndInstallAsset(
  downloadUrl: string,
  filename: string
): Promise<string> {
  isDownloadingAsset.set(true);
  try {
    const savedPath = await api.downloadUpdateAsset(downloadUrl, filename);
    downloadedFilePath.set(savedPath);
    toast.success('Instalador descargado correctamente.');
    return savedPath;
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    toast.error(`Error al descargar: ${msg}`);
    throw err;
  } finally {
    isDownloadingAsset.set(false);
  }
}

/**
 * Abre el instalador descargado en disco para actualizar la app.
 */
export async function launchDownloadedInstaller(filePath: string): Promise<void> {
  try {
    await api.launchInstaller(filePath);
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    toast.error(`No se pudo iniciar el instalador: ${msg}`);
  }
}

/**
 * Abre una URL en el navegador externo predeterminado.
 */
export async function openInBrowser(url: string): Promise<void> {
  try {
    await api.openExternalUrl(url);
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    toast.error(`Error al abrir enlace: ${msg}`);
  }
}

