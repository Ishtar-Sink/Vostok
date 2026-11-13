import { useId, useRef, useState } from 'react';
import type { ComponentPropsWithoutRef, InputEvent, KeyboardEvent } from 'react';
import styles from './tag-field.module.css';

const MAX_SUGGESTIONS = 8;

export interface TagFieldProps extends Omit<
  ComponentPropsWithoutRef<'div'>,
  'onChange' | 'children'
> {
  /** Nome do campo, lido como nome acessível. */
  label: string;
  /** Explicação curta, ligada por `aria-describedby`. */
  help?: string;
  /** Tags já adicionadas, na ordem em que aparecem. */
  values: string[];
  /** Sugestões filtradas pelo texto digitado, sem repetir as já adicionadas. */
  suggestions?: string[];
  /** Recebe o texto sem espaços nas pontas e devolve a tag, ou `null` para recusar. */
  normalize?: (text: string) => string | null;
  /** Mensagem de erro, vinda de fora; marca o campo como inválido. */
  error?: string;
  /** Recebe a lista nova a cada tag acrescentada ou removida. */
  onChange: (values: string[]) => void;
  placeholder?: string;
  /** Aviso quando o `normalize` recusa o texto. Padrão: `Valor não aceito.` */
  refusedMessage?: string;
}

export function TagField({
  label,
  help,
  values,
  suggestions = [],
  normalize,
  error,
  onChange,
  placeholder,
  refusedMessage = 'Valor não aceito.',
  className,
  ...rest
}: TagFieldProps) {
  const [text, setText] = useState('');
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const [refused, setRefused] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const base = useId();
  const inputId = `${base}-input`;
  const listId = `${base}-list`;
  const helpId = `${base}-help`;
  const errorId = `${base}-error`;
  const refusedId = `${base}-refused`;
  const optionId = (index: number) => `${base}-option-${index}`;

  const query = text.trim() ? (normalize?.(text.trim()) ?? text.trim()).toLowerCase() : '';
  const filtered = suggestions
    .filter((item) => !values.includes(item) && item.toLowerCase().includes(query))
    .slice(0, MAX_SUGGESTIONS);
  const hasList = open && filtered.length > 0;
  const describedBy = [help ? helpId : null, refused ? refusedId : null, error ? errorId : null]
    .filter(Boolean)
    .join(' ');

  function close() {
    setOpen(false);
    setActive(-1);
  }

  function commit(raws: string[]): string[] {
    const next = [...values];
    const rejected: string[] = [];
    for (const raw of raws) {
      const trimmed = raw.trim();
      if (!trimmed) continue;
      const tag = normalize ? normalize(trimmed) : trimmed;
      if (tag === null) rejected.push(trimmed);
      else if (!next.includes(tag)) next.push(tag);
    }
    if (next.length > values.length) onChange(next);
    setRefused(rejected.length > 0);
    return rejected;
  }

  function commitText() {
    const rejected = commit([text]);
    setText(rejected.join(', '));
    close();
  }

  function choose(item: string) {
    onChange([...values, item]);
    setText('');
    close();
  }

  function handleInput(event: InputEvent<HTMLInputElement>) {
    const next = event.currentTarget.value;
    if (next.includes(',')) {
      const parts = next.split(',');
      const rejected = commit(parts.slice(0, -1));
      setText([...rejected, parts[parts.length - 1]].filter((part) => part.trim()).join(', '));
    } else {
      setText(next);
      setRefused(false);
    }
    setOpen(true);
    setActive(-1);
  }

  function handleKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setOpen(true);
      setActive((index) => Math.min(filtered.length - 1, index + 1));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive((index) => Math.max(-1, index - 1));
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (hasList && active >= 0) choose(filtered[active]);
      else commitText();
    } else if (event.key === ',') {
      event.preventDefault();
      commitText();
    } else if (event.key === 'Backspace' && text === '' && values.length > 0) {
      onChange(values.slice(0, -1));
    } else if (event.key === 'Escape' && hasList) {
      event.preventDefault();
      event.stopPropagation();
      close();
    } else if (event.key === 'Tab') {
      close();
    }
  }

  return (
    <div {...rest} className={[styles.root, className].filter(Boolean).join(' ')}>
      <label htmlFor={inputId} className={styles.label}>
        {label}
      </label>
      <div className={styles.combo}>
        <input
          ref={input}
          id={inputId}
          type="text"
          autoComplete="off"
          spellCheck={false}
          role="combobox"
          aria-autocomplete="list"
          aria-expanded={hasList}
          aria-controls={hasList ? listId : undefined}
          aria-activedescendant={hasList && active >= 0 ? optionId(active) : undefined}
          aria-invalid={error || refused ? true : undefined}
          aria-describedby={describedBy || undefined}
          placeholder={placeholder}
          className={styles.control}
          value={text}
          onInput={handleInput}
          onKeyDown={handleKeyDown}
          onBlur={close}
        />
        {hasList ? (
          <ul
            id={listId}
            role="listbox"
            aria-label={`Sugestões para ${label}`}
            className={styles.listbox}
          >
            {filtered.map((item, index) => (
              <li
                key={item}
                id={optionId(index)}
                role="option"
                aria-selected={index === active}
                className={styles.option}
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => choose(item)}
              >
                {item}
              </li>
            ))}
          </ul>
        ) : null}
      </div>
      {values.length > 0 ? (
        <ul aria-label={`${label} adicionadas`} className={styles.pills}>
          {values.map((tag) => (
            <li key={tag} className={styles.pill}>
              <span>{tag}</span>
              <button
                type="button"
                className={styles.remove}
                aria-label={`Remover ${tag}`}
                onClick={() => {
                  onChange(values.filter((other) => other !== tag));
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
      {refused ? (
        <p id={refusedId} role="alert" className={styles.error}>
          {refusedMessage}
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
