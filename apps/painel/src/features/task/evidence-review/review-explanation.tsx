import type { ReactNode } from 'react';
import { formatDateTime } from '../../../shared/api';
import { Markdown } from '../../../shared/markdown';
import { Badge } from '../../../ui/badge/badge';
import { CheckList } from '../../../ui/check-list/check-list';
import { CopyButton } from '../../../ui/copy-button/copy-button';
import type { EvidenceEntry } from '../evidence-list/evidence-items';
import { isVisual, type SnippetCheck } from './snippet-checks';
import styles from './evidence-review.module.css';

export type CheckField = 'excerpt' | 'failure';

export interface ReviewExplanationProps {
  entry: EvidenceEntry;
  checks: Record<CheckField, SnippetCheck[]>;
  /** Mensagem no lugar dos resultados enquanto o log não está disponível; vazia com o log lido. */
  logNote: string;
  active: { field: CheckField; id: string } | null;
  onSelect: (field: CheckField, check: SnippetCheck) => void;
  copied: { text: string; ok: boolean };
  /** Leva o vídeo aberto ao tempo, em segundos. */
  onSeek: (seconds: number) => void;
}

const FIELD = {
  excerpt: { title: 'Trecho esperado', legend: 'Cada texto precisa aparecer no log.' },
  failure: { title: 'Sinais de falha', legend: 'Nenhum destes textos pode aparecer no log.' },
} as const;

function TerminalIcon() {
  return (
    <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false">
      <path d="M3 4.5l3 3-3 3M8 11.5h5" />
    </svg>
  );
}

function Block({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className={styles.block}>
      <h3>{title}</h3>
      {children}
    </section>
  );
}

const PREVIOUS_SNIPPET = /(?:^| · )conferida sob o trecho anterior \(ajuste (\d+)\)$/;

/** Separa do aval a marca que a CLI acrescenta à conferência mantida depois de um Trecho ou Falha trocado. */
export function splitPreviousSnippet(reason: string): { reason: string; adjustment?: number } {
  const match = PREVIOUS_SNIPPET.exec(reason);
  if (!match) return { reason };
  return { reason: reason.slice(0, match.index), adjustment: Number(match[1]) };
}

const TIME = /(?<!\d)(\d{1,2}):([0-5]\d)(?!\d)/g;

/** Divide o texto nos tempos `m:ss`, que viram `{ label, seconds }`; o resto fica como texto. */
export function timeParts(text: string): (string | { label: string; seconds: number })[] {
  const parts: (string | { label: string; seconds: number })[] = [];
  let last = 0;
  for (const match of text.matchAll(TIME)) {
    const start = match.index ?? 0;
    if (start > last) parts.push(text.slice(last, start));
    parts.push({ label: match[0], seconds: Number(match[1]) * 60 + Number(match[2]) });
    last = start + match[0].length;
  }
  if (last < text.length) parts.push(text.slice(last));
  return parts;
}

/** Explicação da evidência, na ordem da conferência: o que mostra, onde olhar, a falha e, recolhido, o resto. */
export function ReviewExplanation({ entry, checks, logNote, active, onSelect, copied, onSeek }: ReviewExplanationProps) {
  const { notes, review } = entry;
  const visual = isVisual(entry.kindLabel);
  const video = entry.kindLabel === 'Vídeo';

  function checkList(field: CheckField) {
    const { title, legend } = FIELD[field];
    const items = checks[field];
    if (logNote) return <p className={styles.note}>{logNote}</p>;
    return (
      <>
        <CheckList
          label={title}
          items={items}
          active={active?.field === field ? active.id : null}
          itemTitle="Clique para destacar no log e copiar"
          onSelect={(id) => {
            const check = items.find((item) => item.id === id);
            if (check) onSelect(field, check);
          }}
        />
        <p className={styles.legend}>{legend}</p>
      </>
    );
  }

  const failureChecks = () => checkList('failure');

  function checkBlock(field: CheckField) {
    if (checks[field].length === 0) return null;
    return <Block title={FIELD[field].title}>{checkList(field)}</Block>;
  }

  const { reason: aval, adjustment } = splitPreviousSnippet(review?.reason ?? '');
  const date = review?.reviewed_at ? formatDateTime(review.reviewed_at) : '—';
  const where = notes.whereToLook ?? (visual ? notes.excerpt : undefined);
  const shows = notes.shows ?? notes.proof;
  const reproduce = (notes.reproduce ?? review?.reproduce)?.replace(/^`+|`+$/g, '').trim();
  const covers = (notes.covers ?? '').split(',').map((id) => id.trim()).filter(Boolean);
  const hasHow = Boolean(review?.tool || entry.kind || review?.how);

  return (
    <>
      <Block title="O que mostra">
        {shows ? <Markdown source={shows} /> : <p className={styles.note}>O planejamento não descreveu o que esta evidência mostra.</p>}
      </Block>
      {where ? (
        <Block title="Onde olhar">
          {video ? (
            <p className={styles.note}>
              {timeParts(where).map((part, index) =>
                typeof part === 'string' ? (
                  part
                ) : (
                  <button key={index} type="button" className={styles.time} onClick={() => onSeek(part.seconds)}>
                    {part.label}
                  </button>
                ),
              )}
            </p>
          ) : (
            <Markdown source={where} />
          )}
        </Block>
      ) : null}
      {visual ? (
        notes.failure ? (
          <Block title="Falha">
            <Markdown source={notes.failure} />
          </Block>
        ) : null
      ) : (
        <>
          {checkBlock('excerpt')}
          {notes.failure || checks.failure.length > 0 ? (
            <Block title={FIELD.failure.title}>
              {notes.failure ? <Markdown source={notes.failure} /> : null}
              {checks.failure.length > 0 ? failureChecks() : null}
            </Block>
          ) : null}
        </>
      )}
      {shows && notes.proof && notes.shows ? (
        <details className={styles.fold}>
          <summary>Prova</summary>
          <div>
            <Markdown source={notes.proof} />
          </div>
        </details>
      ) : null}
      {hasHow ? (
        <details className={styles.fold}>
          <summary>Como foi feita</summary>
          <div>
            <div className={styles.tool}>
              <TerminalIcon />
              <span>{review?.tool || '—'}</span>
              {entry.kind ? <Badge tone="neutral">{entry.kind}</Badge> : null}
            </div>
            {review?.how ? <p className={styles.note}>{review.how}</p> : null}
            <p className={styles.derived}>{review?.how ? 'Derivado do tipo e do comando. A frase vem do subitem "Como:".' : 'Derivado do tipo e do comando.'}</p>
          </div>
        </details>
      ) : null}
      {reproduce ? (
        <details className={styles.fold}>
          <summary>Reproduzir</summary>
          <div>
            <div className={styles.command}>
              <code>{reproduce}</code>
              <CopyButton text={reproduce} label="Copiar comando" />
            </div>
          </div>
        </details>
      ) : null}
      {covers.length > 0 ? (
        <details className={styles.fold}>
          <summary>Cobre</summary>
          <div>
            <ul className={styles.tags}>
              {covers.map((id) => (
                <li key={id}>
                  <Badge tone="neutral">{id}</Badge>
                </li>
              ))}
            </ul>
          </div>
        </details>
      ) : null}
      <p role="status" className={styles.copied} data-ok={copied.ok}>
        {copied.text}
      </p>
      {review?.status === 'conferida' ? (
        <Block title="Conferência">
          <p className={styles.note}>{`Conferida em ${date}${aval ? ` · Aval: “${aval}”` : ''}`}</p>
          {adjustment === undefined ? null : (
            <p className={styles.note}>{`Conferida sob o trecho anterior (ajuste ${adjustment}): o Trecho ou a Falha mudaram depois desta conferência.`}</p>
          )}
        </Block>
      ) : null}
      {review?.status === 'recusada' ? (
        <Block title="Recusa">
          <p className={styles.note}>{`Recusada em ${date} · Motivo: “${review.reason}”`}</p>
        </Block>
      ) : null}
    </>
  );
}
