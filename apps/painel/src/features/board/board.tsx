import { useState } from 'react';
import { listOf, type TaskSummary } from '../../shared/api';
import { onLinkClick } from '../../shared/router';
import { BOARD_SECTIONS, statusLabel, statusTone } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import styles from './board.module.css';

/** Cor de status das colunas do fluxo; as de entrada (para aprovar e aprovadas) ficam sem cor. */
const COLUMN_TONE: Record<string, 'info' | 'accent' | 'warning' | 'success'> = { ready: 'success', running: 'info', executed: 'success', validating: 'accent', blocked: 'warning', done: 'success' };

export interface BoardTask extends TaskSummary {
  spec: string;
}

export type BoardGroup = 'etapa' | 'spec' | 'onda';
export type BoardOrder = 'numero' | 'spec' | 'onda';

const taskNumber = (id: string) => Number(id.replace(/\D/g, ''));
const waveNumber = (wave: string) => (wave === '' ? Infinity : Number(wave));
const compareNumbers = (a: number, b: number) => (a === b ? 0 : a < b ? -1 : 1);

/** Cópia dos cartões na ordem pedida; o critério principal desempata pelos seguintes. */
function sortCards(cards: BoardTask[], order: BoardOrder): BoardTask[] {
  const byNumber = (a: BoardTask, b: BoardTask) => taskNumber(a.id) - taskNumber(b.id);
  const bySpec = (a: BoardTask, b: BoardTask) => a.spec.localeCompare(b.spec);
  const byWave = (a: BoardTask, b: BoardTask) => compareNumbers(waveNumber(a.wave), waveNumber(b.wave));
  const criteria = { numero: [byNumber, bySpec], spec: [bySpec, byNumber], onda: [byWave, bySpec, byNumber] }[order];
  return [...cards].sort((a, b) => criteria.reduce((result, compare) => result || compare(a, b), 0));
}

interface CardGroup {
  key: string;
  name: string;
  cards: BoardTask[];
}

/** Grupos por spec ou por onda, na ordem natural da chave. */
function groupCards(tasks: BoardTask[], group: 'spec' | 'onda'): CardGroup[] {
  const groups = new Map<string, CardGroup>();
  for (const task of tasks) {
    const key = group === 'spec' ? task.spec : task.wave;
    const name = group === 'spec' ? task.spec : key ? `Onda ${key}` : 'Sem onda';
    const entry = groups.get(key) ?? { key, name, cards: [] };
    entry.cards.push(task);
    groups.set(key, entry);
  }
  return [...groups.values()].sort((a, b) => (group === 'spec' ? a.key.localeCompare(b.key) : compareNumbers(waveNumber(a.key), waveNumber(b.key))));
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

/** Quadro kanban só de leitura: uma seção por etapa do fluxo da tarefa; as aprovadas ficam agrupadas por onda. */
export function Board({ tasks, showSpec = false, group = 'etapa', order = 'numero' }: { tasks: BoardTask[]; showSpec?: boolean; group?: BoardGroup; order?: BoardOrder }) {
  const done = new Set(tasks.filter((task) => task.status === 'done').map((task) => `${task.spec}/${task.id}`));
  const isReady = (task: BoardTask) => task.status === 'approved' && listOf(task.depends_on).every((dep) => done.has(`${task.spec}/${dep}`));
  const running = (group: WaveGroup) => tasks.some((task) => task.spec === group.spec && task.wave === group.wave && (task.status === 'in_progress' || task.status === 'executed' || task.status === 'validating'));
  const inSection = (task: BoardTask, id: string) => (id === 'ready' ? isReady(task) : id === 'approved' ? !isReady(task) : true);
  const [doneOpen, setDoneOpen] = useState(false);

  const showStatus = group !== 'etapa';
  const renderCard = (task: BoardTask) => {
    const deps = listOf(task.depends_on);
    const pendingDeps = deps.filter((dep) => !done.has(`${task.spec}/${dep}`));
    const waitingOn = listOf(task.waiting_on ?? '');
    const closesAfter = listOf(task.closes_after ?? '').filter((id) => !done.has(`${task.spec}/${id}`));
    const facts = [task.profile, task.persona].filter(Boolean);
    return (
      <li key={`${task.spec}/${task.id}`} className={styles.cardItem}>
        <a href={`/tarefas/${task.spec}/${task.id}`} onClick={onLinkClick} className={styles.card}>
          <span className={styles.cardTop}>
            <span className={styles.id}>{showSpec ? `${task.spec} · ${task.id}` : task.id}</span>
            {task.wave ? <span className={styles.wave}>onda {task.wave}</span> : null}
            {showStatus ? <Badge tone={statusTone(task.status)}>{statusLabel(task.status)}</Badge> : null}
          </span>
          <span className={styles.title}>{task.title}</span>
          {facts.length > 0 ? <span className={styles.facts}>{facts.join(' · ')}</span> : null}
          {waitingOn.length > 0 ? <span className={styles.waiting}>{`Espera ${waitingOn.join(', ')}${task.waiting_reason ? `: ${task.waiting_reason}` : ''}`}</span> : null}
          {closesAfter.map((id) => <span key={id} className={styles.closesAfter}>{`Fecha depois de ${id}`}</span>)}
          {task.status !== 'done' && pendingDeps.length > 0 ? <span className={styles.dependsOn}>Depende de {pendingDeps.join(', ')}</span> : null}
        </a>
      </li>
    );
  };

  if (group !== 'etapa') {
    return (
      <div className={styles.board}>
        {groupCards(tasks, group).map((entry) => (
          <section key={entry.key} className={styles.column} data-group={entry.key} aria-labelledby={`grp-${entry.key}`}>
            <header className={styles.columnHeader}>
              <h3 id={`grp-${entry.key}`}>{entry.name}</h3>
              <span className={styles.count}>{entry.cards.length}</span>
            </header>
            <ul className={styles.cards}>{sortCards(entry.cards, order).map(renderCard)}</ul>
          </section>
        ))}
        {tasks.length === 0 ? <p className={styles.empty}>Nenhuma</p> : null}
      </div>
    );
  }

  return (
      <div className={styles.board}>
        {BOARD_SECTIONS.map((column) => {
          const cards = sortCards(tasks.filter((task) => column.statuses.includes(task.status) && inSection(task, column.id)), order);
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
  );
}
