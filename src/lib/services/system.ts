import { api } from './api';
import {
  enable as enableAutostartPlugin,
  disable as disableAutostartPlugin,
  isEnabled as isAutostartPluginEnabled
} from '@tauri-apps/plugin-autostart';

export interface SystemService {
  isAutostart(): Promise<boolean>;
  setAutostart(enable: boolean): Promise<boolean>;
  isMinimizeToTray(): Promise<boolean>;
  setMinimizeToTray(enabled: boolean): Promise<boolean>;
  isMinimizeOnCopy(): Promise<boolean>;
  setMinimizeOnCopy(enabled: boolean): Promise<boolean>;
  isForceGifLoop(): Promise<boolean>;
  setForceGifLoop(enabled: boolean): Promise<boolean>;
  minimizeWindow(): Promise<boolean>;
  getGlobalShortcut(): Promise<string>;
  setGlobalShortcut(shortcut: string): Promise<string>;
}

export const systemService: SystemService = {
  async isAutostart(): Promise<boolean> {
    try {
      return await isAutostartPluginEnabled();
    } catch {
      return false;
    }
  },

  async setAutostart(enable: boolean): Promise<boolean> {
    if (enable) {
      await enableAutostartPlugin();
    } else {
      await disableAutostartPlugin();
    }
    return true;
  },

  async isMinimizeToTray(): Promise<boolean> {
    try {
      return await api.getMinimizeToTray();
    } catch {
      return false;
    }
  },

  async setMinimizeToTray(enabled: boolean): Promise<boolean> {
    await api.setMinimizeToTray(enabled);
    return true;
  },

  async isMinimizeOnCopy(): Promise<boolean> {
    try {
      return await api.getMinimizeOnCopy();
    } catch {
      return false;
    }
  },

  async setMinimizeOnCopy(enabled: boolean): Promise<boolean> {
    await api.setMinimizeOnCopy(enabled);
    return true;
  },

  async isForceGifLoop(): Promise<boolean> {
    try {
      return await api.getForceGifLoop();
    } catch {
      return true;
    }
  },

  async setForceGifLoop(enabled: boolean): Promise<boolean> {
    await api.setForceGifLoop(enabled);
    return true;
  },

  async minimizeWindow(): Promise<boolean> {
    try {
      await api.minimizeWindow();
      return true;
    } catch {
      return false;
    }
  },

  async getGlobalShortcut(): Promise<string> {
    try {
      return await api.getGlobalShortcut();
    } catch {
      return 'Super+Control+Period';
    }
  },

  async setGlobalShortcut(shortcut: string): Promise<string> {
    return await api.setGlobalShortcut(shortcut);
  }
};

