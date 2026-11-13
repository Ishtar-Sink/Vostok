import { useId } from 'react';
import type { ComponentPropsWithoutRef } from 'react';
import styles from './switch.module.css';

export interface SwitchProps extends Omit<
  ComponentPropsWithoutRef<'div'>,
  'onChange' | 'children'
> {
  /** Texto do interruptor, lido como nome acessível. */
  label: string;
  /** Explicação curta, ligada por `aria-describedby`. */
  help?: string;
  /** Estado atual; quem usa guarda o estado. */
  checked: boolean;
  /** Recebe o estado oposto ao atual a cada clique. */
  onChange: (checked: boolean) => void;
}

export function Switch({ label, help, checked, onChange, className, ...rest }: SwitchProps) {
  const base = useId();
  const controlId = `${base}-control`;
  const helpId = `${base}-help`;

  return (
    <div {...rest} className={[styles.field, className].filter(Boolean).join(' ')}>
      <button
        id={controlId}
        type="button"
        role="switch"
        aria-checked={checked}
        aria-describedby={help ? helpId : undefined}
        className={styles.root}
        onClick={() => onChange(!checked)}
      >
        <span className={styles.track} aria-hidden="true">
          <span className={styles.thumb} />
        </span>
      </button>
      <div className={styles.text}>
        <label htmlFor={controlId} className={styles.label}>
          {label}
        </label>
        {help ? (
          <p id={helpId} className={styles.help}>
            {help}
          </p>
        ) : null}
      </div>
    </div>
  );
}
