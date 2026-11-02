import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { onLinkClick } from './router';
import type { Reference, ReferenceIndex } from './references';
import { statusLabel, statusTone } from './status';
import { Badge } from '../ui/badge/badge';
import styles from './ref-popover.module.css';

interface Open {
  ref: Reference;
  anchor: HTMLElement;
  pinned: boolean;
}

const HOVER_DELAY = 250;

/** Popover das referências: prévia ao passar o mouse ou focar, fixo com um clique. Fecha com Esc,
 *  clique fora ou ao sair do mouse (se não estiver fixo). No celular, ocupa a base da tela. */
export function useReferencePopover(index: ReferenceIndex) {
  const [open, setOpen] = useState<Open | null>(null);
  const timer = useRef(0);

  const show = useCallback(
    (anchor: HTMLElement, pinned: boolean) => {
      const ref = index.get(anchor.dataset.ref ?? '');
      if (!ref) return;
      setOpen((current) => {
        current?.anchor.setAttribute('aria-expanded', 'false');
        anchor.setAttribute('aria-expanded', 'true');
        return { ref, anchor, pinned };
      });
    },
    [index],
  );

  const close = useCallback(() => {
    window.clearTimeout(timer.current);
    setOpen((current) => {
      current?.anchor.setAttribute('aria-expanded', 'false');
      return null;
    });
  }, []);

  /** Liga os eventos ao contêiner que tem os botões `.ref`. */
  const bind = useCallback(
    (container: HTMLElement | null) => {
      if (!container) return;
      const target = (event: Event) => (event.target as HTMLElement).closest<HTMLElement>('button.ref');
      container.onclick = (event) => {
        const anchor = target(event);
        if (anchor) show(anchor, true);
      };
      container.onmouseover = (event) => {
        const anchor = target(event);
        if (!anchor) return;
        window.clearTimeout(timer.current);
        timer.current = window.setTimeout(() => show(anchor, false), HOVER_DELAY);
      };
      container.onmouseout = (event) => {
        if (!target(event)) return;
        window.clearTimeout(timer.current);
        timer.current = window.setTimeout(() => setOpen((current) => (current && !current.pinned ? (current.anchor.setAttribute('aria-expanded', 'false'), null) : current)), HOVER_DELAY);
      };
    },
    [show],
  );

  const popover = open ? <Popover open={open} onClose={close} onKeep={() => window.clearTimeout(timer.current)} /> : null;
  return { bind, popover };
}

function Popover({ open, onClose, onKeep }: { open: Open; onClose: () => void; onKeep: () => void }) {
  const box = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<{ top: number; left: number } | null>(null);
  const { ref } = open;

  useLayoutEffect(() => {
    const rect = open.anchor.getBoundingClientRect();
    const height = box.current?.offsetHeight ?? 0;
    const width = box.current?.offsetWidth ?? 360;
    const below = rect.bottom + 8 + height < window.innerHeight;
    setPosition({
      top: below ? rect.bottom + 8 : Math.max(8, rect.top - 8 - height),
      left: Math.min(Math.max(8, rect.left), window.innerWidth - width - 8),
    });
  }, [open]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
        open.anchor.focus();
      }
    };
    const onDown = (event: MouseEvent) => {
      if (!box.current?.contains(event.target as Node) && event.target !== open.anchor) onClose();
    };
    window.addEventListener('keydown', onKey);
    window.addEventListener('mousedown', onDown);
    window.addEventListener('scroll', onClose, { passive: true, once: true });
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('mousedown', onDown);
      window.removeEventListener('scroll', onClose);
    };
  }, [open, onClose]);

  return createPortal(
    <div
      ref={box}
      role="dialog"
      aria-label={`Referência ${ref.id}`}
      className={styles.popover}
      style={position ? { top: position.top, left: position.left } : { visibility: 'hidden' }}
      onMouseEnter={onKeep}
    >
      <header className={styles.header}>
        <span className={styles.id}>{ref.id}</span>
        <span className={styles.section}>{ref.section}</span>
        {ref.task ? <Badge tone={statusTone(ref.task.status)}>{statusLabel(ref.task.status)}</Badge> : null}
        <button type="button" className={styles.close} aria-label="Fechar" onClick={onClose}>
          ×
        </button>
      </header>
      {ref.text ? <p className={styles.text}>{ref.text}</p> : null}
      {ref.fields.length > 0 ? (
        <dl className={styles.fields}>
          {ref.fields.filter(([, value]) => value && value !== '-').map(([label, value]) => (
            <div key={label}>
              <dt>{label}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      ) : null}
      {ref.task ? (
        <a href={`/tarefas/${ref.task.spec}/${ref.id}`} onClick={(event) => { onClose(); onLinkClick(event); }} className={styles.link}>
          Abrir a tarefa
        </a>
      ) : null}
    </div>,
    document.body,
  );
}
