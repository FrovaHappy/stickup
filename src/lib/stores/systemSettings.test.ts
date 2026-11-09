import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  autostartEnabled,
  minimizeToTrayEnabled,
  minimizeOnCopyEnabled,
  forceGifLoopEnabled,
  globalShortcut,
  DEFAULT_GLOBAL_SHORTCUT,
  systemSettingsLoading,
  loadSystemSettings,
  setAutostartSetting,
  setMinimizeToTraySetting,
  setMinimizeOnCopySetting,
  setForceGifLoopSetting,
  triggerMinimizeWindow,
  setGlobalShortcutSetting,
  formatShortcutKeys,
  setSystemServiceForTesting,
  resetSystemService
} from './systemSettings';
import type { SystemService } from '$lib/services/system';

describe('systemSettings store', () => {
  let mockState = {
    autostart: false,
    minimizeToTray: false,
    minimizeOnCopy: false,
    forceGifLoop: true,
    globalShortcut: DEFAULT_GLOBAL_SHORTCUT,
    minimized: false
  };

  const mockService: SystemService = {
    async isAutostart() {
      return mockState.autostart;
    },
    async setAutostart(enable: boolean) {
      mockState.autostart = enable;
      return true;
    },
    async isMinimizeToTray() {
      return mockState.minimizeToTray;
    },
    async setMinimizeToTray(enable: boolean) {
      mockState.minimizeToTray = enable;
      return true;
    },
    async isMinimizeOnCopy() {
      return mockState.minimizeOnCopy;
    },
    async setMinimizeOnCopy(enable: boolean) {
      mockState.minimizeOnCopy = enable;
      return true;
    },
    async isForceGifLoop() {
      return mockState.forceGifLoop;
    },
    async setForceGifLoop(enable: boolean) {
      mockState.forceGifLoop = enable;
      return true;
    },
    async minimizeWindow() {
      mockState.minimized = true;
      return true;
    },
    async getGlobalShortcut() {
      return mockState.globalShortcut;
    },
    async setGlobalShortcut(shortcut: string) {
      mockState.globalShortcut = shortcut;
      return shortcut;
    }
  };

  beforeEach(() => {
    mockState = {
      autostart: false,
      minimizeToTray: false,
      minimizeOnCopy: false,
      forceGifLoop: true,
      globalShortcut: DEFAULT_GLOBAL_SHORTCUT,
      minimized: false
    };
    setSystemServiceForTesting(mockService);
    autostartEnabled.set(false);
    minimizeToTrayEnabled.set(false);
    minimizeOnCopyEnabled.set(false);
    forceGifLoopEnabled.set(true);
    globalShortcut.set(DEFAULT_GLOBAL_SHORTCUT);
    systemSettingsLoading.set(false);
  });

  it('loads system settings and updates stores', async () => {
    mockState.autostart = true;
    mockState.minimizeToTray = true;
    mockState.minimizeOnCopy = true;
    mockState.forceGifLoop = false;
    mockState.globalShortcut = 'Alt+Shift+Period';

    const res = await loadSystemSettings(mockService);
    expect(res).toEqual({
      autostart: true,
      minimizeToTray: true,
      minimizeOnCopy: true,
      forceGifLoop: false,
      globalShortcut: 'Alt+Shift+Period'
    });

    let autoVal: boolean | undefined;
    let trayVal: boolean | undefined;
    let copyVal: boolean | undefined;
    let forceGifVal: boolean | undefined;
    let shortcutVal: string | undefined;
    let loadingVal: boolean | undefined;

    autostartEnabled.subscribe((v) => (autoVal = v))();
    minimizeToTrayEnabled.subscribe((v) => (trayVal = v))();
    minimizeOnCopyEnabled.subscribe((v) => (copyVal = v))();
    forceGifLoopEnabled.subscribe((v) => (forceGifVal = v))();
    globalShortcut.subscribe((v) => (shortcutVal = v))();
    systemSettingsLoading.subscribe((v) => (loadingVal = v))();

    expect(autoVal).toBe(true);
    expect(trayVal).toBe(true);
    expect(copyVal).toBe(true);
    expect(forceGifVal).toBe(false);
    expect(shortcutVal).toBe('Alt+Shift+Period');
    expect(loadingVal).toBe(false);
  });

  it('toggles and updates minimizeOnCopy setting and triggers minimize', async () => {
    expect(mockState.minimizeOnCopy).toBe(false);
    const success = await setMinimizeOnCopySetting(true, mockService);
    expect(success).toBe(true);
    expect(mockState.minimizeOnCopy).toBe(true);

    let copyVal: boolean | undefined;
    minimizeOnCopyEnabled.subscribe((v) => (copyVal = v))();
    expect(copyVal).toBe(true);

    // Test minimizeWindow trigger
    expect(mockState.minimized).toBe(false);
    const minSuccess = await triggerMinimizeWindow(mockService);
    expect(minSuccess).toBe(true);
    expect(mockState.minimized).toBe(true);

    // Disable
    await setMinimizeOnCopySetting(false, mockService);
    expect(mockState.minimizeOnCopy).toBe(false);
    minimizeOnCopyEnabled.subscribe((v) => (copyVal = v))();
    expect(copyVal).toBe(false);
  });

  it('toggles and updates forceGifLoop setting', async () => {
    expect(mockState.forceGifLoop).toBe(true);
    const success = await setForceGifLoopSetting(false, mockService);
    expect(success).toBe(true);
    expect(mockState.forceGifLoop).toBe(false);

    let loopVal: boolean | undefined;
    forceGifLoopEnabled.subscribe((v) => (loopVal = v))();
    expect(loopVal).toBe(false);

    // Re-enable
    await setForceGifLoopSetting(true, mockService);
    expect(mockState.forceGifLoop).toBe(true);
    forceGifLoopEnabled.subscribe((v) => (loopVal = v))();
    expect(loopVal).toBe(true);
  });

  it('toggles and updates autostart setting', async () => {
    const success = await setAutostartSetting(true, mockService);
    expect(success).toBe(true);
    expect(mockState.autostart).toBe(true);

    let autoVal: boolean | undefined;
    autostartEnabled.subscribe((v) => (autoVal = v))();
    expect(autoVal).toBe(true);

    await setAutostartSetting(false, mockService);
    expect(mockState.autostart).toBe(false);
    autostartEnabled.subscribe((v) => (autoVal = v))();
    expect(autoVal).toBe(false);
  });

  it('toggles and updates minimize to tray setting', async () => {
    const success = await setMinimizeToTraySetting(true, mockService);
    expect(success).toBe(true);
    expect(mockState.minimizeToTray).toBe(true);

    let trayVal: boolean | undefined;
    minimizeToTrayEnabled.subscribe((v) => (trayVal = v))();
    expect(trayVal).toBe(true);

    await setMinimizeToTraySetting(false, mockService);
    expect(mockState.minimizeToTray).toBe(false);
    minimizeToTrayEnabled.subscribe((v) => (trayVal = v))();
    expect(trayVal).toBe(false);
  });

  it('updates global shortcut setting', async () => {
    const success = await setGlobalShortcutSetting('Super+Control+Period', mockService);
    expect(success).toBe(true);
    expect(mockState.globalShortcut).toBe('Super+Control+Period');

    let shortcutVal: string | undefined;
    globalShortcut.subscribe((v) => (shortcutVal = v))();
    expect(shortcutVal).toBe('Super+Control+Period');

    await setGlobalShortcutSetting('Alt+Shift+KeyX', mockService);
    expect(mockState.globalShortcut).toBe('Alt+Shift+KeyX');
    globalShortcut.subscribe((v) => (shortcutVal = v))();
    expect(shortcutVal).toBe('Alt+Shift+KeyX');
  });

  it('formats accelerator shortcut keys for display', () => {
    expect(formatShortcutKeys('Super+Control+Period')).toEqual(['Win', 'Ctrl', '.']);
    expect(formatShortcutKeys('Super+Ctrl+.')).toEqual(['Win', 'Ctrl', '.']);
    expect(formatShortcutKeys('Alt+Shift+KeyP')).toEqual(['Alt', 'Shift', 'P']);
    expect(formatShortcutKeys('Control+Space')).toEqual(['Ctrl', 'Espacio']);
    expect(formatShortcutKeys('')).toEqual([]);
  });

  it('handles service errors gracefully without updating store on failure', async () => {
    const failingService: SystemService = {
      async isAutostart() {
        throw new Error('fail');
      },
      async setAutostart() {
        throw new Error('cannot set autostart');
      },
      async isMinimizeToTray() {
        throw new Error('fail');
      },
      async setMinimizeToTray() {
        throw new Error('cannot set tray');
      },
      async isMinimizeOnCopy() {
        throw new Error('fail');
      },
      async setMinimizeOnCopy() {
        throw new Error('cannot set minimize on copy');
      },
      async isForceGifLoop() {
        throw new Error('fail');
      },
      async setForceGifLoop() {
        throw new Error('cannot set force gif loop');
      },
      async minimizeWindow() {
        throw new Error('cannot minimize window');
      },
      async getGlobalShortcut() {
        throw new Error('fail shortcut');
      },
      async setGlobalShortcut() {
        throw new Error('cannot set shortcut');
      }
    };

    const spyConsole = vi.spyOn(console, 'error').mockImplementation(() => {});

    const autoSuccess = await setAutostartSetting(true, failingService);
    expect(autoSuccess).toBe(false);

    const traySuccess = await setMinimizeToTraySetting(true, failingService);
    expect(traySuccess).toBe(false);

    const copySuccess = await setMinimizeOnCopySetting(true, failingService);
    expect(copySuccess).toBe(false);

    const gifSuccess = await setForceGifLoopSetting(true, failingService);
    expect(gifSuccess).toBe(false);

    const minSuccess = await triggerMinimizeWindow(failingService);
    expect(minSuccess).toBe(false);

    const shortcutSuccess = await setGlobalShortcutSetting('Super+Control+Period', failingService);
    expect(shortcutSuccess).toBe(false);

    spyConsole.mockRestore();
    resetSystemService();
  });
});

