import { useMemo, useState } from 'react';
import { BackLink } from '../../shared/back-link';
import { buildReferences, type ReferenceIndex } from '../../shared/references';
import { useReferencePopover } from '../../shared/ref-popover';
import { Button } from '../../ui/button/button';
import { navigate } from '../../shared/router';
import { formatDate, useResource, type PlanDetail, type SpecDetail } from '../../shared/api';
import { Markdown } from '../../shared/markdown';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { Card } from '../../ui/card/card';
import { PageHeader } from '../../ui/page-header/page-header';
import { Tabs } from '../../ui/tabs/tabs';
import { Board } from '../board/board';
import page from '../../shared/page.module.css';
import styles from './spec-page.module.css';

type View = 'documento' | 'quadro' | 'plano';

const SPEC_TABS = [
  { id: 'visao-geral', label: 'Visão geral', titles: ['Objetivo', 'Estado atual', 'Defeito', 'Escopo', 'Premissas e dependências', 'Riscos'] },
  { id: 'requisitos', label: 'Requisitos', titles: ['Requisitos funcionais', 'Requisitos não funcionais', 'Regras de negócio', 'Fluxos', 'Casos de borda', 'Contratos e interfaces', 'Componentes afetados'] },
  { id: 'testes', label: 'Testes e aceite', titles: ['Casos de teste', 'Critérios de aceite'] },
  { id: 'decisoes', label: 'Decisões', titles: ['O que mudou', 'Registro de decisões', 'Pendências', 'Referências', 'Histórico de ajustes'] },
];

function sectionsOf(markdown: string): { title: string; text: string }[] {
  const sections = [{ title: '', lines: [] as string[] }];
  let fenced = false;
  for (const line of markdown.split('\n')) {
    if (line.trimStart().startsWith('```')) fenced = !fenced;
    if (!fenced && line.startsWith('## ')) sections.push({ title: line.slice(3).trim(), lines: [] });
    sections[sections.length - 1].lines.push(line);
  }
  return sections
    .map(({ title, lines }) => ({ title, text: lines.join('\n') }))
    .filter((section) => section.title !== '' || section.text.trim() !== '');
}

function specTabs(markdown: string): { id: string; label: string; source: string }[] {
  const sections = sectionsOf(markdown);
  const known = new Set(SPEC_TABS.flatMap((tab) => tab.titles));
  return SPEC_TABS.map((tab) => ({
    id: tab.id,
    label: tab.label,
    source: sections
      .filter((section) => (tab.titles.includes(section.title) || (tab.id === 'visao-geral' && !known.has(section.title))))
      .map((section) => section.text)
      .join('\n'),
  })).filter((tab) => tab.source.trim() !== '');
}

function tabFromHash(hash: string, ids: string[]): string {
  const id = hash.replace(/^#/, '');
  return ids.includes(id) ? id : ids[0];
}

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
  const tabs = useMemo(() => specTabs(markdown), [markdown]);
  const [active, setActive] = useState(() => tabFromHash(window.location.hash, tabs.map((tab) => tab.id)));
  const selected = tabs.some((tab) => tab.id === active) ? active : tabs[0]?.id;
  const { bind, popover } = useReferencePopover(references);
  function select(id: string) {
    setActive(id);
    history.replaceState(history.state, '', '#' + id);
  }
  return (
    <div className={page.docLayout}>
      <article className={page.docMain}>
        <Tabs
          label="Partes da spec"
          tabs={tabs.map((tab) => ({ id: tab.id, label: tab.label, panel: <Markdown source={tab.source} references={references} bind={bind} /> }))}
          value={selected}
          onChange={select}
        />
        <div className={styles.pathRow}>
          <p className={styles.path}>{path}</p>
        </div>
      </article>
      <aside className={page.docAside}>{children}</aside>
      {popover}
    </div>
  );
}

function PlanView({ id, version, references, outdated, status }: { id: string; version: number; references: ReferenceIndex; outdated: boolean; status: string }) {
  const plan = useResource<PlanDetail>(`/api/planos/${encodeURIComponent(id)}`, version);
  const { bind, popover } = useReferencePopover(references);
  if (plan.kind === 'loading') return <LoadingState label="Carregando o plano" rows={6} />;
  if (plan.kind === 'error') return <ErrorState title="Nenhum plano aberto" description="O plano só existe entre o planejamento e a aprovação." retryLabel="Tentar de novo" onRetry={plan.retry} />;
  return (
    <div className={page.docLayout}>
      <Card as="article" className={page.docMain}>
        <Markdown source={plan.data.markdown} references={references} bind={bind} />
      </Card>
      <aside className={page.docAside}>
        {outdated ? (
          <>
            <p className={page.asideTitle}>Plano desatualizado</p>
            <p className={styles.note}>
              {status === 'approved'
                ? 'A spec mudou depois deste plano, e ele não cobre a versão aprovada. Um plano novo precisa ser criado.'
                : 'A spec mudou depois deste plano e espera uma nova aprovação. A spec precisa ser aprovada antes de o plano ser refeito.'}
            </p>
          </>
        ) : (
          <>
            <p className={page.asideTitle}>Revisão</p>
            <p className={styles.note}>Aprovar o plano cria e detalha as tarefas. O painel só mostra o plano; a aprovação acontece pelo chat.</p>
          </>
        )}
      </aside>
      {popover}
    </div>
  );
}
