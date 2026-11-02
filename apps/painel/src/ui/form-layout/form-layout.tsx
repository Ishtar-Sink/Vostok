import type { ReactNode } from 'react';
import { Button } from '../button/button';
import styles from './form-layout.module.css';

export interface FormGridProps {
  children: ReactNode;
  className?: string;
}

/** Grade do formulário; um filho com `data-span="full"` ocupa a linha inteira. */
export function FormGrid({ children, className }: FormGridProps) {
  return <div className={[styles.grid, className].filter(Boolean).join(' ')}>{children}</div>;
}

export interface FormActionsProps {
  cancel_label: string;
  submit_label: string;
  onCancel: () => void;
  loading?: boolean;
  submit_disabled?: boolean;
}

export function FormActions({
  cancel_label,
  submit_label,
  onCancel,
  loading = false,
  submit_disabled = false,
}: FormActionsProps) {
  return (
    <div className={styles.actions} data-span="full">
      <Button variant="muted" onClick={onCancel}>
        {cancel_label}
      </Button>
      <Button variant="primary" type="submit" loading={loading} disabled={submit_disabled}>
        {submit_label}
      </Button>
    </div>
  );
}
