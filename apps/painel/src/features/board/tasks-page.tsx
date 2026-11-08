import { useState } from 'react';
import { useResource, type Overview } from '../../shared/api';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { Board } from './board';
import styles from './board.module.css';

export function TasksPage({ version }: { version: number }) {
  const overview = useResource<Overview>('/api/visao', version);
  const [spec, setSpec] = useState('todas');
  if (overview.kind === 'loading') return <LoadingState label="Carregando as tarefas" rows={6} />;
  if (overview.kind === 'error') return <ErrorState title="Não foi possível ler o harness" description={overview.message} retryLabel="Tentar de novo" onRetry={overview.retry} />;

  const specs = overview.data.specs.filter((item) => item.tasks.length > 0);
  const tasks = specs.filter((item) => spec === 'todas' || item.meta.id === spec).flatMap((item) => item.tasks.map((task) => ({ ...task, spec: item.meta.id })));

  return (
    <>
      <PageHeader eyebrow="Tarefas" title="Quadro de tarefas" subtitle="Todas as tarefas das specs abertas, pela etapa em que estão." />
      <div className={styles.filters} role="group" aria-label="Filtrar por spec">
        <button type="button" aria-pressed={spec === 'todas'} onClick={() => setSpec('todas')}>
          Todas <span>{specs.reduce((sum, item) => sum + item.tasks.length, 0)}</span>
        </button>
        {specs.map((item) => (
          <button key={item.meta.id} type="button" aria-pressed={spec === item.meta.id} onClick={() => setSpec(item.meta.id)} title={item.meta.title}>
            {item.meta.id} <span>{item.tasks.length}</span>
          </button>
        ))}
      </div>
      <Board tasks={tasks} showSpec={spec === 'todas'} />
    </>
  );
}
