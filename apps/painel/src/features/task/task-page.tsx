import { useEffect, useMemo, useState } from 'react';
import { formatDate, listOf, useResource, type SpecDetail, type TaskDetail, type TaskSummary } from '../../shared/api';
import { BackLink } from '../../shared/back-link';
import { buildReferences } from '../../shared/references';
import { useReferencePopover } from '../../shared/ref-popover';
import { Markdown, slug } from '../../shared/markdown';
import { navigate, onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import { Button } from '../../ui/button/button';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { Tabs } from '../../ui/tabs/tabs';
import page from '../../shared/page.module.css';
import { EvidenceList } from './evidence-list/evidence-list';
import styles from './task-page.module.css';

/** Ordem de leitura: primeiro o que interessa a quem decide, depois o que é roteiro do agente. */
const HUMAN = [
  { file: 'executivo', title: 'Resumo executivo' },
  { file: 'tecnico', title: 'Detalhes técnicos' },
  { file: 'criterios', title: 'Critérios de conclusão' },
];
const AGENT = [
  { file: 'execucao', title: 'Execução' },
  { file: 'verificacao', title: 'Verificação' },
  { file: 'registro', title: 'Registro de execução' },
];

const TASK_TABS = [
  { id: 'resumo', label: 'Resumo' },
  { id: 'evidencias', label: 'Evidências' },
  { id: 'agente', label: 'Para o agente' },
];

function tabFromHash(hash: string): string {
  const id = hash.replace(/^#/, '');
  return TASK_TABS.some((tab) => tab.id === id) ? id : 'resumo';
}

const NO_REVIEW_MESSAGE = 'Nenhuma outra tarefa da onda tem evidência a conferir.';
const KEY_BLOCKERS = 'input, textarea, select, [contenteditable="true"], [role="tablist"], dialog';

function taskNumber(id: string): number {
  return Number(id.replace(/\D/g, ''));
}

function waveTasks(tasks: TaskSummary[], id: string, wave: string): TaskSummary[] {
  const current = tasks.find((item) => item.id === id);
  if (!wave) return current ? [current] : [];
  const same = tasks.filter((item) => item.wave === wave);
  const list = current && !same.includes(current) ? [...same, current] : same;
  return list.sort((a, b) => taskNumber(a.id) - taskNumber(b.id));
}

async function hasReviewToCheck(spec: string, id: string): Promise<boolean> {
  try {
    const response = await fetch(`/api/tarefas/${encodeURIComponent(spec)}/${encodeURIComponent(id)}`);
    if (!response.ok) return false;
    const detail = (await response.json()) as TaskDetail;
    return (detail.reviews ?? []).some((review) => review.status === 'a conferir');
  } catch {
    return false;
  }
}

function WaveBar({ spec, task, wave, tasks, onReview }: { spec: string; task: string; wave: string; tasks: TaskSummary[]; onReview: () => void }) {
  const list = waveTasks(tasks, task, wave);
  const index = list.findIndex((item) => item.id === task);
  const previous = index > 0 ? list[index - 1] : undefined;
  const next = index >= 0 && index < list.length - 1 ? list[index + 1] : undefined;
  const [searching, setSearching] = useState(false);
  const [message, setMessage] = useState('');
  useEffect(() => setMessage(''), [task]);
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
      if ((event.target as Element | null)?.closest?.(KEY_BLOCKERS)) return;
      const target = event.key === 'ArrowLeft' ? previous : event.key === 'ArrowRight' ? next : undefined;
      if (target) navigate(`/tarefas/${spec}/${target.id}`);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [previous, next, spec]);

  async function nextToReview() {
    setSearching(true);
    setMessage('');
    const candidates = [...list.slice(index + 1), ...list.slice(0, index)];
    for (const candidate of candidates) {
      if (await hasReviewToCheck(spec, candidate.id)) {
        onReview();
        navigate(`/tarefas/${spec}/${candidate.id}#evidencias`);
        setSearching(false);
        return;
      }
    }
    setMessage(NO_REVIEW_MESSAGE);
    setSearching(false);
  }

  return (
    <nav aria-label="Tarefas da onda" className={styles.waveBar}>
      {previous && <a href={`/tarefas/${spec}/${previous.id}`} onClick={onLinkClick} aria-label={`Tarefa anterior: ${previous.id}`} className={styles.waveStep}><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg></a>}
      <span className={styles.wavePosition}>{wave ? `${task} · Onda ${wave} · ${index + 1} de ${list.length}` : `${task} · Sem onda`}</span>
      {next && <a href={`/tarefas/${spec}/${next.id}`} onClick={onLinkClick} aria-label={`Próxima tarefa: ${next.id}`} className={styles.waveStep}><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg></a>}
      {list.length > 1 && <Button variant="ghost" size="small" className={styles.waveAction} loading={searching} onClick={nextToReview}>Próxima a conferir</Button>}
      {list.length > 1 && <p role="status" className={styles.waveMessage}>{message}</p>}
    </nav>
  );
}

export function TaskPage({ spec, task, version }: { spec: string; task: string; version: number }) {
  const detail = useResource<TaskDetail>(`/api/tarefas/${encodeURIComponent(spec)}/${encodeURIComponent(task)}`, version);
  const specDetail = useResource<SpecDetail>(`/api/specs/${encodeURIComponent(spec)}`, version);
  const references = useMemo(() => (specDetail.kind === 'ready' ? buildReferences(specDetail.data.markdown, specDetail.data.tasks, spec) : new Map()), [specDetail, spec]);
  const { bind, popover } = useReferencePopover(references);
  const [tab, setTab] = useState(() => tabFromHash(window.location.hash));
  function select(id: string) {
    setTab(id);
    history.replaceState(history.state, '', '#' + id);
  }

  if (detail.kind === 'loading') return <LoadingState label={`Carregando ${task}`} rows={8} />;
  if (detail.kind === 'error') return <ErrorState title={`Não foi possível abrir ${task}`} description={detail.message} retryLabel="Tentar de novo" onRetry={detail.retry} />;

  const { state, files } = detail.data;
  const results: [string, string][] = [
    ['Testes', state.tests_result],
    ['Regressão', state.regression_result],
    ['Evidências', state.evidence_result],
  ];
  const waitingOn = listOf(state.waiting_on ?? '');
  const closesAfter = specDetail.kind === 'ready' ? listOf(state.closes_after ?? '').filter((id) => specDetail.data.tasks.find((item) => item.id === id)?.status !== 'done') : [];

  return (
    <div className={page.stack}>
      <PageHeader back={<BackLink fallback={`/tarefas?spec=${spec}`} label={`Quadro de ${spec}`} />} eyebrow={`${spec} · ${state.id}`} title={state.title} subtitle={state.description?.replace(/`/g, "")} actions={
          <div className={styles.headerActions}>
            <Badge tone={statusTone(state.status)}>{statusLabel(state.status)}</Badge>
          </div>
        } />
      {specDetail.kind === 'ready' && <WaveBar spec={spec} task={state.id} wave={state.wave ?? ''} tasks={specDetail.data.tasks} onReview={() => setTab('evidencias')} />}
      <div className={page.docLayout}>
        <div className={page.docMain}>
          <Tabs
            label="Partes da tarefa"
            value={tab}
            onChange={select}
            tabs={[
              {
                id: 'resumo',
                label: 'Resumo',
                panel: HUMAN.map((part) => (
                  <section key={part.file} aria-labelledby={slug(part.title)} className={styles.part}>
                    <h2 id={slug(part.title)} className={styles.partTitle}>{part.title}</h2>
                    {files[part.file] ? <Markdown source={stripTitle(files[part.file])} prefix={`${part.file}-`} references={references} bind={bind} /> : <p className={styles.muted}>Ainda não escrito.</p>}
                  </section>
                )),
              },
              {
                id: 'evidencias',
                label: 'Evidências',
                panel: (
                  <section aria-labelledby="evidencias" className={styles.part}>
                    <EvidenceList detail={detail.data} headingId="evidencias" />
                  </section>
                ),
              },
              {
                id: 'agente',
                label: 'Para o agente',
                panel: (
                  <>
                    <p className={styles.tabIntro}>Roteiro que o agente segue: arquivos, passos, comandos de verificação e o registro de cada etapa.</p>
                    {AGENT.map((part) => (
                      <section key={part.file} aria-labelledby={slug(part.title)} className={styles.part}>
                        <h2 id={slug(part.title)} className={styles.partTitle}>{part.title}</h2>
                        {files[part.file] ? <Markdown source={stripTitle(files[part.file])} prefix={`${part.file}-`} references={references} bind={bind} /> : <p className={styles.muted}>Vazio.</p>}
                      </section>
                    ))}
                  </>
                ),
              },
            ]}
          />
        </div>
        <aside className={page.docAside}>
          <dl className={page.meta}>
            <div><dt>Status</dt><dd>{statusLabel(state.status)}</dd></div>
            {state.wave ? <div><dt>Onda</dt><dd>{state.wave}</dd></div> : null}
            {waitingOn.length > 0 ? <div><dt>Espera</dt><dd>{`${waitingOn.join(', ')}${state.waiting_reason ? `: ${state.waiting_reason}` : ''}`}</dd></div> : null}
            {closesAfter.length > 0 ? <div><dt>Fecha depois de</dt><dd>{closesAfter.map((id) => <a key={id} href={`/tarefas/${spec}/${id}`} onClick={onLinkClick} className={styles.dep}>{id}</a>)}</dd></div> : null}
            <div><dt>Depende de</dt><dd>{listOf(state.depends_on ?? '').map((dep) => <a key={dep} href={`/tarefas/${spec}/${dep}`} onClick={onLinkClick} className={styles.dep}>{dep}</a>)}{listOf(state.depends_on ?? '').length === 0 ? '—' : null}</dd></div>
            <div><dt>Cobre</dt><dd ref={bind} className={styles.refList}>{listOf(state.covers ?? '').map((id) => <button key={id} type="button" className="ref" data-ref={id} aria-haspopup="dialog">{id}</button>)}{listOf(state.covers ?? '').length === 0 ? '—' : null}</dd></div>
            {state.profile ? <div><dt>Perfil</dt><dd>{state.profile}</dd></div> : null}
            {state.persona ? <div><dt>Persona</dt><dd>{state.persona}</dd></div> : null}
            <div><dt>Modelo</dt><dd>{state.model || '—'}</dd></div>
            <div><dt>Aprovada</dt><dd>{formatDate(state.approved_at ?? '')}</dd></div>
            <div><dt>Tentativas</dt><dd>{state.attempts ?? '0'}</dd></div>
          </dl>
          <section>
            <p className={page.asideTitle}>Validação</p>
            <ul className={styles.results}>
              {results.map(([label, value]) => (
                <li key={label}>
                  <span>{label}</span>
                  <Badge tone={statusTone(value)}>{value && value !== 'null' ? statusLabel(value) : 'pendente'}</Badge>
                </li>
              ))}
            </ul>
          </section>
        </aside>
      </div>
      {popover}
    </div>
  );
}

function stripTitle(markdown: string): string {
  return markdown.replace(/^##+ .*\n/, '').replace(/^### /gm, '#### ');
}
