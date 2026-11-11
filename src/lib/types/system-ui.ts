export type ButtonVariant =
  | 'primary'
  | 'secondary'
  | 'accent'
  | 'warning'
  | 'success'
  | 'danger'
  | 'outline'
  | 'ghost'
  | 'icon';

export type ButtonSize = 'sm' | 'md' | 'lg';

export type BadgeVariant = 'gray' | 'blue' | 'green' | 'amber' | 'hot' | 'new';

export type BadgeShape = 'rounded' | 'pill' | 'circle';

export type ChipVariant = 'default' | 'indigo' | 'amber' | 'green' | 'sky' | 'hot';

export type InputVariant = 'default' | 'pill';

export type CardHighlight = 'none' | 'new' | 'hot' | 'indigo';

export type CardPadding = 'sm' | 'md' | 'lg';

export type ToastVariant = 'success' | 'error' | 'info' | 'warning';

export type ToastImportance = 'low' | 'medium' | 'high';

export type ToastImportanceFilter = 'all' | 'medium' | 'high' | 'none';

export interface ToastItem {
  id: string;
  title?: string;
  message: string;
  variant?: ToastVariant;
  duration?: number;
  timestamp?: number;
  importance?: ToastImportance;
}

export interface NotificationRecord {
  id: string;
  title: string;
  message: string;
  variant: ToastVariant;
  timestamp: number;
  read: boolean;
  importance?: ToastImportance;
}

export interface ContextMenuItem {
  id?: string;
  label: string;
  icon?: any;
  shortcut?: string;
  danger?: boolean;
  disabled?: boolean;
  divider?: boolean;
  action?: () => void;
}

