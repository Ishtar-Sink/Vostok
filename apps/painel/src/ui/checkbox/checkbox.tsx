import { useId } from 'react';
import type { InputHTMLAttributes } from 'react';
import styles from './checkbox.module.css';

export interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type' | 'id'> {
  label: string;
  id?: string;
}

export function Checkbox({ label, id, className, ...rest }: CheckboxProps) {
  const generated = useId();
  const controlId = id ?? generated;
  return (
    <div className={[styles.root, className].filter(Boolean).join(' ')}>
      <input {...rest} id={controlId} type="checkbox" className={styles.control} />
      <label htmlFor={controlId} className={styles.label}>
        {label}
      </label>
    </div>
  );
}
