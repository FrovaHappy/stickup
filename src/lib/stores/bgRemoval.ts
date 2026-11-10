import { writable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { api } from '$lib/services/api';
import type { BgModelStatus, BgDownloadProgress, ModelPresetInfo } from '$lib/types/bgRemoval';
import { toast } from '$lib/stores/toast';

export const bgModelStatus = writable<BgModelStatus | null>(null);
export const modelPresets = writable<ModelPresetInfo[]>([]);
export const activeModelId = writable<string>('birefnet_lite');
export const isDownloadingModel = writable<boolean>(false);
export const downloadProgress = writable<BgDownloadProgress | null>(null);
export const isProcessingBg = writable<boolean>(false);
export const idleTimeoutSecs = writable<number>(60);
export const isModelLoadedInRam = writable<boolean>(false);

let unlistenProgress: UnlistenFn | null = null;

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

export async function checkBgModelStatus(): Promise<BgModelStatus | null> {
  try {
    const status = await api.getBgModelStatus();
    bgModelStatus.set(status);
    if (status?.active_model_id) {
      activeModelId.set(status.active_model_id);
    }
    return status;
  } catch (err) {
    console.error('Error al consultar estado del modelo de IA:', err);
    return null;
  }
}

export async function loadModelPresets(): Promise<ModelPresetInfo[]> {
  try {
    const presets = await api.getBgModelPresets();
    modelPresets.set(presets);
    const active = presets.find((p) => p.is_active);
    if (active) {
      activeModelId.set(active.id);
    }
    return presets;
  } catch (err) {
    console.error('Error al cargar lista de presets de modelos:', err);
    return [];
  }
}

export async function setActiveModel(modelId: string): Promise<boolean> {
  try {
    const updatedStatus = await api.setActiveBgModel(modelId);
    bgModelStatus.set(updatedStatus);
    activeModelId.set(modelId);
    await loadModelPresets();
    toast.success('Modelo de IA activo actualizado.');
    return true;
  } catch (err) {
    const msg = typeof err === 'string' ? err : 'Error al cambiar de modelo de IA';
    toast.error(msg);
    return false;
  }
}

export async function downloadBgModel(modelId?: string): Promise<boolean> {
  try {
    isDownloadingModel.set(true);
    downloadProgress.set({
      downloaded_bytes: 0,
      total_bytes: null,
      percentage: 0,
      done: false
    });

    if (!unlistenProgress) {
      unlistenProgress = await listen<BgDownloadProgress>(
        'bg-model-download-progress',
        (event) => {
          downloadProgress.set(event.payload);
        }
      );
    }

    const updated = await api.downloadBgModel(modelId);
    bgModelStatus.set(updated);
    await loadModelPresets();
    toast.success('Modelo descargado con éxito.');
    return true;
  } catch (err) {
    const msg = typeof err === 'string' ? err : 'Error al descargar el modelo de recorte.';
    toast.error(msg);
    return false;
  } finally {
    isDownloadingModel.set(false);
    downloadProgress.set(null);
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
  }
}

export async function deleteBgModel(modelId?: string): Promise<boolean> {
  try {
    const updated = await api.deleteBgModel(modelId);
    bgModelStatus.set(updated);
    await loadModelPresets();
    isModelLoadedInRam.set(false);
    toast.info('Modelo eliminado para liberar espacio.');
    return true;
  } catch (err) {
    toast.error(`Error al eliminar modelo: ${err}`);
    return false;
  }
}

export async function loadIdleTimeout(): Promise<number> {
  try {
    const timeout = await api.getBgModelIdleTimeout();
    idleTimeoutSecs.set(timeout);
    return timeout;
  } catch (err) {
    console.error('Error al cargar tiempo de espera de modelo IA:', err);
    return 60;
  }
}

export async function setIdleTimeout(secs: number): Promise<boolean> {
  try {
    await api.setBgModelIdleTimeout(secs);
    idleTimeoutSecs.set(secs);
    await checkModelLoadedInRam();
    toast.success('Tiempo de inactividad de memoria actualizado.');
    return true;
  } catch (err) {
    const msg = typeof err === 'string' ? err : 'Error al guardar tiempo de inactividad';
    toast.error(msg);
    return false;
  }
}

export async function checkModelLoadedInRam(): Promise<boolean> {
  try {
    const loaded = await api.isBgModelLoadedInRam();
    isModelLoadedInRam.set(loaded);
    return loaded;
  } catch (err) {
    console.error('Error al comprobar carga de modelo en RAM:', err);
    return false;
  }
}

export async function unloadModelSession(): Promise<boolean> {
  try {
    const wasLoaded = await api.unloadBgModelSession();
    isModelLoadedInRam.set(false);
    if (wasLoaded) {
      toast.success('Memoria del modelo liberada correctamente.');
    } else {
      toast.info('El modelo no estaba ocupando memoria RAM.');
    }
    return true;
  } catch (err) {
    const msg = typeof err === 'string' ? err : 'Error al descargar modelo de memoria';
    toast.error(msg);
    return false;
  }
}
