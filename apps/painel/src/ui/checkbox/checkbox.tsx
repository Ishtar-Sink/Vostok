import { useId } from 'react';
import type { ComponentPropsWithoutRef } from 'react';
import styles from './checkbox.module.css';

export interface CheckboxProps extends Omit<
  ComponentPropsWithoutRef<'div'>,
  'onChange' | 'children'
> {
  /** Texto da caixa, lido como nome acessível. */
  label: string;
  /** Explicação curta, ligada por `aria-describedby`. */
  help?: string;
  /** Estado atual; quem usa guarda o estado. */
  checked: boolean;
  /** Recebe o novo estado a cada clique. */
  onChange: (checked: boolean) => void;
  /** Bloqueia o clique e atenua a caixa. */
  disabled?: boolean;
}

export function Checkbox({
  label,
  help,
  checked,
  onChange,
  disabled,
  className,
  ...rest
}: CheckboxProps) {
  const base = useId();
  const inputId = `${base}-input`;
  const helpId = `${base}-help`;

  return (
    <div {...rest} className={[styles.root, className].filter(Boolean).join(' ')}>
      <input
        id={inputId}
        type="checkbox"
        className={styles.control}
        checked={checked}
        disabled={disabled}
        aria-describedby={help ? helpId : undefined}
        onChange={(event) => {
          if (!disabled) onChange(event.currentTarget.checked);
        }}
      />
      <label htmlFor={inputId} className={styles.label}>
        {label}
      </label>
      {help ? (
        <p id={helpId} className={styles.help}>
          {help}
        </p>
      ) : null}
    </div>
  );
}
