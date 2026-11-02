import { useState } from 'react';
import { ActionDialog } from '../../shared/action-bar';
import { useResource, type Overview } from '../../shared/api';
import { ACTION_LABELS, attentionItems, isPrimary, type AttentionItem } from '../../shared/attention';
import { onLinkClick } from '../../shared/router';
import { Badge } from '../../ui/badge/badge';
import { Button } from '../../ui/button/button';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import styles from './inbox-page.module.css';

const TAG_TONE = { Aprovação: 'primary', 'Pedido ao agente': 'info', Decisão: 'warning', Pergunta: 'accent', Commit: 'info' } as const;

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
