import { useId } from 'react';
import type { ComponentPropsWithoutRef } from 'react';
import styles from './number-field.module.css';

export interface NumberFieldProps extends Omit<
  ComponentPropsWithoutRef<'div'>,
  'onChange' | 'children'
> {
  /** Nome do campo, lido como nome acessível. */
  label: string;
  /** Explicação curta, ligada por `aria-describedby`. */
  help?: string;
  /** Valor atual; `null` deixa o campo vazio. */
  value: number | null;
  /** Unidade mostrada ao lado do número. */
  unit?: string;
  /** Limite inferior repassado ao campo; o componente não corrige o valor. */
  min?: number;
  /** Limite superior repassado ao campo; o componente não corrige o valor. */
  max?: number;
  /** Mensagem de erro, vinda de fora; marca o campo como inválido. */
  error?: string;
  /** Recebe o número digitado, ou `null` com o campo vazio ou sem número. */
  onChange: (value: number | null) => void;
}

function parse(raw: string): number | null {
  if (raw.trim() === '' || Number.isNaN(Number(raw))) return null;
  return Number(raw);
}

export function NumberField({
  label,
  help,
  value,
  unit,
  min,
  max,
  error,
  onChange,
  className,
  ...rest
}: NumberFieldProps) {
  const base = useId();
  const inputId = `${base}-input`;
  const unitId = `${base}-unit`;
  const helpId = `${base}-help`;
  const errorId = `${base}-error`;
  const describedBy = [unit ? unitId : null, help ? helpId : null, error ? errorId : null]
    .filter(Boolean)
    .join(' ');

  return (
    <div {...rest} className={[styles.root, className].filter(Boolean).join(' ')}>
      <label htmlFor={inputId} className={styles.label}>
        {label}
      </label>
      <span className={styles.row}>
        <input
          id={inputId}
          type="number"
          inputMode="numeric"
          step={1}
          min={min}
          max={max}
          className={styles.control}
          value={value ?? ''}
          aria-invalid={error ? true : undefined}
          aria-describedby={describedBy || undefined}
          onInput={(event) => onChange(parse(event.currentTarget.value))}
        />
        {unit ? (
          <span id={unitId} className={styles.unit}>
            {unit}
          </span>
        ) : null}
      </span>
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
