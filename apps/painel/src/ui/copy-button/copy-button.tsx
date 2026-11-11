import type { ComponentPropsWithRef } from 'react';
import { useEffect, useRef, useState } from 'react';
import styles from './copy-button.module.css';

const COPIED_MS = 2000;
const COPIED = 'Copiado';
const FAILED = 'Não foi possível copiar';

export interface CopyButtonProps extends Omit<ComponentPropsWithRef<'button'>, 'children' | 'onClick'> {
  /** Texto a copiar; a função é lida só na hora do clique. */
  text: string | (() => string);
  /** Nome acessível do botão. */
  label?: string;
}

export function CopyButton({ text, label = 'Copiar', className, ...rest }: CopyButtonProps) {
  const [message, setMessage] = useState('');
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  function announce(next: string) {
    setMessage(next);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setMessage(''), COPIED_MS);
  }

  function copy() {
    const value = typeof text === 'function' ? text() : text;
    const written = navigator.clipboard?.writeText(value);
    if (!written) {
      announce(FAILED);
      return;
    }
    written.then(() => announce(COPIED), () => announce(FAILED));
  }

  return (
    <span className={[styles.root, className].filter(Boolean).join(' ')}>
      <button {...rest} type="button" aria-label={label} className={styles.button} data-copied={message === COPIED ? '' : undefined} onClick={copy}>
        <span aria-hidden="true" className={styles.icon}>
          {message === COPIED ? (
            <svg data-icon="check" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M3 8.5l3.5 3.5L13 4.5" />
            </svg>
          ) : (
            <svg data-icon="copy" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round">
              <rect x="5.5" y="5.5" width="8" height="8" rx="1.5" />
              <path d="M10.5 5.5V3.5a1 1 0 0 0-1-1h-6a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2" />
            </svg>
          )}
        </span>
      </button>
      <span role="status" className={styles.visuallyHidden}>{message}</span>
    </span>
  );
}
