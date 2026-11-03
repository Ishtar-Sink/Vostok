import { useEffect, useState } from 'react';
import { fetchActivities, formatDatesIn, formatDateTime, type Activity, type ActivityType } from '../../shared/api';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { SelectField } from '../../ui/select-field/select-field';
import styles from './activity-page.module.css';

const ACTIVITY_LIMIT = 1000;
const TYPES: ActivityType[] = ['spec', 'plano', 'tarefa', 'pedido', 'aval'];
const TYPE_OPTIONS = [{ value: '', label: 'Todos' }, ...TYPES.map((type) => ({ value: type, label: type }))];

type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; items: Activity[] };

function itemHref(item: Activity): string | null {
  if (item.spec && item.task) return `/tarefas/${item.spec}/${item.task}`;
  return item.spec ? `/specs/${item.spec}` : null;
}

export function ActivityPage({ version }: { version: number }) {
  const [type, setType] = useState('');
  const [spec, setSpec] = useState('');
  const [knownSpecs, setKnownSpecs] = useState<string[]>([]);
  const [state, setState] = useState<State>({ kind: 'loading' });
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let active = true;
    fetchActivities({ tipo: (type || undefined) as ActivityType | undefined, spec: spec || undefined, limite: ACTIVITY_LIMIT })
      .then((items) => {
        if (!active) return;
        setState({ kind: 'ready', items });
        setKnownSpecs((known) => [...new Set([...known, ...items.map((item) => item.spec).filter(Boolean)])].sort());
      })
      .catch((error: Error) => active && setState({ kind: 'error', message: error.message }));
    return () => {
      active = false;
    };
  }, [type, spec, version, attempt]);

  const specOptions = [{ value: '', label: 'Todas' }, ...knownSpecs.map((id) => ({ value: id, label: id }))];

  return (
    <>
      <PageHeader eyebrow="Atividades" title="Atividades" subtitle="Tudo o que aconteceu no projeto, da mais nova para a mais antiga." />
      <div className={styles.filters}>
        <SelectField label="Tipo" options={TYPE_OPTIONS} value={type} onChange={setType} />
        <SelectField label="Spec" options={specOptions} value={spec} onChange={setSpec} />
      </div>
      {state.kind === 'loading' ? <LoadingState label="Carregando as atividades" rows={6} /> : null}
      {state.kind === 'error' ? <ErrorState title="Não foi possível ler as atividades" description={state.message} retryLabel="Tentar de novo" onRetry={() => setAttempt((value) => value + 1)} /> : null}
      {state.kind === 'ready' && state.items.length === 0 ? <EmptyState title="Nenhuma atividade com esses filtros." description="Mude o tipo ou a spec para ver outras." /> : null}
      {state.kind === 'ready' && state.items.length > 0 ? (
        <ol className={styles.timeline}>
          {state.items.map((item, index) => {
            const href = itemHref(item);
            const title = `${item.target || item.task || item.spec} · ${item.step}`;
            return (
              <li key={`${item.date}-${item.target}-${index}`} data-result={item.result}>
                <span className={styles.when}>{formatDateTime(item.date)} · {item.type}</span>
                {href ? <a href={href} onClick={onLinkClick} className={styles.what}>{title}</a> : <span className={styles.what}>{title}</span>}
                <span className={styles.detail}>
                  <Badge tone={statusTone(item.result)}>{statusLabel(item.result)}</Badge> {formatDatesIn(item.detail)}
                </span>
              </li>
            );
          })}
        </ol>
      ) : null}
    </>
  );
}
