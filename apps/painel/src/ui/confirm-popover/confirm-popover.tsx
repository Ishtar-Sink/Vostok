import { useEffect, useId, useRef, useState } from 'react';
import type { ComponentPropsWithoutRef, KeyboardEvent } from 'react';
import { Button } from '../button/button';
import styles from './confirm-popover.module.css';

export type ConfirmPopoverTone = 'primary' | 'danger' | 'neutral';

export interface ConfirmPopoverProps extends Omit<
  ComponentPropsWithoutRef<'div'>,
  'title' | 'onSubmit' | 'children'
> {
  /** Mostra a caixa; ao abrir, limpa o campo e leva o foco a ele. */
  open: boolean;
  /** Título da caixa, lido como nome acessível. */
  title: string;
  /** Explicação curta, ligada por `aria-describedby`. */
  description: string;
  /** Nome acessível do campo de texto. */
  fieldLabel: string;
  placeholder?: string;
  /** Exige texto para confirmar. Padrão: texto opcional. */
  required?: boolean;
  /** Aviso quando falta o texto obrigatório. Padrão: `Obrigatório`. */
  requiredMessage?: string;
  confirmLabel: string;
  /** Padrão: `Cancelar`. */
  cancelLabel?: string;
  /** `danger` pinta o confirmar como recusa. Padrão: `primary`. */
  tone?: ConfirmPopoverTone;
  /** Mostra o botão de confirmar ocupado. */
  loading?: boolean;
  /** Erro de quem confirmou, mostrado abaixo do campo. */
  error?: string;
  /** Chamado com o texto sem espaços nas pontas. */
  onConfirm: (text: string) => void;
  /** Chamado no botão Cancelar e no Esc; o Esc não sobe para o que está atrás. */
  onCancel: () => void;
}

/** Caixa ancorada com um campo de texto, variante do `ConfirmDialog` da Vega para o rodapé. */
export function ConfirmPopover({
  open,
  title,
  description,
  fieldLabel,
  placeholder,
  required = false,
  requiredMessage = 'Obrigatório',
  confirmLabel,
  cancelLabel = 'Cancelar',
  tone = 'primary',
  loading = false,
  error,
  onConfirm,
  onCancel,
  className,
  ...rest
}: ConfirmPopoverProps) {
  const base = useId();
  const fieldRef = useRef<HTMLTextAreaElement>(null);
  const [text, setText] = useState('');
  const [missing, setMissing] = useState(false);

  useEffect(() => {
    if (!open) return;
    setText('');
    setMissing(false);
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    fieldRef.current?.focus();
    return () => previous?.focus();
  }, [open]);

  function confirm() {
    const value = text.trim();
    if (required && value === '') {
      setMissing(true);
      fieldRef.current?.focus();
      return;
    }
    onConfirm(value);
  }

  function onKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    rest.onKeyDown?.(event);
    if (event.key !== 'Escape') return;
    event.preventDefault();
    event.stopPropagation();
    onCancel();
  }

  if (!open) return null;

  const message = missing ? requiredMessage : error;

  return (
    <div
      {...rest}
      role="dialog"
      aria-modal="false"
      aria-labelledby={`${base}-title`}
      aria-describedby={`${base}-desc`}
      className={[styles.root, className].filter(Boolean).join(' ')}
      onKeyDown={onKeyDown}
    >
      <h3 id={`${base}-title`} className={styles.title}>
        {title}
      </h3>
      <p id={`${base}-desc`} className={styles.description}>
        {description}
      </p>
      <textarea
        ref={fieldRef}
        className={styles.field}
        aria-label={fieldLabel}
        placeholder={placeholder}
        value={text}
        onInput={(event) => {
          setText(event.currentTarget.value);
          setMissing(false);
        }}
        aria-required={required || undefined}
        aria-invalid={missing || undefined}
        aria-describedby={message ? `${base}-msg` : undefined}
      />
      {message ? (
        <p id={`${base}-msg`} role="alert" className={styles.message}>
          {message}
        </p>
      ) : null}
      <div className={styles.actions}>
        <Button variant="ghost" onClick={onCancel}>
          {cancelLabel}
        </Button>
        <Button
          variant={tone === 'primary' ? 'primary' : 'secondary'}
          className={tone === 'danger' ? styles.danger : undefined}
          loading={loading}
          onClick={confirm}
        >
          {confirmLabel}
        </Button>
      </div>
    </div>
  );
}
