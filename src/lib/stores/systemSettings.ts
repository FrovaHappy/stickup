import { writable } from 'svelte/store';
import { systemService, type SystemService } from '$lib/services/system';
import { toast } from '$lib/stores/toast';

export const DEFAULT_GLOBAL_SHORTCUT = 'Super+Control+Period';

export const autostartEnabled = writable<boolean>(false);
export const minimizeToTrayEnabled = writable<boolean>(false);
export const globalShortcut = writable<string>(DEFAULT_GLOBAL_SHORTCUT);
export const systemSettingsLoading = writable<boolean>(true);

export function formatShortcutKeys(shortcut: string): string[] {
  if (!shortcut || !shortcut.trim()) return [];
  return shortcut.split('+').map((part) => {
    const p = part.trim();
    const lower = p.toLowerCase();
    if (lower === 'super' || lower === 'meta') return 'Win';
    if (lower === 'control' || lower === 'ctrl') return 'Ctrl';
    if (lower === 'alt') return 'Alt';
    if (lower === 'shift') return 'Shift';
    if (lower === 'period' || p === '.') return '.';
    if (lower === 'comma' || p === ',') return ',';
    if (lower === 'space') return 'Espacio';
    if (/^key[a-z0-9]$/i.test(p)) return p.slice(3).toUpperCase();
    if (/^digit[0-9]$/i.test(p)) return p.slice(5);
    return p;
  });
}

let activeService: SystemService = systemService;

export function setSystemServiceForTesting(mock: SystemService) {
  activeService = mock;
}

export function resetSystemService() {
  activeService = systemService;
}

export async function loadSystemSettings(
  service: SystemService = activeService
): Promise<{ autostart: boolean; minimizeToTray: boolean; globalShortcut: string }> {
  systemSettingsLoading.set(true);
  try {
    const [auto, tray, shortcut] = await Promise.all([
      service.isAutostart(),
      service.isMinimizeToTray(),
      service.getGlobalShortcut()
    ]);
    autostartEnabled.set(auto);
    minimizeToTrayEnabled.set(tray);
    const resolvedShortcut = shortcut || DEFAULT_GLOBAL_SHORTCUT;
    globalShortcut.set(resolvedShortcut);
    return { autostart: auto, minimizeToTray: tray, globalShortcut: resolvedShortcut };
  } catch (e) {
    console.error('Error cargando configuración del sistema:', e);
    return { autostart: false, minimizeToTray: false, globalShortcut: DEFAULT_GLOBAL_SHORTCUT };
  } finally {
    systemSettingsLoading.set(false);
  }
}

export async function setAutostartSetting(
  enable: boolean,
  service: SystemService = activeService
): Promise<boolean> {
  try {
    await service.setAutostart(enable);
    autostartEnabled.set(enable);
    if (enable) {
      toast.success('StickUp iniciará automáticamente con el sistema operativo');
    } else {
      toast.info('Inicio automático con el sistema desactivado');
    }
    return true;
  } catch (e) {
    console.error('Error modificando autostart:', e);
    toast.error('No se pudo modificar el inicio automático con el sistema');
    return false;
  }
}

export async function setMinimizeToTraySetting(
  enable: boolean,
  service: SystemService = activeService
): Promise<boolean> {
  try {
    await service.setMinimizeToTray(enable);
    minimizeToTrayEnabled.set(enable);
    if (enable) {
      toast.success('StickUp se minimizará en la bandeja al cerrar la ventana');
    } else {
      toast.info('Minimizar en la bandeja desactivado');
    }
    return true;
  } catch (e) {
    console.error('Error modificando minimizar a bandeja:', e);
    toast.error('No se pudo guardar la preferencia de bandeja');
    return false;
  }
}

export async function setGlobalShortcutSetting(
  shortcut: string,
  service: SystemService = activeService
): Promise<boolean> {
  try {
    const updated = await service.setGlobalShortcut(shortcut);
    globalShortcut.set(updated);
    toast.success('Atajo de teclado actualizado');
    return true;
  } catch (e) {
    console.error('Error modificando atajo global:', e);
    const msg = e instanceof Error ? e.message : String(e);
    toast.error(msg || 'No se pudo actualizar el atajo de teclado');
    return false;
  }
}

