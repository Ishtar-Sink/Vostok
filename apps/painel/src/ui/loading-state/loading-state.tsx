import styles from './loading-state.module.css';

export interface SkeletonProps {
  rows?: number;
}

export function Skeleton({ rows = 3 }: SkeletonProps) {
  return (
    <div className={styles.skeleton} aria-hidden="true">
      {Array.from({ length: rows }, (_, index) => (
        <i key={index} className={styles.row} />
      ))}
    </div>
  );
}

export interface LoadingStateProps {
  /** Texto lido por leitores de tela enquanto carrega. */
  label: string;
  rows?: number;
}

export function LoadingState({ label, rows }: LoadingStateProps) {
  return (
    <div role="status" aria-busy="true" className={styles.root}>
      <span className={styles.visuallyHidden}>{label}</span>
      <Skeleton rows={rows} />
    </div>
  );
}
