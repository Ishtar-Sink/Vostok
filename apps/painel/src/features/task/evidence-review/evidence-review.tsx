import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import { readTextPreview } from '../../../shared/api';
import { Badge, type BadgeTone } from '../../../ui/badge/badge';
import { SplitDialog, type SplitDialogPane } from '../../../ui/split-dialog/split-dialog';
import { CATEGORIES, type EvidenceEntry } from '../evidence-list/evidence-items';
import { EvidenceViewer, TEXT, type TextState } from './evidence-viewer';
import { nextToReview } from './next-to-review';
import { ReviewActions, type ReviewAction } from './review-actions';
import { ReviewExplanation, type CheckField } from './review-explanation';
import { checksOf, isVisual, snippetsOf, type SnippetCheck } from './snippet-checks';
import styles from './evidence-review.module.css';

const COPY_MESSAGE_MS = 1500;
const MOBILE = '(max-width: 760px)';
const NO_TEXT: TextState = { value: null, total: null, failed: false };

function statusBadge(entry: EvidenceEntry): { label: string; tone: BadgeTone } {
  const status = entry.review?.status;
  if (status !== 'a conferir' && status !== 'recusada' && status !== 'conferida') return { label: 'Sem aval', tone: 'neutral' };
  const category = CATEGORIES.find((item) => item.id === entry.category);
  const label = status === 'a conferir' ? 'A conferir' : status === 'recusada' ? 'Recusada' : 'Conferida';
  return { label, tone: category?.tone ?? 'neutral' };
}

function readsText(entry: EvidenceEntry): boolean {
  return Boolean(entry.file) && TEXT.test(entry.name) && !(entry.manual && !entry.manual.attached);
}

function useEvidenceText(entry: EvidenceEntry, reload: number): TextState {
  const [text, setText] = useState<TextState>(NO_TEXT);
  useEffect(() => {
    setText(NO_TEXT);
    if (!readsText(entry) || !entry.file) return;
    if (entry.file.size === 0) {
      setText({ value: '', total: null, failed: false });
      return;
    }
    let alive = true;
    readTextPreview(entry.file.path)
      .then((preview) => alive && setText({ value: preview.text, total: preview.total, failed: false }))
      .catch(() => alive && setText({ value: null, total: null, failed: true }));
    return () => {
      alive = false;
    };
  }, [entry.name, reload]);
  return text;
}

function useCopyMessage() {
  const [message, setMessage] = useState({ text: '', ok: true });
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);
  function show(text: string, ok: boolean) {
    setMessage({ text, ok });
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setMessage({ text: '', ok: true }), COPY_MESSAGE_MS);
  }
  function copy(value: string) {
    const clipboard = navigator.clipboard;
    if (!clipboard?.writeText) {
      show('Não foi possível copiar', false);
      return;
    }
    clipboard.writeText(value).then(
      () => show(`Copiado: ${value}`, true),
      () => show('Não foi possível copiar', false),
    );
  }
  return { message, copy };
}

export interface EvidenceReviewProps {
  spec: string;
  task: string;
  /** Status da tarefa: concluída (`done`) não mostra as ações. */
  taskStatus: string;
  /** Evidências por onde ‹ › e ← → navegam: as das categorias filtradas, na ordem da lista. */
  entries: EvidenceEntry[];
  current: EvidenceEntry;
  onNavigate: (name: string) => void;
  onClose: () => void;
}

/** Modal de conferência: a explicação da evidência à esquerda e a evidência aberta à direita. */
export function EvidenceReview({ spec, task, taskStatus, entries, current, onNavigate, onClose }: EvidenceReviewProps) {
  const [reload, setReload] = useState(0);
  const [notice, setNotice] = useState<{ text: string; alert: boolean } | null>(null);
  const text = useEvidenceText(current, reload);
  const [active, setActive] = useState<{ field: CheckField; id: string } | null>(null);
  const [pane, setPane] = useState<SplitDialogPane>('side');
  const { message, copy } = useCopyMessage();

  useEffect(() => {
    setActive(null);
    setPane('side');
    setNotice(null);
  }, [current.name]);

  const checks = useMemo<Record<CheckField, SnippetCheck[]>>(() => {
    if (isVisual(current.kindLabel)) return { excerpt: [], failure: [] };
    const log = text.value ?? '';
    return {
      excerpt: checksOf(log, snippetsOf(current.notes.excerpt), true),
      failure: checksOf(log, snippetsOf(current.notes.failure), false),
    };
  }, [current, text.value]);

  const logNote = !readsText(current)
    ? 'Esta evidência não é texto: confira abrindo o arquivo.'
    : text.failed
      ? 'Não foi possível ler o log.'
      : text.value === null
        ? 'Lendo o log…'
        : '';

  const hits = useMemo(() => {
    const check = active ? checks[active.field].find((item) => item.id === active.id) : undefined;
    return new Set(check?.lines ?? []);
  }, [active, checks]);

  function select(field: CheckField, check: SnippetCheck) {
    if (active?.field === field && active.id === check.id) {
      setActive(null);
      return;
    }
    setActive({ field, id: check.id });
    copy(check.label);
    if (check.lines.length > 0 && window.matchMedia?.(MOBILE)?.matches) setPane('main');
  }

  const index = entries.findIndex((item) => item.name === current.name);
  const previous = index > 0 ? entries[index - 1] : undefined;
  const next = index === -1 ? entries[0] : entries[index + 1];

  function onKeys(event: KeyboardEvent<HTMLDialogElement>) {
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if ((event.target as Element).closest?.('input, textarea')) return;
    const target = event.key === 'ArrowLeft' ? previous : event.key === 'ArrowRight' ? next : undefined;
    if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
    event.preventDefault();
    if (target) onNavigate(target.name);
  }

  function done(action: ReviewAction) {
    if (action === 'retirar') {
      setNotice({ text: 'Decisão retirada: a evidência volta para A conferir.', alert: false });
      return;
    }
    const following = nextToReview(entries, current.name);
    if (following) onNavigate(following.name);
    else setNotice({ text: 'Não há outra evidência a conferir neste filtro.', alert: false });
  }

  function changed(message: string) {
    setNotice({ text: message, alert: true });
    setReload((value) => value + 1);
  }

  const badge = statusBadge(current);

  return (
    <SplitDialog
      open
      title={current.title}
      eyebrow={current.id || undefined}
      status={<Badge tone={badge.tone}>{badge.label}</Badge>}
      onClose={onClose}
      pane={pane}
      onPaneChange={setPane}
      onKeyDown={onKeys}
      side={<ReviewExplanation entry={current} checks={checks} logNote={logNote} active={active} onSelect={select} copied={message} />}
      main={<EvidenceViewer entry={current} text={text} hits={hits} version={reload} />}
      footer={
        <>
          <nav aria-label="Navegar entre as evidências" className={styles.nav}>
            <button type="button" className={styles.navButton} aria-label="Anterior (←)" title="Anterior (←)" disabled={!previous} onClick={() => previous && onNavigate(previous.name)}>
              ‹
            </button>
            <span className={styles.position}>{index >= 0 ? `${index + 1} de ${entries.length}` : ''}</span>
            <button type="button" className={styles.navButton} aria-label="Próxima (→)" title="Próxima (→)" disabled={!next} onClick={() => next && onNavigate(next.name)}>
              ›
            </button>
          </nav>
          {notice ? (
            <p role={notice.alert ? 'alert' : 'status'} className={styles.notice}>
              {notice.text}
            </p>
          ) : null}
          {taskStatus !== 'done' ? <ReviewActions spec={spec} task={task} entry={current} onDone={done} onChanged={changed} /> : null}
        </>
      }
    />
  );
}
