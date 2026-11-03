import { useMemo, useState } from 'react';
import { BackLink } from '../../shared/back-link';
import { EditDialog } from '../../shared/edit-dialog';
import { buildReferences, type ReferenceIndex } from '../../shared/references';
import { useReferencePopover } from '../../shared/ref-popover';
import { Button } from '../../ui/button/button';
import { navigate } from '../../shared/router';
import { formatDate, useResource, type PlanDetail, type SpecDetail } from '../../shared/api';
import { headingsOf, Markdown } from '../../shared/markdown';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Toc } from '../../shared/toc';
import { Badge } from '../../ui/badge/badge';
import { Card } from '../../ui/card/card';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { Board } from '../board/board';
import { specActions } from '../../shared/attention';
import { ActionBar } from '../../shared/action-bar';
import page from '../../shared/page.module.css';
import styles from './spec-page.module.css';

type View = 'documento' | 'quadro' | 'plano';

const TYPE_LABELS: Record<string, string> = { feature: 'Recurso', improvement: 'Melhoria', fix: 'Correção', study: 'Estudo', analysis: 'Análise' };

export function SpecPage({ id, view, version }: { id: string; view: View; version: number }) {
  const spec = useResource<SpecDetail>(`/api/specs/${encodeURIComponent(id)}`, version);
  const references = useMemo(() => (spec.kind === 'ready' ? buildReferences(spec.data.markdown, spec.data.tasks, id) : new Map()), [spec, id]);
  if (spec.kind === 'loading') return <LoadingState label={`Carregando ${id}`} rows={8} />;
  if (spec.kind === 'error') return <ErrorState title={`Não foi possível abrir ${id}`} description={spec.message} retryLabel="Tentar de novo" onRetry={spec.retry} />;

  const { meta, tasks, plan, plan_outdated } = spec.data;
  const outdated = plan && plan_outdated;
  const done = tasks.filter((task) => task.status === 'done').length;
  const views: { id: View; label: string; show: boolean }[] = [
    { id: 'documento', label: 'Documento', show: true },
    { id: 'quadro', label: `Quadro (${tasks.length})`, show: tasks.length > 0 },
    { id: 'plano', label: 'Plano', show: plan },
  ];

  return (
    <div className={page.stack}>
      <div>
        <PageHeader
          back={<BackLink fallback="/specs" label="Specs" />}
          eyebrow={`${meta.id} · ${TYPE_LABELS[meta.type] ?? meta.type}`}
          title={meta.title}
          actions={
            <div className={styles.headerActions}>
              <Badge tone={statusTone(meta.status)}>{statusLabel(meta.status)}</Badge>
              {outdated ? <Badge tone="warning">Plano desatualizado</Badge> : null}
              <ActionBar items={specActions(meta, plan, tasks, outdated)} />
              {tasks.length > 0 && view !== 'quadro' ? (
                <Button variant="secondary" onClick={() => navigate(`/specs/${meta.id}/quadro`)}>Ver o quadro</Button>
              ) : null}
            </div>
          }
        />
        <nav className={styles.views} aria-label="Visões da spec">
          {views
            .filter((item) => item.show)
            .map((item) => (
              <a key={item.id} href={`/specs/${meta.id}${item.id === 'documento' ? '' : `/${item.id}`}`} onClick={onLinkClick} aria-current={view === item.id ? 'page' : undefined}>
                {item.label}
              </a>
            ))}
        </nav>
      </div>
      {view === 'quadro' ? (
        <Board tasks={tasks.map((task) => ({ ...task, spec: meta.id }))} />
      ) : view === 'plano' ? (
        <PlanView id={meta.id} version={version} references={references} outdated={outdated} status={meta.status} />
      ) : (
        <SpecDocument markdown={spec.data.markdown} path={spec.data.path} references={references}>
          <dl className={page.meta}>
            <div><dt>Status</dt><dd>{statusLabel(meta.status)}</dd></div>
            <div><dt>Tipo</dt><dd>{TYPE_LABELS[meta.type] ?? meta.type}</dd></div>
            <div><dt>Autor</dt><dd>{meta.author}</dd></div>
            <div><dt>Criada</dt><dd>{formatDate(meta.created_at)}</dd></div>
            <div><dt>Aprovada</dt><dd>{formatDate(meta.approved_at)}</dd></div>
            <div><dt>Ajustes</dt><dd>{meta.adjustments || '0'}</dd></div>
            {outdated ? <div><dt>Plano</dt><dd>Desatualizado: a spec mudou depois do plano.</dd></div> : null}
            {tasks.length > 0 ? <div><dt>Tarefas</dt><dd>{done} de {tasks.length} concluídas</dd></div> : null}
          </dl>
          {tasks.length > 0 ? (
            <section>
              <div className={styles.asideHeader}>
                <p className={page.asideTitle}>Tarefas</p>
                <a href={`/specs/${meta.id}/quadro`} onClick={onLinkClick} className={page.link}>Quadro</a>
              </div>
              <ul className={styles.taskList}>
                {tasks.map((task) => (
                  <li key={task.id}>
                    <a href={`/tarefas/${meta.id}/${task.id}`} onClick={onLinkClick}>
                      <span className={styles.taskId}>{task.id}</span>
                      <span className={styles.taskTitle}>{task.title}</span>
                    </a>
                    <Badge tone={statusTone(task.status)}>{statusLabel(task.status)}</Badge>
                  </li>
                ))}
              </ul>
            </section>
          ) : null}
        </SpecDocument>
      )}
    </div>
  );
}

function SpecDocument({ markdown, path, references, children }: { markdown: string; path: string; references: ReferenceIndex; children: React.ReactNode }) {
  const headings = useMemo(() => headingsOf(markdown), [markdown]);
  const { bind, popover } = useReferencePopover(references);
  const [editing, setEditing] = useState(false);
  const editable = path.startsWith('vostok/work/') && !path.includes('/.archive/');
  return (
    <div className={page.docLayout}>
      <Toc headings={headings} />
      <Card as="article" className={page.docMain}>
        <Markdown source={markdown} references={references} bind={bind} />
        <div className={styles.pathRow}>
          <p className={styles.path}>{path}</p>
          {editable ? <Button variant="ghost" onClick={() => setEditing(true)}>Editar o arquivo</Button> : null}
        </div>
      </Card>
      <aside className={page.docAside}>{children}</aside>
      {popover}
      {editing ? <EditDialog path={path} title={path.split('/').pop() ?? path} onClose={() => setEditing(false)} /> : null}
    </div>
  );
}

function PlanView({ id, version, references, outdated, status }: { id: string; version: number; references: ReferenceIndex; outdated: boolean; status: string }) {
  const plan = useResource<PlanDetail>(`/api/planos/${encodeURIComponent(id)}`, version);
  const { bind, popover } = useReferencePopover(references);
  const headings = useMemo(() => (plan.kind === 'ready' ? headingsOf(plan.data.markdown) : []), [plan]);
  if (plan.kind === 'loading') return <LoadingState label="Carregando o plano" rows={6} />;
  if (plan.kind === 'error') return <ErrorState title="Nenhum plano aberto" description="O plano só existe entre o planejamento e a aprovação." retryLabel="Tentar de novo" onRetry={plan.retry} />;
  return (
    <div className={page.docLayout}>
      <Toc headings={headings} />
      <Card as="article" className={page.docMain}>
        <Markdown source={plan.data.markdown} references={references} bind={bind} />
      </Card>
      <aside className={page.docAside}>
        {outdated ? (
          <>
            <p className={page.asideTitle}>Plano desatualizado</p>
            <p className={styles.note}>
              {status === 'approved'
                ? 'A spec mudou depois deste plano, e ele não cobre a versão aprovada. Use "Refazer o plano" para criar um plano novo.'
                : 'A spec mudou depois deste plano e espera uma nova aprovação. Aprove a spec antes de refazer o plano.'}
            </p>
          </>
        ) : (
          <>
            <p className={page.asideTitle}>Revisão</p>
            <p className={styles.note}>Aprovar o plano cria e detalha as tarefas. Pelo painel, a aprovação vira um aval e um pedido para o agente em loop (TASK-04).</p>
          </>
        )}
      </aside>
      {popover}
    </div>
  );
}
