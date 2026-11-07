import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  autostartEnabled,
  minimizeToTrayEnabled,
  systemSettingsLoading,
  loadSystemSettings,
  setAutostartSetting,
  setMinimizeToTraySetting,
  setSystemServiceForTesting,
  resetSystemService
} from './systemSettings';
import type { SystemService } from '$lib/services/system';

describe('systemSettings store', () => {
  let mockState = {
    autostart: false,
    minimizeToTray: false
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
    }
  };

  beforeEach(() => {
    mockState = {
      autostart: false,
      minimizeToTray: false
    };
    setSystemServiceForTesting(mockService);
    autostartEnabled.set(false);
    minimizeToTrayEnabled.set(false);
    systemSettingsLoading.set(false);
  });

  it('loads system settings and updates stores', async () => {
    mockState.autostart = true;
    mockState.minimizeToTray = true;

    const res = await loadSystemSettings(mockService);
    expect(res).toEqual({ autostart: true, minimizeToTray: true });

    let autoVal: boolean | undefined;
    let trayVal: boolean | undefined;
    let loadingVal: boolean | undefined;

    autostartEnabled.subscribe((v) => (autoVal = v))();
    minimizeToTrayEnabled.subscribe((v) => (trayVal = v))();
    systemSettingsLoading.subscribe((v) => (loadingVal = v))();

    expect(autoVal).toBe(true);
    expect(trayVal).toBe(true);
    expect(loadingVal).toBe(false);
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
      }
    };

    const spyConsole = vi.spyOn(console, 'error').mockImplementation(() => {});

    const autoSuccess = await setAutostartSetting(true, failingService);
    expect(autoSuccess).toBe(false);

    const traySuccess = await setMinimizeToTraySetting(true, failingService);
    expect(traySuccess).toBe(false);

    spyConsole.mockRestore();
    resetSystemService();
  });
});

