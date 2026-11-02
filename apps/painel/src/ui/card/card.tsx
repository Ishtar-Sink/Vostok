import type { HTMLAttributes } from 'react';
import styles from './card.module.css';

export type CardTone = 'default' | 'primary' | 'accent' | 'success' | 'warning' | 'danger' | 'info';

export interface CardProps extends HTMLAttributes<HTMLElement> {
  as?: 'div' | 'section' | 'article';
  tone?: CardTone;
}

export function Card({ as: Tag = 'div', tone = 'default', className, ...rest }: CardProps) {
  const classes = [styles.root, tone === 'default' ? null : styles[tone], className]
    .filter(Boolean)
    .join(' ');
  return <Tag {...rest} className={classes} />;
}
