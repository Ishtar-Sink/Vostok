import { useResource, type Overview } from '../../shared/api';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { remainingTasks, type RemainingTask } from '../../shared/attention';
import { EmptyState } from '../../ui/empty-state/empty-state';
import page from '../../shared/page.module.css';
import styles from './dashboard-page.module.css';

const STATUS_ORDER = ['pending', 'approved', 'in_progress', 'executed', 'validating', 'blocked', 'done'];

export function DashboardPage({ version, name }: { version: number; name: string }) {
  const overview = useResource<Overview>('/api/visao', version);
  if (overview.kind === 'loading') return <LoadingState label="Carregando o painel" rows={6} />;
  if (overview.kind === 'error') return <ErrorState title="Não foi possível ler o harness" description={overview.message} retryLabel="Tentar de novo" onRetry={overview.retry} />;

  const { specs, archived } = overview.data;
  const tasks = specs.flatMap((spec) => spec.tasks);
  const running = tasks.filter((task) => task.status === 'in_progress' || task.status === 'executed' || task.status === 'validating').length;
  const blocked = tasks.filter((task) => task.status === 'blocked').length;
  const remaining = remainingTasks(specs);
  const active = specs.filter((spec) => spec.tasks.length > 0 || spec.plan);
  const early = specs.filter((spec) => spec.tasks.length === 0 && !spec.plan);

  return (
    <div className={page.stack}>
      <PageHeader eyebrow="Painel" title={name || overview.data.project} subtitle="O estado do harness desta pasta, atualizado ao vivo." />
      <dl className={styles.kpis}>
        <Kpi label="Tarefas que faltam" value={remaining.length} tone={remaining.length > 0 ? 'primary' : undefined} href="/tarefas" />
        <Kpi label="Specs abertas" value={specs.length} hint={`${archived.length} arquivadas`} href="/specs" />
        <Kpi label="Tarefas em execução" value={running} hint={blocked > 0 ? `${blocked} bloqueada(s)` : 'nenhuma bloqueada'} tone={blocked > 0 ? 'warning' : undefined} href="/tarefas" />
        <Kpi label="Tarefas concluídas" value={tasks.filter((task) => task.status === 'done').length} hint={`de ${tasks.length} abertas`} href="/tarefas" />
      </dl>

      <div className={styles.grid}>
        <div className={page.stack}>
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
                      {spec.plan && spec.plan_outdated ? <Badge tone="warning">Plano desatualizado</Badge> : null}
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
                    <a href={`/tarefas?spec=${spec.meta.id}`} onClick={onLinkClick} className={page.link}>Abrir o quadro</a>
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

        <section className={styles.remaining} aria-labelledby="faltam">
          <div className={page.sectionHeader}>
            <h2 id="faltam" className={page.sectionTitle}>Tarefas que faltam</h2>
          </div>
          <p className={page.sectionHint}>Status e onda de cada tarefa ainda não concluída</p>
          <RemainingList specs={specs} remaining={remaining} />
        </section>
      </div>
    </div>
  );
}

function RemainingList({ specs, remaining }: { specs: Overview['specs']; remaining: RemainingTask[] }) {
  if (remaining.length === 0) {
    return <EmptyState title="Nenhuma tarefa faltando" description="As tarefas das specs abertas aparecem aqui até serem concluídas." />;
  }
  return (
    <>
      {specs
        .filter((spec) => remaining.some((task) => task.spec === spec.meta.id))
        .map((spec) => (
          <div key={spec.meta.id} className={page.stack}>
            <h3 className={styles.remainingSpec}>
              <a href={`/tarefas?spec=${spec.meta.id}`} onClick={onLinkClick}>{spec.meta.id} · {spec.meta.title}</a>
            </h3>
            <ul className={styles.remainingList}>
              {remaining
                .filter((task) => task.spec === spec.meta.id)
                .map((task) => (
                  <li key={task.id}>
                    <a href={`/tarefas/${spec.meta.id}/${task.id}`} onClick={onLinkClick}>
                      <span className={styles.specId}>{task.id}</span> {task.title}
                    </a>
                    <Badge tone={statusTone(task.status)}>{statusLabel(task.status)}</Badge>
                    {task.wave ? <span className={styles.remainingWave}>onda {task.wave}</span> : null}
                  </li>
                ))}
            </ul>
          </div>
        ))}
    </>
  );
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
