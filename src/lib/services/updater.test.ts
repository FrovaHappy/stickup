import { describe, it, expect } from 'vitest';
import {
  parseSemver,
  compareSemver,
  isHotRelease,
  findPlatformAsset,
  checkUpdateFromReleases
} from './updater';
import type { GitHubRelease, ReleaseAsset } from '$lib/types/updater';

describe('Updater Service - SemVer Parser & Comparator', () => {
  it('parses standard semver versions', () => {
    expect(parseSemver('0.2.0')).toEqual({
      raw: '0.2.0',
      major: 0,
      minor: 2,
      patch: 0,
      prereleaseTag: null,
      prereleaseNum: null
    });

    expect(parseSemver('v1.14.9')).toEqual({
      raw: 'v1.14.9',
      major: 1,
      minor: 14,
      patch: 9,
      prereleaseTag: null,
      prereleaseNum: null
    });
  });

  it('parses prerelease versions with dev/beta tags', () => {
    expect(parseSemver('v0.2.0-dev.3')).toEqual({
      raw: 'v0.2.0-dev.3',
      major: 0,
      minor: 2,
      patch: 0,
      prereleaseTag: 'dev',
      prereleaseNum: 3
    });

    expect(parseSemver('1.0.0-beta.1')).toEqual({
      raw: '1.0.0-beta.1',
      major: 1,
      minor: 0,
      patch: 0,
      prereleaseTag: 'beta',
      prereleaseNum: 1
    });
  });

  it('returns null for invalid versions', () => {
    expect(parseSemver('invalid')).toBeNull();
    expect(parseSemver('')).toBeNull();
  });

  it('compares versions correctly', () => {
    // Normal upgrades
    expect(compareSemver('0.2.1', '0.2.0')).toBe(1);
    expect(compareSemver('0.2.0', '0.2.1')).toBe(-1);
    expect(compareSemver('0.2.0', '0.2.0')).toBe(0);
    expect(compareSemver('1.0.0', '0.9.9')).toBe(1);

    // Stable vs Prerelease: stable is higher for same base
    expect(compareSemver('0.2.0', '0.2.0-dev.1')).toBe(1);
    expect(compareSemver('0.2.0-dev.1', '0.2.0')).toBe(-1);

    // Between prereleases of same base
    expect(compareSemver('0.2.0-dev.2', '0.2.0-dev.1')).toBe(1);
    expect(compareSemver('0.2.0-dev.1', '0.2.0-dev.2')).toBe(-1);
    expect(compareSemver('0.2.0-dev.1', '0.2.0-dev.1')).toBe(0);

    // Higher base with prerelease vs lower stable base
    expect(compareSemver('0.3.0-dev.1', '0.2.0')).toBe(1);
  });
});

describe('Updater Service - Hot Release Detection', () => {
  const BASE_TIME = 1700000000000;

  it('marks a release as hot if published under 24 hours ago', () => {
    const publishedAt = new Date(BASE_TIME - 3 * 3600 * 1000).toISOString(); // 3 hours ago
    const result = isHotRelease(publishedAt, 24, BASE_TIME);

    expect(result.isHot).toBe(true);
    expect(result.hours).toBe(3);
  });

  it('does not mark a release as hot if published 25 hours ago', () => {
    const publishedAt = new Date(BASE_TIME - 25 * 3600 * 1000).toISOString(); // 25 hours ago
    const result = isHotRelease(publishedAt, 24, BASE_TIME);

    expect(result.isHot).toBe(false);
    expect(result.hours).toBe(25);
  });

  it('handles edge case at exactly 24 hours', () => {
    const publishedAt = new Date(BASE_TIME - 24 * 3600 * 1000).toISOString();
    const result = isHotRelease(publishedAt, 24, BASE_TIME);

    expect(result.isHot).toBe(false);
    expect(result.hours).toBe(24);
  });
});

describe('Updater Service - Asset Matching by Platform', () => {
  const mockAssets: ReleaseAsset[] = [
    {
      id: 1,
      name: 'StickUp_0.2.1_x64-setup.exe',
      browser_download_url: 'https://github.com/.../setup.exe',
      size: 15000000,
      content_type: 'application/x-msdownload'
    },
    {
      id: 2,
      name: 'stickup_0.2.1_amd64.AppImage',
      browser_download_url: 'https://github.com/.../app.AppImage',
      size: 70000000,
      content_type: 'application/octet-stream'
    },
    {
      id: 3,
      name: 'stickup_0.2.1_amd64.deb',
      browser_download_url: 'https://github.com/.../app.deb',
      size: 12000000,
      content_type: 'application/vnd.debian.binary-package'
    }
  ];

  it('matches Windows installer (.exe)', () => {
    const asset = findPlatformAsset(mockAssets, 'windows');
    expect(asset).not.toBeNull();
    expect(asset?.name).toBe('StickUp_0.2.1_x64-setup.exe');
  });

  it('matches Linux AppImage installer', () => {
    const asset = findPlatformAsset(mockAssets, 'linux');
    expect(asset).not.toBeNull();
    expect(asset?.name).toBe('stickup_0.2.1_amd64.AppImage');
  });

  it('falls back to deb if AppImage is not found on Linux', () => {
    const assetsWithoutAppImage = mockAssets.filter((a) => !a.name.endsWith('.AppImage'));
    const asset = findPlatformAsset(assetsWithoutAppImage, 'linux');
    expect(asset).not.toBeNull();
    expect(asset?.name).toBe('stickup_0.2.1_amd64.deb');
  });
});

describe('Updater Service - checkUpdateFromReleases', () => {
  const mockReleases: GitHubRelease[] = [
    {
      id: 101,
      tag_name: 'v0.2.1',
      name: 'v0.2.1 Release',
      body: '### Added\n- New version system',
      published_at: new Date(1700000000000 - 5 * 3600 * 1000).toISOString(),
      html_url: 'https://github.com/FrovaHappy/stickup/releases/tag/v0.2.1',
      prerelease: false,
      draft: false,
      assets: [
        {
          id: 1,
          name: 'StickUp_0.2.1_x64-setup.exe',
          browser_download_url: 'https://github.com/download.exe',
          size: 1000,
          content_type: 'application/octet-stream'
        }
      ]
    },
    {
      id: 102,
      tag_name: 'v0.3.0-dev.1',
      name: 'v0.3.0-dev.1',
      body: 'Dev testing',
      published_at: new Date(1700000000000 - 2 * 3600 * 1000).toISOString(),
      html_url: 'https://github.com/FrovaHappy/stickup/releases/tag/v0.3.0-dev.1',
      prerelease: true,
      draft: false,
      assets: []
    }
  ];

  it('detects available update for older version', () => {
    const result = checkUpdateFromReleases('0.2.0', mockReleases, {
      includePrereleases: false,
      platform: 'windows',
      nowMs: 1700000000000
    });

    expect(result.hasUpdate).toBe(true);
    expect(result.latestVersion).toBe('0.2.1');
    expect(result.isHot).toBe(true);
    expect(result.hoursSincePublished).toBe(5);
    expect(result.matchedAsset?.name).toBe('StickUp_0.2.1_x64-setup.exe');
  });

  it('ignores prerelease by default', () => {
    const result = checkUpdateFromReleases('0.2.1', mockReleases, {
      includePrereleases: false,
      platform: 'windows'
    });

    expect(result.hasUpdate).toBe(false);
  });

  it('considers prerelease when includePrereleases is true', () => {
    const result = checkUpdateFromReleases('0.2.1', mockReleases, {
      includePrereleases: true,
      platform: 'windows'
    });

    expect(result.hasUpdate).toBe(true);
    expect(result.latestVersion).toBe('0.3.0-dev.1');
  });

  it('returns hasUpdate: false when currently on latest version', () => {
    const result = checkUpdateFromReleases('0.2.1', mockReleases, {
      includePrereleases: false
    });

    expect(result.hasUpdate).toBe(false);
  });
});

