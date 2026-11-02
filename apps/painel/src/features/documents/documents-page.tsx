import { fileUrl, formatDate, useResource, type DocumentGroup } from '../../shared/api';
import { onLinkClick } from '../../shared/router';
import { DataTable, PrimaryCell } from '../../ui/data-table/data-table';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import styles from './documents-page.module.css';

const DOC_LABELS: Record<string, string> = { executivo: 'Executivo', tecnico: 'Técnico', evidencias: 'Evidências' };

export function DocumentsPage({ version }: { version: number }) {
  const groups = useResource<DocumentGroup[]>('/api/documentos', version);
  if (groups.kind === 'loading') return <LoadingState label="Carregando os documentos" rows={6} />;
  if (groups.kind === 'error') return <ErrorState title="Não foi possível ler os documentos" description={groups.message} retryLabel="Tentar de novo" onRetry={groups.retry} />;

  return (
    <>
      <PageHeader eyebrow="Documentos" title="Documentos finais" subtitle="Executivo, técnico e evidências de cada spec finalizada, como foram gerados." />
      <DataTable
        caption="Documentos finais"
        rows={groups.data}
        rowKey={(row) => row.meta.id}
        empty={<EmptyState title="Nenhuma spec finalizada" description="Os documentos aparecem aqui quando uma spec é finalizada e arquivada." />}
        columns={[
          { id: 'spec', header: 'Spec', primary: true, cell: (row) => <PrimaryCell id={row.meta.id} title={row.meta.title} href={`/specs/${row.meta.id}`} onClick={onLinkClick} /> },
          {
            id: 'docs',
            header: 'Documentos',
            width: '320px',
            cell: (row) => (
              <span className={styles.docs}>
                {row.docs.map((doc) => (
                  <a key={doc.path} href={fileUrl(doc.path)} target="_blank" rel="noreferrer" className={styles.doc}>
                    {DOC_LABELS[doc.name] ?? doc.name}
                  </a>
                ))}
              </span>
            ),
          },
          { id: 'updated', header: 'Finalizada', width: '120px', align: 'end', cell: (row) => formatDate(row.meta.updated_at) },
        ]}
      />
    </>
  );
}
