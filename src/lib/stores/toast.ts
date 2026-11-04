import { writable } from 'svelte/store';
import type { ToastItem, ToastVariant } from '$lib/types/system-ui';

export const toasts = writable<ToastItem[]>([]);

let toastCounter = 0;

export function showToast(
  message: string,
  variant: ToastVariant = 'success',
  duration = 3000
): string {
  const id = `toast-${Date.now()}-${++toastCounter}`;
  const item: ToastItem = { id, message, variant, duration };

  toasts.update((all) => [...all, item]);

  if (duration > 0) {
    setTimeout(() => {
      dismissToast(id);
    }, duration);
  }

  return id;
}

export function dismissToast(id: string): void {
  toasts.update((all) => all.filter((t) => t.id !== id));
}

export const toast = {
  success: (msg: string, duration = 3000) => showToast(msg, 'success', duration),
  error: (msg: string, duration = 4000) => showToast(msg, 'error', duration),
  info: (msg: string, duration = 3000) => showToast(msg, 'info', duration),
  warning: (msg: string, duration = 3500) => showToast(msg, 'warning', duration),
  dismiss: dismissToast
};

