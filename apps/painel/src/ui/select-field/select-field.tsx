import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';
import styles from './select-field.module.css';

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectFieldProps {
  label: string;
  help?: string;
  error?: string;
  id?: string;
  className?: string;
  options: readonly SelectOption[];
  value: string;
  onChange: (value: string) => void;
  onBlur?: () => void;
  placeholder?: string;
  disabled?: boolean;
  name?: string;
}

function Chevron() {
  return (
    <svg
      viewBox="0 0 24 24"
      width="16"
      height="16"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      <path d="m6 9 6 6 6-6" />
    </svg>
  );
}

const TYPE_AHEAD_MS = 700;
/** Rolagens logo após abrir ou mover a opção ativa são da própria lista e não a fecham. */
const SETTLE_MS = 250;

export function SelectField({
  label,
  help,
  error,
  id,
  className,
  options,
  value,
  onChange,
  onBlur,
  placeholder,
  disabled,
  name,
}: SelectFieldProps) {
  const generated = useId();
  const controlId = id ?? generated;
  const labelId = `${controlId}-label`;
  const valueId = `${controlId}-value`;
  const listId = `${controlId}-list`;
  const helpId = `${controlId}-help`;
  const errorId = `${controlId}-error`;
  const describedBy = [help ? helpId : null, error ? errorId : null].filter(Boolean).join(' ');

  const root = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLUListElement>(null);
  const quiet = useRef(0);
  const typed = useRef({ text: '', at: 0 });
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const [above, setAbove] = useState(false);

  const selected = options.find((option) => option.value === value);

  function openList() {
    if (disabled) return;
    setActive(
      Math.max(
        0,
        options.findIndex((option) => option.value === value),
      ),
    );
    setOpen(true);
  }

  function close(focus: boolean) {
    setOpen(false);
    if (focus) trigger.current?.focus();
  }

  function choose(index: number) {
    const option = options[index];
    if (option && option.value !== value) onChange(option.value);
    close(true);
  }

  useLayoutEffect(() => {
    if (!open || !list.current || !root.current) return;
    const box = root.current.getBoundingClientRect();
    const room = window.innerHeight - box.bottom;
    setAbove(room < list.current.offsetHeight + 8 && box.top > room);
    quiet.current = performance.now() + SETTLE_MS;
    list.current.focus({ preventScroll: true });
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onPointer = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) close(false);
    };
    const onScroll = (event: Event) => {
      if (performance.now() < quiet.current) return;
      if (!list.current?.contains(event.target as Node)) close(false);
    };
    const onResize = () => {
      close(false);
    };
    document.addEventListener('pointerdown', onPointer, true);
    document.addEventListener('scroll', onScroll, true);
    window.addEventListener('resize', onResize);
    return () => {
      document.removeEventListener('pointerdown', onPointer, true);
      document.removeEventListener('scroll', onScroll, true);
      window.removeEventListener('resize', onResize);
    };
  }, [open]);

  useEffect(() => {
    if (!open) return;
    quiet.current = performance.now() + SETTLE_MS;
    document.getElementById(`${listId}-${String(active)}`)?.scrollIntoView({ block: 'nearest' });
  }, [open, active, listId]);

  function onTriggerKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (['ArrowDown', 'ArrowUp'].includes(event.key)) {
      event.preventDefault();
      openList();
    }
  }

  function onListKeyDown(event: KeyboardEvent<HTMLUListElement>) {
    const last = options.length - 1;
    switch (event.key) {
      case 'ArrowDown':
        setActive((index) => Math.min(last, index + 1));
        break;
      case 'ArrowUp':
        setActive((index) => Math.max(0, index - 1));
        break;
      case 'Home':
        setActive(0);
        break;
      case 'End':
        setActive(last);
        break;
      case 'Enter':
      case ' ':
        event.preventDefault();
        choose(active);
        return;
      case 'Escape':
        event.preventDefault();
        event.stopPropagation();
        close(true);
        return;
      case 'Tab':
        close(false);
        return;
      default: {
        if (event.key.length !== 1 || event.ctrlKey || event.metaKey) return;
        const now = event.timeStamp;
        const text =
          (now - typed.current.at < TYPE_AHEAD_MS ? typed.current.text : '') +
          event.key.toLowerCase();
        typed.current = { text, at: now };
        const found = options.findIndex((option) => option.label.toLowerCase().startsWith(text));
        if (found >= 0) setActive(found);
      }
    }
    event.preventDefault();
  }

  const rootClasses = [styles.root, className].filter(Boolean).join(' ');
  return (
    <div className={rootClasses} ref={root}>
      <label id={labelId} htmlFor={controlId} className={styles.label}>
        {label}
      </label>
      <div className={styles.wrap}>
        {/* eslint-disable-next-line jsx-a11y/role-supports-aria-props -- o contrato pede aria-invalid no botão */}
        <button
          ref={trigger}
          type="button"
          id={controlId}
          className={styles.control}
          aria-haspopup="listbox"
          aria-expanded={open}
          aria-controls={open ? listId : undefined}
          aria-labelledby={`${labelId} ${valueId}`}
          aria-invalid={error ? true : undefined}
          aria-describedby={describedBy || undefined}
          disabled={disabled}
          onClick={() => {
            if (open) close(false);
            else openList();
          }}
          onKeyDown={onTriggerKeyDown}
          onBlur={() => {
            if (!open) onBlur?.();
          }}
        >
          <span id={valueId} className={selected ? styles.value : styles.placeholder}>
            {selected?.label ?? placeholder ?? ''}
          </span>
          <span className={[styles.chevron, open ? styles.chevronOpen : ''].join(' ')}>
            <Chevron />
          </span>
        </button>
        {open ? (
          <ul
            ref={list}
            id={listId}
            role="listbox"
            tabIndex={0}
            aria-labelledby={labelId}
            aria-activedescendant={`${listId}-${String(active)}`}
            className={[styles.list, above ? styles.above : ''].join(' ')}
            onKeyDown={onListKeyDown}
          >
            {options.map((option, index) => (
              // eslint-disable-next-line jsx-a11y/click-events-have-key-events
              <li
                key={option.value}
                id={`${listId}-${String(index)}`}
                role="option"
                aria-selected={option.value === value}
                className={[styles.option, index === active ? styles.active : ''].join(' ')}
                onMouseMove={() => {
                  if (index !== active) setActive(index);
                }}
                onClick={() => {
                  choose(index);
                }}
              >
                {option.label}
              </li>
            ))}
          </ul>
        ) : null}
      </div>
      {name ? <input type="hidden" name={name} value={value} /> : null}
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
