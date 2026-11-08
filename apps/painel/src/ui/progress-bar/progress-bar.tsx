import styles from './progress-bar.module.css';

export interface ProgressBarProps {
  label: string;
  value: number;
  max?: number;
}

export function ProgressBar({ label, value, max = 100 }: ProgressBarProps) {
  const clamped = Math.min(Math.max(value, 0), max);
  const percent = max > 0 ? (clamped / max) * 100 : 0;
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuenow={clamped}
      aria-valuemin={0}
      aria-valuemax={max}
      className={styles.root}
    >
      <span className={styles.fill} style={{ width: `${String(percent)}%` }} />
    </div>
  );
}
