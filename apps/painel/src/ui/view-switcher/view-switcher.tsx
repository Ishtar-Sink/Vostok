import type { ComponentPropsWithRef, ReactNode } from 'react';
import styles from './view-switcher.module.css';

export interface ViewOption {
  value: string;
  /** Nome acessível e dica do botão. */
  label: string;
  /** Obrigatório: o botão mostra só o ícone. */
  icon: ReactNode;
}

export interface ViewSwitcherProps extends Omit<
  ComponentPropsWithRef<'div'>,
  'onChange' | 'defaultValue'
> {
  /** Nome acessível do grupo de botões. */
  label: string;
  /** Visualizações disponíveis. */
  options: readonly ViewOption[];
  /** Valor da visualização ativa. */
  value: string;
  /** Chamado com o valor da visualização escolhida. */
  onChange: (value: string) => void;
}

export function ViewSwitcher({
  label,
  options,
  value,
  onChange,
  className,
  ...rest
}: ViewSwitcherProps) {
  return (
    <div
      {...rest}
      role="group"
      aria-label={label}
      className={[styles.root, className].filter(Boolean).join(' ')}
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          aria-pressed={option.value === value}
          aria-label={option.label}
          title={option.label}
          className={styles.option}
          onClick={() => {
            onChange(option.value);
          }}
        >
          {option.icon}
        </button>
      ))}
    </div>
  );
}
