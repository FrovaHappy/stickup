import { writable } from 'svelte/store';
import { systemService, type SystemService } from '$lib/services/system';
import { toast } from '$lib/stores/toast';

export const autostartEnabled = writable<boolean>(false);
export const minimizeToTrayEnabled = writable<boolean>(false);
export const systemSettingsLoading = writable<boolean>(true);

let activeService: SystemService = systemService;

export function setSystemServiceForTesting(mock: SystemService) {
  activeService = mock;
}

export function resetSystemService() {
  activeService = systemService;
}

export async function loadSystemSettings(
  service: SystemService = activeService
): Promise<{ autostart: boolean; minimizeToTray: boolean }> {
  systemSettingsLoading.set(true);
  try {
    const [auto, tray] = await Promise.all([
      service.isAutostart(),
      service.isMinimizeToTray()
    ]);
    autostartEnabled.set(auto);
    minimizeToTrayEnabled.set(tray);
    return { autostart: auto, minimizeToTray: tray };
  } catch (e) {
    console.error('Error cargando configuración del sistema:', e);
    return { autostart: false, minimizeToTray: false };
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

