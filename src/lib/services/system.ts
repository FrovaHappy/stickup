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
  }
};

