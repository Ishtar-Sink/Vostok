import { useId, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';
import styles from './tag-field.module.css';

export interface TagFieldProps {
  label: string;
  value: readonly string[];
  onChange: (value: string[]) => void;
  onBlur?: () => void;
  /** Nome da lista de pills: "{label} adicionadas". */
  list_label: string;
  /** Nome do "×" de cada pill: "Remover {tag}". */
  remove_label: (tag: string) => string;
  placeholder?: string;
  help?: string;
  error?: string;
  id?: string;
  className?: string;
}

export function TagField({
  label,
  value,
  onChange,
  onBlur,
  list_label,
  remove_label,
  placeholder,
  help,
  error,
  id,
  className,
}: TagFieldProps) {
  const generated = useId();
  const controlId = id ?? generated;
  const helpId = `${controlId}-help`;
  const errorId = `${controlId}-error`;
  const describedBy = [help ? helpId : null, error ? errorId : null].filter(Boolean).join(' ');
  const input = useRef<HTMLInputElement>(null);
  const [text, setText] = useState('');

  function commit(raw: string) {
    const tag = raw.trim().replace(/^#+/, '').trim();
    setText('');
    if (!tag) return;
    if (value.some((existing) => existing.toLowerCase() === tag.toLowerCase())) return;
    onChange([...value, tag]);
  }

  function onKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === 'Enter' || event.key === ',') {
      event.preventDefault();
      commit(text);
    } else if (event.key === 'Backspace' && text === '' && value.length > 0) {
      onChange(value.slice(0, -1));
    }
  }

  return (
    <div className={[styles.root, className].filter(Boolean).join(' ')}>
      <label htmlFor={controlId} className={styles.label}>
        {label}
      </label>
      <input
        ref={input}
        id={controlId}
        type="text"
        autoComplete="off"
        className={styles.control}
        placeholder={placeholder}
        value={text}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy || undefined}
        onChange={(event) => {
          const next = event.target.value;
          if (next.includes(',')) {
            next
              .split(',')
              .slice(0, -1)
              .forEach((part) => {
                commit(part);
              });
            setText(next.slice(next.lastIndexOf(',') + 1));
          } else {
            setText(next);
          }
        }}
        onKeyDown={onKeyDown}
        onBlur={() => {
          commit(text);
          onBlur?.();
        }}
      />
      {value.length > 0 ? (
        <ul aria-label={list_label} className={styles.pills}>
          {value.map((tag) => (
            <li key={tag} className={styles.pill}>
              <span>{tag}</span>
              <button
                type="button"
                className={styles.remove}
                aria-label={remove_label(tag)}
                onClick={() => {
                  onChange(value.filter((existing) => existing !== tag));
                  input.current?.focus();
                }}
              >
                <span aria-hidden="true">×</span>
              </button>
            </li>
          ))}
        </ul>
      ) : null}
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
