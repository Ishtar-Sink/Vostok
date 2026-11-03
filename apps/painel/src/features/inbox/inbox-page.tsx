import { useState } from 'react';
import { ActionDialog, performAction } from '../../shared/action-bar';
import { useResource, type Overview } from '../../shared/api';
import { ACTION_LABELS, attentionItems, isPrimary, type AttentionItem } from '../../shared/attention';
import { onLinkClick } from '../../shared/router';
import { Badge } from '../../ui/badge/badge';
import { Button } from '../../ui/button/button';
import { ConfirmDialog } from '../../ui/confirm-dialog/confirm-dialog';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import styles from './inbox-page.module.css';

const TAG_TONE = { Aprovação: 'primary', 'Pedido ao agente': 'info', Decisão: 'warning', Pergunta: 'accent', Commit: 'info' } as const;

type UndoOutcome = { kind: 'idle' } | { kind: 'sending' } | { kind: 'done'; message: string } | { kind: 'failed'; message: string };

/** Commits apontados pelo `vostok commit check`, com os pedidos de liberar e desfazer (RF-07). */
function PendingCommits({ version }: { version: number }) {
  const commits = useResource<{ linhas?: string[] }>('/api/commits', version);
  const [releasing, setReleasing] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [undo, setUndo] = useState<UndoOutcome>({ kind: 'idle' });
  const lines = commits.kind === 'ready' ? (commits.data.linhas ?? []) : [];
  if (lines.length === 0) return null;
  const text = lines.join('\n');
  const release: AttentionItem = { key: 'release', kind: 'release-commits', spec: '', title: text, reason: '', href: '/pendencias' };

  async function confirmUndo() {
    setConfirming(false);
    setUndo({ kind: 'sending' });
    const result = await performAction({ key: 'undo', kind: 'undo-commits', spec: '', title: text, reason: '', href: '/pendencias' }, '');
    setUndo(result.ok ? { kind: 'done', message: result.message } : { kind: 'failed', message: result.message });
  }

  return (
    <section className={styles.commits} aria-labelledby="commits-sem-liberacao">
      <h2 id="commits-sem-liberacao" className={styles.commitsTitle}>Commits sem liberação</h2>
      <p className={styles.reason}>Commits feitos sem liberação, apontados pela verificação de commits.</p>
      <ul className={styles.commitLines}>
        {lines.map((line) => <li key={line}>{line}</li>)}
      </ul>
      <div className={styles.commitActions}>
        <Button variant="primary" onClick={() => setReleasing(true)}>{ACTION_LABELS['release-commits'].button}</Button>
        <Button variant="secondary" disabled={undo.kind === 'sending'} onClick={() => setConfirming(true)}>{ACTION_LABELS['undo-commits'].button}</Button>
      </div>
      {undo.kind === 'done' ? <p className={styles.reason} role="status">{undo.message}</p> : null}
      {undo.kind === 'failed' ? <p className={styles.reason} role="alert">{undo.message}</p> : null}
      <ActionDialog item={releasing ? release : null} onClose={() => setReleasing(false)} />
      <ConfirmDialog
        open={confirming}
        title="Desfazer os commits apontados?"
        description="Os commits novos saem da branch atual; as alterações ficam na pasta de trabalho."
        confirmLabel={ACTION_LABELS['undo-commits'].button}
        cancelLabel="Cancelar"
        onConfirm={() => void confirmUndo()}
        onCancel={() => setConfirming(false)}
      />
    </section>
  );
}

export function InboxPage({ version }: { version: number }) {
  const overview = useResource<Overview>('/api/visao', version);
  if (overview.kind === 'loading') return <LoadingState label="Carregando as pendências" rows={4} />;
  if (overview.kind === 'error') return <ErrorState title="Não foi possível ler o harness" description={overview.message} retryLabel="Tentar de novo" onRetry={overview.retry} />;
  return (
    <>
      <PageHeader
        eyebrow="Pendências"
        title="Aguardando você"
        subtitle="Aprovações, finalizações, bloqueios e perguntas do agente, em todas as specs abertas."
        actions={<RequestCommit />}
      />
      <PendingCommits version={version} />
      <PendingList items={attentionItems(overview.data)} />
    </>
  );
}

const COMMIT_REQUEST: AttentionItem = { key: 'commit', kind: 'fazer-commit', spec: '', title: '', reason: '', href: '/pendencias' };

function RequestCommit() {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button variant="secondary" onClick={() => setOpen(true)}>
        {ACTION_LABELS['fazer-commit'].button}
      </Button>
      <ActionDialog item={open ? COMMIT_REQUEST : null} onClose={() => setOpen(false)} />
    </>
  );
}

function PendingList({ items }: { items: AttentionItem[] }) {
  const [selected, setSelected] = useState<AttentionItem | null>(null);
  if (items.length === 0) {
    return <EmptyState title="Nada esperando você" description="Aprovações, finalizações, bloqueios e perguntas do agente aparecem aqui assim que surgirem." />;
  }
  return (
    <>
      <ul className={styles.list}>
        {items.map((item) => {
          const label = ACTION_LABELS[item.kind];
          return (
            <li key={item.key} className={styles.item} data-kind={item.kind}>
              <div className={styles.text}>
                <div className={styles.meta}>
                  <Badge tone={TAG_TONE[label.tag as keyof typeof TAG_TONE]}>{label.tag}</Badge>
                  <a href={item.href} onClick={onLinkClick} className={styles.ref}>
                    {item.spec}
                    {item.task ? ` · ${item.task}` : ''}
                  </a>
                </div>
                <p className={styles.title}>{item.title}</p>
                <p className={styles.reason}>{item.reason}</p>
              </div>
              <Button variant={isPrimary(item.kind) ? 'primary' : 'secondary'} onClick={() => setSelected(item)}>
                {label.button}
              </Button>
            </li>
          );
        })}
      </ul>
      <ActionDialog item={selected} onClose={() => setSelected(null)} />
    </>
  );
}
