import { useEffect, useId, useRef, useState } from 'react';
import type { ComponentPropsWithoutRef, KeyboardEvent, ReactNode } from 'react';
import styles from './split-dialog.module.css';

export type SplitDialogPane = 'side' | 'main';

export interface SplitDialogProps extends Omit<
  ComponentPropsWithoutRef<'dialog'>,
  'title' | 'open' | 'onClose' | 'role' | 'children'
> {
  /** Abre o diálogo como modal (`showModal()`); ao ficar `false`, fecha e devolve o foco. */
  open: boolean;
  /** Título do diálogo, lido como nome acessível. */
  title: string;
  /** Selo antes do título, por exemplo o ID da evidência. */
  eyebrow?: ReactNode;
  /** Estado mostrado depois do título; some no celular. */
  status?: ReactNode;
  /** Chamado no Esc, no `cancel` nativo e no botão de fechar. */
  onClose: () => void;
  /** Painel da explicação (esquerda). */
  side: ReactNode;
  /** Painel da evidência (direita). */
  main: ReactNode;
  /** Ações do rodapé. */
  footer: ReactNode;
  /** Nome do painel da explicação e da aba do celular. Padrão: `Explicação`. */
  sideLabel?: string;
  /** Nome do painel da evidência e da aba do celular. Padrão: `Evidência`. */
  mainLabel?: string;
  /** Nome do botão de fechar. Padrão: `Fechar (Esc)`. */
  closeLabel?: string;
  /** Painel ativo no celular, quando controlado. */
  pane?: SplitDialogPane;
  /** Chamado com o painel escolhido pelas abas. */
  onPaneChange?: (pane: SplitDialogPane) => void;
}

const FOCUSABLE = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';

function focusables(root: ParentNode): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
    (element) => !element.hasAttribute('disabled'),
  );
}

/** Diálogo modal em dois painéis (explicação e evidência), com abas no celular. */
export function SplitDialog({
  open,
  title,
  eyebrow,
  status,
  onClose,
  side,
  main,
  footer,
  sideLabel = 'Explicação',
  mainLabel = 'Evidência',
  closeLabel = 'Fechar (Esc)',
  pane,
  onPaneChange,
  className,
  ...rest
}: SplitDialogProps) {
  const base = useId();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [own, setOwn] = useState<SplitDialogPane>('side');
  const current = pane ?? own;

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog || !open) return;
    const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialog.showModal();
    const body = dialog.querySelector('[data-split-body]');
    const target =
      dialog.querySelector<HTMLElement>('[data-autofocus]') ??
      (body ? focusables(body)[0] : undefined);
    target?.focus();
    return () => {
      dialog.close();
      trigger?.focus();
    };
  }, [open]);

  function choose(next: SplitDialogPane) {
    setOwn(next);
    onPaneChange?.(next);
  }

  function onKeyDown(event: KeyboardEvent<HTMLDialogElement>) {
    rest.onKeyDown?.(event);
    if (event.defaultPrevented) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key !== 'Tab') return;
    const list = focusables(event.currentTarget);
    const first = list[0];
    const last = list[list.length - 1];
    if (!first || !last) return;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  if (!open) return null;

  return (
    <dialog
      {...rest}
      ref={dialogRef}
      aria-labelledby={`${base}-title`}
      className={[styles.root, className].filter(Boolean).join(' ')}
      onKeyDown={onKeyDown}
      onCancel={(event) => {
        rest.onCancel?.(event);
        event.preventDefault();
        onClose();
      }}
    >
      <div className={styles.frame}>
        <header className={styles.head}>
          {eyebrow ? <span className={styles.eyebrow}>{eyebrow}</span> : null}
          <h2 id={`${base}-title`} className={styles.title}>
            {title}
          </h2>
          {status ? <span className={styles.status}>{status}</span> : null}
          <button
            type="button"
            className={styles.close}
            aria-label={closeLabel}
            title={closeLabel}
            onClick={onClose}
          >
            <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" aria-hidden="true" focusable="false">
              <path d="M4 4l8 8M12 4l-8 8" />
            </svg>
          </button>
        </header>
        <div role="tablist" aria-label="Partes da conferência" className={styles.tabs}>
          <button
            type="button"
            role="tab"
            id={`${base}-tab-side`}
            aria-selected={current === 'side'}
            aria-controls={`${base}-side`}
            onClick={() => choose('side')}
          >
            {sideLabel}
          </button>
          <button
            type="button"
            role="tab"
            id={`${base}-tab-main`}
            aria-selected={current === 'main'}
            aria-controls={`${base}-main`}
            onClick={() => choose('main')}
          >
            {mainLabel}
          </button>
        </div>
        <div className={styles.split} data-pane={current} data-split-body="">
          <section id={`${base}-side`} aria-label={sideLabel} className={styles.side}>
            {side}
          </section>
          <section id={`${base}-main`} aria-label={mainLabel} className={styles.main}>
            {main}
          </section>
        </div>
        <footer className={styles.footer}>{footer}</footer>
      </div>
    </dialog>
  );
}
