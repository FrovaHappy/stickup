import { writable, derived } from 'svelte/store';
import type { NotificationRecord, ToastVariant } from '$lib/types/system-ui';

const STORAGE_KEY = 'stickup_notifications_history';
const MAX_NOTIFICATIONS = 50;

function loadStoredNotifications(): NotificationRecord[] {
  if (typeof window === 'undefined') return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      return parsed;
    }
  } catch (err) {
    console.error('Error al cargar historial de notificaciones:', err);
  }
  return [];
}

function saveNotifications(items: NotificationRecord[]): void {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
  } catch (err) {
    console.error('Error al guardar historial de notificaciones:', err);
  }
}

export const notifications = writable<NotificationRecord[]>(loadStoredNotifications());

export const unreadCount = derived(notifications, ($notifications) =>
  $notifications.filter((n) => !n.read).length
);

export function addNotification(entry: {
  title?: string;
  message: string;
  variant?: ToastVariant;
  id?: string;
  timestamp?: number;
  importance?: import('$lib/types/system-ui').ToastImportance;
}): NotificationRecord {
  const variant: ToastVariant = entry.variant ?? 'info';
  const defaultTitle =
    variant === 'success'
      ? 'Éxito'
      : variant === 'error'
        ? 'Error'
        : variant === 'warning'
          ? 'Atención'
          : 'Información';

  const record: NotificationRecord = {
    id: entry.id || `notif-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
    title: entry.title || defaultTitle,
    message: entry.message,
    variant,
    timestamp: entry.timestamp || Date.now(),
    read: false,
    importance: entry.importance
  };

  notifications.update((all) => {
    const updated = [record, ...all.filter((item) => item.id !== record.id)].slice(
      0,
      MAX_NOTIFICATIONS
    );
    saveNotifications(updated);
    return updated;
  });

  return record;
}

export function markAsRead(id: string): void {
  notifications.update((all) => {
    const updated = all.map((item) => (item.id === id ? { ...item, read: true } : item));
    saveNotifications(updated);
    return updated;
  });
}

export function markAllAsRead(): void {
  notifications.update((all) => {
    const updated = all.map((item) => ({ ...item, read: true }));
    saveNotifications(updated);
    return updated;
  });
}

export function deleteNotification(id: string): void {
  notifications.update((all) => {
    const updated = all.filter((item) => item.id !== id);
    saveNotifications(updated);
    return updated;
  });
}

export function clearNotifications(): void {
  notifications.set([]);
  saveNotifications([]);
}

export function formatTimeAgo(timestamp: number): string {
  const diffSec = Math.floor((Date.now() - timestamp) / 1000);
  if (diffSec < 45) return 'Hace un momento';
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `Hace ${diffMin} min`;
  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) return `Hace ${diffHours} h`;
  const diffDays = Math.floor(diffHours / 24);
  if (diffDays === 1) return 'Ayer';
  if (diffDays < 7) return `Hace ${diffDays} días`;

  const d = new Date(timestamp);
  return `${d.getDate()}/${d.getMonth() + 1} ${d.getHours().toString().padStart(2, '0')}:${d
    .getMinutes()
    .toString()
    .padStart(2, '0')}`;
}

