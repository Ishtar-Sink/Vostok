import { useEffect, useRef, useState, type ReactNode } from 'react';
import { fetchActivities, formatDatesIn, formatDateTime, useResource, type Activity, type ActivityType, type Overview, type PanelRequest } from '../../shared/api';
import { ActionDialog } from '../../shared/action-bar';
import { ACTION_LABELS, attentionItems, requestLabel, type AttentionItem } from '../../shared/attention';
import { onLinkClick } from '../../shared/router';
import { activityLabel, statusLabel, statusTone } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import { Button } from '../../ui/button/button';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { SelectField } from '../../ui/select-field/select-field';
import styles from './activity-page.module.css';

const PAGE_SIZE = 50;
const TYPES: ActivityType[] = ['spec', 'plano', 'tarefa', 'pedido', 'aval'];
type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; items: Activity[]; hasMore: boolean; loadingMore: boolean; moreFailed: boolean };

const TYPE_LABELS: Record<ActivityType, string> = { spec: 'Spec', plano: 'Plano', tarefa: 'Tarefa', pedido: 'Pedido', aval: 'Aval' };
const TYPE_OPTIONS = [{ value: '', label: 'Todos' }, ...TYPES.map((type) => ({ value: type, label: TYPE_LABELS[type] }))];

const ICON_PATHS: Record<ActivityType, ReactNode> = {
  spec: <><path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" /><path d="M14 3v5h5" /></>,
  plano: <path d="M9 6h11M9 12h11M9 18h11M4 6h.01M4 12h.01M4 18h.01" />,
  tarefa: <><rect x="3" y="3" width="18" height="18" rx="2" /><path d="m8 12 3 3 5-6" /></>,
  pedido: <path d="M22 2 11 13M22 2l-7 20-4-9-9-4z" />,
  aval: <><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" /><path d="m9 12 2 2 4-4" /></>,
};

function TypeIcon({ type }: { type: ActivityType }) {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" className={styles.icon}>
      {ICON_PATHS[type]}
    </svg>
  );
}

function dayKey(value: string): string {
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return value;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  const pad = (number: number) => String(number).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

function timeOf(value: string): string {
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return '';
  return formatDateTime(value).split(', ')[1] ?? '';
}

function groupByDay(items: Activity[]): { key: string; entries: { item: Activity; index: number }[] }[] {
  const groups: { key: string; entries: { item: Activity; index: number }[] }[] = [];
  items.forEach((item, index) => {
    const key = dayKey(item.date);
    const last = groups.at(-1);
    if (last && last.key === key) last.entries.push({ item, index });
    else groups.push({ key, entries: [{ item, index }] });
  });
  return groups;
}

function actionsByActivity(items: Activity[], pending: AttentionItem[]): Map<number, AttentionItem[]> {
  const map = new Map<number, AttentionItem[]>();
  for (const action of pending) {
    const index = items.findIndex((activity) => activity.spec === action.spec && (!action.task || activity.task === action.task));
    if (index < 0) continue;
    map.set(index, [...(map.get(index) ?? []), action]);
  }
  return map;
}

function buttonLabel(item: AttentionItem): string {
  return item.kind === 'approve-task' && (item.tasks?.length ?? 0) >= 2 ? `Aprovar ${item.tasks!.length} tarefas` : ACTION_LABELS[item.kind].button;
}

function RunningRequests({ requests }: { requests: PanelRequest[] }) {
  return (
    <section aria-labelledby="atividades-em-andamento" className={styles.running}>
      <h2 id="atividades-em-andamento" className={styles.runningTitle}>Em andamento</h2>
      <ul className={styles.runningList}>
        {requests.map((request) => (
          <li key={request.id}>
            <Badge tone="info">{requestLabel(request)}</Badge>
            <a href={request.tarefa ? `/tarefas/${request.spec}/${request.tarefa}` : `/specs/${request.spec}`} onClick={onLinkClick} className={styles.what}>
              {request.spec}{request.tarefa ? ` · ${request.tarefa}` : ''}
            </a>
            <span className={styles.when}>{request.id} · desde {formatDateTime(request.atualizado_em)}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function activityDetail(item: Activity): string {
  if (item.type === 'aval') return '';
  if (item.type === 'pedido') return activityLabel(item.detail);
  return formatDatesIn(item.detail);
}

function itemHref(item: Activity): string | null {
  if (item.spec && item.task) return `/tarefas/${item.spec}/${item.task}`;
  return item.spec ? `/specs/${item.spec}` : null;
}

export function ActivityPage({ version }: { version: number }) {
  const [type, setType] = useState('');
  const [spec, setSpec] = useState('');
  const [state, setState] = useState<State>({ kind: 'loading' });
  const [attempt, setAttempt] = useState(0);
  const overview = useResource<Overview>('/api/visao', version);
  const view = overview.kind === 'ready' ? overview.data : null;
  const pending = view ? attentionItems(view) : [];
  const running = view?.requests ?? [];
  const [selected, setSelected] = useState<AttentionItem | null>(null);

  const loaded = useRef(0);
  const filters = useRef('');
  const run = useRef(0);
  const sentinel = useRef<HTMLDivElement>(null);
  const loadMoreRef = useRef<() => void>(() => {});

  useEffect(() => {
    const key = `${type}|${spec}`;
    const count = key === filters.current ? Math.max(loaded.current, PAGE_SIZE) : PAGE_SIZE;
    filters.current = key;
    const current = ++run.current;
    const tipo = (type || undefined) as ActivityType | undefined;
    fetchActivities({ tipo, spec: spec || undefined, limite: count })
      .then((items) => {
        if (current !== run.current) return;
        loaded.current = items.length;
        setState({ kind: 'ready', items, hasMore: items.length >= count, loadingMore: false, moreFailed: false });
      })
      .catch((error: Error) => current === run.current && setState({ kind: 'error', message: error.message }));
  }, [type, spec, version, attempt]);

  function loadMore() {
    if (state.kind !== 'ready' || !state.hasMore || state.loadingMore) return;
    const current = run.current;
    const loadedItems = state.items;
    setState({ ...state, loadingMore: true, moreFailed: false });
    fetchActivities({ tipo: (type || undefined) as ActivityType | undefined, spec: spec || undefined, limite: PAGE_SIZE, deslocamento: loadedItems.length })
      .then((page) => {
        if (current !== run.current) return;
        const items = [...loadedItems, ...page];
        loaded.current = items.length;
        setState({ kind: 'ready', items, hasMore: page.length >= PAGE_SIZE, loadingMore: false, moreFailed: false });
      })
      .catch(() => current === run.current && setState({ kind: 'ready', items: loadedItems, hasMore: true, loadingMore: false, moreFailed: true }));
  }
  loadMoreRef.current = loadMore;

  const ready = state.kind === 'ready' ? state : null;
  useEffect(() => {
    if (typeof IntersectionObserver === 'undefined' || !sentinel.current || !ready || !ready.hasMore || ready.moreFailed || ready.loadingMore) return;
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) loadMoreRef.current();
    }, { rootMargin: '200px' });
    observer.observe(sentinel.current);
    return () => observer.disconnect();
  }, [ready?.items.length, ready?.hasMore, ready?.loadingMore, ready?.moreFailed]);

  const specIds = [...new Set([...(view?.specs ?? []), ...(view?.archived ?? [])].map((item) => item.meta.id).concat(spec ? [spec] : []))].sort();
  const specOptions = [{ value: '', label: 'Todas' }, ...specIds.map((id) => ({ value: id, label: id }))];

  const actionMap = state.kind === 'ready' ? actionsByActivity(state.items, pending) : new Map<number, AttentionItem[]>();

  return (
    <>
      <PageHeader eyebrow="Atividades" title="Atividades" subtitle="Tudo o que aconteceu no projeto, por dia, da mais nova para a mais antiga. O botão ao lado de uma atividade é o que ainda espera você." />
      {running.length > 0 ? <RunningRequests requests={running} /> : null}
      <div className={styles.filters}>
        <SelectField label="Tipo" options={TYPE_OPTIONS} value={type} onChange={setType} />
        <SelectField label="Spec" options={specOptions} value={spec} onChange={setSpec} />
      </div>
      {state.kind === 'loading' ? <LoadingState label="Carregando as atividades" rows={6} /> : null}
      {state.kind === 'error' ? <ErrorState title="Não foi possível ler as atividades" description={state.message} retryLabel="Tentar de novo" onRetry={() => setAttempt((value) => value + 1)} /> : null}
      {state.kind === 'ready' && state.items.length === 0 ? <EmptyState title="Nenhuma atividade com esses filtros." description="Mude o tipo ou a spec para ver outras." /> : null}
      {state.kind === 'ready' && state.items.length > 0 ? groupByDay(state.items).map((group) => (
        <section key={group.key} aria-labelledby={`dia-${group.key}`} className={styles.day}>
          <h2 id={`dia-${group.key}`} className={styles.dayTitle}>{formatDateTime(group.key)}</h2>
          <ol className={styles.timeline}>
            {group.entries.map(({ item, index }) => {
              const href = itemHref(item);
              const title = `${item.target || item.task || item.spec} · ${activityLabel(item.step)}`;
              const actions = actionMap.get(index);
              return (
                <li key={`${item.date}-${item.target}-${index}`} data-type={item.type} data-result={item.result}>
                  <span className={styles.when}><TypeIcon type={item.type} /> {[timeOf(item.date), TYPE_LABELS[item.type]].filter(Boolean).join(' · ')}</span>
                  {href ? <a href={href} onClick={onLinkClick} className={styles.what}>{title}</a> : <span className={styles.what}>{title}</span>}
                  <span className={styles.detail}>
                    <Badge tone={statusTone(item.result)}>{statusLabel(item.result)}</Badge> {activityDetail(item)}
                  </span>
                  {actions ? (
                    <div className={styles.actions}>
                      {actions.map((action) => (
                        <Button key={action.key} size="small" variant={action.now ? 'primary' : 'secondary'} onClick={() => setSelected(action)}>{buttonLabel(action)}</Button>
                      ))}
                    </div>
                  ) : null}
                </li>
              );
            })}
          </ol>
        </section>
      )) : null}
      {ready?.hasMore ? (
        <>
          {ready.moreFailed ? <p className={styles.detail} role="alert">Não foi possível carregar mais atividades. Tente de novo.</p> : null}
          <div ref={sentinel} className={styles.actions}>
            <Button onClick={loadMore}>{ready.loadingMore ? 'Carregando…' : 'Carregar mais'}</Button>
          </div>
        </>
      ) : null}
      <ActionDialog item={selected} onClose={() => setSelected(null)} />
    </>
  );
}
