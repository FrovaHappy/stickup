import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  formatBytes,
  bgModelStatus,
  modelPresets,
  activeModelId,
  checkBgModelStatus,
  loadModelPresets,
  setActiveModel,
  downloadBgModel,
  deleteBgModel,
  idleTimeoutSecs,
  isModelLoadedInRam,
  loadIdleTimeout,
  setIdleTimeout,
  checkModelLoadedInRam,
  unloadModelSession
} from './bgRemoval';
import { api } from '$lib/services/api';
import { get } from 'svelte/store';

vi.mock('$lib/services/api', () => ({
  api: {
    getBgModelStatus: vi.fn(),
    getBgModelPresets: vi.fn(),
    setActiveBgModel: vi.fn(),
    downloadBgModel: vi.fn(),
    deleteBgModel: vi.fn(),
    getBgModelIdleTimeout: vi.fn(),
    setBgModelIdleTimeout: vi.fn(),
    unloadBgModelSession: vi.fn(),
    isBgModelLoadedInRam: vi.fn()
  }
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {})
}));

vi.mock('$lib/stores/toast', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn()
  }
}));

describe('bgRemoval store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    bgModelStatus.set(null);
    modelPresets.set([]);
    activeModelId.set('birefnet_lite');
  });

  describe('formatBytes', () => {
    it('formatea 0 bytes', () => {
      expect(formatBytes(0)).toBe('0 B');
    });

    it('formatea kilobytes', () => {
      expect(formatBytes(1024)).toBe('1.0 KB');
    });

    it('formatea megabytes', () => {
      expect(formatBytes(43 * 1024 * 1024)).toBe('43.0 MB');
    });
  });

  describe('checkBgModelStatus', () => {
    it('obtiene el estado y actualiza el store', async () => {
      const mockStatus = {
        is_downloaded: true,
        file_size_bytes: 45000000,
        model_path: 'C:/models/birefnet-lite.onnx',
        active_model_id: 'birefnet_lite'
      };
      vi.mocked(api.getBgModelStatus).mockResolvedValueOnce(mockStatus);

      const res = await checkBgModelStatus();
      expect(res).toEqual(mockStatus);
      expect(get(bgModelStatus)).toEqual(mockStatus);
      expect(get(activeModelId)).toBe('birefnet_lite');
    });
  });

  describe('loadModelPresets', () => {
    it('carga lista de presets y setea el activo', async () => {
      const mockPresets = [
        {
          id: 'birefnet_lite',
          name: 'BiRefNet Lite',
          description: 'SOTA',
          filename: 'birefnet-lite.onnx',
          download_url: 'url',
          approx_size_mb: 224,
          is_downloaded: false,
          file_size_bytes: 0,
          is_active: true
        }
      ];
      vi.mocked(api.getBgModelPresets).mockResolvedValueOnce(mockPresets);

      const presets = await loadModelPresets();
      expect(presets).toEqual(mockPresets);
      expect(get(modelPresets)).toEqual(mockPresets);
      expect(get(activeModelId)).toBe('birefnet_lite');
    });
  });

  describe('setActiveModel', () => {
    it('actualiza el modelo activo', async () => {
      const mockStatus = {
        is_downloaded: true,
        file_size_bytes: 180000000,
        model_path: 'C:/models/rmbg-1.4.onnx',
        active_model_id: 'rmbg14_full'
      };
      vi.mocked(api.setActiveBgModel).mockResolvedValueOnce(mockStatus);
      vi.mocked(api.getBgModelPresets).mockResolvedValueOnce([]);

      const ok = await setActiveModel('rmbg14_full');
      expect(ok).toBe(true);
      expect(get(activeModelId)).toBe('rmbg14_full');
    });
  });

  describe('downloadBgModel', () => {
    it('inicia descarga y actualiza store si tiene éxito', async () => {
      const mockStatus = {
        is_downloaded: true,
        file_size_bytes: 45000000,
        model_path: 'C:/models/birefnet-lite.onnx',
        active_model_id: 'birefnet_lite'
      };
      vi.mocked(api.downloadBgModel).mockResolvedValueOnce(mockStatus);
      vi.mocked(api.getBgModelPresets).mockResolvedValueOnce([]);

      const success = await downloadBgModel();
      expect(success).toBe(true);
      expect(get(bgModelStatus)).toEqual(mockStatus);
    });
  });

  describe('deleteBgModel', () => {
    it('elimina modelo y actualiza store', async () => {
      const mockStatus = {
        is_downloaded: false,
        file_size_bytes: 0,
        model_path: 'C:/models/birefnet-lite.onnx',
        active_model_id: 'birefnet_lite'
      };
      vi.mocked(api.deleteBgModel).mockResolvedValueOnce(mockStatus);
      vi.mocked(api.getBgModelPresets).mockResolvedValueOnce([]);

      const success = await deleteBgModel();
      expect(success).toBe(true);
      expect(get(bgModelStatus)).toEqual(mockStatus);
    });
  });

  describe('idleTimeout y memoria RAM', () => {
    it('carga timeout desde la api', async () => {
      vi.mocked(api.getBgModelIdleTimeout).mockResolvedValueOnce(60);
      const res = await loadIdleTimeout();
      expect(res).toBe(60);
      expect(get(idleTimeoutSecs)).toBe(60);
    });

    it('establece nuevo timeout', async () => {
      vi.mocked(api.setBgModelIdleTimeout).mockResolvedValueOnce(undefined);
      vi.mocked(api.isBgModelLoadedInRam).mockResolvedValueOnce(false);
      const ok = await setIdleTimeout(0);
      expect(ok).toBe(true);
      expect(get(idleTimeoutSecs)).toBe(0);
    });

    it('descarga modelo de memoria RAM', async () => {
      vi.mocked(api.unloadBgModelSession).mockResolvedValueOnce(true);
      isModelLoadedInRam.set(true);
      const ok = await unloadModelSession();
      expect(ok).toBe(true);
      expect(get(isModelLoadedInRam)).toBe(false);
    });
  });
});
