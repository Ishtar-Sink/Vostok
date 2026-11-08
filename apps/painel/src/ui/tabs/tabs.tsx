import { useId, useRef, useState } from 'react';
import type { KeyboardEvent, ReactNode } from 'react';
import styles from './tabs.module.css';

export interface TabItem {
  id: string;
  label: string;
  panel: ReactNode;
}

export interface TabsProps {
  /** Nome acessível da lista de abas. */
  label: string;
  tabs: readonly TabItem[];
  value?: string;
  defaultValue?: string;
  onChange?: (id: string) => void;
}

export function Tabs({ label, tabs, value, defaultValue, onChange }: TabsProps) {
  const base = useId();
  const [internal, setInternal] = useState(defaultValue ?? tabs[0]?.id);
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});
  const [focused, setFocused] = useState<string | undefined>(undefined);
  const selected = value ?? internal;
  const tabStop = focused ?? selected;

  function select(id: string) {
    setInternal(id);
    onChange?.(id);
  }

  function moveFocus(index: number) {
    const target = tabs[(index + tabs.length) % tabs.length];
    if (!target) return;
    setFocused(target.id);
    refs.current[target.id]?.focus();
  }

  function onKeyDown(event: KeyboardEvent, index: number) {
    const moves: Record<string, number> = {
      ArrowRight: index + 1,
      ArrowLeft: index - 1,
      Home: 0,
      End: tabs.length - 1,
    };
    const next = moves[event.key];
    if (next === undefined) return;
    event.preventDefault();
    moveFocus(next);
  }

  return (
    <div>
      <div role="tablist" aria-label={label} className={styles.list}>
        {tabs.map((tab, index) => {
          const active = tab.id === selected;
          return (
            <button
              key={tab.id}
              ref={(node) => {
                refs.current[tab.id] = node;
              }}
              type="button"
              role="tab"
              id={`${base}-tab-${tab.id}`}
              aria-selected={active}
              aria-controls={`${base}-panel-${tab.id}`}
              tabIndex={tab.id === tabStop ? 0 : -1}
              className={styles.tab}
              onClick={() => {
                select(tab.id);
              }}
              onFocus={() => {
                setFocused(tab.id);
              }}
              onBlur={() => {
                setFocused(undefined);
              }}
              onKeyDown={(event) => {
                onKeyDown(event, index);
              }}
            >
              {tab.label}
            </button>
          );
        })}
      </div>
      {tabs.map((tab) => (
        <div
          key={tab.id}
          role="tabpanel"
          id={`${base}-panel-${tab.id}`}
          aria-labelledby={`${base}-tab-${tab.id}`}
          hidden={tab.id !== selected}
          className={styles.panel}
        >
          {tab.id === selected ? tab.panel : null}
        </div>
      ))}
    </div>
  );
}
