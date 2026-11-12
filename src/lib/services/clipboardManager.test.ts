import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  shouldThrottlePaste,
  isInteractiveInputElement,
  handleClipboardPasteAction,
  pasteConfirmState,
  closePasteConfirmation,
  isAddModalOpen,
  openAddModal,
  closeAddModal
} from './clipboardManager';
import { api } from './api';
import { askBeforePaste, lastCopiedEmote } from '$lib/stores/preferences';
import { get } from 'svelte/store';

vi.mock('./api', () => ({
  api: {
    inspectClipboard: vi.fn(),
    importFromClipboard: vi.fn(),
    importRawBytes: vi.fn()
  }
}));

vi.mock('$lib/stores/emotes', () => ({
  loadEmotes: vi.fn()
}));

vi.mock('$lib/stores/toast', () => ({
  toast: {
    info: vi.fn(),
    error: vi.fn(),
    success: vi.fn()
  }
}));

describe('clipboardManager service', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    closePasteConfirmation();
    closeAddModal();
    askBeforePaste.set(false);
    lastCopiedEmote.set(null);
  });

  describe('modal and throttle utilities', () => {
    it('controls add modal open state', () => {
      expect(get(isAddModalOpen)).toBe(false);
      openAddModal();
      expect(get(isAddModalOpen)).toBe(true);
      closeAddModal();
      expect(get(isAddModalOpen)).toBe(false);
    });

    it('identifies interactive elements', () => {
      const input = { tagName: 'INPUT' } as any;
      const textarea = { tagName: 'TEXTAREA' } as any;
      const div = { tagName: 'DIV' } as any;
      const editableDiv = { tagName: 'DIV', isContentEditable: true } as any;

      expect(isInteractiveInputElement(input)).toBe(true);
      expect(isInteractiveInputElement(textarea)).toBe(true);
      expect(isInteractiveInputElement(editableDiv)).toBe(true);
      expect(isInteractiveInputElement(div)).toBe(false);
      expect(isInteractiveInputElement(null)).toBe(false);
    });

    it('throttles rapid paste calls', () => {
      const t1 = 10000;
      expect(shouldThrottlePaste(t1)).toBe(false);
      expect(shouldThrottlePaste(t1 + 200)).toBe(true);
      expect(shouldThrottlePaste(t1 + 1500)).toBe(false);
    });
  });

  describe('handleClipboardPasteAction', () => {
    it('aborts when clipboard has no image', async () => {
      (api.inspectClipboard as any).mockResolvedValue({
        has_image: false,
        preview_data_url: null
      });

      const handled = await handleClipboardPasteAction();
      expect(handled).toBe(false);
      expect(api.importFromClipboard).not.toHaveBeenCalled();
    });

    it('imports directly when askBeforePaste is false', async () => {
      (api.inspectClipboard as any).mockResolvedValue({
        has_image: true,
        preview_data_url: 'https://cdn.discordapp.com/emojis/1539337442191212695.webp?size=96&animated=true',
        suggested_name: 'DiscordEmote1539337442191212695',
        is_duplicate: false
      });
      (api.importFromClipboard as any).mockResolvedValue({
        id: 1,
        name: 'DiscordEmote1539337442191212695',
        format: 'gif'
      });

      const onSucc = vi.fn();
      const handled = await handleClipboardPasteAction({ onSuccess: onSucc });

      expect(handled).toBe(true);
      expect(api.importFromClipboard).toHaveBeenCalledWith('DiscordEmote1539337442191212695');
      expect(onSucc).toHaveBeenCalledWith('DiscordEmote1539337442191212695', 'GIF');
    });

    it('opens confirmation popup when askBeforePaste is true', async () => {
      askBeforePaste.set(true);
      (api.inspectClipboard as any).mockResolvedValue({
        has_image: true,
        preview_data_url: 'https://cdn.discordapp.com/emojis/123.webp',
        suggested_name: 'DiscordEmote123',
        is_duplicate: false
      });

      const handled = await handleClipboardPasteAction();
      expect(handled).toBe(true);
      expect(api.importFromClipboard).not.toHaveBeenCalled();

      const popup = get(pasteConfirmState);
      expect(popup.open).toBe(true);
      expect(popup.name).toBe('DiscordEmote123');
      expect(popup.previewUrl).toBe('https://cdn.discordapp.com/emojis/123.webp');
    });

    it('blocks duplicate emotes', async () => {
      (api.inspectClipboard as any).mockResolvedValue({
        has_image: true,
        is_duplicate: true,
        duplicate_name: 'ExistingEmote'
      });

      const handled = await handleClipboardPasteAction();
      expect(handled).toBe(false);
      expect(api.importFromClipboard).not.toHaveBeenCalled();
    });
  });
});
