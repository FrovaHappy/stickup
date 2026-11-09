import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import {
  autoCheckUpdates,
  includePrereleases,
  preferredModal,
  isCheckingUpdates,
  availableUpdate,
  updateModalOpen,
  setAutoCheck,
  setIncludePrereleases,
  setPreferredModal,
  checkForUpdates,
  CHECK_INTERVAL_MS
} from './updater';
import type { GitHubRelease } from '$lib/types/updater';

const storage: Record<string, string> = {};
const mockLocalStorage = {
  getItem: (key: string) => storage[key] ?? null,
  setItem: (key: string, value: string) => {
    storage[key] = String(value);
  },
  removeItem: (key: string) => {
    delete storage[key];
  },
  clear: () => {
    for (const k in storage) {
      delete storage[k];
    }
  }
};

Object.defineProperty(globalThis, 'localStorage', {
  value: mockLocalStorage,
  writable: true,
  configurable: true
});

describe('Updater Store', () => {
  beforeEach(() => {
    mockLocalStorage.clear();
    autoCheckUpdates.set(true);
    includePrereleases.set(false);
    preferredModal.set('app');
    isCheckingUpdates.set(false);
    availableUpdate.set(null);
    updateModalOpen.set(false);
  });

  it('updates preferences and persists to localStorage', () => {
    setAutoCheck(false);
    expect(get(autoCheckUpdates)).toBe(false);
    expect(localStorage.getItem('stickup_update_auto_check')).toBe('false');

    setIncludePrereleases(true);
    expect(get(includePrereleases)).toBe(true);
    expect(localStorage.getItem('stickup_update_include_prerelease')).toBe('true');

    setPreferredModal('browser');
    expect(get(preferredModal)).toBe('browser');
    expect(localStorage.getItem('stickup_update_preferred_modal')).toBe('browser');
  });

  it('skips automatic check if less than 24 hours have passed', async () => {
    const recentCheck = Date.now() - 3600 * 1000; // 1 hour ago
    localStorage.setItem('stickup_update_last_check_ts', String(recentCheck));

    const mockFetcher = vi.fn().mockResolvedValue([]);

    await checkForUpdates(false, mockFetcher, '0.2.0');

    expect(mockFetcher).not.toHaveBeenCalled();
    expect(get(isCheckingUpdates)).toBe(false);
  });

  it('runs automatic check if 24 hours have passed', async () => {
    const oldCheck = Date.now() - (CHECK_INTERVAL_MS + 1000); // 24+ hours ago
    localStorage.setItem('stickup_update_last_check_ts', String(oldCheck));

    const mockReleases: GitHubRelease[] = [
      {
        id: 1,
        tag_name: 'v0.2.2',
        name: 'v0.2.2',
        body: 'Notes',
        published_at: new Date().toISOString(),
        html_url: 'https://github.com/...',
        prerelease: false,
        draft: false,
        assets: []
      }
    ];

    const mockFetcher = vi.fn().mockResolvedValue(mockReleases);

    await checkForUpdates(false, mockFetcher, '0.2.0');

    expect(mockFetcher).toHaveBeenCalledTimes(1);
    expect(get(availableUpdate)?.hasUpdate).toBe(true);
    expect(get(availableUpdate)?.latestVersion).toBe('0.2.2');
    expect(get(updateModalOpen)).toBe(true);
  });

  it('always runs manual check regardless of 24h margin', async () => {
    const recentCheck = Date.now() - 1000; // 1 second ago
    localStorage.setItem('stickup_update_last_check_ts', String(recentCheck));

    const mockFetcher = vi.fn().mockResolvedValue([]);

    await checkForUpdates(true, mockFetcher, '0.2.0');

    expect(mockFetcher).toHaveBeenCalledTimes(1);
  });
});
