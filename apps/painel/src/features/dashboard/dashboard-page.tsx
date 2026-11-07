import { useEffect, useState } from 'react';
import { fetchActivities, formatDatesIn, formatDateTime, useResource, type Activity, type Overview } from '../../shared/api';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { ActionDialog } from '../../shared/action-bar';
import { ACTION_LABELS, attentionItems, isPrimary, type AttentionItem } from '../../shared/attention';
import { Button } from '../../ui/button/button';
import { EmptyState } from '../../ui/empty-state/empty-state';
import page from '../../shared/page.module.css';
import styles from './dashboard-page.module.css';

const STATUS_ORDER = ['pending', 'approved', 'in_progress', 'validating', 'blocked', 'done'];
const ACTIVITY_LIMITS = [5, 10, 15];

export function DashboardPage({ version, name }: { version: number; name: string }) {
  const overview = useResource<Overview>('/api/visao', version);
  if (overview.kind === 'loading') return <LoadingState label="Carregando o painel" rows={6} />;
  if (overview.kind === 'error') return <ErrorState title="Não foi possível ler o harness" description={overview.message} retryLabel="Tentar de novo" onRetry={overview.retry} />;

  const { specs, archived } = overview.data;
  const tasks = specs.flatMap((spec) => spec.tasks);
  const running = tasks.filter((task) => task.status === 'in_progress' || task.status === 'validating').length;
  const blocked = tasks.filter((task) => task.status === 'blocked').length;
  const attention = attentionItems(overview.data);
  const active = specs.filter((spec) => spec.tasks.length > 0 || spec.plan);
  const early = specs.filter((spec) => spec.tasks.length === 0 && !spec.plan);

  return (
    <div className={page.stack}>
      <PageHeader eyebrow="Painel" title={name || overview.data.project} subtitle="O estado do harness desta pasta, atualizado ao vivo." />
      <dl className={styles.kpis}>
        <Kpi label="Aguardando você" value={attention.length} tone={attention.length > 0 ? 'primary' : undefined} href="/pendencias" />
        <Kpi label="Specs abertas" value={specs.length} hint={`${archived.length} arquivadas`} href="/specs" />
        <Kpi label="Tarefas em execução" value={running} hint={blocked > 0 ? `${blocked} bloqueada(s)` : 'nenhuma bloqueada'} tone={blocked > 0 ? 'warning' : undefined} href="/tarefas" />
        <Kpi label="Tarefas concluídas" value={tasks.filter((task) => task.status === 'done').length} hint={`de ${tasks.length} abertas`} href="/tarefas" />
      </dl>

      <div className={styles.grid}>
        <div className={page.stack}>
          <section className={page.section} aria-labelledby="aguardando">
            <div className={page.sectionHeader}>
              <h2 id="aguardando" className={page.sectionTitle}>Aguardando você</h2>
              <p className={page.sectionHint}>Decisões que destravam o agente</p>
            </div>
            <Waiting items={attention} limit={5} />
          </section>

          <section className={page.section} aria-labelledby="andamento">
            <div className={page.sectionHeader}>
              <h2 id="andamento" className={page.sectionTitle}>Em andamento</h2>
              <a href="/tarefas" onClick={onLinkClick} className={page.link}>Quadro de tarefas</a>
            </div>
            {active.length === 0 ? <p className={page.sectionHint}>Nenhuma spec com plano ou tarefas.</p> : null}
            <div className={styles.specCards}>
              {active.map((spec) => {
                const counts = STATUS_ORDER.map((status) => ({ status, count: spec.tasks.filter((task) => task.status === status).length })).filter((item) => item.count > 0);
                const waves = [...new Set(spec.tasks.map((task) => task.wave).filter(Boolean))].sort((a, b) => Number(a) - Number(b));
                return (
                  <article key={spec.meta.id} className={styles.specCard}>
                    <header className={styles.specHeader}>
                      <a href={`/specs/${spec.meta.id}`} onClick={onLinkClick} className={styles.specLink}>
                        <span className={styles.specId}>{spec.meta.id}</span>
                        <span className={styles.specTitle}>{spec.meta.title}</span>
                      </a>
                      <Badge tone={statusTone(spec.meta.status)}>{statusLabel(spec.meta.status)}</Badge>
                    </header>
                    <div className={styles.stacked} role="img" aria-label={counts.map((item) => `${item.count} ${statusLabel(item.status)}`).join(', ')}>
                      {counts.map((item) => (
                        <span key={item.status} data-status={item.status} style={{ flexGrow: item.count }} />
                      ))}
                    </div>
                    <ul className={styles.legend}>
                      {counts.map((item) => (
                        <li key={item.status} data-status={item.status}>
                          {item.count} {statusLabel(item.status)}
                        </li>
                      ))}
                      {spec.plan ? <li data-status="plan">plano para revisar</li> : null}
                    </ul>
                    {waves.length > 0 ? (
                      <ol className={styles.waves} aria-label="Ondas">
                        {waves.map((wave) => {
                          const inWave = spec.tasks.filter((task) => task.wave === wave);
                          const doneInWave = inWave.filter((task) => task.status === 'done').length;
                          const state = doneInWave === inWave.length ? 'done' : inWave.some((task) => task.status !== 'pending' && task.status !== 'approved') ? 'running' : 'todo';
                          return (
                            <li key={wave} data-state={state} title={`Onda ${wave}: ${doneInWave} de ${inWave.length} concluídas`}>
                              <span>Onda {wave}</span>
                              <span>{doneInWave}/{inWave.length}</span>
                            </li>
                          );
                        })}
                      </ol>
                    ) : null}
                    <a href={`/specs/${spec.meta.id}/quadro`} onClick={onLinkClick} className={page.link}>Abrir o quadro</a>
                  </article>
                );
              })}
            </div>
            {early.length > 0 ? (
              <p className={page.sectionHint}>
                Em preparação: {early.map((spec) => `${spec.meta.id} (${statusLabel(spec.meta.status)})`).join(', ')}
              </p>
            ) : null}
          </section>
        </div>

        <RecentActivity version={version} />
      </div>
    </div>
  );
}

type ActivityState = { kind: 'loading' } | { kind: 'error' } | { kind: 'ready'; items: Activity[] };

function RecentActivity({ version }: { version: number }) {
  const [limit, setLimit] = useState(ACTIVITY_LIMITS[0]);
  const state = useActivities(limit, version);
  return (
    <section className={styles.activity} aria-labelledby="atividade">
      <div className={page.sectionHeader}>
        <h2 id="atividade" className={page.sectionTitle}>Atividade recente</h2>
        <div className={styles.limits} role="group" aria-label="Quantidade de atividades">
          {ACTIVITY_LIMITS.map((option) => (
            <button key={option} type="button" aria-pressed={limit === option} onClick={() => setLimit(option)}>
              {option}
            </button>
          ))}
        </div>
      </div>
      <ActivityList state={state} />
      <a href="/atividades" onClick={onLinkClick} className={page.link}>Ver todas</a>
    </section>
  );
}

function useActivities(limit: number, version: number): ActivityState {
  const [state, setState] = useState<ActivityState>({ kind: 'loading' });
  useEffect(() => {
    let active = true;
    fetchActivities({ limite: limit })
      .then((items) => {
        if (active) setState({ kind: 'ready', items });
      })
      .catch(() => {
        if (active) setState({ kind: 'error' });
      });
    return () => {
      active = false;
    };
  }, [limit, version]);
  return state;
}

function ActivityList({ state }: { state: ActivityState }) {
  if (state.kind === 'loading') return <p className={page.sectionHint}>Carregando atividades.</p>;
  if (state.kind === 'error') return <p className={page.sectionHint}>Não foi possível ler as atividades.</p>;
  if (state.items.length === 0) return <p className={page.sectionHint}>Sem registros ainda.</p>;
  return (
    <ol className={styles.timeline}>
      {state.items.map((item, index) => (
        <li key={`${item.spec}-${item.target}-${index}`} data-result={item.result}>
          <span className={styles.when}>{formatDateTime(item.date)}</span>
          <a href={activityHref(item)} onClick={onLinkClick} className={styles.what}>
            {[item.task, item.step].filter(Boolean).join(' · ')}
          </a>
          <span className={styles.detail}>
            <Badge tone={statusTone(item.result)}>{statusLabel(item.result)}</Badge> {formatDatesIn(item.detail)}
          </span>
        </li>
      ))}
    </ol>
  );
}

function activityHref(item: Activity): string {
  switch (item.type) {
    case 'tarefa':
      return `/tarefas/${item.spec}/${item.target}`;
    case 'spec':
    case 'plano':
      return `/specs/${item.spec}`;
    case 'pedido':
    case 'aval':
      return '/pendencias';
  }
}

function Kpi({ label, value, hint, tone, href }: { label: string; value: number; hint?: string; tone?: 'primary' | 'warning'; href: string }) {
  return (
    <a href={href} onClick={onLinkClick} className={styles.kpi} data-tone={tone}>
      <dt>{label}</dt>
      <dd>{value}</dd>
      {hint ? <span className={styles.kpiHint}>{hint}</span> : null}
    </a>
  );
}

const TAG_TONE = { Aprovação: 'primary', 'Pedido ao agente': 'info', Decisão: 'warning', Pergunta: 'accent' } as const;

function Waiting({ items, limit }: { items: AttentionItem[]; limit: number }) {
  const [selected, setSelected] = useState<AttentionItem | null>(null);
  if (items.length === 0) {
    return <EmptyState title="Nada esperando você" description="Aprovações, finalizações, bloqueios e perguntas do agente aparecem aqui assim que surgirem." />;
  }
  return (
    <>
      <ul className={styles.waiting}>
        {items.slice(0, limit).map((item) => {
          const label = ACTION_LABELS[item.kind];
          return (
            <li key={item.key} className={styles.waitingItem} data-kind={item.kind}>
              <div className={styles.waitingText}>
                <div className={styles.waitingMeta}>
                  <Badge tone={TAG_TONE[label.tag as keyof typeof TAG_TONE]}>{label.tag}</Badge>
                  <a href={item.href} onClick={onLinkClick} className={styles.waitingRef}>
                    {item.spec}
                    {item.task ? ` · ${item.task}` : ''}
                  </a>
                </div>
                <p className={styles.waitingTitle}>{item.title}</p>
                <p className={styles.waitingReason}>{item.reason}</p>
              </div>
              <Button variant={isPrimary(item.kind) ? 'primary' : 'secondary'} onClick={() => setSelected(item)}>
                {label.button}
              </Button>
            </li>
          );
        })}
      </ul>
      {items.length > limit ? (
        <a href="/pendencias" onClick={onLinkClick} className={page.link}>
          Ver as {items.length} pendências
        </a>
      ) : null}
      <ActionDialog item={selected} onClose={() => setSelected(null)} />
    </>
  );
}
