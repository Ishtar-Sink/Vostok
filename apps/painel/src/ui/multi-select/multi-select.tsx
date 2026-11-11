import { useEffect, useId, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';
import styles from './multi-select.module.css';

export interface MultiSelectOption {
  value: string;
  label: string;
}

export interface MultiSelectShortcut {
  label: string;
  values: string[];
}

export interface MultiSelectProps {
  /** Nome do filtro: aparece no gatilho e nomeia o menu. */
  label: string;
  /** Opções, na ordem em que aparecem e em que `onChange` devolve os valores. */
  options: readonly MultiSelectOption[];
  selected: readonly string[];
  onChange: (values: string[]) => void;
  /** Resumo da seleção mostrado no gatilho (ex.: `Ativas`, `4 de 8`). */
  summary: string;
  /** Atalhos que trocam a seleção inteira de uma vez. */
  shortcuts?: readonly MultiSelectShortcut[];
  className?: string;
}

function ChevronIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="m6 9 6 6 6-6" />
    </svg>
  );
}

/** Filtro de seleção múltipla em menu suspenso: gatilho com o resumo, atalhos e uma caixa por opção. */
export function MultiSelect({ label, options, selected, onChange, summary, shortcuts = [], className }: MultiSelectProps) {
  const menuId = useId();
  const root = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);

  const items = () => Array.from(menu.current?.querySelectorAll<HTMLElement>('button, input') ?? []);

  function close(returnFocus: boolean) {
    setOpen(false);
    if (returnFocus) trigger.current?.focus();
  }

  useEffect(() => {
    if (!open) return;
    items()[0]?.focus();
    const onPointer = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener('pointerdown', onPointer, true);
    return () => document.removeEventListener('pointerdown', onPointer, true);
  }, [open]);

  function toggle(value: string) {
    onChange(options.map((option) => option.value).filter((other) => (other === value ? !selected.includes(value) : selected.includes(other))));
  }

  function onKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    const list = items();
    const index = list.indexOf(document.activeElement as HTMLElement);
    if (event.key === 'ArrowDown') list[(index + 1) % list.length]?.focus();
    else if (event.key === 'ArrowUp') list[(index - 1 + list.length) % list.length]?.focus();
    else if (event.key === 'Enter' && document.activeElement instanceof HTMLInputElement) document.activeElement.click();
    else if (event.key === 'Escape') close(true);
    else if (event.key === 'Tab') {
      close(false);
      return;
    } else return;
    event.preventDefault();
    event.stopPropagation();
  }

  return (
    <div ref={root} className={[styles.root, className].filter(Boolean).join(' ')}>
      <button ref={trigger} type="button" className={styles.trigger} aria-haspopup="true" aria-expanded={open} aria-controls={open ? menuId : undefined} onClick={() => setOpen((current) => !current)}>
        <span className={styles.label}>{label}:</span>{' '}
        <span className={styles.summary}>{summary}</span>
        <span className={styles.chevron} data-open={open}><ChevronIcon /></span>
      </button>
      {open ? (
        <div ref={menu} id={menuId} role="group" aria-label={label} className={styles.menu} onKeyDown={onKeyDown}>
          {shortcuts.length > 0 ? (
            <div className={styles.shortcuts}>
              {shortcuts.map((shortcut) => (
                <button key={shortcut.label} type="button" className={styles.shortcut} onClick={() => onChange([...shortcut.values])}>{shortcut.label}</button>
              ))}
            </div>
          ) : null}
          <ul className={styles.options}>
            {options.map((option) => (
              <li key={option.value}>
                <label className={styles.option}>
                  <input type="checkbox" className={styles.checkbox} checked={selected.includes(option.value)} onChange={() => toggle(option.value)} />
                  {option.label}
                </label>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
