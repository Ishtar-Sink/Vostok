import type { ComponentPropsWithoutRef } from 'react';
import styles from './check-list.module.css';

export type CheckTone = 'good' | 'bad';

export interface CheckItem {
  id: string;
  /** Texto conferido, mostrado em código. */
  label: string;
  /** Resultado escrito, por exemplo `apareceu · 1 linha`. */
  result: string;
  /** `good` pinta de sucesso (✓) e `bad` de perigo (✕). */
  tone: CheckTone;
}

export interface CheckListProps extends Omit<ComponentPropsWithoutRef<'ul'>, 'onSelect' | 'children'> {
  /** Nome acessível da lista. */
  label: string;
  /** Itens, um por linha. */
  items: readonly CheckItem[];
  /** Id do item ativo. */
  active?: string | null;
  /** Chamado com o id do item clicado; sem ele os itens não são botões. */
  onSelect?: (id: string) => void;
  /** Dica (`title`) dos botões. */
  itemTitle?: string;
}

function ToneIcon({ tone }: { tone: CheckTone }) {
  return (
    <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false">
      <path d={tone === 'good' ? 'M3 8.5l3.5 3.5L13 4.5' : 'M4 4l8 8M12 4l-8 8'} />
    </svg>
  );
}

/** Lista de itens conferidos, com o resultado escrito e o tom bom ou ruim. */
export function CheckList({
  label,
  items,
  active = null,
  onSelect,
  itemTitle,
  className,
  ...rest
}: CheckListProps) {
  return (
    <ul {...rest} aria-label={label} className={[styles.root, className].filter(Boolean).join(' ')}>
      {items.map((item) => {
        const content = (
          <>
            <span className={styles.icon} aria-hidden="true">
              <ToneIcon tone={item.tone} />
            </span>
            <code className={styles.text}>{item.label}</code>
            {' '}
            <span className={styles.result}>{item.result}</span>
          </>
        );
        return (
          <li key={item.id}>
            {onSelect ? (
              <button
                type="button"
                className={styles.item}
                data-tone={item.tone}
                aria-pressed={active === item.id}
                title={itemTitle}
                onClick={() => onSelect(item.id)}
              >
                {content}
              </button>
            ) : (
              <div className={styles.item} data-tone={item.tone}>
                {content}
              </div>
            )}
          </li>
        );
      })}
    </ul>
  );
}
