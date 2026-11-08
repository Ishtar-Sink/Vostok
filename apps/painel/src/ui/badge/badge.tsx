import type { HTMLAttributes } from 'react';
import styles from './badge.module.css';

export type BadgeTone =
  'neutral' | 'primary' | 'accent' | 'success' | 'warning' | 'danger' | 'info';

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  /** Para uma pendência comum use `warning`, nunca `danger` ([project/frontend-14]). */
  tone?: BadgeTone;
}

export function Badge({ tone = 'primary', className, ...rest }: BadgeProps) {
  const classes = [styles.root, styles[tone], className].filter(Boolean).join(' ');
  return <span {...rest} className={classes} />;
}
