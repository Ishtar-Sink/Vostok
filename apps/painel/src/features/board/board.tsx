import { useState } from 'react';
import { formatDateTime, listOf, type PanelRequest, type TaskSummary } from '../../shared/api';
import { ActionDialog } from '../../shared/action-bar';
import { activeRequest, FREE_QUESTION, requestLabel, type AttentionItem } from '../../shared/attention';
import { RequestNotes } from '../../shared/request-notes';
import { onLinkClick } from '../../shared/router';
import { BOARD_SECTIONS } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import { Button } from '../../ui/button/button';
import styles from './board.module.css';

/** Cor de status das colunas do fluxo; as de entrada (para aprovar e aprovadas) ficam sem cor. */
const COLUMN_TONE: Record<string, 'info' | 'accent' | 'warning' | 'success'> = { ready: 'success', running: 'info', validating: 'accent', blocked: 'warning', done: 'success' };

export interface BoardTask extends TaskSummary {
  spec: string;
}

interface WaveGroup {
  spec: string;
  wave: string;
  cards: BoardTask[];
}

/** Cartões aprovados por spec e onda, na ordem das ondas. */
function waveGroups(cards: BoardTask[]): WaveGroup[] {
  const groups = new Map<string, WaveGroup>();
  for (const task of cards) {
    const key = `${task.spec}/${task.wave}`;
    const group = groups.get(key) ?? { spec: task.spec, wave: task.wave, cards: [] };
    group.cards.push(task);
    groups.set(key, group);
  }
  return [...groups.values()].sort((a, b) => a.spec.localeCompare(b.spec) || Number(a.wave === '') - Number(b.wave === '') || Number(a.wave) - Number(b.wave));
}

function executeWave(group: WaveGroup, ready: BoardTask[]): AttentionItem {
  const ids = ready.map((task) => task.id);
  return { key: `wave-${group.spec}-${group.wave}`, kind: 'execute-wave', spec: group.spec, wave: group.wave, tasks: ids, title: `Onda ${group.wave}: ${ids.join(', ')}`, reason: '', href: `/specs/${group.spec}/quadro` };
}

function executeTask(task: BoardTask): AttentionItem {
  return { key: `execute-${task.spec}-${task.id}`, kind: 'execute-task', spec: task.spec, task: task.id, title: task.title, reason: '', href: `/tarefas/${task.spec}/${task.id}` };
}

/** Pedido que mostra o selo "Com o agente" no cartão: o da própria tarefa, o `aprovar-plano` da spec
 *  enquanto a tarefa está pendente, o `executar-onda` da onda dela ou, na tarefa pronta, outro pedido da spec. */
export function agentRequest(task: BoardTask, requests: PanelRequest[], ready: boolean): PanelRequest | undefined {
  const own = activeRequest(requests, task.spec, task.id);
  if (own) return own;
  const spec = activeRequest(requests, task.spec);
  if (spec?.acao === 'aprovar-plano' && task.status === 'pending') return spec;
  if (spec?.acao === 'executar-onda' && spec.onda === task.wave && task.status !== 'done') return spec;
  if (spec && ready && spec.acao !== 'executar-onda') return spec;
  return undefined;
}

/** Quadro kanban: uma seção por etapa do fluxo da tarefa; as aprovadas ficam agrupadas por onda, e os botões de execução somem durante um pedido da spec ou da tarefa. */
export function Board({ tasks, showSpec = false, requests = [] }: { tasks: BoardTask[]; showSpec?: boolean; requests?: PanelRequest[] }) {
  const done = new Set(tasks.filter((task) => task.status === 'done').map((task) => `${task.spec}/${task.id}`));
  const [selected, setSelected] = useState<AttentionItem | null>(null);
  const [notesOf, setNotesOf] = useState<string | null>(null);
  const isReady = (task: BoardTask) => task.status === 'approved' && listOf(task.depends_on).every((dep) => done.has(`${task.spec}/${dep}`));
  const running = (group: WaveGroup) => tasks.some((task) => task.spec === group.spec && task.wave === group.wave && (task.status === 'in_progress' || task.status === 'validating'));
  const inSection = (task: BoardTask, id: string) => (id === 'ready' ? isReady(task) : id === 'approved' ? !isReady(task) : true);
  const [doneOpen, setDoneOpen] = useState(false);
  const specBusy = (spec: string) => requests.some((request) => request.spec === spec && request.acao !== FREE_QUESTION);
  const taskBusy = (task: BoardTask) => Boolean(activeRequest(requests, task.spec, task.id) || activeRequest(requests, task.spec));

  const renderCard = (task: BoardTask) => {
    const deps = listOf(task.depends_on);
    const declared = listOf(task.waiting_on ?? '');
    const waiting = declared.length > 0 ? declared : deps.filter((dep) => !done.has(`${task.spec}/${dep}`));
    const ready = isReady(task);
    const request = agentRequest(task, requests, ready);
    const facts = [task.profile, task.persona].filter(Boolean);
    return (
      <li key={`${task.spec}/${task.id}`} className={styles.cardItem}>
        <a href={`/tarefas/${task.spec}/${task.id}`} onClick={onLinkClick} className={styles.card}>
          <span className={styles.cardTop}>
            <span className={styles.id}>{showSpec ? `${task.spec} · ${task.id}` : task.id}</span>
            {task.wave ? <span className={styles.wave}>onda {task.wave}</span> : null}
          </span>
          <span className={styles.title}>{task.title}</span>
          {facts.length > 0 ? <span className={styles.facts}>{facts.join(' · ')}</span> : null}
          {task.status !== 'done' && waiting.length > 0 ? <span className={styles.waiting}>Espera {waiting.join(', ')}</span> : null}
          {declared.length > 0 && task.waiting_reason ? <span className={styles.facts}>{task.waiting_reason}</span> : null}
          {request ? <span className={styles.agentWork}>Com o agente: {requestLabel(request)}</span> : null}
          {request?.nota ? <span className={styles.agentNote}>{request.nota} · {formatDateTime(request.nota_em ?? '')}</span> : null}
        </a>
        {request?.nota ? <Button variant="secondary" className={styles.cardAction} aria-label={`Ver as notas de ${request.id} em ${task.id}`} onClick={() => setNotesOf(request.id)}>Ver notas</Button> : null}
        {ready && !taskBusy(task) ? <Button variant="secondary" className={styles.cardAction} aria-label={`Executar ${task.id}`} onClick={() => setSelected(executeTask(task))}>Executar</Button> : null}
      </li>
    );
  };

  return (
    <>
      <div className={styles.board}>
        {BOARD_SECTIONS.map((column) => {
          const cards = tasks.filter((task) => column.statuses.includes(task.status) && inSection(task, column.id));
          return (
            <section key={column.id} className={styles.column} data-column={column.id} aria-labelledby={`col-${column.id}`}>
              <header className={styles.columnHeader}>
                <h3 id={`col-${column.id}`}>
                  {column.id === 'done' && cards.length > 0 ? (
                    <button type="button" className={styles.sectionToggle} aria-expanded={doneOpen} onClick={() => setDoneOpen((open) => !open)}>
                      {COLUMN_TONE[column.id] ? <span className={styles.dot} data-tone={COLUMN_TONE[column.id]} aria-hidden="true" /> : null}
                      {column.label}
                    </button>
                  ) : (
                    <>
                      {COLUMN_TONE[column.id] ? <span className={styles.dot} data-tone={COLUMN_TONE[column.id]} aria-hidden="true" /> : null}
                      {column.label}
                    </>
                  )}
                </h3>
                <span className={styles.count}>{cards.length}</span>
              </header>
              {column.id === 'done' && cards.length > 0 && !doneOpen ? null : (
                <ul className={styles.cards}>
                  {column.id === 'approved' || column.id === 'ready'
                    ? waveGroups(cards).map((group) => {
                        const name = `${showSpec ? `${group.spec} · ` : ''}${group.wave ? `Onda ${group.wave}` : 'Sem onda'}`;
                        return (
                          <li key={`${group.spec}/${group.wave}`}>
                            <div role="group" aria-label={name} className={styles.waveGroup}>
                              <div className={styles.waveHeader}>
                                <span className={styles.waveTitle}>{name}</span>
                                {group.wave && running(group) ? <Badge tone="info">em execução</Badge> : null}
                                {column.id === 'ready' && group.wave && !specBusy(group.spec) ? (
                                  <Button variant="secondary" onClick={() => setSelected(executeWave(group, group.cards))}>
                                    Executar onda {group.wave}
                                  </Button>
                                ) : null}
                              </div>
                              <ul className={styles.cards}>{group.cards.map(renderCard)}</ul>
                            </div>
                          </li>
                        );
                      })
                    : cards.map(renderCard)}
                  {cards.length === 0 ? <li className={styles.empty}>Nenhuma</li> : null}
                </ul>
              )}
            </section>
          );
        })}
      </div>
      <ActionDialog item={selected} onClose={() => setSelected(null)} />
      {notesOf ? <RequestNotes id={notesOf} onClose={() => setNotesOf(null)} /> : null}
    </>
  );
}
