import type { ReactNode } from 'react';
import { formatDateTime } from '../../../shared/api';
import { Markdown } from '../../../shared/markdown';
import { Badge } from '../../../ui/badge/badge';
import { CheckList } from '../../../ui/check-list/check-list';
import { CopyButton } from '../../../ui/copy-button/copy-button';
import type { EvidenceEntry } from '../evidence-list/evidence-items';
import { isVisual, notApplicable, type SnippetCheck } from './snippet-checks';
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

function NotApplicable({ reason }: { reason: string }) {
  return (
    <div className={styles.na}>
      <strong>Não se aplica.</strong> {reason}
    </div>
  );
}

/** Explicação da evidência, na ordem do RF-08: o que prova, como foi feita, como reproduzir, o que conferir e a decisão. */
export function ReviewExplanation({ entry, checks, logNote, active, onSelect, copied }: ReviewExplanationProps) {
  const { notes, review } = entry;
  const visual = isVisual(entry.kindLabel);

  function checkBlock(field: CheckField) {
    const { title, legend } = FIELD[field];
    const items = checks[field];
    if (visual || items.length === 0) {
      return (
        <Block title={title}>
          <NotApplicable reason={notApplicable(field, entry.kindLabel)} />
        </Block>
      );
    }
    if (logNote) {
      return (
        <Block title={title}>
          <p className={styles.note}>{logNote}</p>
        </Block>
      );
    }
    return (
      <Block title={title}>
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
      </Block>
    );
  }

  const date = review?.reviewed_at ? formatDateTime(review.reviewed_at) : '—';

  return (
    <>
      <Block title="O que prova">
        {notes.proof ? <Markdown source={notes.proof} /> : <p className={styles.note}>O planejamento não descreveu o que esta evidência prova.</p>}
      </Block>
      <Block title="Como foi feita">
        <div className={styles.tool}>
          <TerminalIcon />
          <span>{review?.tool || '—'}</span>
          {entry.kind ? <Badge tone="neutral">{entry.kind}</Badge> : null}
        </div>
        {review?.how ? <p className={styles.note}>{review.how}</p> : null}
        <p className={styles.derived}>{review?.how ? 'Derivado do tipo e do comando. A frase vem do subitem "Como:".' : 'Derivado do tipo e do comando.'}</p>
      </Block>
      <Block title="Como reproduzir">
        {review?.reproduce ? (
          <div className={styles.command}>
            <code>{review.reproduce}</code>
            <CopyButton text={review.reproduce} label="Copiar comando" />
          </div>
        ) : (
          <p className={styles.note}>Anexada pelo usuário; não há comando.</p>
        )}
      </Block>
      {visual ? (
        <Block title="Como ler">
          <div className={styles.read}>
            <strong>O que olhar:</strong>
            {notes.excerpt ? <Markdown source={notes.excerpt} /> : <p>—</p>}
          </div>
          <div className={styles.read}>
            <strong>Sinal de falha:</strong>
            {notes.failure ? <Markdown source={notes.failure} /> : <p>—</p>}
          </div>
        </Block>
      ) : null}
      {checkBlock('excerpt')}
      {checkBlock('failure')}
      <p role="status" className={styles.copied} data-ok={copied.ok}>
        {copied.text}
      </p>
      {review?.status === 'conferida' ? (
        <Block title="Conferência">
          <p className={styles.note}>{`Conferida em ${date}${review.reason ? ` · Aval: “${review.reason}”` : ''}`}</p>
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
