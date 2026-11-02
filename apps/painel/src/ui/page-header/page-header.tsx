import type { ReactNode, Ref } from 'react';
import styles from './page-header.module.css';

export interface PageHeaderProps {
  title: string;
  eyebrow?: string;
  subtitle?: string;
  actions?: ReactNode;
  /** "Voltar" acima do eyebrow e do título. */
  back?: ReactNode;
  /** Preso sob a topbar: mostra o fundo de desfoque. */
  stuck?: boolean;
  /** Título menor e sem subtítulo, depois de rolar a página. */
  compact?: boolean;
  headingRef?: Ref<HTMLHeadingElement>;
}

export function PageHeader({
  title,
  eyebrow,
  subtitle,
  actions,
  back,
  stuck = false,
  compact = false,
  headingRef,
}: PageHeaderProps) {
  const classes = [styles.root, stuck ? styles.stuck : '', compact ? styles.compact : '']
    .filter(Boolean)
    .join(' ');
  return (
    <header
      className={classes}
      {...(stuck ? { 'data-stuck': '' } : {})}
      {...(compact ? { 'data-compact': '' } : {})}
    >
      <div className={styles.titleBlock}>
        {back ? <div className={styles.back}>{back}</div> : null}
        {eyebrow ? <p className={styles.eyebrow}>{eyebrow}</p> : null}
        <h1 ref={headingRef} tabIndex={-1} data-page-heading="" className={styles.pageHeading}>
          {title}
        </h1>
        {subtitle && !compact ? <p className={styles.subtitle}>{subtitle}</p> : null}
      </div>
      {actions ? <div className={styles.actions}>{actions}</div> : null}
    </header>
  );
}
