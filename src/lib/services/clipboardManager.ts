import { writable, get } from 'svelte/store';
import { api } from './api';
import { toast } from '$lib/stores/toast';
import { askBeforePaste, lastCopiedEmote } from '$lib/stores/preferences';
import { loadEmotes } from '$lib/stores/emotes';
import { toPascalCase } from '$lib/utils/format';
import type { Emote } from '$lib/types/emote';

export const isAddModalOpen = writable<boolean>(false);

export function openAddModal() {
  isAddModalOpen.set(true);
}

export function closeAddModal() {
  isAddModalOpen.set(false);
}

export interface PasteConfirmPopupState {
  open: boolean;
  previewUrl: string;
  name: string;
  isBusy: boolean;
  onSave: (dontAskAgain: boolean) => Promise<void>;
  onCancel: () => void;
}

export const pasteConfirmState = writable<PasteConfirmPopupState>({
  open: false,
  previewUrl: '',
  name: '',
  isBusy: false,
  onSave: async () => {},
  onCancel: () => {}
});

export function closePasteConfirmation() {
  pasteConfirmState.set({
    open: false,
    previewUrl: '',
    name: '',
    isBusy: false,
    onSave: async () => {},
    onCancel: () => {}
  });
}

export const PASTE_COOLDOWN_MS = 1000;
let lastPasteTimestamp = 0;

export function shouldThrottlePaste(now = Date.now()): boolean {
  if (now - lastPasteTimestamp < PASTE_COOLDOWN_MS) {
    return true;
  }
  lastPasteTimestamp = now;
  return false;
}

export function isInteractiveInputElement(target: EventTarget | null): boolean {
  if (!target) return false;
  const el = target as Partial<HTMLElement>;
  const tag = (el.tagName || '').toUpperCase();
  return tag === 'INPUT' || tag === 'TEXTAREA' || Boolean(el.isContentEditable);
}

export interface ClipboardImportCallbacks {
  onSuccess?: (emoteName: string, format?: string) => void;
  onError?: (error: string) => void;
  onStatus?: (status: string) => void;
}

/**
 * Ejecuta la importación directa desde el portapapeles nativo y refresca los emotes.
 */
export async function executeDirectClipboardImport(
  nameToUse?: string,
  callbacks?: ClipboardImportCallbacks
): Promise<Emote[]> {
  callbacks?.onStatus?.('Guardando emote...');
  try {
    const result = await api.importFromClipboard(nameToUse);
    await loadEmotes();

    const list: Emote[] = Array.isArray(result)
      ? result
      : result && typeof result === 'object' && ('id' in result || 'name' in result)
      ? [result as Emote]
      : [];

    if (list.length === 1) {
      const item = list[0];
      const fmt = item?.format ? item.format.toUpperCase() : undefined;
      const name = item?.name || nameToUse || 'emote';
      callbacks?.onSuccess?.(name, fmt);
    } else if (list.length > 1) {
      callbacks?.onSuccess?.(`${list.length} emotes`);
    }
    return list;
  } catch (err: any) {
    const msg = err?.message || String(err);
    if (msg.includes('ya existe')) {
      toast.info(msg);
    } else {
      callbacks?.onError?.(msg);
      toast.error(`Error: ${msg}`);
    }
    throw err;
  }
}

/**
 * Procesa la acción de pegar desde el portapapeles:
 * 1. Inspecciona el portapapeles nativo con `api.inspectClipboard`.
 * 2. Valida duplicados con base de datos y con el último emote copiado.
 * 3. Si `askBeforePaste` está activo: abre el popup de confirmación con la previsualización.
 * 4. Si `askBeforePaste` está inactivo: importa directamente.
 */
export async function handleClipboardPasteAction(
  callbacks?: ClipboardImportCallbacks
): Promise<boolean> {
  const inspect = await api.inspectClipboard();

  if (inspect.is_duplicate) {
    const name = inspect.duplicate_name || 'seleccionado';
    toast.info(`El emote "${name}" ya existe en tu colección`);
    return false;
  }

  const lastCopied = get(lastCopiedEmote);
  if (
    lastCopied &&
    inspect.suggested_name &&
    inspect.suggested_name.toLowerCase() === lastCopied.name.toLowerCase()
  ) {
    toast.info(`El emote "${lastCopied.name}" ya existe en tu colección`);
    return false;
  }

  if (!inspect.has_image || !inspect.preview_data_url) {
    const msg = 'No se encontraron imágenes compatibles (.png, .jpg, .webp, .gif).';
    callbacks?.onError?.(msg);
    toast.error(msg);
    return false;
  }

  const normalizedName = inspect.suggested_name || 'Clipboard';
  const ask = get(askBeforePaste);

  if (!ask) {
    await executeDirectClipboardImport(normalizedName, callbacks);
    return true;
  }

  // Si requiere confirmación previa:
  pasteConfirmState.set({
    open: true,
    previewUrl: inspect.preview_data_url,
    name: normalizedName,
    isBusy: false,
    onSave: async () => {
      pasteConfirmState.update((s) => ({ ...s, isBusy: true }));
      try {
        await executeDirectClipboardImport(normalizedName, callbacks);
        closePasteConfirmation();
      } catch {
        pasteConfirmState.update((s) => ({ ...s, isBusy: false }));
      }
    },
    onCancel: () => {
      closePasteConfirmation();
    }
  });

  return true;
}

/**
 * Procesa archivos recibidos directamente en un ClipboardEvent del navegador.
 */
export async function handleFilesFromClipboard(
  files: FileList | File[],
  callbacks?: ClipboardImportCallbacks
): Promise<boolean> {
  const validFiles: File[] = [];
  let hadUnsupported = false;

  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    const isSupported =
      file.type.startsWith('image/') || /\.(png|jpe?g|gif|webp)$/i.test(file.name);
    if (isSupported) {
      validFiles.push(file);
    } else {
      hadUnsupported = true;
    }
  }

  if (validFiles.length === 0) {
    if (hadUnsupported) {
      const msg =
        'El archivo pegado no es compatible. Solo se admiten imágenes PNG, JPG, JPEG, GIF o WebP.';
      callbacks?.onError?.(msg);
      toast.error(msg);
    }
    return false;
  }

  const file = validFiles[0];
  const stem = file.name
    ? toPascalCase(file.name.replace(/\.[^/.]+$/, ''))
    : 'Clipboard';

  const lastCopied = get(lastCopiedEmote);
  if (lastCopied && stem.toLowerCase() === lastCopied.name.toLowerCase()) {
    toast.info(`El emote "${lastCopied.name}" ya existe en tu colección`);
    return false;
  }

  const buf = await file.arrayBuffer();
  const bytes = Array.from(new Uint8Array(buf));
  const ask = get(askBeforePaste);

  const saveAction = async () => {
    callbacks?.onStatus?.('Importando imagen pegada...');
    await api.importRawBytes(bytes, stem || 'Clipboard');
    await loadEmotes();
    callbacks?.onSuccess?.(stem);
  };

  if (!ask) {
    try {
      await saveAction();
      return true;
    } catch (err: any) {
      const msg = err?.message || String(err);
      callbacks?.onError?.(msg);
      toast.error(`Error: ${msg}`);
      return false;
    }
  }

  // askBeforePaste está activo
  const preview = URL.createObjectURL(file);
  pasteConfirmState.set({
    open: true,
    previewUrl: preview,
    name: stem,
    isBusy: false,
    onSave: async () => {
      pasteConfirmState.update((s) => ({ ...s, isBusy: true }));
      try {
        await saveAction();
        URL.revokeObjectURL(preview);
        closePasteConfirmation();
      } catch {
        pasteConfirmState.update((s) => ({ ...s, isBusy: false }));
      }
    },
    onCancel: () => {
      URL.revokeObjectURL(preview);
      closePasteConfirmation();
    }
  });

  return true;
}

/**
 * Atajo centralizado para eventos globales on:paste y on:keydown (Ctrl+V)
 */
export async function handleGlobalPasteEvent(
  e: ClipboardEvent,
  callbacks?: ClipboardImportCallbacks
) {
  if (isInteractiveInputElement(e.target)) {
    return;
  }

  if (shouldThrottlePaste()) {
    e.preventDefault();
    return;
  }

  e.preventDefault();

  const dt = e.clipboardData;
  if (dt && dt.files && dt.files.length > 0) {
    const handled = await handleFilesFromClipboard(dt.files, callbacks);
    if (handled) return;
  }

  try {
    await handleClipboardPasteAction(callbacks);
  } catch (err: any) {
    if (typeof err === 'string' && err.includes('ya está en curso')) {
      return;
    }
    const msg = err?.message || String(err);
    callbacks?.onError?.(msg);
  }
}

export async function handleGlobalKeyDownEvent(
  e: KeyboardEvent,
  callbacks?: ClipboardImportCallbacks
) {
  if ((e.ctrlKey || e.metaKey) && (e.key === 'v' || e.key === 'V')) {
    if (isInteractiveInputElement(e.target)) {
      return;
    }

    if (shouldThrottlePaste()) {
      e.preventDefault();
      return;
    }

    e.preventDefault();
    try {
      await handleClipboardPasteAction(callbacks);
    } catch (err: any) {
      if (typeof err === 'string' && err.includes('ya está en curso')) {
        return;
      }
      const msg = err?.message || String(err);
      callbacks?.onError?.(msg);
    }
  }
}
