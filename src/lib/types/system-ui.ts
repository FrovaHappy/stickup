export type ButtonVariant =
  | 'primary'
  | 'secondary'
  | 'accent'
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

export type AlertVariant = 'info' | 'success' | 'warning' | 'error' | 'toast';

export type CardHighlight = 'none' | 'new' | 'hot' | 'indigo';

export type CardPadding = 'sm' | 'md' | 'lg';

export type ToastVariant = 'success' | 'error' | 'info' | 'warning';

export interface ToastItem {
  id: string;
  message: string;
  variant?: ToastVariant;
  duration?: number;
}

export interface ContextMenuItem {
  id?: string;
  label: string;
  icon?: string;
  shortcut?: string;
  danger?: boolean;
  disabled?: boolean;
  divider?: boolean;
  action?: () => void;
}

