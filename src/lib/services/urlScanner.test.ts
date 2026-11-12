import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  isDiscordUrl,
  isDiscordAnimatedUrl,
  isDirectImageUrl,
  cleanUrlInput,
  processUrlOrPath
} from './urlScanner';
import { api } from './api';

vi.mock('./api', () => ({
  api: {
    importPaths: vi.fn(),
    importFromUrl: vi.fn(),
    scanUrl: vi.fn()
  }
}));

describe('urlScanner service', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('isDiscordUrl', () => {
    it('detects discord cdn and media domains', () => {
      expect(isDiscordUrl('https://cdn.discordapp.com/emojis/123.webp')).toBe(true);
      expect(isDiscordUrl('https://media.discordapp.net/attachments/1/2/test.png')).toBe(true);
      expect(isDiscordUrl('https://example.com/image.png')).toBe(false);
    });
  });

  describe('isDiscordAnimatedUrl', () => {
    it('identifies animated query parameter or gif extension', () => {
      expect(
        isDiscordAnimatedUrl(
          'https://cdn.discordapp.com/emojis/1539337442191212695.webp?size=96&animated=true'
        )
      ).toBe(true);
      expect(isDiscordAnimatedUrl('https://cdn.discordapp.com/emojis/123.gif')).toBe(true);
      expect(isDiscordAnimatedUrl('https://cdn.discordapp.com/avatars/1/a_12345.webp')).toBe(true);
      expect(
        isDiscordAnimatedUrl('https://cdn.discordapp.com/emojis/123.webp?size=96')
      ).toBe(false);
      expect(isDiscordAnimatedUrl('https://example.com/test.gif')).toBe(false);
    });
  });

  describe('isDirectImageUrl', () => {
    it('recognizes standard image extensions and discord emojis', () => {
      expect(
        isDirectImageUrl(
          'https://cdn.discordapp.com/emojis/1539337442191212695.webp?size=96&animated=true'
        )
      ).toBe(true);
      expect(isDirectImageUrl('https://cdn.discordapp.com/emojis/12345')).toBe(true);
      expect(isDirectImageUrl('https://example.com/cat.png')).toBe(true);
      expect(isDirectImageUrl('https://example.com/cat.webp?size=64#frag')).toBe(true);
      expect(isDirectImageUrl('https://example.com/page.html')).toBe(false);
      expect(isDirectImageUrl('ftp://example.com/image.png')).toBe(false);
    });
  });

  describe('cleanUrlInput', () => {
    it('removes surrounding quotes and unescapes amp;', () => {
      expect(
        cleanUrlInput('"https://cdn.discordapp.com/emojis/123.webp?size=96&amp;animated=true"')
      ).toBe('https://cdn.discordapp.com/emojis/123.webp?size=96&animated=true');
      expect(cleanUrlInput(" 'https://example.com/test.png' ")).toBe('https://example.com/test.png');
    });
  });

  describe('processUrlOrPath', () => {
    it('handles direct image URLs via importFromUrl', async () => {
      const mockEmote = { id: 1, name: 'DiscordEmote1539337442191212695', format: 'gif' } as any;
      (api.importFromUrl as any).mockResolvedValue(mockEmote);

      const result = await processUrlOrPath(
        'https://cdn.discordapp.com/emojis/1539337442191212695.webp?size=96&animated=true'
      );

      expect(result.type).toBe('direct');
      expect(result.directEmote).toEqual(mockEmote);
      expect(api.importFromUrl).toHaveBeenCalledTimes(1);
    });

    it('falls back to scanUrl if importFromUrl fails', async () => {
      (api.importFromUrl as any).mockRejectedValue(new Error('Network error'));
      (api.scanUrl as any).mockResolvedValue([
        { url: 'https://example.com/test.png', suggested_name: 'Test' }
      ]);

      const result = await processUrlOrPath('https://example.com/test.png');

      expect(result.type).toBe('scanned');
      expect(result.scannedImages?.length).toBe(1);
      expect(api.scanUrl).toHaveBeenCalledTimes(1);
    });

    it('scans webpage URLs directly', async () => {
      (api.scanUrl as any).mockResolvedValue([
        { url: 'https://example.com/1.png', suggested_name: 'One' }
      ]);

      const result = await processUrlOrPath('https://example.com/gallery');

      expect(result.type).toBe('scanned');
      expect(result.scannedImages?.length).toBe(1);
      expect(api.importFromUrl).not.toHaveBeenCalled();
      expect(api.scanUrl).toHaveBeenCalledWith('https://example.com/gallery');
    });
  });
});

