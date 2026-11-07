import { useState } from 'react';
import { ActionDialog } from '../../shared/action-bar';
import { formatDate, useResource, type Overview, type PanelRequest, type SpecMeta } from '../../shared/api';
import { ACTION_LABELS, activeRequest, requestLabel, type AttentionItem } from '../../shared/attention';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import { Button } from '../../ui/button/button';
import { DataTable, PrimaryCell, ProgressCell } from '../../ui/data-table/data-table';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import styles from './specs-page.module.css';

interface Row {
  meta: SpecMeta;
  done: number;
  total: number;
  request?: PanelRequest;
}

const FILTERS = [
  { id: 'abertas', label: 'Abertas' },
  { id: 'arquivadas', label: 'Arquivadas' },
  { id: 'todas', label: 'Todas' },
] as const;

const TYPE_LABELS: Record<string, string> = { feature: 'Recurso', improvement: 'Melhoria', fix: 'Correção', study: 'Estudo', analysis: 'Análise' };

const NEW_SPEC: AttentionItem = { key: 'new-spec', kind: 'create-spec', spec: '', title: '', reason: '', href: '/specs' };

function NewSpec() {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button variant="primary" onClick={() => setOpen(true)}>
        {ACTION_LABELS['create-spec'].button}
      </Button>
      <ActionDialog item={open ? NEW_SPEC : null} onClose={() => setOpen(false)} />
    </>
  );
}

export function SpecsPage({ version }: { version: number }) {
  const overview = useResource<Overview>('/api/visao', version);
  const [filter, setFilter] = useState<(typeof FILTERS)[number]['id']>('abertas');
  if (overview.kind === 'loading') return <LoadingState label="Carregando as specs" rows={6} />;
  if (overview.kind === 'error') return <ErrorState title="Não foi possível ler o harness" description={overview.message} retryLabel="Tentar de novo" onRetry={overview.retry} />;

  const requests = overview.data.requests ?? [];
  const working = (id: string) => activeRequest(requests, id) ?? requests.find((request) => request.spec === id);

  const open: Row[] = overview.data.specs.map((spec) => ({ meta: spec.meta, request: working(spec.meta.id), done: spec.tasks.filter((task) => task.status === 'done').length, total: spec.tasks.length }));
  const archived: Row[] = overview.data.archived.map((spec) => ({ meta: spec.meta, done: spec.tasks_total, total: spec.tasks_total }));
  const rows = filter === 'abertas' ? open : filter === 'arquivadas' ? archived : [...open, ...archived];
  const counts = { abertas: open.length, arquivadas: archived.length, todas: open.length + archived.length };

  return (
    <>
      <PageHeader eyebrow="Specs" title="Especificações" subtitle="Da ideia em rascunho à spec arquivada com os documentos finais." actions={<NewSpec />} />
      <div className={styles.filters} role="group" aria-label="Filtrar specs">
        {FILTERS.map((item) => (
          <button key={item.id} type="button" aria-pressed={filter === item.id} onClick={() => setFilter(item.id)}>
            {item.label} <span>{counts[item.id]}</span>
          </button>
        ))}
      </div>
      <DataTable
        caption="Specs"
        rows={rows}
        rowKey={(row) => row.meta.id}
        rowHref={(row) => `/specs/${row.meta.id}`}
        empty={<EmptyState title="Nenhuma spec aqui" description='Troque o filtro ou crie uma spec com "Nova spec".' />}
        columns={[
          { id: 'spec', header: 'Spec', primary: true, cell: (row) => <PrimaryCell id={row.meta.id} title={row.meta.title} href={`/specs/${row.meta.id}`} onClick={onLinkClick} /> },
          { id: 'type', header: 'Tipo', width: '120px', cell: (row) => TYPE_LABELS[row.meta.type] ?? (row.meta.type || '—') },
          { id: 'progress', header: 'Tarefas', width: '150px', cell: (row) => <ProgressCell done={row.done} total={row.total} /> },
          { id: 'updated', header: 'Atualizada', width: '120px', align: 'end', cell: (row) => formatDate(row.meta.updated_at) },
          {
            id: 'status',
            header: 'Status',
            width: '190px',
            cell: (row) => (
              <span className={styles.status}>
                <Badge tone={statusTone(row.meta.status)}>{statusLabel(row.meta.status)}</Badge>
                {row.request ? <span className={styles.agentWork}>{requestLabel(row.request)}</span> : null}
              </span>
            ),
          },
        ]}
      />
    </>
  );
}
