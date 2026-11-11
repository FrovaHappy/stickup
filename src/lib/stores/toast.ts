import { writable, get } from 'svelte/store';
import type {
  ToastItem,
  ToastVariant,
  ToastImportance,
  ToastImportanceFilter
} from '$lib/types/system-ui';
import { addNotification } from '$lib/stores/notifications';

const TOAST_IMPORTANCE_FILTER_KEY = 'stickup_pref_toast_importance';

function getStoredImportanceFilter(): ToastImportanceFilter {
  if (typeof localStorage === 'undefined') return 'all';
  try {
    const val = localStorage.getItem(TOAST_IMPORTANCE_FILTER_KEY);
    if (val === 'all' || val === 'medium' || val === 'high' || val === 'none') {
      return val as ToastImportanceFilter;
    }
  } catch {
    // Ignore storage errors
  }
  return 'all';
}

export const toastImportanceFilter = writable<ToastImportanceFilter>(getStoredImportanceFilter());

export function setToastImportanceFilter(filter: ToastImportanceFilter): void {
  toastImportanceFilter.set(filter);
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(TOAST_IMPORTANCE_FILTER_KEY, filter);
    } catch {
      // Ignore
    }
  }
}

export const toasts = writable<ToastItem[]>([]);

let toastCounter = 0;

export function showToast(
  message: string,
  variant: ToastVariant = 'success',
  duration = 3000,
  title?: string,
  recordHistory?: boolean,
  importance?: ToastImportance
): string {
  const timestamp = Date.now();
  const id = `toast-${timestamp}-${++toastCounter}`;
  const defaultTitle =
    variant === 'success'
      ? 'Éxito'
      : variant === 'error'
        ? 'Error'
        : variant === 'warning'
          ? 'Atención'
          : 'Información';

  const resolvedTitle = title || defaultTitle;
  const resolvedImportance: ToastImportance =
    importance ||
    (variant === 'error' || variant === 'warning'
      ? 'high'
      : variant === 'success'
        ? 'medium'
        : 'low');

  const item: ToastItem = {
    id,
    title: resolvedTitle,
    message,
    variant,
    duration,
    timestamp,
    importance: resolvedImportance
  };

  // Determinar si se muestra el toast emergente según el filtro configurado
  const currentFilter = get(toastImportanceFilter);
  const shouldDisplayToast = (() => {
    if (currentFilter === 'none') return false;
    if (currentFilter === 'high') return resolvedImportance === 'high';
    if (currentFilter === 'medium') {
      return resolvedImportance === 'medium' || resolvedImportance === 'high';
    }
    return true; // 'all'
  })();

  if (shouldDisplayToast) {
    toasts.update((all) => [...all, item]);

    if (duration > 0) {
      setTimeout(() => {
        dismissToast(id);
      }, duration);
    }
  }

  // Se registra siempre en el historial de notificaciones si duration > 0 (incluso si está silenciado en pantalla)
  const shouldRecord = recordHistory !== undefined ? recordHistory : duration > 0;
  if (shouldRecord) {
    addNotification({
      id,
      title: resolvedTitle,
      message,
      variant,
      timestamp,
      importance: resolvedImportance
    });
  }

  return id;
}

export function dismissToast(id: string): void {
  toasts.update((all) => all.filter((t) => t.id !== id));
}

export const toast = {
  success: (msg: string, duration = 3000, title?: string, importance?: ToastImportance) =>
    showToast(msg, 'success', duration, title, undefined, importance),
  error: (msg: string, duration = 4000, title?: string, importance?: ToastImportance) =>
    showToast(msg, 'error', duration, title, undefined, importance),
  info: (msg: string, duration = 3000, title?: string, importance?: ToastImportance) =>
    showToast(msg, 'info', duration, title, undefined, importance),
  warning: (msg: string, duration = 3500, title?: string, importance?: ToastImportance) =>
    showToast(msg, 'warning', duration, title, undefined, importance),
  dismiss: dismissToast
};
