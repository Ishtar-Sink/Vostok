import type { HTMLAttributes } from 'react';
import styles from './toast.module.css';

export type ToastTone = 'info' | 'success' | 'warning' | 'danger';

export function ToastRegion({ className, ...rest }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      {...rest}
      aria-live="polite"
      className={[styles.region, className].filter(Boolean).join(' ')}
    />
  );
}

export interface ToastProps extends HTMLAttributes<HTMLDivElement> {
  tone?: ToastTone;
}

export function Toast({ tone = 'info', className, ...rest }: ToastProps) {
  return (
    <div {...rest} className={[styles.root, styles[tone], className].filter(Boolean).join(' ')} />
  );
}
