import { useId } from 'react';
import styles from './segmented-field.module.css';

export interface SegmentedOption {
  value: string;
  label: string;
}

export interface SegmentedFieldProps {
  label: string;
  options: readonly SegmentedOption[];
  value: string;
  onChange: (value: string) => void;
  onBlur?: () => void;
  name?: string;
  error?: string;
  help?: string;
  className?: string;
}

export function SegmentedField({
  label,
  options,
  value,
  onChange,
  onBlur,
  name,
  error,
  help,
  className,
}: SegmentedFieldProps) {
  const generated = useId();
  const group = name ?? generated;
  const helpId = `${generated}-help`;
  const errorId = `${generated}-error`;
  const describedBy = [help ? helpId : null, error ? errorId : null].filter(Boolean).join(' ');
  return (
    <fieldset
      className={[styles.root, className].filter(Boolean).join(' ')}
      aria-describedby={describedBy || undefined}
    >
      <legend className={styles.legend}>{label}</legend>
      <div role="radiogroup" aria-label={label} className={styles.group}>
        {options.map((option) => (
          <label key={option.value} className={styles.segment}>
            <input
              type="radio"
              name={group}
              value={option.value}
              checked={option.value === value}
              className={styles.input}
              onChange={() => {
                onChange(option.value);
              }}
              onBlur={onBlur}
            />
            <span className={styles.text}>{option.label}</span>
          </label>
        ))}
      </div>
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
    </fieldset>
  );
}
