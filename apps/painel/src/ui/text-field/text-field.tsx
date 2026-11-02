import { useId } from 'react';
import type { InputHTMLAttributes } from 'react';
import styles from './text-field.module.css';

export interface TextFieldProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'id'> {
  label: string;
  help?: string;
  error?: string;
  id?: string;
}

export function TextField({ label, help, error, id, className, ...rest }: TextFieldProps) {
  const generated = useId();
  const controlId = id ?? generated;
  const helpId = `${controlId}-help`;
  const errorId = `${controlId}-error`;
  const describedBy = [help ? helpId : null, error ? errorId : null].filter(Boolean).join(' ');
  return (
    <div className={[styles.root, className].filter(Boolean).join(' ')}>
      <label htmlFor={controlId} className={styles.label}>
        {label}
      </label>
      <input
        {...rest}
        id={controlId}
        className={styles.control}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy || undefined}
      />
      {help ? (
        <p id={helpId} className={styles.help}>
          {help}
        </p>
      ) : null}
      {error ? (
        <p id={errorId} role="alert" className={styles.error}>
          {error}
        </p>
      ) : null}
    </div>
  );
}
